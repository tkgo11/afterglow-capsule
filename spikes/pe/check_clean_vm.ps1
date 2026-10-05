# SPIKE ONLY. Windows PowerShell 5.1 compatible; no SDK, Rust, Python or admin needed.
# Does not install certificates or change signature/trust policy.
param(
    [string]$ArtifactDirectory = $PSScriptRoot,
    [string]$Report = (Join-Path $PSScriptRoot "spike-c-clean-vm-result.json"),
    [switch]$CleanRecipientVm,
    [switch]$ShellIconVisible
)
$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
$Exe = Join-Path $ArtifactDirectory "SpikeC-Standalone.exe"
$Expected = Get-Content -Raw (Join-Path $ArtifactDirectory "spike-c-expected.json") | ConvertFrom-Json
if ($Expected.format_name -ne "afterglow-spike-c-evidence" -or $Expected.format_version -ne 1 -or $Expected.minimum_reader_version -ne 1) {
    throw "Unsupported evidence metadata"
}
if ((Get-FileHash -Algorithm SHA256 $Exe).Hash -ne $Expected.signed_exe_sha256) { throw "Signed EXE checksum differs" }
$Signature = Get-AuthenticodeSignature $Exe
if (-not $Signature.SignerCertificate) { throw "Embedded test signer was not found" }
# The disposable CI trust root has been removed. Record Windows trust status;
# a self-signed test artifact is not expected to establish publisher trust here.
$Scratch = Join-Path ([IO.Path]::GetTempPath()) ("afterglow-clean-vm-" + [guid]::NewGuid())
$Observed = @{}
try {
    New-Item -ItemType Directory -Path $Scratch | Out-Null
    $Standalone = Join-Path $Scratch "SpikeC-Standalone.exe"
    Copy-Item $Exe $Standalone
    foreach ($Kind in @("capsule", "icon", "group-icon", "version")) {
        $Start = New-Object Diagnostics.ProcessStartInfo
        $Start.FileName = $Standalone
        $Start.WorkingDirectory = $Scratch
        $Start.Arguments = $Kind
        $Start.UseShellExecute = $false
        $Start.RedirectStandardOutput = $true
        $Process = [Diagnostics.Process]::Start($Start)
        $Readback = New-Object IO.MemoryStream
        try {
            $Copy = $Process.StandardOutput.BaseStream.CopyToAsync($Readback)
            if (-not $Process.WaitForExit(30000)) {
                $Process.Kill()
                throw "Standalone reader timed out for $Kind"
            }
            if (-not $Copy.Wait(5000)) { throw "Resource output timed out for $Kind" }
            if ($Process.ExitCode -ne 0) { throw "Standalone reader failed for $Kind" }
            $Hasher = [Security.Cryptography.SHA256]::Create()
            try { $Digest = [BitConverter]::ToString($Hasher.ComputeHash($Readback.ToArray())).Replace("-", "") }
            finally { $Hasher.Dispose() }
            if ($Digest -ne $Expected.resource_sha256.$Kind) { throw "Resource checksum differs for $Kind" }
            $Observed[$Kind] = $Digest
        } finally { $Process.Dispose(); $Readback.Dispose() }
    }
    $Version = [Diagnostics.FileVersionInfo]::GetVersionInfo($Standalone)
    $Numbers = "{0}.{1}.{2}.{3}" -f $Version.FileMajorPart, $Version.FileMinorPart, $Version.FileBuildPart, $Version.FilePrivatePart
    if ($Numbers -ne $Expected.file_version) { throw "Native version metadata differs" }
    if ((Get-FileHash -Algorithm SHA256 $Standalone).Hash -ne $Expected.signed_exe_sha256) { throw "Standalone execution changed the EXE" }
    $Tools = @{}
    foreach ($Name in @("cargo", "rustc", "go", "node", "python", "signtool")) {
        $Tools[$Name] = [bool](Get-Command $Name -ErrorAction SilentlyContinue)
    }
    @{
        format_name = "afterglow-spike-c-clean-vm-result"; format_version = 1; minimum_reader_version = 1;
        windows = [Environment]::OSVersion.VersionString; process_64_bit = [Environment]::Is64BitProcess;
        powershell = $PSVersionTable.PSVersion.ToString(); exe_sha256 = $Expected.signed_exe_sha256;
        resource_sha256 = $Observed; file_version = $Numbers; one_file_execution = "passed";
        authenticode_status = $Signature.Status.ToString(); authenticode_status_message = $Signature.StatusMessage;
        signer_subject = $Signature.SignerCertificate.Subject; available_commands = $Tools;
        clean_recipient_vm_attested = [bool]$CleanRecipientVm; shell_icon_visible_attested = [bool]$ShellIconVisible;
        acceptance_review = "pending human review of VM provenance, Explorer screenshot and CI signing evidence"
    } | ConvertTo-Json -Depth 5 | Set-Content -Encoding UTF8 $Report
    Write-Output "Standalone resource and version checks passed. Evidence written to $Report; human acceptance remains required."
} finally {
    if (Test-Path $Scratch) { Remove-Item $Scratch -Recurse -Force }
}
