# Phase 2 evidence validator (isolated spike tooling)

This Rust executable reviews evidence; it has no production release or preview
capability. It performs no network requests, changes no evidence, and grants no
cryptographic release capability. The CI bundles contain the Windows x64 binary
as `EvidenceValidator.exe`; local builds use `phase2-evidence`.

```powershell
./EvidenceValidator.exe validate-spike-d ./results/d
./EvidenceValidator.exe validate-spike-c ./results/c
./EvidenceValidator.exe validate-phase2-evidence ./results --trusted-provenance ./reviewed-provenance.json
```

Output is the versioned `afterglow-phase2-validation-result` JSON document. Exit
codes are **0 PASS**, **1 FAIL**, and **2 PENDING**. Missing physical evidence or
human observations remains PENDING. A machine failure, inconsistent/tampered log,
wrong GPU/DPI, invalid numeric field, dirty recipient VM or hash mismatch fails.

All commands require independently reviewed trusted provenance to accept a gate.
A collector's artifact manifest is a consistency input, never an independent
trust anchor. Supplying an arbitrary file under `--trusted-provenance` does not
establish that it was independently reviewed. The reviewer must check original
GitHub workflow source commits, successful jobs, and downloaded artifact hashes.
The trusted document must be kept separately from untrusted returned evidence.

The trusted `afterglow-phase2-trusted-provenance` version 2 document contains
`format_name`, `format_version`, and `minimum_reader_version`, plus:

- `spike_a` and `spike_b`: `status: "PASS"`, `repository`, full `source_commit`,
  `workflow_run_id`, `workflow_run_attempt`, `workflow_path`, `job_id`, exact
  `evidence_url`, and a nonempty `review_note`.
- `spike_c` and `spike_d`: exact `provenance` copied from the independently
  reviewed artifact, `expected_manifest_sha256`, and `exe_sha256`.

`provenance` contains `repository: "tkgo11/afterglow-capsule"`, a full 40-character
source commit, positive numeric workflow run/attempt IDs, and
`workflow_path: ".github/workflows/spikes.yml"`.

The full evidence directory contains `c/` and `d/`. C contains
`spike-c-clean-vm-result.json`, `spike-c-expected.json`, `SpikeC-Standalone.exe`,
the report's referenced VM provenance, and PNG shell-icon evidence. D contains
`spike-d-expected.json`, `SpikeD-Glass.exe`, and immutable
`cells/<attempt-id>/cell.json`, `probe.jsonl`, and `stderr.log` files.

Every selected D cell must have exactly three 300-frame windows, actual matching
GPU class/native DPI, correctly recorded input and event-to-present counters,
finite timing statistics, correct automatic quality changes, hashed raw logs,
and completed human foreground/input observations. Synthetic input receipts and
CPU acknowledgement/submission timings do not prove human perceived response.
The 54-cell matrix must contain 27 integrated and 27 discrete cells.

Repeated attempts are never implicitly resolved by timestamp. Without a
selection document, repeated coordinates remain unresolved. A reviewed
`matrix-selection.json` (`afterglow-spike-d-matrix-selection`, version 2) may
contain `selections: [{"cell_path":"cells/<id>/cell.json"}]` and
`exclusions: [{"cell_path":"cells/<id>/cell.json","reason":"reviewed reason"}]`.
Every preserved attempt must be selected or explicitly excluded; selected
coordinates must be unique. Excluded original failures and their explanations
remain in the result's `audit` array. Old version 1 artifacts are obsolete and
remain PENDING; the tool does not rewrite or delete them.

Parsers reject duplicate JSON fields, traversal, symlinks and unsafe relative
paths. JSON, raw logs and executables have allocation bounds. Unit/regression
tests create only temporary synthetic fixtures; those fixtures are not repository
physical evidence and are never presented as physical-machine test results.
