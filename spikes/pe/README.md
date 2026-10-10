# Spike C: PE resources and Authenticode

The separate injector copies public fixture resources into a copied template.
The template only reads its own resources through native Windows APIs. They are
isolated experiments; neither is a production Builder or Viewer dependency.

Run as Administrator on a disposable native Windows x64 environment with PowerShell 7, Rust,
Python and Windows SDK SignTool:

```powershell
./spikes/pe/run_spike.ps1 -SignTool '<Windows SDK x64 signtool.exe path>'
```

The script builds the template once and seeds it with stock public resources,
then copies it and replaces RCDATA/icon/group-icon/version at the same IDs without
recompilation. The stock orange icon and version 1.0.0.0 become a cyan icon and
version 2.0.0.0. It verifies exact native readback, checks
Windows file version metadata, creates a disposable nonexportable test signer,
signs, verifies, and reads the resources again. An isolated negative artifact is
then mutated after signing and must fail signature verification. Test certificates,
keys and scratch files are removed in `finally`. The optional `-EvidenceDirectory`
preserves a byte-identical unmutated signed public test EXE and resource hashes
before the separate negative artifact is modified. It never exports signing keys.

The test installs its disposable public signing certificate in the VM's machine root
store for the experiment; its nonexportable signing key stays in CurrentUser/My.
CurrentUser root import displayed protected-root UI and stalled hosted CI, so the
experiment uses an elevated unattended machine import. Use a disposable VM and ensure cleanup succeeded before
reusing it. No production signing/trust setup is selected by this experiment.

Three pure Python fixture tests validate the resource formats and distinguish
stock/project assets, including visible opaque icon pixels. Windows x64 Rust API
cross-check passed on Linux. Native Windows signing/readback and post-sign mutation
rejection passed in CI, including the strengthened stock-resource replacement run.
Shell icon replacement and clean recipient VM execution remain unvalidated.
Hosted Windows automation does not
replace the mandatory clean-VM and shell checks. See the
[evidence record](../../docs/decisions/spike-c-pe.md) and
[manual validation protocol](../manual/README.md).
