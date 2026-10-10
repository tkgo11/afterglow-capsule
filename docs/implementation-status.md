# Implementation status — 2026-10-10

The repository does not yet meet the final Builder/Viewer product target.
Phases 0–1 are complete. Phase 2 A/B passed; C/D remain PENDING. The user deferred
work requiring their machine or observations. That deferral does not accept a
mandatory spike or waive SPEC §148. SPEC.md is unchanged.

## Phase status

| Phase | Status                 | Actual implementation and remaining work                                                                                                                                                                                                           |
| ----- | ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 0–1   | Complete               | Foundation, versioned schema/project models, bounded capsule header and application scaffolds.                                                                                                                                                     |
| 2     | PENDING                | A timelock and B NTS gates passed. C PE/signing CI passed; clean recipient and shell observation missing. D tooling complete; genuine physical matrix and observations missing.                                                                    |
| 3     | Prepared, not accepted | OS CEK, per-object HKDF, chunked AES-GCM, beneficial compression, authenticated failure and independent vectors. Production integration and acceptance deferred.                                                                                   |
| 4     | Partially prepared     | Exact-round adapter, reviewed network pins, local BLS verification and relay race; official Go differential passed. Production bounded HTTPS transport and future-round wrapper/application integration remain.                                    |
| 5     | Partially prepared     | Versioned portable capsule assembly, digests, encrypted metadata binding, bounded readback and capsule leak checks. Production PE injection/icon/version, frozen assets/build report, final EXE scans, sign-last pipeline and distribution remain. |
| 6     | Prepared, not accepted | Release-owned state engine, encrypted required index, ceremony gates and entry/media navigation. Running Viewer integration remains.                                                                                                               |
| 7     | Partially prepared     | Pure provider/consensus/monotonic time engine. Authenticated NTS metadata API issue, network adapters and Windows integration remain.                                                                                                              |
| 8–9   | Gated                  | Functional native Viewer, accessibility, Temporal Glass/Motion and actual reference GPU acceptance remain. The Phase 2 probe is not the production renderer.                                                                                       |
| 10–11 | Gated                  | Working Builder wizard/editors/preview/build pipeline and advanced features remain. The frontend is still a scaffold.                                                                                                                              |
| 12    | Gated                  | Complete hardening, product failure/release paths, accessibility, clean install and physical performance acceptance remain. Limited parser fuzz/advisory checks are not this phase's completion.                                                   |

## Cloud work prepared

The excluded [preparation workspace](../preparation/README.md) has six small crates.
Neither production application may depend on them while the gate is pending;
bootstrap tests enforce that boundary. No backend, local-clock authorization,
preview unlock or Viewer admin surface was added.

Reviewable commits in [draft PR #1](https://github.com/tkgo11/afterglow-capsule/pull/1):

- `652e036`: bounded object cryptography and independent Python vectors.
- `4243398`: exact-round timelock and official Go CEK differential.
- `c5cbfc1`: portable capsule assembly and metadata validation.
- `d4d6476`: release-owned Viewer state and encrypted-capsule failure tests.
- `800790b`: pure auxiliary time engine and explicit NTS metadata issue.
- `2014be8`: encrypted-record fuzz target, codec bounds and clock separation tests.
- `0150115`: indexed capsule lookup, immutable validated metadata and large shuffled-table regression.

Local checks passed: **48 preparation tests** (14 crypto, 9 timelock, 8 capsule,
7 state, 10 time), formatting and Clippy; official Go/prepared-Rust historical
32-byte CEK checks in both directions and corrupt/truncated ciphertext rejection;
root Rust/frontend checks; six topology and nine advisory-report regressions;
five instrumented parser targets at 30 seconds each. Fuzz evidence keeps raw logs,
counter/run counts and SHA-256 hashes under ignored `fuzz/target/smoke-evidence/`.
These bounded smoke runs do not establish exhaustive fuzzing or product readiness.

At `c5cbfc1`, all 14 hosted jobs passed across the
[workspace](https://github.com/tkgo11/afterglow-capsule/actions/runs/38032622851),
[mandatory spikes](https://github.com/tkgo11/afterglow-capsule/actions/runs/38032622786)
and [cloud feasibility diagnostic](https://github.com/tkgo11/afterglow-capsule/actions/runs/38032622779).
The Windows preparation job's logs confirm the actual prepared adapter's Go
differential passed, rather than inferring compatibility from Spike A alone.
Later source changes require their own CI result; PR #1 records the latest reviewed run.

## Security findings and limits

Four locked Rust graphs were scanned without advisory/platform/severity exclusions
or scanner errors; no known vulnerabilities were reported in the recorded RustSec
snapshot `7eebec69c352c7191b1f13eb95dd510eeca5d1de`. The
[raw reports and summary](decisions/preparation-advisory-evidence/summary.json)
preserve exact lock/report hashes and empty stderr logs. The `proc-macro-error2`
unmaintained warning remains visible in the spike
and preparation age localization chains. It requires maintenance review before
production adoption; changing the validated timelock path requires renewed tests.
This is not a comprehensive security audit.

The [NTS metadata issue](decisions/0006-nts-metadata-boundary.md) remains a software
integration blocker: the evaluated high-level API discards authenticated precision,
root delay/dispersion and leap metadata. RTT alone cannot honestly supply source
uncertainty. No fabricated precision or production adapter was added.

Capsule digests detect corruption, not publisher authenticity. Private objects
require successful AEAD authentication. Secret buffers use zeroization where
practical; codec, allocator, OS and debugger copies are not guaranteed erased.
Final EXE packaging, publisher credentials, SmartScreen reputation and one-file
product distribution have not been established by isolated preparation tests.

The parser review replaced repeated object-table scans with indexed lookups and
kept validated metadata read-only. A 4,096-object shuffled-table test checks exact
readback, missing IDs and isolation from edits to creator-owned metadata clones.

## Deferred physical evidence

The strict validator still returns **PENDING**, with exact reasons: no canonical C
clean-VM report and no D physical matrix. Existing obsolete/failed runs and reviewed
provenance were preserved. Hosted CI with developer tools and its CPU adapter
cannot satisfy these requirements.

The reviewed bundles remain anchored to `95c028697cd4dd028deb84c9b27196f9d0e1ead4`
in [trusted provenance](decisions/phase2-trusted-provenance.json). Newer CI rebuilds
are not automatically trusted substitutes. The reviewed download artifacts expire
October 20 UTC; preserve the public bundles locally, or independently review new
artifacts and provenance before collecting acceptance evidence.

If the deferred tests are resumed, use the complete reviewed bundles and the
[manual protocol](../spikes/manual/README.md). Exact Windows entry points are:

```powershell
# On physical hybrid integrated/discrete hardware, in the reviewed GPU bundle:
.\run_all_gpu_validation.ps1 -RequireIntelNvidia

# On a supported client with Sandbox already enabled, in the reviewed C bundle:
.\launch_clean_windows_sandbox.ps1
# Alternatively, inside a genuinely clean external Windows x64 VM:
.\run_clean_recipient_validation.ps1

# After assembling genuine C/D evidence, with separately reviewed provenance:
.\EvidenceValidator.exe validate-phase2-evidence C:\path\phase2-evidence --trusted-provenance C:\path\reviewed-provenance.json
```

Supported 100%, 150%, 200% Display Settings changes, honest per-cell GPU visual/input
observations and one Shell-icon image observation remain human tasks. The collectors
detect actual native DPI, exact adapter classes and actual received input; null
observations, zero required input or absent sample windows cannot become PASS.

Genuine validated evidence enables the next production integration work. It does
not automatically complete later phases. Those require the implementation and
acceptance tests listed above, in SPEC order. No confirmation is needed for routine
continuation once prerequisites are genuinely satisfied.
