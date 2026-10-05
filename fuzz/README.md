# Parser fuzzing

Phase 1 supplies runnable skeletons for capsule headers, public manifests, object
tables and editable project documents. Production parsers enforce size, version
and recursion bounds; fuzzing does not disable them.

```sh
cargo check --manifest-path fuzz/Cargo.toml --locked --bins
cargo +nightly fuzz run capsule_header -- -max_len=4096 -max_total_time=60
cargo +nightly fuzz run manifest -- -max_len=65536 -max_total_time=60
cargo +nightly fuzz run object_table -- -max_len=65536 -max_total_time=60
cargo +nightly fuzz run workspace -- -max_len=65536 -max_total_time=60
```

For the reproducible CI smoke, install the pinned development tools and run:

```sh
rustup toolchain install nightly-2025-09-15 --profile minimal
cargo +nightly-2025-09-15 install cargo-fuzz --version 0.13.2 --locked
python scripts/run_fuzz_smoke.py
```

The runner verifies libFuzzer instrumentation, runs each existing target for
30 seconds, preserves hashed logs under `fuzz/target/smoke-evidence`, and requires
unchanged lockfiles. CI uploads those logs. This bounded smoke is useful regression
evidence, not exhaustive fuzzing or completed Phase 12 hardening. Production Rust
remains pinned to 1.90.0; the nightly is isolated development tooling. The
committed fuzz lockfile is separate from the production workspace lockfile.
`cargo check` only checks harness compilation; it is not an instrumented fuzz run.

Add encrypted chunk, network response and timelock decoder targets alongside those
implementations after the mandatory spikes, and complete hardening in Phase 12.
Only public/synthetic seeds belong in a corpus; never use creator secrets.
