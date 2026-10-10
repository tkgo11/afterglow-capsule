use super::*;
use sha2::{Digest, Sha256};

fn fixture() -> serde_json::Value {
    let text = include_str!("../../../docs/external-assumptions.md");
    let json = text
        .split("<!-- BEGIN AFTERGLOW HISTORICAL QUICKNET FIXTURE -->")
        .nth(1)
        .unwrap()
        .split("```json")
        .nth(1)
        .unwrap()
        .split("```")
        .next()
        .unwrap();
    serde_json::from_str(json).unwrap()
}

fn engine(round: u64) -> QuicknetEngine {
    let chain = fixture()["chain"].clone();
    let genesis = chain["genesis_time"].as_u64().unwrap();
    let period = chain["period"].as_u64().unwrap();
    serde_json::from_value::<PinnedRelease>(serde_json::json!({
        "utc": timestamp(genesis + (round - 1) * period),
        "timezone": "UTC", "target_round": round, "timelock_profile": "quicknet-v1",
        "network": { "profile_id": "quicknet-v1", "chain_hash": chain["hash"],
            "public_key": chain["public_key"], "genesis_utc": timestamp(genesis),
            "period_seconds": period, "scheme_id": chain["schemeID"],
            "relays": ["https://relay-one.example", "https://relay-two.example"] }
    }))
    .map(QuicknetEngine::new)
    .unwrap()
    .unwrap()
}

fn timestamp(seconds: u64) -> String {
    chrono::DateTime::from_timestamp(i64::try_from(seconds).unwrap(), 0)
        .unwrap()
        .to_rfc3339()
}

fn beacon(round: u64, signature: &str) -> Vec<u8> {
    let bytes = hex::decode(signature).unwrap();
    serde_json::to_vec(&serde_json::json!({ "round": round, "signature": signature,
        "randomness": hex::encode(Sha256::digest(bytes)) }))
    .unwrap()
}

fn valid_beacon() -> Vec<u8> {
    let fixture = fixture();
    beacon(
        1000,
        fixture["additional_quicknet_beacon"]["signature"]
            .as_str()
            .unwrap(),
    )
}

#[test]
fn verified_exact_round_recovers_exact_cek() {
    let engine = engine(1000);
    let original = ContentKey::generate().unwrap();
    let envelope = engine.lock_cek(&original).unwrap();
    let verified = engine.verify_beacon(&valid_beacon()).unwrap();
    let recovered = engine.unlock_cek(&envelope, &verified).unwrap();
    original.with_secret(|a| recovered.with_secret(|b| assert_eq!(a, b)));
}

#[test]
fn wrong_round_wrong_chain_invalid_signature_and_randomness_fail() {
    let engine = engine(1000);
    let mut value: serde_json::Value = serde_json::from_slice(&valid_beacon()).unwrap();
    value["round"] = 1001.into();
    assert!(
        engine
            .verify_beacon(&serde_json::to_vec(&value).unwrap())
            .is_err()
    );
    let foreign = fixture()["historical_foreign_beacon"]["signature"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(engine.verify_beacon(&beacon(1000, &foreign)).is_err());
    assert!(
        engine
            .verify_beacon(&beacon(1000, &"00".repeat(48)))
            .is_err()
    );
    value = serde_json::from_slice(&valid_beacon()).unwrap();
    value["randomness"] = "00".repeat(32).into();
    assert!(
        engine
            .verify_beacon(&serde_json::to_vec(&value).unwrap())
            .is_err()
    );
}

#[test]
fn malformed_large_duplicate_and_chained_responses_fail_safely() {
    let engine = engine(1000);
    for input in [
        vec![],
        vec![b' '; MAX_BEACON_BYTES + 1],
        br#"{"round":1000,"round":1000,"signature":"00","randomness":"00"}"#.to_vec(),
        br#"{"round":1000,"signature":"00","randomness":"00","previous_signature":"00"}"#.to_vec(),
    ] {
        assert!(engine.verify_beacon(&input).is_err());
    }
}

#[test]
fn exact_round_pin_is_not_replaceable_by_another_engine_beacon() {
    let first = engine(1000);
    let future = engine(1001);
    let cek = ContentKey::generate().unwrap();
    let verified = first.verify_beacon(&valid_beacon()).unwrap();
    assert!(
        future
            .unlock_cek(&future.lock_cek(&cek).unwrap(), &verified)
            .is_err()
    );
    assert!(
        future
            .validate_envelope(&first.lock_cek(&cek).unwrap())
            .is_err()
    );
}

#[test]
fn envelope_tampering_truncation_size_and_cek_lengths_fail_closed() {
    let engine = engine(1000);
    let envelope = engine.lock_cek(&ContentKey::generate().unwrap()).unwrap();
    let verified = engine.verify_beacon(&valid_beacon()).unwrap();
    let mut altered = envelope.clone();
    *altered.last_mut().unwrap() ^= 1;
    assert!(engine.unlock_cek(&altered, &verified).is_err());
    assert!(
        engine
            .unlock_cek(&envelope[..envelope.len() - 1], &verified)
            .is_err()
    );
    assert!(
        engine
            .validate_envelope(&vec![0; MAX_ENVELOPE_BYTES + 1])
            .is_err()
    );
    for length in [0, 31, 33, 1024] {
        let mut wrong = Vec::new();
        tlock_age::encrypt(
            &mut wrong,
            vec![0; length].as_slice(),
            &engine.chain.hash(),
            &engine.chain.public_key(),
            1000,
        )
        .unwrap();
        assert!(engine.unlock_cek(&wrong, &verified).is_err());
    }
}

#[test]
fn all_pinned_network_fields_and_release_metadata_are_checked() {
    let engine = engine(1000);
    for field in ["chain_hash", "public_key", "scheme_id"] {
        let mut value = serde_json::to_value(&engine.release.network).unwrap();
        value[field] = "00".into();
        let profile: NetworkProfile = serde_json::from_value(value).unwrap();
        assert!(engine.verify_network_profile(&profile).is_err());
    }
    let mut wrong = engine.release.clone();
    wrong.network.period_seconds += 1;
    assert!(QuicknetEngine::new(wrong).is_err());
    let mut wrong = engine.release.clone();
    wrong.target_round += 1;
    assert!(QuicknetEngine::new(wrong).is_err());
}

struct Transport {
    bodies: Vec<Result<Vec<u8>, RelayFailure>>,
}

impl RelayTransport for Transport {
    fn fetch(&self, url: String) -> RelayFuture {
        let index = usize::from(url.contains("relay-two"));
        let result = self.bodies[index].clone();
        Box::pin(async move { result })
    }
}

#[tokio::test]
async fn relay_failure_invalid_first_and_valid_second_are_raced() {
    for first in [
        Err(RelayFailure::Transport),
        Ok(b"invalid HTTP 200".to_vec()),
        Err(RelayFailure::Unavailable),
    ] {
        let transport = Arc::new(Transport {
            bodies: vec![first, Ok(valid_beacon())],
        });
        assert!(matches!(
            engine(1000)
                .fetch_target(transport, Duration::from_secs(2))
                .await,
            FetchOutcome::Verified(_)
        ));
    }
}

#[tokio::test]
async fn absence_timeout_and_all_invalid_responses_remain_waiting() {
    let engine = engine(1000);
    let transport = Arc::new(Transport {
        bodies: vec![
            Err(RelayFailure::Unavailable),
            Err(RelayFailure::Unavailable),
        ],
    });
    assert!(
        matches!(engine.fetch_target(Arc::clone(&transport), Duration::from_secs(1)).await,
        FetchOutcome::Waiting(failures) if failures.len() == 2)
    );
    assert!(matches!(
        engine.fetch_target(transport, Duration::ZERO).await,
        FetchOutcome::Waiting(_)
    ));
}

#[test]
fn urls_contain_only_pinned_chain_and_round() {
    let engine = engine(1000);
    for url in engine.relay_urls().unwrap() {
        let url = url::Url::parse(&url).unwrap();
        assert!(url.path().ends_with("/public/1000"));
        assert!(url.path().contains(&engine.release.network.chain_hash));
        assert!(url.query().is_none());
    }
}
