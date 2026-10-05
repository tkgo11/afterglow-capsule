# SPIKE ONLY. Run on a disposable clean Windows x64 VM with the Windows SDK.
# Private signing material is nonexportable in CurrentUser/My; the disposable
# public test trust root is installed in LocalMachine/Root and removed afterward.
param([Parameter(Mandatory=$true)][string]$SignTool, [string]$EvidenceDirectory)
$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
if (-not $IsWindows) { throw "Native Windows is required (PowerShell 7)." }
$Principal = [Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $Principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw "Run this isolated signing experiment as Administrator on a disposable VM."
}
Write-Output "Windows=$([Environment]::OSVersion.VersionString); PowerShell=$($PSVersionTable.PSVersion); SignTool=$SignTool; SDK tool version=$([Diagnostics.FileVersionInfo]::GetVersionInfo($SignTool).FileVersion)"
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
        Run-Native "python" @("spikes/pe/fixtures.py", (Join-Path $Scratch "stock"), "--variant", "baseline")
    } finally { Pop-Location }
    $Template = Join-Path $Repo "spikes/target/debug/spike-c-template.exe"
    $Injector = Join-Path $Repo "spikes/target/debug/spike-c-injector.exe"
    $Stock = Join-Path $Scratch "stock"
    $StockTemplate = Join-Path $Scratch "UnsignedTemplate.exe"
    Copy-Item $Template $StockTemplate
    Run-Native $Injector @($StockTemplate, (Join-Path $Stock "capsule.bin"), (Join-Path $Stock "icon.bin"), (Join-Path $Stock "group-icon.bin"), (Join-Path $Stock "version.bin"))
    $TemplateHash = (Get-FileHash -Algorithm SHA256 $StockTemplate).Hash
    $Exe = Join-Path $Scratch "ProjectName.exe"
    Copy-Item $StockTemplate $Exe
    Write-Output "Copied precompiled template; injecting public test resources."
    $Capsule = Join-Path $Scratch "capsule.bin"
    $InjectArgs = @($Exe, $Capsule, (Join-Path $Scratch "icon.bin"), (Join-Path $Scratch "group-icon.bin"), (Join-Path $Scratch "version.bin"))
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
    # Verify stock data in the copied precompiled template, then replace every
    # resource at the same IDs without changing/recompiling the stock template.
    foreach ($Kind in @("capsule", "icon", "group-icon", "version")) { Verify-Resource $Kind (Join-Path $Stock "$Kind.bin") }
    if ([Diagnostics.FileVersionInfo]::GetVersionInfo($Exe).FileMajorPart -ne 1) { throw "Stock version is missing" }
    Run-Native $Injector $InjectArgs
    Verify-Resource "capsule" $Capsule
    foreach ($Kind in @("icon", "group-icon", "version")) { Verify-Resource $Kind (Join-Path $Scratch "$Kind.bin") }
    $Version = [Diagnostics.FileVersionInfo]::GetVersionInfo($Exe)
    if ($Version.FileMajorPart -ne 2 -or $Version.FileMinorPart -ne 0 -or $Version.FileBuildPart -ne 0 -or $Version.FilePrivatePart -ne 0) { throw "Version resource replacement did not apply" }
    if ((Get-FileHash -Algorithm SHA256 $StockTemplate).Hash -ne $TemplateHash) { throw "Project injection modified the stock runtime template" }
    # Icon/group-icon replacement is exercised above; visual shell verification remains required.
    Write-Output "Creating disposable nonexportable test signer."
    $Cert = New-SelfSignedCertificate -Type CodeSigningCert -Subject "CN=AFTERGLOW disposable Spike C" -CertStoreLocation "Cert:\CurrentUser\My" -KeyExportPolicy NonExportable
    $PublicCert = Join-Path $Scratch "signer.cer"
    Export-Certificate -Cert $Cert -FilePath $PublicCert | Out-Null
    # CurrentUser root import displays protected-root UI even with certutil -f.
    # On a disposable elevated VM, LocalMachine import is unattended. Only this
    # ephemeral public test certificate is installed and removed in finally.
    $RootPath = "Cert:\LocalMachine\Root\$($Cert.Thumbprint)"
    Run-Native "certutil" @("-f", "-addstore", "Root", $PublicCert)
    Write-Output "Signing and verifying the copied test EXE."
    Run-Native $SignTool @("sign", "/fd", "SHA256", "/s", "My", "/sha1", $Cert.Thumbprint, $Exe)
    Run-Native $SignTool @("verify", "/pa", "/v", $Exe)
    Verify-Resource "capsule" $Capsule
    foreach ($Kind in @("icon", "group-icon", "version")) { Verify-Resource $Kind (Join-Path $Scratch "$Kind.bin") }
    $SignedHash = (Get-FileHash -Algorithm SHA256 $Exe).Hash
    if ($EvidenceDirectory) {
        New-Item -ItemType Directory -Path $EvidenceDirectory -Force | Out-Null
        Copy-Item $Exe (Join-Path $EvidenceDirectory "SpikeC-Standalone.exe")
        if ((Get-FileHash -Algorithm SHA256 (Join-Path $EvidenceDirectory "SpikeC-Standalone.exe")).Hash -ne $SignedHash) { throw "Evidence copy changed signed bytes" }
        $Resources = @{}
        foreach ($Kind in @("capsule", "icon", "group-icon", "version")) {
            $Resources[$Kind] = (Get-FileHash -Algorithm SHA256 (Join-Path $Scratch "$Kind.bin")).Hash
        }
        @{
            format_name = "afterglow-spike-c-evidence"; format_version = 1; minimum_reader_version = 1;
            signed_exe_sha256 = $SignedHash; resource_sha256 = $Resources; file_version = "2.0.0.0";
            windows = [Environment]::OSVersion.VersionString;
            sdk_tool_version = [Diagnostics.FileVersionInfo]::GetVersionInfo($SignTool).FileVersion
        } | ConvertTo-Json -Depth 5 | Set-Content -Encoding utf8 (Join-Path $EvidenceDirectory "spike-c-expected.json")
    }
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
    if (($RootPath -and (Test-Path $RootPath)) -or ($Cert -and (Test-Path "Cert:\CurrentUser\My\$($Cert.Thumbprint)")) -or (Test-Path $Scratch)) { throw "Disposable signing experiment cleanup was incomplete" }
}
