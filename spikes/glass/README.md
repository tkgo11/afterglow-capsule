# Spike D: explicit-adapter Temporal Glass probe

Isolated winit/wgpu public synthetic scene; no production Viewer, project content,
release engine or debug unlock exists here. It renders atmosphere, sampled glass,
opaque foreground bars and pointer feedback. This tests platform feasibility;
archive readability and the full design system remain production phase work.

```sh
cargo test --manifest-path spikes/Cargo.toml --locked -p afterglow-spike-d
cargo run --manifest-path spikes/Cargo.toml --locked --release -p afterglow-spike-d -- --list-adapters
```

Windows example for one bounded physical cell:

```powershell
.\SpikeD-Glass.exe 1440 900 full --gpu integrated --expected-dpi 100 --drive-input
```

`--gpu integrated|discrete` requires the reported IntegratedGpu/DiscreteGpu class;
there is no fallback. Optional `--adapter-name <substring>` resolves ambiguity.
`--expected-dpi 100|150|200` must match native per-monitor window DPI. Defaults are
three 300-interval sample windows, three-second warmup and 120-second timeout.
The JSONL v2 protocol includes actual adapter name/vendor/device/backend/driver/
info/type, native DPI, dimensions, mean/p95 cadence and input/acknowledgement/
event-to-present-return timings. Unknown/software adapters cannot count. Focus
loss, resize, wrong DPI, missing received input or timeout fail a recorded cell.

Standard Windows SendInput drives pointer/F8 events while the own window is
foreground; counters come from received winit events. Human responsiveness and
foreground observations are separately required. Slower/jittery cadence
conservatively degrades effects using max(mean,p95); explicit opaque preference
remains opaque. Cadence and CPU timings are not GPU timestamps or photon latency.

Use [the complete manual protocol](../manual/README.md): one command collects the
54-cell matrix and preserves all attempts. Native hardware/visual evidence remains
pending until actually returned and reviewed. See the
[spike decision](../../docs/decisions/spike-d-glass.md) and
[validator](../evidence/README.md); compilation/unit tests close no physical cell.
