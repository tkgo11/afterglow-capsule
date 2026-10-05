# Spike C — PE resources and Authenticode

Status: **BLOCKED — native replacement/signing/readback passed; clean VM/shell outstanding**.
Review date: 2026-10-05.
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

Three public-fixture tests passed (PNG CRC/group-icon references, aligned
`VS_VERSION_INFO`, and distinct stock/project resources with visible opaque icons).
Windows-target compilation/lint checks passed. Those cross-check commands alone
do not prove Windows loader/signature behavior.

The [PowerShell experiment](../../spikes/pe/run_spike.ps1) prepares a copied
precompiled template with stock resources, replaces resources before signing,
uses an isolated disposable test signer, verifies signature and byte-for-byte
resource readback, then requires
an intentionally mutated negative artifact to fail verification. Templates only
read resources; the separate injector owns mutation. Resource-update failures
discard the transaction. No signing key is committed or exported. The public
test certificate exists only in the disposable signing environment.
The optional evidence directory retains an exact signed public test copy before
negative mutation; only the separate scratch artifact is mutated and removed.

[Native Windows evidence](https://github.com/tkgo11/afterglow-capsule/actions/runs/37290773182/job/111700375403)
passed on Windows Server 2025 10.0.26100 x64 with Rust 1.90.0 and Python 3.12.10.
Exact native resource readback passed before/after signing; Windows SDK SignTool
signed and verified with zero errors/warnings. Post-sign resource mutation changed
EXE bytes and made verification fail (`No signature found` after resource update).
The strengthened [replacement/signing/collector run](https://github.com/tkgo11/afterglow-capsule/actions/runs/37292698552/job/111706611179)
passed at commit `f797ac893c67bfe6e2f36fdc647b3ddd7be9228d`. It verified stock
resources in the copy, replaced every resource at the same IDs, changed the
opaque icon and fixed version from 1.0.0.0 to 2.0.0.0, and verified the stock
template's SHA-256 remained unchanged. Signing, exact post-sign reads, mutation
rejection and cleanup assertions passed. Windows SDK directory was 10.0.26100.0;
SignTool reported file version `4.00 (WinBuild.160101.0800)`.

The downloadable [clean-VM bundle](https://github.com/tkgo11/afterglow-capsule/actions/runs/37292698552/artifacts/11336838628)
contains only the unmutated signed public EXE, expected hashes, collector, protocol
and hosted collector smoke report. The ZIP and embedded EXE SHA-256 were checked
after download, and the x64 PE still contains its signature directory. No keys,
certificate files or dependency libraries are included. It expires 2026-10-19;
the workflow can rebuild it. The built-in collector passed on Windows PowerShell
5.1.26100.33438, executing the EXE alone from an empty temporary working directory.
Both human clean-VM and icon attestations remained false in hosted CI.

CurrentUser trust-root import stalled at Windows protected-root UI in the earlier
hosted experiment. On a disposable elevated VM, the revised script installs only
the ephemeral public test certificate in LocalMachine/Root; its nonexportable key
stays in CurrentUser/My. Both stores, the key and scratch files are cleaned up.
This test trust setup is not a production signing policy. After cleanup, the
collector reported `UnknownError`: the certificate chain terminates in an
untrusted root. It preserved that trust result; it did not install a root or
claim trusted publisher identity. The prior SignTool verification supplies the
test's cryptographic signing evidence under its disposable test trust setup.

## Acceptance matrix

| SPEC §23 procedure                                        | Evidence                                                                          |
| --------------------------------------------------------- | --------------------------------------------------------------------------------- |
| Copy precompiled template without recompiling per project | Passed; stock template hash unchanged                                             |
| Insert RCDATA                                             | Native exact readback passed before/after signing                                 |
| Replace icon/version                                      | Native same-ID replacement and exact readback passed; Explorer inspection pending |
| Runtime resource load                                     | Native own-module resource reads passed                                           |
| Sign EXE and verify signature                             | Windows SDK SignTool passed with disposable test trust root                       |
| Capsule readable after signing                            | Exact native readback passed                                                      |
| Post-sign mutation fails verification                     | Verification rejected disposable mutated artifact                                 |
| Clean Windows VM                                          | Pending; hosted developer CI is insufficient                                      |

Record Windows/SDK versions and every native result, visually inspect the shell
icon, and test the one-file copied artifact on a clean recipient Windows VM with
no development tools/sidecars before closing the gate. The
[manual protocol](../../spikes/manual/README.md) explains the downloadable signed
public test copy and built-in Windows collector. **All-steps-pass acceptance
has not been met.** The negative mutation is limited to disposable spike artifacts;
it cannot justify mutation of a signed final Viewer.
