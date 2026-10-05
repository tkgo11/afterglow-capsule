# Spike D — wgpu Temporal Glass

Status: **BLOCKED — shader/policy tests and cross-check only**.
Review date: 2026-10-06. Canonical acceptance: SPEC.md §24 and §148.

## Evaluated stack and environment

Linux x64, Rust 1.90.0. Isolated `wgpu 27.0.1`, `winit 0.30.13`, `pollster 0.4.0`
and test-only `naga 27.0.3`, with exact lock/checksums. Windows x64 API cross-check
passes. These versions support the pinned Rust toolchain; production renderer
selection, performance and driver compatibility remain unvalidated. See
[dependencies](../dependencies.md) for licenses/upstreams/security status.

## Commands and observed results

```sh
cargo test --manifest-path spikes/Cargo.toml --locked -p afterglow-spike-d
cargo check --manifest-path spikes/Cargo.toml --locked -p afterglow-spike-d --target x86_64-pc-windows-msvc
```

Three tests passed: complete WGSL parse/validation, monotonic degradation under
slow/invalid samples, and explicit opaque preference retention. The ready
[windowed experiment](../../spikes/glass/README.md) has separate sampled atmosphere
and glass passes, a stable opaque synthetic foreground and pointer feedback,
bounded cadence sample windows, resize/focus handling and automatic fallback.
The scene does not bind its own render target as an input. Animation uses elapsed
time. No project content or release engine is present.

The same three tests and the full isolated-workspace lint check also passed on
Windows Server 2025 x64 with Rust 1.90.0:
[Windows CI evidence](https://github.com/tkgo11/afterglow-capsule/actions/runs/37288622769/job/111693472911).
These unit/validation results execute without creating a GPU window and do not
close any physical reference-hardware matrix cell.

Shader/policy tests do not establish real GPU performance, visual fidelity,
readable archive text, measured input latency or Windows window behavior. This
workspace exposes no physical GPU device or windowed reference Windows hardware.
Software/unknown adapters are rejected as reference evidence by the probe.

## Required matrix and acceptance

| Axis                 | Required cells                                            | Observed evidence                                                       |
| -------------------- | --------------------------------------------------------- | ----------------------------------------------------------------------- |
| Physical client size | 960×640; 1440×900; 1920×1080                              | Pending                                                                 |
| Actual DPI scaling   | 100%; 150%; 200%                                          | Pending                                                                 |
| Reference adapter    | Integrated GPU; discrete GPU                              | Pending                                                                 |
| Effects              | Full; Reduced Transparency; low-quality fallback          | Pending                                                                 |
| Input                | Responsive pointer/keyboard throughout and after fallback | Pending hardware observation                                            |
| Cadence              | 60 FPS achievable or effects degrade automatically        | Policy tested; hardware result pending                                  |
| Meaning              | Essential foreground independent of glass                 | Synthetic foreground present in every mode; hardware inspection pending |

All 54 resolution/DPI/GPU/effect cells remain pending. Use the protocol in the
probe README; preserve actual adapter/driver/OS/DPI/size, frame interval average/
p95, mode transitions and interaction observations. The log measures application
presentation cadence, not GPU timestamps. Do not fabricate FPS or count a software
container render as reference hardware. **SPEC acceptance remains unresolved.**

The workflow builds a native Windows x64 release probe as the `phase-2-gpu-probe`
artifact. The [published bundle](https://github.com/tkgo11/afterglow-capsule/actions/runs/37292698552/artifacts/11337491857)
was built at `f797ac893c67bfe6e2f36fdc647b3ddd7be9228d`; its downloaded ZIP and
x64 EXE SHA-256 were verified. It contains only the probe, expected hash,
collector and protocol, and expires 2026-10-19. PowerShell 5.1 parsed the collector
successfully; no window was run in hosted CI. The [manual protocol](../../spikes/manual/README.md) and PowerShell
collector cover nine cells at one actual DPI/GPU combination, repeated across
all six combinations. Collection leaves human performance, responsiveness and
foreground observations pending. Artifact compilation closes no hardware cell.

## Version 2 evidence tooling

The original version 1 bundle above is obsolete for new acceptance evidence;
its historical results remain available for audit. See the
[collection/validation decision](0003-phase2-evidence.md) for explicit adapter
selection, real input receipts, static CRT builds, native Shell extraction,
clean-recipient inventory, strict trusted provenance and preserved retries.
The [new one-command protocol](../../spikes/manual/README.md) supersedes the
old collector instructions. Physical C/D acceptance remains PENDING; automated
regression fixtures and hosted success do not count as physical evidence.
