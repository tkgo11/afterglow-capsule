# SPIKE ONLY: optional fresh Windows Sandbox VM. Does not enable features or reboot.
# Requires an already enabled supported Windows 10/11 Pro/Enterprise Sandbox.
# This launcher never emits PASS; only the collector inside the actual VM can.
param(
    [string]$ArtifactDirectory = $PSScriptRoot,
    [string]$ResultsDirectory = (Join-Path $PSScriptRoot ('sandbox-results-' + [guid]::NewGuid()))
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
. (Join-Path $PSScriptRoot 'collector_logic.ps1')
$Os = Get-CimInstance Win32_OperatingSystem
if ($Os.ProductType -ne 1 -or $Os.Caption -notmatch 'Windows (10|11)\b' -or -not [Environment]::Is64BitProcess) { throw 'Windows 10/11 x64 client is required.' }
$Sandbox = Join-Path $env:WINDIR 'System32\WindowsSandbox.exe'
$Feature = Get-CimInstance Win32_OptionalFeature -Filter "Name='Containers-DisposableClientVM'"
if (-not (Test-Path -LiteralPath $Sandbox) -or -not $Feature -or $Feature.InstallState -ne 1) {
    throw 'Windows Sandbox is unavailable/not enabled. Use a fresh external Windows VM; this script will not install features or change host settings.'
}
$ArtifactDirectory = [IO.Path]::GetFullPath($ArtifactDirectory)
$ResultsDirectory = [IO.Path]::GetFullPath($ResultsDirectory)
if (Test-Path -LiteralPath $ResultsDirectory) { throw 'ResultsDirectory must be a new path so prior evidence is preserved.' }
New-Item -ItemType Directory -Path $ResultsDirectory | Out-Null
$Input = Join-Path ([IO.Path]::GetTempPath()) ('afterglow-sandbox-input-' + [guid]::NewGuid())
New-Item -ItemType Directory -Path $Input | Out-Null
try {
    # Only public fixture executable and exact collector verification inputs.
    foreach ($Name in @('SpikeC-Standalone.exe','spike-c-expected.json','run_clean_recipient_validation.ps1','collector_logic.ps1','native_collector.cs')) {
        # Stage only the allowlisted reviewed public bundle as fresh local bytes.
        # Original downloaded files/host trust annotations are left unchanged.
        # This avoids carrying Internet-zone ADS into the disposable guest's
        # RemoteSigned process. No executable byte/signature is changed.
        [IO.File]::WriteAllBytes((Join-Path $Input $Name), [IO.File]::ReadAllBytes((Join-Path $ArtifactDirectory $Name)))
    }
    @{
        format_name='afterglow-spike-c-vm-provenance';format_version=2;minimum_reader_version=2;
        installation_kind='clean-windows-vm';clean_recipient_attested=$true;
        origin='Microsoft Windows Sandbox fresh disposable VM via WindowsSandbox.exe';
        base_image=('Supported Windows Sandbox base image generated from Windows client host '+$Os.Version+' build '+$Os.BuildNumber);
        note='Fresh supported Windows Sandbox launch. Network/vGPU/clipboard/printer redirection disabled. Only public fixture and collector inputs mapped read-only; results directory mapped writable. Attestation requires actual inside-VM collector inventory; launcher itself cannot pass the gate.';
        host_os_caption=$Os.Caption;host_os_build=$Os.BuildNumber;launched_utc=[DateTime]::UtcNow.ToString('o')
    } | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $Input 'vm-provenance.json') -Encoding UTF8
    $Configuration = New-SpikeCSandboxConfiguration $Input $ResultsDirectory
    $Wsb = Join-Path $Input 'afterglow-clean-recipient.wsb'
    [IO.File]::WriteAllText($Wsb,$Configuration,[Text.Encoding]::UTF8)
    Copy-Item -LiteralPath $Wsb -Destination (Join-Path $ResultsDirectory 'sandbox-configuration.wsb')
    Write-Output 'Launching actual fresh Windows Sandbox; no host display/feature/trust settings are changed.'
    Write-Output 'Only the disposable guest collector process uses supported RemoteSigned execution policy. Host/persistent policies and Group Policy restrictions are unchanged.'
    Write-Output ('Results: '+$ResultsDirectory)
    Write-Output 'Wait for collector output, then close Sandbox. Inspect shell-icon.png and record the visual observation in the canonical report.'
    $Process = Start-Process -FilePath $Sandbox -ArgumentList ('"'+$Wsb+'"') -PassThru
    $Process.WaitForExit()
    $Report = Join-Path $ResultsDirectory 'spike-c-clean-vm-result.json'
    if (-not (Test-Path -LiteralPath $Report)) { throw 'Sandbox did not produce a canonical collector report; no clean-VM evidence exists.' }
    $Observed = Get-Content -LiteralPath $Report -Raw | ConvertFrom-Json
    Write-Output ($Observed.machine_status+': actual Sandbox machine checks; visual observation/Phase 2 validator remain required.')
    foreach ($Reason in $Observed.failures) { Write-Output ('FAIL: '+$Reason) }
    if ($Observed.machine_status -ne 'PASS') { exit 1 }
} finally {
    Remove-Item -LiteralPath $Input -Recurse -Force
}
