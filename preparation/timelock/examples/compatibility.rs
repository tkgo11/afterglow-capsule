//! Development-only differential harness, outside both production applications.
use std::{
    error::Error,
    fs,
    io::{self, Read},
    process::ExitCode,
};

use ag_prepared_object_crypto::ContentKey;
use ag_prepared_timelock::{QuicknetEngine, TimelockEngine};
use ag_schema::PinnedRelease;
use zeroize::Zeroizing;

fn bounded(path: &str, limit: usize) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "fixture size"));
    }
    Ok(bytes)
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 3 {
        return Err("expected lock|unlock|verify release.json ...".into());
    }
    let release: PinnedRelease = serde_json::from_slice(&bounded(&args[2], 32 * 1024)?)?;
    let engine = QuicknetEngine::new(release)?;
    match args[1].as_str() {
        "lock" if args.len() == 5 => {
            let secret = Zeroizing::new(bounded(&args[3], 32)?);
            let bytes: [u8; 32] = secret.as_slice().try_into()?;
            let cek = ContentKey::from_unwrapped(Zeroizing::new(bytes));
            fs::write(&args[4], engine.lock_cek(&cek)?)?;
        }
        "unlock" if args.len() == 6 => {
            let envelope = bounded(&args[3], 16 * 1024)?;
            let beacon = engine.verify_beacon(&bounded(&args[4], 16 * 1024)?)?;
            let cek = engine.unlock_cek(&envelope, &beacon)?;
            cek.with_secret(|bytes| fs::write(&args[5], bytes))?;
        }
        "verify" if args.len() == 4 => {
            engine.verify_beacon(&bounded(&args[3], 16 * 1024)?)?;
        }
        _ => return Err("invalid compatibility command".into()),
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Prepared adapter compatibility: {error}");
            ExitCode::FAILURE
        }
    }
}
