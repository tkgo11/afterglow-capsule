# Compatibility entry point. Old v1 switch-only attestations/artifacts are obsolete.
# Supply genuine vm-provenance.json; no switch can bypass cleanliness/provenance.
param(
    [string]$ArtifactDirectory = $PSScriptRoot,
    [string]$Report = (Join-Path $ArtifactDirectory 'spike-c-clean-vm-result.json'),
    [switch]$CleanRecipientVm,
    [switch]$ShellIconVisible
)
if ($CleanRecipientVm -or $ShellIconVisible) {
    Write-Warning 'Old attestation switches are ignored. v2 requires actual inventory, vm-provenance.json and recorded shell-icon observation.'
}
& (Join-Path $PSScriptRoot 'run_clean_recipient_validation.ps1') -ArtifactDirectory $ArtifactDirectory -Report $Report
exit $LASTEXITCODE
