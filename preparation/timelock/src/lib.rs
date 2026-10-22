//! Isolated exact-round adapter using the Spike A interoperability path.
//! No clock, mock beacon, preview switch or release bypass in non-test code.

use std::{
    future::Future,
    io::{self, Write},
    pin::Pin,
    sync::Arc,
    time::Duration,
};

use ag_prepared_object_crypto::ContentKey;
use ag_schema::{NetworkProfile, PinnedRelease, Validate};
use drand_core::{beacon::ApiBeacon, chain::ChainInfo};
use serde::Deserialize;
use tokio::task::JoinSet;
use zeroize::Zeroizing;

pub const MAX_BEACON_BYTES: usize = 16 * 1024;
pub const MAX_ENVELOPE_BYTES: usize = 16 * 1024;

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum TimelockError {
    #[error("network profile does not match the reviewed Quicknet identity")]
    Profile,
    #[error("release metadata or exact target round is invalid")]
    Release,
    #[error("beacon has invalid format, size, signature, randomness or round")]
    Beacon,
    #[error("timelock envelope has invalid format, size, chain or exact round")]
    Envelope,
    #[error("timelock authentication failed or CEK length is not 32 bytes")]
    Authentication,
    #[error("timelock encryption failed")]
    Encryption,
}

/// Only cryptographic verification creates this capability.
pub struct VerifiedBeacon {
    round: u64,
    chain_hash: Vec<u8>,
    signature: Vec<u8>,
}

impl std::fmt::Debug for VerifiedBeacon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VerifiedBeacon")
            .field("round", &self.round)
            .finish_non_exhaustive()
    }
}

pub trait TimelockEngine {
    fn verify_network_profile(&self, profile: &NetworkProfile) -> Result<(), TimelockError>;
    fn lock_cek(&self, cek: &ContentKey) -> Result<Vec<u8>, TimelockError>;
    fn verify_beacon(&self, bytes: &[u8]) -> Result<VerifiedBeacon, TimelockError>;
    fn unlock_cek(
        &self,
        envelope: &[u8],
        beacon: &VerifiedBeacon,
    ) -> Result<ContentKey, TimelockError>;
}

#[derive(Clone)]
pub struct QuicknetEngine {
    release: PinnedRelease,
    chain: ChainInfo,
}

impl QuicknetEngine {
    pub fn new(release: PinnedRelease) -> Result<Self, TimelockError> {
        release.validate().map_err(|_| TimelockError::Release)?;
        let chain: ChainInfo = serde_json::from_str(include_str!(concat!(
            env!("OUT_DIR"),
            "/reviewed-chain.json"
        )))
        .map_err(|_| TimelockError::Profile)?;
        let engine = Self { release, chain };
        engine.verify_network_profile(&engine.release.network)?;
        Ok(engine)
    }

    pub fn pinned_release(&self) -> &PinnedRelease {
        &self.release
    }

    pub fn validate_envelope(&self, envelope: &[u8]) -> Result<(), TimelockError> {
        if envelope.is_empty() || envelope.len() > MAX_ENVELOPE_BYTES {
            return Err(TimelockError::Envelope);
        }
        let header = tlock_age::decrypt_header(envelope).map_err(|_| TimelockError::Envelope)?;
        if header.round() != self.release.target_round || header.hash() != self.chain.hash() {
            return Err(TimelockError::Envelope);
        }
        Ok(())
    }

    /// Constructs only the exact target-round URL on the pinned relay allowlist.
    pub fn relay_urls(&self) -> Result<Vec<String>, TimelockError> {
        self.release
            .network
            .relays
            .iter()
            .map(|base| {
                let mut url = url::Url::parse(base).map_err(|_| TimelockError::Profile)?;
                url.path_segments_mut()
                    .map_err(|_| TimelockError::Profile)?
                    .pop_if_empty()
                    .push(&hex::encode(self.chain.hash()))
                    .push("public")
                    .push(&self.release.target_round.to_string());
                Ok(url.into())
            })
            .collect()
    }

    /// An HTTP 200 is only input to verification, never release authorization.
    /// Timeouts and ordinary absence remain WAITING, not cryptographic success.
    pub async fn fetch_target<T: RelayTransport>(
        &self,
        transport: Arc<T>,
        budget: Duration,
    ) -> FetchOutcome {
        let Ok(urls) = self.relay_urls() else {
            return FetchOutcome::Waiting(vec![RelayAttempt {
                relay_index: 0,
                failure: RelayFailure::Configuration,
            }]);
        };
        let deadline = tokio::time::Instant::now() + budget.min(Duration::from_secs(30));
        let mut jobs = JoinSet::new();
        for (index, url) in urls.into_iter().enumerate() {
            let transport = Arc::clone(&transport);
            let engine = self.clone();
            jobs.spawn(async move {
                tokio::time::sleep(Duration::from_millis(index as u64 * 200)).await;
                let result = match tokio::time::timeout(
                    Duration::from_secs(5),
                    transport.fetch(url),
                )
                .await
                {
                    Err(_) => Err(RelayFailure::Timeout),
                    Ok(Err(error)) => Err(error),
                    Ok(Ok(body)) => engine
                        .verify_beacon(&body)
                        .map_err(|_| RelayFailure::InvalidBeacon),
                };
                (index, result)
            });
        }
        let mut failures = Vec::new();
        loop {
            match tokio::time::timeout_at(deadline, jobs.join_next()).await {
                Ok(Some(Ok((_, Ok(beacon))))) => {
                    jobs.abort_all();
                    return FetchOutcome::Verified(beacon);
                }
                Ok(Some(Ok((index, Err(failure))))) => failures.push(RelayAttempt {
                    relay_index: index,
                    failure,
                }),
                Ok(Some(Err(_))) => failures.push(RelayAttempt {
                    relay_index: usize::MAX,
                    failure: RelayFailure::Transport,
                }),
                Ok(None) => return FetchOutcome::Waiting(failures),
                Err(_) => {
                    jobs.abort_all();
                    failures.push(RelayAttempt {
                        relay_index: usize::MAX,
                        failure: RelayFailure::Timeout,
                    });
                    return FetchOutcome::Waiting(failures);
                }
            }
        }
    }
}

impl TimelockEngine for QuicknetEngine {
    fn verify_network_profile(&self, profile: &NetworkProfile) -> Result<(), TimelockError> {
        profile.validate().map_err(|_| TimelockError::Profile)?;
        if profile.scheme_id != self.chain.scheme_id()
            || hex::decode(&profile.chain_hash).ok() != Some(self.chain.hash())
            || hex::decode(&profile.public_key).ok() != Some(self.chain.public_key())
            || profile.genesis_utc.timestamp() < 0
            || profile.genesis_utc.timestamp() as u64 != self.chain.genesis_time()
            || profile.genesis_utc.timestamp_subsec_nanos() != 0
            || profile.period_seconds != self.chain.period()
        {
            return Err(TimelockError::Profile);
        }
        Ok(())
    }

    fn lock_cek(&self, cek: &ContentKey) -> Result<Vec<u8>, TimelockError> {
        let mut ciphertext = Vec::new();
        cek.with_secret(|secret| {
            tlock_age::encrypt(
                &mut ciphertext,
                secret.as_slice(),
                &self.chain.hash(),
                &self.chain.public_key(),
                self.release.target_round,
            )
        })
        .map_err(|_| TimelockError::Encryption)?;
        self.validate_envelope(&ciphertext)?;
        Ok(ciphertext)
    }

    fn verify_beacon(&self, bytes: &[u8]) -> Result<VerifiedBeacon, TimelockError> {
        if bytes.is_empty() || bytes.len() > MAX_BEACON_BYTES {
            return Err(TimelockError::Beacon);
        }
        let wire: WireBeacon = serde_json::from_slice(bytes).map_err(|_| TimelockError::Beacon)?;
        let signature = hex::decode(&wire.signature).map_err(|_| TimelockError::Beacon)?;
        let randomness = hex::decode(&wire.randomness).map_err(|_| TimelockError::Beacon)?;
        if wire.round != self.release.target_round
            || signature.len() != 48
            || randomness.len() != 32
        {
            return Err(TimelockError::Beacon);
        }
        // Force the supported unchained type. Never dispatch to chained parsing
        // where untrusted previous_signature lengths can trigger upstream panics.
        let api: ApiBeacon = serde_json::from_value(serde_json::json!({
            "round": wire.round, "signature": wire.signature, "randomness": wire.randomness
        }))
        .map_err(|_| TimelockError::Beacon)?;
        if !api.is_unchained()
            || !api
                .verify(self.chain.clone())
                .map_err(|_| TimelockError::Beacon)?
        {
            return Err(TimelockError::Beacon);
        }
        Ok(VerifiedBeacon {
            round: wire.round,
            chain_hash: self.chain.hash(),
            signature,
        })
    }

    fn unlock_cek(
        &self,
        envelope: &[u8],
        beacon: &VerifiedBeacon,
    ) -> Result<ContentKey, TimelockError> {
        self.validate_envelope(envelope)?;
        if beacon.round != self.release.target_round || beacon.chain_hash != self.chain.hash() {
            return Err(TimelockError::Beacon);
        }
        let mut output = CekWriter {
            secret: Zeroizing::new([0; 32]),
            written: 0,
        };
        tlock_age::decrypt(&mut output, envelope, &self.chain.hash(), &beacon.signature)
            .map_err(|_| TimelockError::Authentication)?;
        if output.written != 32 {
            return Err(TimelockError::Authentication);
        }
        Ok(ContentKey::from_unwrapped(output.secret))
    }
}

#[derive(Deserialize)]
struct WireBeacon {
    round: u64,
    signature: String,
    randomness: String,
}

struct CekWriter {
    secret: Zeroizing<[u8; 32]>,
    written: usize,
}

impl Write for CekWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let end = self
            .written
            .checked_add(bytes.len())
            .filter(|n| *n <= 32)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid CEK length"))?;
        self.secret[self.written..end].copy_from_slice(bytes);
        self.written = end;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelayFailure {
    Unavailable,
    Timeout,
    Transport,
    InvalidBeacon,
    Configuration,
}

#[derive(Debug)]
pub struct RelayAttempt {
    pub relay_index: usize,
    pub failure: RelayFailure,
}

#[derive(Debug)]
pub enum FetchOutcome {
    Verified(VerifiedBeacon),
    Waiting(Vec<RelayAttempt>),
}

pub type RelayFuture = Pin<Box<dyn Future<Output = Result<Vec<u8>, RelayFailure>> + Send>>;

/// Transport implementations must enforce body limits, TLS verification, no
/// redirects, allowlist destinations and bounded I/O. No production HTTP adapter
/// is adopted by this preparation; a test transport exists only under cfg(test).
pub trait RelayTransport: Send + Sync + 'static {
    fn fetch(&self, url: String) -> RelayFuture;
}

#[cfg(test)]
mod tests;
