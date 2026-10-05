# SPIKE ONLY. Run on a disposable clean Windows x64 VM with the Windows SDK.
# Test signing material lives only in CurrentUser certificate stores and is removed.
param([Parameter(Mandatory=$true)][string]$SignTool)
$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
if (-not $IsWindows) { throw "Native Windows is required (PowerShell 7)." }
$Repo = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$Scratch = Join-Path ([IO.Path]::GetTempPath()) ("afterglow-spike-c-" + [guid]::NewGuid())
$Cert = $null
$RootPath = $null
function Run-Native([string]$Tool, [string[]]$Arguments) {
    & $Tool @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$Tool failed with exit code $LASTEXITCODE" }
}
try {
    New-Item -ItemType Directory -Path $Scratch | Out-Null
    Push-Location $Repo
    try {
        Run-Native "cargo" @("build", "--manifest-path", "spikes/Cargo.toml", "--locked", "-p", "afterglow-spike-c")
        Run-Native "python" @("spikes/pe/fixtures.py", $Scratch)
    } finally { Pop-Location }
    $Template = Join-Path $Repo "spikes/target/debug/spike-c-template.exe"
    $Injector = Join-Path $Repo "spikes/target/debug/spike-c-injector.exe"
    $Exe = Join-Path $Scratch "ProjectName.exe"
    Copy-Item $Template $Exe
    Write-Output "Copied precompiled template; injecting public test resources."
    $Capsule = Join-Path $Scratch "capsule.bin"
    $InjectArgs = @($Exe, $Capsule, (Join-Path $Scratch "icon.bin"), (Join-Path $Scratch "group-icon.bin"), (Join-Path $Scratch "version.bin"))
    Run-Native $Injector $InjectArgs
    function Verify-Resource([string]$Kind, [string]$ExpectedFile) {
        $Start = [Diagnostics.ProcessStartInfo]::new($Exe)
        $Start.UseShellExecute = $false
        $Start.RedirectStandardOutput = $true
        $Start.Arguments = $Kind
        $Process = [Diagnostics.Process]::Start($Start)
        $Readback = [IO.MemoryStream]::new()
        $Copy = $Process.StandardOutput.BaseStream.CopyToAsync($Readback)
        if (-not $Process.WaitForExit(30000)) {
            $Process.Kill($true)
            throw "Native resource reader timed out for $Kind"
        }
        if (-not $Copy.Wait(5000)) { throw "Resource stdout copy timed out for $Kind" }
        if ($Process.ExitCode -ne 0) { throw "Native resource reader failed" }
        $Expected = [IO.File]::ReadAllBytes($ExpectedFile)
        if ([Convert]::ToBase64String($Readback.ToArray()) -ne [Convert]::ToBase64String($Expected)) { throw "RCDATA readback differs" }
        $Process.Dispose()
        $Readback.Dispose()
        Write-Output "Verified native $Kind resource readback."
    }
    Verify-Resource "capsule" $Capsule
    foreach ($Kind in @("icon", "group-icon", "version")) { Verify-Resource $Kind (Join-Path $Scratch "$Kind.bin") }
    $Version = [Diagnostics.FileVersionInfo]::GetVersionInfo($Exe)
    if ($Version.FileMajorPart -ne 1 -or $Version.FileMinorPart -ne 0 -or $Version.FileBuildPart -ne 0 -or $Version.FilePrivatePart -ne 0) { throw "Version resource did not apply" }
    # Icon/group-icon insertion is exercised above; visual shell verification remains required.
    Write-Output "Creating disposable nonexportable test signer."
    $Cert = New-SelfSignedCertificate -Type CodeSigningCert -Subject "CN=AFTERGLOW disposable Spike C" -CertStoreLocation "Cert:\CurrentUser\My" -KeyExportPolicy NonExportable
    $PublicCert = Join-Path $Scratch "signer.cer"
    Export-Certificate -Cert $Cert -FilePath $PublicCert | Out-Null
    # Use Windows' unattended import for this disposable public certificate.
    # Interactive root-store UI cannot be serviced on a hosted CI desktop.
    $RootPath = "Cert:\CurrentUser\Root\$($Cert.Thumbprint)"
    Run-Native "certutil" @("-user", "-f", "-addstore", "Root", $PublicCert)
    Write-Output "Signing and verifying the copied test EXE."
    Run-Native $SignTool @("sign", "/fd", "SHA256", "/s", "My", "/sha1", $Cert.Thumbprint, $Exe)
    Run-Native $SignTool @("verify", "/pa", "/v", $Exe)
    Verify-Resource "capsule" $Capsule
    foreach ($Kind in @("icon", "group-icon", "version")) { Verify-Resource $Kind (Join-Path $Scratch "$Kind.bin") }
    $SignedHash = (Get-FileHash -Algorithm SHA256 $Exe).Hash
    # Negative test only: this mutated test EXE is never a final/distributed artifact.
    [IO.File]::WriteAllBytes($Capsule, [Text.Encoding]::UTF8.GetBytes("changed public test resource"))
    Run-Native $Injector $InjectArgs
    & $SignTool verify /pa /v $Exe
    if ($LASTEXITCODE -eq 0) { throw "Post-sign resource mutation incorrectly verified" }
    if ((Get-FileHash -Algorithm SHA256 $Exe).Hash -eq $SignedHash) { throw "Negative mutation did not change EXE bytes" }
    # The expected negative native exit was asserted above; do not propagate it
    # through GitHub's PowerShell wrapper as a failure of the successful test.
    $global:LASTEXITCODE = 0
    Write-Output "SPIKE C automation passed; record Windows/SDK versions and verify shell icon/clean-VM launch before closing the gate."
} finally {
    if ($RootPath -and (Test-Path $RootPath)) { Remove-Item $RootPath }
    if ($Cert) { Remove-Item "Cert:\CurrentUser\My\$($Cert.Thumbprint)" -DeleteKey }
    if (Test-Path $Scratch) { Remove-Item $Scratch -Recurse -Force }
}
