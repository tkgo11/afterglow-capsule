// Read the one central reviewed chain fixture. Only chain metadata is emitted;
// historical beacons and test keys cannot enter the non-test library this way.
use std::{env, fs, path::PathBuf};

fn main() {
    let source = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("../../docs/external-assumptions.md");
    println!("cargo:rerun-if-changed={}", source.display());
    let text = fs::read_to_string(&source).expect("central external assumptions");
    let fixture = text
        .split("<!-- BEGIN AFTERGLOW HISTORICAL QUICKNET FIXTURE -->")
        .nth(1)
        .expect("fixture start")
        .split("<!-- END AFTERGLOW HISTORICAL QUICKNET FIXTURE -->")
        .next()
        .expect("fixture end")
        .split("```json")
        .nth(1)
        .expect("JSON start")
        .split("```")
        .next()
        .expect("JSON end");
    let fixture: serde_json::Value = serde_json::from_str(fixture).expect("central fixture JSON");
    assert_eq!(fixture["format_version"], 1);
    let chain = fixture.get("chain").expect("chain metadata");
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("reviewed-chain.json");
    fs::write(out, serde_json::to_vec(chain).unwrap()).unwrap();
}
