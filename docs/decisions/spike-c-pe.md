# Spike C — PE resources and Authenticode

Status: **BLOCKED — cross-check and fixtures only**. Review date: 2026-10-05.
Canonical acceptance: SPEC.md §23 and §148.

## Evaluated stack and environment

Linux x64, Rust 1.90.0 with installed `x86_64-pc-windows-msvc` target, Python
3.12.14. The isolated injector/template use exact `windows-sys 0.61.2` native
resource APIs. License/upstream/security status are in [dependencies](../dependencies.md).
No production PE packager or runtime-resource loader is selected yet.

## Commands and observed results

```sh
python spikes/pe/test_fixtures.py
cargo check --manifest-path spikes/Cargo.toml --locked -p afterglow-spike-c --target x86_64-pc-windows-msvc
cargo clippy --manifest-path spikes/Cargo.toml --locked -p afterglow-spike-c --all-targets --target x86_64-pc-windows-msvc -- -D warnings
```

Two public-fixture tests passed (PNG CRC/group-icon references and aligned
`VS_VERSION_INFO`). Windows-target compilation/lint checks passed. These commands
do not execute native APIs or prove Windows loader/signature behavior. This host
has no native Windows environment; its NSS `signtool` signs JARs, not PE files.

The [PowerShell experiment](../../spikes/pe/run_spike.ps1) prepares a copied
precompiled template, adds resources before signing, uses an isolated disposable
test signer, verifies signature and byte-for-byte resource readback, then requires
an intentionally mutated negative artifact to fail verification. Templates only
read resources; the separate injector owns mutation. Resource-update failures
discard the transaction. No signing key or certificate is committed and no
signed artifact is distributed by this test.

## Acceptance matrix

| SPEC §23 procedure                                        | Evidence                                                               |
| --------------------------------------------------------- | ---------------------------------------------------------------------- |
| Copy precompiled template without recompiling per project | Script prepared; native run pending                                    |
| Insert RCDATA                                             | Native API cross-check passed; execution pending                       |
| Replace icon/version                                      | Fixtures validated; resource/readback/version and shell checks pending |
| Runtime resource load                                     | Reader cross-check passed; execution pending                           |
| Sign EXE and verify signature                             | Windows SDK SignTool run pending                                       |
| Capsule readable after signing                            | Native readback pending                                                |
| Post-sign mutation fails verification                     | Isolated negative test prepared; native result pending                 |
| Clean Windows VM                                          | Pending; hosted developer CI is insufficient                           |

Record Windows/SDK versions and every native result, visually inspect the shell
icon, and test the one-file copied artifact on a clean recipient Windows VM with
no development tools/sidecars before closing the gate. **All-steps-pass acceptance
has not been met.** The negative mutation is limited to disposable spike artifacts;
it cannot justify mutation of a signed final Viewer.
