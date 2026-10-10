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

## Reviewed version 2 artifacts and cloud checks

Review date for this result: 2026-10-07 (Asia/Seoul).

[All five mandatory spike CI jobs](https://github.com/tkgo11/afterglow-capsule/actions/runs/37493855142)
passed. Source commit `95c028697cd4dd028deb84c9b27196f9d0e1ead4` produced the
following independently downloaded and reviewed bundles. Use these exact
artifacts with [the reviewed trusted provenance](phase2-trusted-provenance.json),
not an arbitrary later rebuild. Both ZIP hashes match GitHub's upload digest;
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

The [current validator result](phase2-validation-status.json), generated using
the exact downloaded bundles and separately reviewed provenance, is **PENDING**:
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
