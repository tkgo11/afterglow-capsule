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

Install `cargo-fuzz` and a nightly Rust toolchain for instrumented runs. The
committed fuzz lockfile is separate from the production workspace lockfile.
`cargo check` only checks harness compilation; it is not an instrumented fuzz run.

Add encrypted chunk, network response and timelock decoder targets alongside those
implementations after the mandatory spikes, and complete hardening in Phase 12.
Only public/synthetic seeds belong in a corpus; never use creator secrets.
