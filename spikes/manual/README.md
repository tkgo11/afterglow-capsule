# Phase 2 manual validation

Download the two artifacts from the latest passing **Mandatory spike evidence**
workflow run on the draft PR. These executables are isolated experiments with
public synthetic data; they are not a working AFTERGLOW Builder or Viewer.
Preserve the run URL and commit with your results. No signing key or certificate
is included. The signing experiment deletes its ephemeral signer and trust root.

## C: clean recipient Windows x64 VM

Use a fresh Windows x64 VM without Rust, Python, SDK, build outputs or resource
sidecars. Extract `phase-2-clean-vm-evidence`. In Explorer, check that
`SpikeC-Standalone.exe` has the cyan square/white diagonal icon, and that its
Properties version is 2.0.0.0. Save a screenshot and record Windows edition/build,
VM source and the absence of developer prerequisites.

Use Windows PowerShell (5.1 supported):

```powershell
cd '<extracted phase-2-clean-vm-evidence folder>'
./check_clean_vm.ps1 -CleanRecipientVm -ShellIconVisible
```

Pass the two switches only after confirming those observations. If local script
policy requires it, review the script and use `Unblock-File ./check_clean_vm.ps1`;
the collector does not change execution or signature policy. It copies **only the
EXE** into an empty temporary working directory and checks all four embedded
resources and native version metadata using Windows built-ins. Expected hashes
are verification inputs, not runtime sidecar dependencies. No admin is needed.

The public test EXE is self-signed with the disposable CI certificate. Windows
may report `NotTrusted` after that root is removed; the collector records the
status and never installs a root. Cryptographic signing, verification and
post-sign mutation rejection are separately evidenced in the CI SignTool log.
This does not establish production publisher trust or SmartScreen reputation.

Return `spike-c-clean-vm-result.json`, screenshot, VM provenance and run URL. A
successful collector on a development machine does not close the clean-VM gate.

## D: physical integrated and discrete GPUs

Extract `phase-2-gpu-probe` on representative physical Windows x64 hardware. The
required matrix has 54 cells: 3 resolutions × 3 actual DPI scales × 2 GPU classes
× 3 modes. Use each GPU class at Windows display scaling 100%, 150% and 200%.
Set the OS scale before each session; the command's label does not change DPI.
On hybrid systems choose the adapter through Windows graphics preferences and
confirm the actual adapter/type in every log. Software adapters cannot pass.

For each of the six GPU/DPI combinations:

```powershell
cd '<extracted phase-2-gpu-probe folder>'
./collect_matrix.ps1 -GpuClass integrated -DpiPercent 100
```

This opens the nine resolution/mode combinations in sequence. `full` is normal
glass, `opaque` is Reduced Transparency, and `reduced` is low quality. Keep the
probe focused; warm up, move the pointer, collect at least three 300-frame sample
windows, then press Escape. The live log is redirected to `results`; inspect it
from another window without keeping the probe unfocused. Alternatively run an
individual cell with `./SpikeD-Glass.exe 1440 900 full` for live console output.
Do not change modes or resize during a recorded cell; rerun any mismatched cell.

Record GPU model/type, driver, Windows build, display resolution/refresh, DPI,
power settings and any errors. A display must be large enough to accommodate the
requested physical client area: verify actual dimensions and DPI in the logs.
Document input responsiveness, stable foreground/pointer visibility, average/p95
frame intervals and observed automatic quality changes. The target is 60 FPS or
automatic degradation with responsive input; exit code zero alone is insufficient.
Presentation intervals are not GPU timestamps or measured input latency.

Fill the human observation fields in each `observations.json` and return the six
result folders and hardware notes. Preserve failures and incomplete cells; the
collector leaves acceptance pending and never manufactures passing results.

Production Phases 3–12 remain blocked by SPEC.md §148 until all C/D evidence has
been reviewed and accepted. These collectors cannot bypass the phase gates.
