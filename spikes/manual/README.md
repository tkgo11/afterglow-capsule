# Phase 2 physical evidence — version 2

Use the complete `phase-2-clean-vm-evidence` and `phase-2-gpu-probe` artifacts
identified in the [evidence decision](https://github.com/tkgo11/afterglow-capsule/blob/codex/phase-1-mandatory-spikes/docs/decisions/0003-phase2-evidence.md).
Save [the reviewed provenance JSON](https://raw.githubusercontent.com/tkgo11/afterglow-capsule/codex/phase-1-mandatory-spikes/docs/decisions/phase2-trusted-provenance.json)
separately as `reviewed-provenance.json`; it anchors the exact reviewed artifacts.
These are isolated experiments using public synthetic content, not a working
Builder or Viewer. No signing keys are distributed. Version 1 bundles are
obsolete for new acceptance evidence; retain their logs/results for audit.

## Running the reviewed bundle on stock Windows PowerShell

First verify the downloaded ZIP against the separately reviewed hash in the
evidence decision. Extract only that reviewed bundle into a new folder. If stock
Windows PowerShell restricts unsigned scripts, use this supported process-only
invocation in that folder (it leaves host/persistent policies and Group Policy
restrictions effective):

```powershell
powershell.exe -NoProfile -ExecutionPolicy RemoteSigned -Command "Get-ChildItem -LiteralPath . -File -Filter '*.ps1' | Unblock-File; & .\run_all_gpu_validation.ps1 -RequireIntelNvidia"
```

`Unblock-File` applies only to the reviewed bundle's local PowerShell files. For
C, replace the invoked script with `launch_clean_windows_sandbox.ps1`, or
`run_clean_recipient_validation.ps1` inside an actual clean VM. No executable,
signature, trust root or machine policy is modified. A corporate policy refusal
remains a refusal; the collector cannot turn it into acceptance.

## D: one command on the physical hybrid-GPU Windows x64 laptop

Extract the GPU bundle, open Windows PowerShell 5.1 in its directory, then run:

```powershell
.\run_all_gpu_validation.ps1 -RequireIntelNvidia
```

This inventories actual adapters, requires Intel `IntegratedGpu` and NVIDIA
`DiscreteGpu`, and collects all **54 cells**: both adapter classes × actual
100%, 150%, 200% DPI × 960×640, 1440×900, 1920×1080 physical client dimensions ×
`full`, `reduced`, `opaque` modes. Every probe receives `--gpu` and
`--expected-dpi`. A missing/ambiguous class, wrong actual adapter or wrong native
DPI fails; OS graphics preferences are supplemental only. `--adapter-name` and
the collector's `-IntegratedAdapterName`/`-DiscreteAdapterName` filters can resolve
multiple adapters of the same class without allowing class fallback.

Three Display Settings scale changes remain manual. Windows awareness APIs do
not change effective display scale, and Microsoft's logical-DPI override is
explicitly unsupported. The script opens Display Settings and waits for the
normal scale change; do not use custom scaling requiring sign-out. Set the primary test display; the probe uses that display. Native `GetDpiForWindow` and per-monitor awareness are checked
in every sample; a label or registry edit cannot substitute for actual DPI.
The primary display must accommodate each requested physical client size. Cells
use borderless windows; a cell matching the whole display uses supported
borderless fullscreen without changing its resolution or exclusive display mode.

Keep each probe in the foreground and watch the visible pointer feedback and
opaque bars. The spike uses standard Windows `SendInput` pointer movement and
F8 key down/up only while its own window is foreground. It does not click,
change OS preferences or inject renderer state. Each cell warms up, records
three 300-frame-interval windows, then exits automatically. Every window must
contain actual received pointer and keyboard events. Do not resize, switch
windows or press Escape during a valid recording. The collector then asks for
one direct observation of input responsiveness and preserved foreground per
cell. Answer pending or failed when appropriate; automation cannot answer this
for you. `-SkipHumanObservations` leaves the required observations null.

Logs record average/p95 presentation intervals, input receipt counts,
SendInput acknowledgement and event-to-present-return CPU timings, actual
adapter/backend/driver, native DPI, dimensions and automatic mode degradation.
These are machine-verifiable application timings, not GPU timestamps, photon
latency or proof of human-perceived responsiveness. Human observation is
mandatory. Slower/jittery windows conservatively reduce effects using the worse
of mean and p95 cadence; opaque preference remains opaque.

Results are self-contained under `results/d/`; raw JSONL/stderr and immutable
`cells/<attempt-id>/cell.json` preserve every attempt. The command emits a final
versioned JSON result (exit 0 PASS, 1 FAIL, 2 PENDING). Without separately reviewed
provenance, even complete local evidence remains PENDING. Preserve the entire
folder, including failures and inventory. `-Resume` avoids repeating unique
machine-passed coordinates; unresolved human observations still need completion.
For a single scale session use `-DpiPercent 100` (or 150/200). The default hybrid
collector requires Intel/NVIDIA; `-GenericHardware` collects the same generic
class matrix on other physical integrated/discrete reference hardware.

To review pending observations or select successful retries explicitly:

```powershell
.\review_gpu_evidence.ps1 -OutputDirectory .\results\d
```

Only confirm an observation if you directly watched the original cell. This
command preserves original report/selection revisions and every raw attempt;
unobserved cells stay pending, and a failed human observation requires a new
attempt rather than rewriting it as success.

If a cell fails, correct the cause and rerun the affected session. Never delete
or alter the original failed cell/raw log to claim success. Multiple attempts
require explicit reviewed `matrix-selection.json` selections/exclusions and
nonempty reasons as described in the [validator contract](../evidence/README.md).
Excluded failures remain visible in the validator's audit output. Old failed
version 1 runs belong in a separately identified audit folder, not the v2 matrix.

## C: one fresh clean Windows x64 VM execution

Use a fresh Windows 10/11 x64 VM without developer prerequisites (Rust, Python,
Node, Go, SDK, Visual Studio/build tools). The complete bundle needs no project
runtime sidecars, SDK, Python, Rust, administrator privilege or installed trust
root. The collector uses Windows built-ins and runs the EXE alone from an empty
temporary directory. Collector inputs are verification data, not Viewer runtime
dependencies. Windows PowerShell 5.1 x64 is supported.

If Windows Sandbox is **already enabled** on a supported Windows client host,
extract the C bundle and run this optional one-command fresh-VM route:

```powershell
.\launch_clean_windows_sandbox.ps1
```

It launches an actual fresh Sandbox VM, disables networking/vGPU/clipboard/
printer redirection, maps allowlisted public inputs read-only and a new results
folder writable, and records VM provenance. It does not enable features or
reboot. Only the disposable guest PowerShell process uses `RemoteSigned`; host
and persistent execution policies and Group Policy restrictions remain effective.
Wait for the collector, then close Sandbox. If unavailable, use a fresh external
Windows VM; the launcher fails clearly rather than treating the host as clean.

In a regular clean VM, extract the bundle and create `vm-provenance.json` with
truthful installation provenance (replace all descriptions):

```json
{
  "format_name": "afterglow-spike-c-vm-provenance",
  "format_version": 2,
  "minimum_reader_version": 2,
  "installation_kind": "clean-windows-vm",
  "clean_recipient_attested": true,
  "origin": "Actual VM creation method and owner",
  "base_image": "Actual Windows image source, edition and build",
  "note": "Fresh installation and how absence of developer tooling was established"
}
```

Then run:

```powershell
.\run_clean_recipient_validation.ps1
```

Do not attest clean when it is not clean. Missing provenance remains pending;
dirty, hosted-CI, Server or emulated-architecture environments fail. The collector
checks expected source/workflow provenance, SHA-256, native AMD64 PE architecture,
embedded RCDATA/icon/group-icon/version exact readback and version 2.0.0.0,
installed developer tooling, Windows edition/build and VM hardware. Bounded EXE
execution and an empty-directory check verify that project sidecars are not needed.

`SHGetFileInfoW` extracts the **actual Shell icon for the actual EXE path**, then
saves the returned native HICON as `shell-icon.png`. It does not substitute the
embedded icon resource or pretend to capture Explorer. Inspect that PNG (and the
Explorer icon if desired) for the cyan square with white diagonal. Fill only
`spike-c-clean-vm-result.json`'s `shell_icon_observation.correct` and `note` with
your actual visual observation. This is the only required human visual check;
a further desktop screenshot is optional. Keep machine fields and raw evidence
unchanged. Prior reports/files are preserved before a rerun.

The disposable CI signer is removed. The public EXE can report an untrusted
certificate-chain status on the recipient, which is recorded without installing
a root. CI separately proves signing, verification and post-sign mutation
rejection. This does not establish production publisher trust or SmartScreen
reputation. A normal hosted developer runner does not qualify as a clean VM.

## Assemble and validate the gate

Keep the reviewed provenance JSON from the repository separately from returned
untrusted evidence. Copy the complete C output into `phase2-evidence/c/` and the
complete `results/d/` folder into `phase2-evidence/d/`. From either bundle run:

```powershell
.\EvidenceValidator.exe validate-phase2-evidence C:\path\phase2-evidence --trusted-provenance C:\path\reviewed-provenance.json
```

The exact reviewed manifest/executable hashes must match those artifacts. Rebuilt
bundles need a new independent CI/provenance review; never edit the trusted file
to match an unreviewed report. Validation does not prove the honesty of human
attestations or replace examination of raw evidence. Incomplete evidence cannot
PASS. Preserve the JSON result and its audit reasons with the evidence.

SPEC.md §148 still gates production Phases 3–12. Supply the entire evidence folder
and validation result to the worktree/repository for review and continuation;
the validator never grants a cryptographic unlock or bypasses the phase gate.
