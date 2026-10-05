# CI ONLY. Attach reproducible source/workflow provenance to public spike bundles.
param(
    [Parameter(Mandatory=$true)][ValidateSet("c", "d")][string]$Kind,
    [Parameter(Mandatory=$true)][string]$EvidenceDirectory
)
$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
$Source = (& git rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or $Source -notmatch '^[a-f0-9]{40}$') { throw "Cannot identify checked-out source" }
if ($env:GITHUB_REPOSITORY -ne "tkgo11/afterglow-capsule" -or $env:GITHUB_RUN_ID -notmatch '^[0-9]+$' -or $env:GITHUB_RUN_ATTEMPT -notmatch '^[0-9]+$') { throw "GitHub workflow provenance is missing" }
if ($env:GITHUB_WORKFLOW_REF -notlike "tkgo11/afterglow-capsule/.github/workflows/spikes.yml@*") { throw "Unexpected bundle workflow" }
$Provenance = @{
    repository = $env:GITHUB_REPOSITORY; source_commit = $Source;
    workflow_run_id = [long]$env:GITHUB_RUN_ID; workflow_run_attempt = [int]$env:GITHUB_RUN_ATTEMPT;
    workflow_path = ".github/workflows/spikes.yml"
}
Copy-Item spikes/target/release/phase2-evidence.exe "$EvidenceDirectory/EvidenceValidator.exe"
Copy-Item spikes/manual/README.md "$EvidenceDirectory/README.md"
if ($Kind -eq "c") {
    $ExpectedPath = Join-Path $EvidenceDirectory "spike-c-expected.json"
    $Expected = Get-Content -Raw $ExpectedPath | ConvertFrom-Json
    $Expected.format_version = 2
    $Expected.minimum_reader_version = 2
    $Expected | Add-Member -NotePropertyName provenance -NotePropertyValue $Provenance
    $Expected | ConvertTo-Json -Depth 8 | Set-Content -Encoding UTF8 $ExpectedPath
    foreach ($Name in @("check_clean_vm.ps1", "run_clean_recipient_validation.ps1", "launch_clean_windows_sandbox.ps1", "collector_logic.ps1", "native_collector.cs")) {
        Copy-Item "spikes/pe/$Name" $EvidenceDirectory
    }
} else {
    Copy-Item spikes/target/release/afterglow-spike-d.exe "$EvidenceDirectory/SpikeD-Glass.exe"
    foreach ($Name in @("collect_matrix.ps1", "run_all_gpu_validation.ps1", "review_gpu_evidence.ps1", "collector_common.ps1", "test_collectors.ps1")) { Copy-Item "spikes/glass/$Name" $EvidenceDirectory }
    @{
        format_name = "afterglow-spike-d-evidence"; format_version = 2; minimum_reader_version = 2;
        exe_sha256 = (Get-FileHash -Algorithm SHA256 "$EvidenceDirectory/SpikeD-Glass.exe").Hash;
        provenance = $Provenance; windows = [Environment]::OSVersion.VersionString;
        protocol = "explicit-adapter-received-input-v2";
        obsolete_protocols = @(1)
    } | ConvertTo-Json -Depth 8 | Set-Content -Encoding UTF8 "$EvidenceDirectory/spike-d-expected.json"
}
$Files = @{}
foreach ($File in Get-ChildItem -File $EvidenceDirectory) { $Files[$File.Name] = (Get-FileHash -Algorithm SHA256 $File.FullName).Hash }
@{
    format_name = "afterglow-spike-bundle-files"; format_version = 1; minimum_reader_version = 1;
    provenance = $Provenance; files_sha256 = $Files
} | ConvertTo-Json -Depth 8 | Set-Content -Encoding UTF8 "$EvidenceDirectory/bundle-files.json"
