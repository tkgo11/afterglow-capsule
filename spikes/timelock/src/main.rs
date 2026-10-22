//! SPIKE ONLY. This executable is outside the production Cargo workspace.
//! It performs no networking and never manufactures a beacon or test private key.

use std::{error::Error, fs, io, process::ExitCode};

use drand_core::{beacon::ApiBeacon, chain::ChainInfo};
use serde::Deserialize;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Deserialize)]
struct Fixture {
    format_name: String,
    format_version: u16,
    minimum_reader_version: u16,
    chain: ChainInfo,
}

fn invalid(message: &'static str) -> Box<dyn Error> {
    Box::new(io::Error::new(io::ErrorKind::InvalidData, message))
}

fn read_bounded(path: &str, max: u64) -> Result<Vec<u8>> {
    if fs::metadata(path)?.len() > max {
        return Err(invalid("spike input exceeds its size limit"));
    }
    // Fixtures are controlled test files, not a production filesystem boundary.
    let bytes = fs::read(path)?;
    if bytes.len() as u64 > max {
        return Err(invalid("spike input exceeds its size limit"));
    }
    Ok(bytes)
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        return Err(invalid("usage: spike-a encrypt|decrypt|verify fixture ..."));
    }
    let fixture: Fixture = serde_json::from_slice(&read_bounded(&args[2], 16384)?)?;
    if fixture.format_name != "afterglow-spike-a-public-fixture"
        || fixture.format_version != 1
        || fixture.minimum_reader_version != 1
        || fixture.chain.scheme_id() != "bls-unchained-g1-rfc9380"
    {
        return Err(invalid("unsupported fixture format or scheme"));
    }
    match args[1].as_str() {
        "encrypt" if args.len() == 6 => {
            let plaintext = read_bounded(&args[3], 1024)?;
            let round: u64 = args[5].parse()?;
            if round == 0 {
                return Err(invalid("round zero is invalid"));
            }
            let mut ciphertext = Vec::new();
            tlock_age::encrypt(
                &mut ciphertext,
                plaintext.as_slice(),
                &fixture.chain.hash(),
                &fixture.chain.public_key(),
                round,
            )?;
            fs::write(&args[4], ciphertext)?;
        }
        "decrypt" if args.len() == 7 => {
            let ciphertext = read_bounded(&args[3], 16384)?;
            let beacon: ApiBeacon = serde_json::from_slice(&read_bounded(&args[5], 16384)?)?;
            let pinned_round: u64 = args[6].parse()?;
            let header = tlock_age::decrypt_header(ciphertext.as_slice())?;
            if pinned_round == 0
                || header.round() != pinned_round
                || header.hash() != fixture.chain.hash()
                || beacon.round() != pinned_round
                || !beacon.verify(fixture.chain.clone())?
            {
                return Err(invalid("pinned chain/round or beacon verification failed"));
            }
            let mut plaintext = Vec::new();
            tlock_age::decrypt(
                &mut plaintext,
                ciphertext.as_slice(),
                &fixture.chain.hash(),
                &beacon.signature(),
            )?;
            // A failed decryption never creates an output file or prints plaintext.
            fs::write(&args[4], plaintext)?;
        }
        "verify" if args.len() == 5 => {
            let beacon: ApiBeacon = serde_json::from_slice(&read_bounded(&args[3], 16384)?)?;
            let pinned_round: u64 = args[4].parse()?;
            if pinned_round == 0
                || beacon.round() != pinned_round
                || !beacon.verify(fixture.chain)?
            {
                return Err(invalid("beacon verification failed"));
            }
        }
        _ => return Err(invalid("invalid spike command or argument count")),
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Spike A: {error}");
            ExitCode::FAILURE
        }
    }
}
