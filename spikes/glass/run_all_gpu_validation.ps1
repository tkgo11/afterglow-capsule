# SPIKE ONLY. Physical GPU evidence; does not alter Windows scaling or GPU preferences.
# Supported awareness/GetDpiForWindow APIs measure per-window effective DPI; they do
# not provide a supported, isolated, reversible per-session global scaling setter.
# Use Display Settings, then prove actual DPI inside every probe; never registry hacks.
param(
    [string]$ArtifactDirectory = $PSScriptRoot,
    [string]$OutputDirectory = (Join-Path $PSScriptRoot 'results/d'),
    [ValidateSet(0, 100, 150, 200)][int]$DpiPercent = 0,
    [ValidatePattern('^[^"\\\r\n]*$')][string]$IntegratedAdapterName,
    [ValidatePattern('^[^"\\\r\n]*$')][string]$DiscreteAdapterName,
    [string]$PowerNote = '',
    [switch]$SkipHumanObservations,
    [switch]$RequireIntelNvidia,
    [switch]$Resume
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
. (Join-Path $PSScriptRoot 'collector_common.ps1')
if ($env:OS -ne 'Windows_NT' -or -not [Environment]::Is64BitOperatingSystem) { throw 'Physical Windows x64 required' }
$Exe = Join-Path $ArtifactDirectory 'SpikeD-Glass.exe'
$Expected = Get-Content -Raw (Join-Path $ArtifactDirectory 'spike-d-expected.json') | ConvertFrom-Json
if ($Expected.format_name -ne 'afterglow-spike-d-evidence' -or $Expected.format_version -ne 2 -or $Expected.minimum_reader_version -ne 2) { throw 'Version 1 artifacts are obsolete: obtain the new complete version 2 GPU probe bundle.' }
if ((Get-FileHash -Algorithm SHA256 $Exe).Hash -ne $Expected.exe_sha256) { throw 'GPU probe checksum differs' }
$Validator = Join-Path $ArtifactDirectory 'EvidenceValidator.exe'
if (-not (Test-Path $Validator)) { throw 'Self-contained bundle is incomplete: EvidenceValidator.exe missing' }
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
$InventoryId = [guid]::NewGuid().ToString()
$InventoryDirectory = Join-Path $OutputDirectory "inventory/$InventoryId"
New-Item -ItemType Directory -Path $InventoryDirectory -Force | Out-Null
$Raw = Join-Path $InventoryDirectory 'adapters.jsonl'
$Stderr = Join-Path $InventoryDirectory 'stderr.log'
$Process = Start-Process -FilePath $Exe -ArgumentList '--list-adapters' -NoNewWindow -PassThru -RedirectStandardOutput $Raw -RedirectStandardError $Stderr
try {
    if (-not $Process.WaitForExit(30000)) { $Process.Kill(); $Process.WaitForExit(); throw 'Adapter inventory timed out' }
    if ($Process.ExitCode -ne 0) { throw "Adapter inventory failed; inspect $InventoryDirectory" }
} finally { $Process.Dispose() }
$Records = @(Read-SpikeDRecords $Raw)
$Inventories = @($Records | Where-Object { $_.format_name -eq 'afterglow-spike-d-event' -and $_.format_version -eq 2 -and $_.minimum_reader_version -eq 2 -and $_.event -eq 'inventory' })
if ($Inventories.Count -ne 1) { throw 'Exactly one version 2 adapter inventory required' }
$Integrated = @($Inventories[0].adapters | Where-Object { $_.device_type -eq 'IntegratedGpu' })
$Discrete = @($Inventories[0].adapters | Where-Object { $_.device_type -eq 'DiscreteGpu' })
$Hardware = @(Get-CimInstance Win32_VideoController | Select-Object Name, PNPDeviceID, DriverVersion, DriverDate, CurrentHorizontalResolution, CurrentVerticalResolution, CurrentRefreshRate)
Write-SpikeDJson @{
    format_name = 'afterglow-spike-d-hardware-inventory'; format_version = 2; minimum_reader_version = 2;
    provenance = $Expected.provenance; exe_sha256 = $Expected.exe_sha256;
    adapters = $Inventories[0].adapters; windows_video_controllers = $Hardware;
    inventory_log = 'adapters.jsonl'; inventory_log_sha256 = (Get-FileHash -Algorithm SHA256 $Raw).Hash;
    inventory_stderr_sha256 = (Get-FileHash -Algorithm SHA256 $Stderr).Hash
} (Join-Path $InventoryDirectory 'inventory.json')
if (-not $Integrated.Count -or -not $Discrete.Count) { throw "Both physical adapter classes are required; integrated=$($Integrated.Count), discrete=$($Discrete.Count). No class fallback is permitted." }
if ($RequireIntelNvidia -and (-not @($Integrated | Where-Object { $_.vendor -eq 32902 }).Count -or -not @($Discrete | Where-Object { $_.vendor -eq 4318 }).Count)) { throw 'Requested hybrid inventory was not proved: Intel integrated (vendor 0x8086) and NVIDIA discrete (0x10DE) required.' }
Write-Host 'Actual enumerated adapters (OS graphics preferences are supplemental):'
$Inventories[0].adapters | Format-Table name, device_type, vendor, device, backend, driver, driver_info | Out-Host
Write-Host 'Each DPI session has 18 cells (both GPU classes, three resolutions and three effect modes).'
Write-Host 'The probe uses Windows SendInput only while focused. Keep its window visible. Actual received events and event-to-present submission latency are recorded; human perception is separately observed.'
$Scales = if ($DpiPercent) { @($DpiPercent) } else { @(100, 150, 200) }
foreach ($Scale in $Scales) {
    if (-not $DpiPercent) {
        Start-Process 'ms-settings:display' | Out-Null
        Write-Host "Set the test display's Windows scale to $Scale%. Do not use custom scaling requiring sign-out. Keep the probe on that display; every cell independently verifies real DPI."
        Read-Host 'After applying that scale, close Settings and press Enter' | Out-Null
    } else { Write-Host "Requested one actual $Scale% DPI session. This parameter does not change Windows scaling." }
    foreach ($Class in @('integrated', 'discrete')) {
        $Name = if ($Class -eq 'integrated') { $IntegratedAdapterName } else { $DiscreteAdapterName }
        $Parameters = @{
            GpuClass = $Class; DpiPercent = $Scale; ArtifactDirectory = $ArtifactDirectory; OutputDirectory = $OutputDirectory;
            PowerNote = $PowerNote; HumanObservations = (-not $SkipHumanObservations); Resume = [bool]$Resume
        }
        if ($Name) { $Parameters.AdapterName = $Name }
        & (Join-Path $PSScriptRoot 'collect_matrix.ps1') @Parameters
    }
}
$SummaryPath = Join-Path $OutputDirectory ('validation-' + [guid]::NewGuid().ToString() + '.json')
$SummaryError = $SummaryPath + '.stderr.log'
$Process = Start-Process -FilePath $Validator -ArgumentList @('validate-spike-d', ('"' + $OutputDirectory + '"')) -NoNewWindow -PassThru -RedirectStandardOutput $SummaryPath -RedirectStandardError $SummaryError
try {
    if (-not $Process.WaitForExit(60000)) { $Process.Kill(); $Process.WaitForExit(); throw 'Strict evidence validator timed out' }
    $Code = $Process.ExitCode
} finally { $Process.Dispose() }
Get-Content -Raw $SummaryPath | Write-Output
Write-Host "Strict validator exit code $Code (0 PASS, 1 FAIL, 2 PENDING). Raw logs, failed attempts and null human observations are preserved in $OutputDirectory."
if ($Code -notin @(0, 1, 2)) { throw "Evidence validator could not run: exit $Code" }
exit $Code
