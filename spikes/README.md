# Mandatory Phase 2 experiments

These executables are excluded from the production Cargo workspace and are not
dependencies of Builder or Viewer. They use public synthetic inputs, historical
public beacons and disposable test artifacts. They have no release-state, capsule
CEK, recovery or preview capability. A passing subset does not close Phase 2.

Use Rust 1.90.0 and the committed separate Cargo lock. The Go reference uses
Go 1.27.1 and its committed module lock.

```sh
cargo fmt --manifest-path spikes/Cargo.toml --all -- --check
cargo test --manifest-path spikes/Cargo.toml --locked --workspace
cargo clippy --manifest-path spikes/Cargo.toml --locked --all-targets -- -D warnings
python spikes/timelock/build_and_check.py
python spikes/timelock/test_live_inputs.py
python spikes/pe/test_fixtures.py
```

The [spike workflow](../.github/workflows/spikes.yml) collects partial Linux and
Windows evidence on pushes, pull requests and manual dispatch. It includes real
network tests and will fail if external services cannot be reached or verified.
It does not provision reference GPUs or substitute a hosted runner for a clean
recipient VM. Results must be reviewed against every SPEC acceptance criterion.

| Experiment               | Commands and limitations       | Gate evidence                                     |
| ------------------------ | ------------------------------ | ------------------------------------------------- |
| A — Go/Rust timelock     | [timelock](timelock/README.md) | [decision](../docs/decisions/spike-a-timelock.md) |
| B — Windows NTS          | [NTS](nts/README.md)           | [decision](../docs/decisions/spike-b-nts.md)      |
| C — PE injection/signing | [PE](pe/README.md)             | [decision](../docs/decisions/spike-c-pe.md)       |
| D — Temporal Glass       | [glass](glass/README.md)       | [decision](../docs/decisions/spike-d-glass.md)    |
