# Spike C: PE resources and Authenticode

The separate injector copies public fixture resources into a copied template.
The template only reads its own resources through native Windows APIs. They are
isolated experiments; neither is a production Builder or Viewer dependency.

Run on a disposable native Windows x64 environment with PowerShell 7, Rust,
Python and Windows SDK SignTool:

```powershell
./spikes/pe/run_spike.ps1 -SignTool '<Windows SDK x64 signtool.exe path>'
```

The script builds the template once, copies it, injects RCDATA/icon/group-icon/
version resources without recompilation, verifies exact native readback, checks
Windows file version metadata, creates a disposable nonexportable test signer,
signs, verifies, and reads the resources again. An isolated negative artifact is
then mutated after signing and must fail signature verification. Test certificates,
keys and files are removed in `finally`; no final EXE is distributed by the script.

The test installs its disposable public signing certificate in the user's root
store for the experiment. Use a disposable VM and ensure cleanup succeeded before
reusing it. No production signing/trust setup is selected by this experiment.

The two pure Python fixture tests and Windows x64 Rust API cross-check passed on
Linux. Native PE execution, SignTool behavior, shell icon replacement and clean
recipient VM execution remain unvalidated. In particular, Linux's NSS `signtool`
is a JAR tool and cannot validate Authenticode. Hosted Windows automation does not
replace the mandatory clean-VM and shell checks. See the
[evidence record](../../docs/decisions/spike-c-pe.md).
