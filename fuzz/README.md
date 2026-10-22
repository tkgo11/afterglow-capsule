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

The excluded prepared encrypted-object reader now has a separately named
`prepared_encrypted_object` target. It parses borrowing records, exercises AAD and
nonce construction and uses the independent public AES-GCM object vector as a
structural seed. The smoke copies that seed into an ignored mutable corpus; it
never mutates the committed fixture or uses creator files. The fifth target runs
for 30 seconds with a 1 MiB input cap. It is preparation evidence and does not
accept the production crypto phase or complete Phase 12.

Add production network response and timelock decoder targets after their validated
adoption, and complete the broader hardening requirements in Phase 12.
Only public/synthetic seeds belong in a corpus; never use creator secrets.
