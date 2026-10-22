# Phase 2 evidence collection and acceptance

Status: implemented tooling; **Phase 2 PENDING** pending genuine C/D evidence.
Review date: 2026-10-06. SPEC.md remains unchanged and canonical.

## Decision and evidence limits

The isolated GPU probe enumerates adapters and chooses the exact requested
reported class, rather than using a performance preference. Ambiguous physical
adapters fail unless a name filter resolves them; equivalent backend views prefer
DX12 deterministically. Every actual adapter and native per-monitor DPI is
reported and checked. Input driving uses Windows SendInput only in the probe's
own foreground window; only actual winit events increment received counters.
Three 300-interval windows and raw hashed JSONL allow independent consistency
review. CPU acknowledgement/presentation-return timings cannot prove human
perception, so responsiveness/foreground observations stay explicitly nullable.

Clean-recipient collection verifies native resources by executing the EXE alone
in an empty directory, records the environment and genuine VM provenance, and
extracts the actual Windows Shell-returned icon into PNG for human review. A
fresh supported Windows Sandbox is an optional local VM, not a simulated clean
host. Installed tooling, hosted CI, Server or ARM emulation disqualify recipients.
No trust root, host policy or display scale is changed by collection.

The validator is read-only, bounded, duplicate-field rejecting and versioned.
It requires 54 unique selected GPU coordinates with exact adapter/DPI/window/
input/timing/log consistency, completed observations and a qualifying C report.
It also requires independently reviewed A/B CI references and exact C/D source,
workflow, manifest and EXE hashes supplied separately as trusted provenance.
Self-reported collector metadata cannot promote itself into acceptance. Human
observations/environment attestations are evidence subject to review, not
cryptographic proof of hardware identity or human honesty. Temporary synthetic
unit fixtures are never physical acceptance evidence.

Failed/pending attempts are immutable and preserved. Explicit reviewed retry
selection can choose a later valid attempt only while retaining excluded failures
and reasons in the audit output. No automatic latest-result rule hides failures.
No production cryptography, renderer or packager is selected by this tooling.

## Findings resolved or still pending

- Old Spike D HighPerformance selection could select NVIDIA while labeled
  integrated. Explicit class enumeration replaces that behavior without fallback.
- Reported discrete measurements had zero input; those reports cannot pass.
  Raw physical evidence has not been supplied in this worktree and is not invented.
- The old C EXE imported `VCRUNTIME140.dll`. New C/D/validator artifacts use a
  static CRT so a fresh recipient need not install that redistributable.
- Windows scaling has no supported isolated per-session setter here. Microsoft's
  `SPI_SETLOGICALDPIOVERRIDE` documentation explicitly says "Do not use."
  Three normal Display Settings changes remain; every probe verifies actual DPI.
- A hosted developer Windows runner is not a clean recipient. CI records actual
  environment/Sandbox feasibility without installing features or provisioning
  an unreviewed VM. Local supported Sandbox/external clean VM remains required.

## Historical artifact audit

Version 1 bundles from commit `f797ac893c67bfe6e2f36fdc647b3ddd7be9228d`
are **obsolete for new C/D acceptance**, not rewritten as passing runs:

| Bundle | Workflow/artifact         | Downloaded ZIP SHA-256                                           | EXE SHA-256                                                      |
| ------ | ------------------------- | ---------------------------------------------------------------- | ---------------------------------------------------------------- |
| C      | 37292698552 / 11336838628 | 2ffbc1ff31de688152db215997f38bc87f765e1600f97c7481517b6a98bfb0e3 | d997b0727ed7072022a47a3c2c97555191b3e7147aa22888c40790ac3ba2ce93 |
| D      | 37292698552 / 11337491857 | 4540d16539d4a0f33a95bd4264d15cf0e9b846b7de6795be8747611dd13733e6 | 1065bcf4b17733e5fee1f7fe6f9da8bdb15edb406477cd88c2ff55b4d57779bd |

Previously passing native signing and A/B interoperability results remain valid
historical evidence within their stated limits. Old physical runs, if returned,
must be retained separately with their original protocol and failure status.

## Commands and continuation

See the [one-command collection protocol](../../spikes/manual/README.md) and
[strict validator contract](../../spikes/evidence/README.md). Cloud regression,
Windows-native collection smoke, live timelock/NTS, artifact provenance and hash
review must finish before a new artifact is recommended. CI completion alone
never accepts C/D. A genuine reviewed Phase 2 PASS is necessary before starting
Phase 3 and proceeding in SPEC order; pending evidence leaves Phases 3–12 pending.

## Historical reviewed version 2 artifacts and cloud checks

Review date for this result: 2026-10-07 (Asia/Seoul).

[All five mandatory spike CI jobs](https://github.com/tkgo11/afterglow-capsule/actions/runs/37493855142)
passed. Source commit `95c028697cd4dd028deb84c9b27196f9d0e1ead4` produced the
following independently downloaded and reviewed bundles. These are superseded
for new C/D acceptance by the current reviewed bundles below. Both ZIP hashes
matched GitHub's upload digest;
bundle-listed file hashes and actual x64 PE/EXE hashes match. No keys, certificate
files, project sidecars or redistributable DLLs are included. `objdump -p` confirms
C/D/validator imports have no VCRUNTIME/MSVCP redistributable dependency.

| Bundle | Exact artifact                                                                                            | ZIP SHA-256                                                      | EXE SHA-256                                                      |
| ------ | --------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------- | ---------------------------------------------------------------- |
| C      | [11426901800](https://github.com/tkgo11/afterglow-capsule/actions/runs/37493855142/artifacts/11426901800) | e518e60d55fdd786a1ace8c75c030fc05c5c69f916aaadbb03664521796accd1 | 3e63634cb7fed8e31f87dfb4b81bcd0fb19e960c29ce2e97fbd274fd2aa1e17d |
| D      | [11426458174](https://github.com/tkgo11/afterglow-capsule/actions/runs/37493855142/artifacts/11426458174) | 62b4f34d292de2874eecacac971394c336bfcb0d290a979eadcad66467eaf361 | a76b8c0b83fd7a0e6c7ade4203db6b8d0035c76ffb3af370b1c9a022141408c0 |

Artifact expiration is **2026-10-21 in Asia/Seoul** (October 20 UTC). A rebuild
requires new independent provenance review because workflow/manifest hashes and
disposable signing hashes change. Preserve reviewed bundles and returned evidence.

[Workspace CI](https://github.com/tkgo11/afterglow-capsule/actions/runs/37493855079)
passed all four Linux/Windows/bootstrap/frontend/fuzz jobs. The instrumented
30-second parser targets completed 15,487,125 / 3,007,365 / 3,382,201 / 2,837,870
runs respectively; this is bounded smoke, not Phase 12 acceptance. The
[Windows native C job](https://github.com/tkgo11/afterglow-capsule/actions/runs/37493855142/job/112373230770)
passed signing/readback/post-sign rejection/cleanup and actual WinPS 5.1 native
resource/Shell-icon extraction smoke, while correctly reporting hosted cleanliness
FAIL. The [D artifact job](https://github.com/tkgo11/afterglow-capsule/actions/runs/37493855142/job/112373230630)
passed the 39 pure WinPS 5.1 collector regressions and native enumeration; its
actual hosted adapter is Microsoft Basic Render Driver (`Cpu`), closing no cell.

The validator result at the time, later regenerated with the current reviewed
provenance, is **PENDING**:
A/B references pass; the genuine recipient report and physical matrix are absent.
No synthetic fixture or hosted diagnostic was inserted as returned physical
evidence.

The actual hosted feasibility report records Windows Server 2025 Datacenter,
installed developer tooling and no WindowsSandbox.exe. Hyper-V management and
features are present; that alone neither proves working nested client execution
nor supplies a licensed clean client image. The separate
[clean-Windows investigation](0004-clean-windows-cloud.md) records supported
prerequisites without treating Server, containers or development appliances as
recipient acceptance. No cloud VM subscription/credentials/image are configured.

## Current reviewed bundles and hosted checks

Review date: 2026-10-19 (Asia/Seoul). Source commit
`bdc78a81578aca9b7e97629894d58ab8ece8bc4f` passed all 14 hosted jobs:
[workspace](https://github.com/tkgo11/afterglow-capsule/actions/runs/39127563876),
[mandatory spikes](https://github.com/tkgo11/afterglow-capsule/actions/runs/39127564125),
and [cloud feasibility](https://github.com/tkgo11/afterglow-capsule/actions/runs/39127564021).
The four locked Rust graphs passed the advisory scan against RustSec revision
`999ec89d5d3f10f20be14750ec8c2d8b44e0ac9b`, with no reported known
vulnerabilities or scanner errors. The existing informational unmaintained
`proc-macro-error2` warning remains visible in spike and preparation graphs.
The [raw advisory artifact](https://github.com/tkgo11/afterglow-capsule/actions/runs/39127563876/artifacts/12137090355)
has ZIP SHA-256 `c0e6979961f7211a9d6d03fa42bb1e99ed26db2d051f07f7fe2ffefee7157768`.

The Windows A/B job verified future round `33174510`: ciphertext was rejected
before the round and decrypted only after pinned-chain signature verification,
with Go↔Rust compatibility in both directions. Independent Cloudflare and Netnod
NTS receipts authenticated using the patched `rustls 0.23.46` lock. The C job
passed resource replacement, readback, signing, post-sign mutation rejection,
collector regressions and actual no-argument hosted EXE launch; hosted cleanliness
correctly failed. The D job passed native collector and GPU probe build checks;
its hosted adapter does not supply physical matrix evidence.

The C/D bundles below were downloaded independently. Their ZIP SHA-256 values
match GitHub's artifact digests. Every listed file matches its bundle manifest,
the manifests record the exact source commit and workflow run, and the bundles
contain no unexpected files beyond named hosted diagnostics. Parsed PE imports
for both probe EXEs have no `VCRUNTIME` or `MSVCP` redistributable dependency.
Use these exact bundles with [the updated trusted provenance](phase2-trusted-provenance.json)
for new physical collection; newer rebuilds require separate review.

| Bundle | Exact artifact                                                                                            | ZIP SHA-256                                                      | Bundle-files SHA-256                                             | EXE SHA-256                                                      |
| ------ | --------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------- | ---------------------------------------------------------------- | ---------------------------------------------------------------- |
| C      | [12135914286](https://github.com/tkgo11/afterglow-capsule/actions/runs/39127564125/artifacts/12135914286) | 859316acc7645deee65b6d945c38ed01e2121f17011246f73abc49a0c9f52c68 | d982922ee5c8faff7963be3d602be167f5192cc25164973db5e66370a1088dfb | c155d1653e8c125341e29abc3bd32c0ffc994e4ec27d6cd433dffa4cbb614a8e |
| D      | [12136394485](https://github.com/tkgo11/afterglow-capsule/actions/runs/39127564125/artifacts/12136394485) | 9c641ddfa457bb620a8d49e42a22d526f2abf60cad86daba8e3a3aedf5a303ff | b2e7c66deb1a72219f60c0d771ac7ffd3715ec50108d52546dc3c004b4cff79f | c3e56b7d6f2ee7334ac02c96df0aaf5cd68a7bce12f0fbdc703804cd179d9edb |

The trusted document separately pins the expected-evidence manifest hashes:
C `4569cf87cd1c452dbd1c52a30dcaa0b92370261305c0609438c006d5b9738eb7`
and D `15f0c4ca82786d9abd234a284b8d1056cb3f33e5160eeee53347e5ac1105e6ad`.
The bundled Windows validator confirmed A/B and trust PASS, with C/D PENDING
because the genuine returned reports are absent; the exact
[validator output](phase2-validation-status.json) is preserved.

GitHub lists these artifacts as expiring 2026-11-01 UTC. Their collector
smoke results do not close the physical gate. Phase 2 remains **PENDING** until
the genuine clean-recipient report, Shell-icon observation and 54 physical GPU
cells with human observations pass the strict validator.
