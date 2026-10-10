//! SPIKE ONLY: no release engine/CEK access, TLS bypass, key logging or fallback NTP.

use std::{process::ExitCode, time::Duration};

use rkik_nts::{NtsClient, NtsClientConfig};

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3
        || args[1].trim().is_empty()
        || args[2].trim().is_empty()
        || args[1] == args[2]
    {
        eprintln!("usage: spike-b independent-operator-one-host independent-operator-two-host");
        return ExitCode::FAILURE;
    }
    // Hostnames must come from the centrally reviewed external assumptions.
    // Distinct hostnames alone do not establish operator independence.
    let mut failed = false;
    for host in &args[1..] {
        match tokio::time::timeout(Duration::from_secs(12), observe(host)).await {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                eprintln!("{host}: {error}");
                failed = true;
            }
            Err(_) => {
                eprintln!("{host}: timeout");
                failed = true;
            }
        }
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

async fn observe(host: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = NtsClientConfig::new(host)
        .with_timeout(Duration::from_secs(5))
        .with_max_retries(0);
    let mut client = NtsClient::new(config);
    client.connect().await?;
    let time = client.get_time().await?;
    if !time.authenticated || client.nts_ke_info().is_none() {
        return Err("NTS key exchange/authentication evidence is missing".into());
    }
    println!(
        "{host}: NTS authenticated; RTT={}us; network UTC epoch={}ms",
        time.round_trip_delay.as_micros(),
        time.network_time
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn zero_timeout_configuration_fails_without_network_io() {
        let config = NtsClientConfig::new("invalid.test").with_timeout(Duration::ZERO);
        let mut client = NtsClient::new(config);
        assert!(client.connect().await.is_err());
    }

    #[tokio::test]
    async fn certificate_verification_cannot_be_disabled_in_this_build() {
        let config = NtsClientConfig::new("invalid.test").with_tls_verification(false);
        let mut client = NtsClient::new(config);
        assert!(client.connect().await.is_err());
    }

    #[tokio::test]
    async fn stalled_local_tls_peer_times_out_without_downgrading_to_ntp() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let peer = tokio::spawn(async move {
            let (_stream, _) = listener.accept().await.unwrap();
            std::future::pending::<()>().await;
        });
        let config = NtsClientConfig::new("127.0.0.1")
            .with_port(port)
            .with_timeout(Duration::from_millis(100))
            .with_max_retries(0);
        let mut client = NtsClient::new(config);
        let result = tokio::time::timeout(Duration::from_secs(3), client.connect())
            .await
            .expect("library did not enforce its TLS timeout");
        peer.abort();
        assert!(
            matches!(result, Err(rkik_nts::Error::Timeout)),
            "{result:?}"
        );
        assert!(!client.is_connected());
        assert!(client.nts_ke_info().is_none());
    }

    #[tokio::test]
    async fn local_untrusted_certificate_is_rejected_before_key_exchange() {
        use rustls::pki_types::PrivatePkcs8KeyDer;
        use std::sync::Arc;

        // Ephemeral test-only TLS material. Nothing is persisted or installed as
        // a trust root, and the client retains its ordinary certificate verifier.
        let generated = rcgen::generate_simple_self_signed(vec!["127.0.0.1".into()]).unwrap();
        let key = PrivatePkcs8KeyDer::from(generated.signing_key.serialize_der()).into();
        let mut server_config = rustls::ServerConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_protocol_versions(&[&rustls::version::TLS13])
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![generated.cert.der().clone()], key)
        .unwrap();
        server_config.alpn_protocols = vec![b"ntske/1".to_vec()];
        let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(server_config));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let peer = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            acceptor.accept(stream).await.is_err()
        });
        let config = NtsClientConfig::new("127.0.0.1")
            .with_port(port)
            .with_timeout(Duration::from_secs(1))
            .with_max_retries(0);
        let mut client = NtsClient::new(config);
        let result = tokio::time::timeout(Duration::from_secs(3), client.connect())
            .await
            .unwrap();
        match result {
            Err(rkik_nts::Error::Tls(message)) => {
                assert!(message.contains("UnknownIssuer"), "{message}")
            }
            other => panic!("expected an untrusted certificate failure, got {other:?}"),
        }
        assert!(
            tokio::time::timeout(Duration::from_secs(3), peer)
                .await
                .unwrap()
                .unwrap()
        );
        assert!(!client.is_connected());
        assert!(client.nts_ke_info().is_none());
    }
}
