// Emit only the reviewed relay policy from the central external-assumptions record.
use std::{env, fs, path::PathBuf};

fn main() {
    let source = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("../../docs/external-assumptions.md");
    println!("cargo:rerun-if-changed={}", source.display());
    let text = fs::read_to_string(&source).expect("central external assumptions");
    let policy = text
        .split("<!-- BEGIN AFTERGLOW REVIEWED PUBLIC DRAND RELAYS -->")
        .nth(1)
        .expect("reviewed relay policy start")
        .split("<!-- END AFTERGLOW REVIEWED PUBLIC DRAND RELAYS -->")
        .next()
        .expect("reviewed relay policy end")
        .split("```json")
        .nth(1)
        .expect("reviewed relay policy JSON start")
        .split("```")
        .next()
        .expect("reviewed relay policy JSON end");
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("reviewed-relays.json");
    fs::write(out, policy).expect("write reviewed relay policy");
}
