//! The bootstrap binary must never claim to produce or open an archive.

#[test]
fn scaffold_exits_unsuccessfully_with_an_explanation() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_afterglow-viewer"))
        .output()
        .expect("bootstrap binary should start");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let diagnostic = String::from_utf8(output.stderr).expect("diagnostic should be UTF-8");
    assert!(diagnostic.contains("Phase 0 scaffold"));
    assert!(diagnostic.contains("unavailable"));
}
