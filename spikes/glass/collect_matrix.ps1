# SPIKE ONLY. Explicitly selects the requested physical adapter class for every cell.
# Machine evidence never substitutes for a human visual responsiveness observation.
param(
    [Parameter(Mandatory=$true)][ValidateSet('integrated', 'discrete')][string]$GpuClass,
    [Parameter(Mandatory=$true)][ValidateSet(100, 150, 200)][int]$DpiPercent,
    [string]$ArtifactDirectory = $PSScriptRoot,
    [string]$OutputDirectory = (Join-Path $PSScriptRoot 'results/d'),
    [ValidatePattern('^[^"\\\r\n]*$')][ValidateLength(0,512)][string]$AdapterName,
    [ValidateLength(0,4096)][string]$PowerNote = '',
    [ValidateSet(0, 32902, 4318)][int]$RequiredVendor = 0,
    [switch]$HumanObservations,
    [switch]$Resume
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
. (Join-Path $PSScriptRoot 'collector_common.ps1')
$Exe = Join-Path $ArtifactDirectory 'SpikeD-Glass.exe'
$ExpectedPath = Join-Path $ArtifactDirectory 'spike-d-expected.json'
$Expected = Get-Content -Raw $ExpectedPath | ConvertFrom-Json
if ($Expected.format_name -ne 'afterglow-spike-d-evidence' -or $Expected.format_version -ne 2 -or $Expected.minimum_reader_version -ne 2) { throw 'Obsolete/incompatible artifact: download the version 2 GPU probe bundle.' }
if ((Get-FileHash -Algorithm SHA256 $Exe).Hash -ne $Expected.exe_sha256) { throw 'GPU probe checksum differs' }
$Provenance = Get-SpikeDField $Expected 'provenance'
if ($null -eq $Provenance -or $Provenance.source_commit -notmatch '^[a-fA-F0-9]{40}$' -or $Provenance.repository -ne 'tkgo11/afterglow-capsule' -or $Provenance.workflow_run_id -le 0 -or $Provenance.workflow_run_attempt -le 0 -or $Provenance.workflow_path -ne '.github/workflows/spikes.yml') { throw 'Required source/workflow provenance is absent or invalid' }
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
# A result directory belongs to one immutable artifact. Preserve incompatible old evidence.
foreach ($Pair in @(@($Exe, 'SpikeD-Glass.exe'), @($ExpectedPath, 'spike-d-expected.json'))) {
    $Destination = Join-Path $OutputDirectory $Pair[1]
    if (Test-Path $Destination) {
        if ((Get-FileHash -Algorithm SHA256 $Destination).Hash -ne (Get-FileHash -Algorithm SHA256 $Pair[0]).Hash) { throw 'Output directory belongs to a different artifact. Use a new OutputDirectory; old evidence is preserved.' }
    } else { Copy-Item $Pair[0] $Destination }
}
$Os = Get-CimInstance Win32_OperatingSystem
$Drivers = @(Get-CimInstance Win32_VideoController | Select-Object Name, PNPDeviceID, DriverVersion, DriverDate, CurrentHorizontalResolution, CurrentVerticalResolution, CurrentRefreshRate)
$PowerScheme = (& powercfg.exe /getactivescheme | Out-String).Trim()
$Battery = @(Get-CimInstance Win32_Battery | Select-Object BatteryStatus, EstimatedChargeRemaining)
$RecordedPowerNote = "Active Windows power scheme: $PowerScheme. Battery snapshot: $($Battery | ConvertTo-Json -Compress -Depth 5). User note: $PowerNote"
$SessionId = [guid]::NewGuid().ToString()
$Cells = @()
foreach ($Size in @(@(960, 640), @(1440, 900), @(1920, 1080))) {
    foreach ($Mode in @('full', 'opaque', 'reduced')) {
        if ($Resume) {
            $PriorPass = @(Get-ChildItem (Join-Path $OutputDirectory 'cells/*/cell.json') -ErrorAction SilentlyContinue | ForEach-Object { Get-Content -Raw $_.FullName | ConvertFrom-Json } | Where-Object {
                $_.machine_status -eq 'PASS' -and $_.exe_sha256 -eq $Expected.exe_sha256 -and $_.requested_gpu_class -eq $GpuClass -and $_.requested_dpi_percent -eq $DpiPercent -and $_.width -eq $Size[0] -and $_.height -eq $Size[1] -and $_.requested_mode -eq $Mode
            })
            if ($PriorPass.Count -eq 1) { Write-Host "Retained prior machine evidence: $GpuClass $DpiPercent% $($Size[0])x$($Size[1]) $Mode"; continue }
            if ($PriorPass.Count -gt 1) { Write-Warning 'Duplicate prior passing attempts remain unresolved; no attempt is silently selected.'; continue }
        }
        $CellId = [guid]::NewGuid().ToString()
        $Directory = Join-Path $OutputDirectory "cells/$CellId"
        New-Item -ItemType Directory -Path $Directory -Force | Out-Null
        $Raw = Join-Path $Directory 'probe.jsonl'
        $Stderr = Join-Path $Directory 'stderr.log'
        $Arguments = @($Size[0], $Size[1], $Mode, '--gpu', $GpuClass, '--expected-dpi', $DpiPercent, '--sample-windows', 3, '--warmup-seconds', 3, '--timeout-seconds', 120, '--drive-input')
        if ($AdapterName) { $Arguments += @('--adapter-name', ('"' + $AdapterName + '"')) }
        Write-Host "$GpuClass $DpiPercent% $($Size[0])x$($Size[1]) ${Mode}: keep the physical probe visible and focused; Windows SendInput moves the pointer inside it."
        $ExitCode = -1
        $Process = $null
        $TransportFailure = $null
        $AbortSession = $false
        try {
            $Process = Start-Process -FilePath $Exe -ArgumentList $Arguments -PassThru -NoNewWindow -RedirectStandardOutput $Raw -RedirectStandardError $Stderr
            if (-not $Process.WaitForExit(140000)) {
                $Process.Kill()
                if (-not $Process.WaitForExit(5000)) { $AbortSession = $true; throw 'Terminated probe failed to exit within the bounded cleanup deadline' }
                $TransportFailure = 'collector bounded timeout: probe was terminated'
            }
            $ExitCode = $Process.ExitCode
        } catch { $TransportFailure = $_.Exception.Message }
        finally { if ($null -ne $Process) { $Process.Dispose() } }
        if (-not (Test-Path $Raw)) { [IO.File]::WriteAllText($Raw, '') }
        if (-not (Test-Path $Stderr)) { [IO.File]::WriteAllText($Stderr, '') }
        $Records = @()
        try {
            if ((Get-Item $Raw).Length -gt 10485760) { throw 'raw log exceeds 10 MiB bound' }
            $Records = @(Read-SpikeDRecords $Raw)
        } catch { $TransportFailure = 'invalid JSONL output: ' + $_.Exception.Message }
        $Checked = Test-SpikeDProtocol -Records $Records -GpuClass $GpuClass -DpiPercent $DpiPercent -Width $Size[0] -Height $Size[1] -Mode $Mode -ExitCode $ExitCode -RequiredVendor $RequiredVendor
        $Failures = @($Checked.failures)
        if ($TransportFailure) { $Failures += $TransportFailure }
        $Observation = @{ responsive_input = $null; foreground_preserved = $null; note = $null }
        if ($HumanObservations) {
            Write-Host 'Machine timing and SendInput receipts do not prove perceived responsiveness.'
            $Answer = Read-Host 'For the cell just observed, was input visibly responsive AND the opaque foreground/pointer feedback preserved? yes / input-failed / foreground-failed / both-failed / pending'
            if ($Answer -eq 'yes') { $Observation.responsive_input = $true; $Observation.foreground_preserved = $true; $Observation.note = 'Human directly observed this cell and confirmed responsive feedback and stable opaque foreground.' }
            elseif ($Answer -in @('input-failed', 'foreground-failed', 'both-failed')) {
                if ($Answer -in @('input-failed', 'both-failed')) { $Observation.responsive_input = $false }
                if ($Answer -in @('foreground-failed', 'both-failed')) { $Observation.foreground_preserved = $false }
                $Observation.note = Read-Host 'Describe the directly observed failure'
            }
        }
        $Cell = @{
            format_name = 'afterglow-spike-d-cell-result'; format_version = 2; minimum_reader_version = 2;
            cell_id = $CellId; session_id = $SessionId; captured_utc = [DateTime]::UtcNow.ToString('o'); provenance = $Provenance;
            exe_sha256 = $Expected.exe_sha256; requested_gpu_class = $GpuClass; requested_dpi_percent = $DpiPercent;
            width = $Size[0]; height = $Size[1]; requested_mode = $Mode; requested_adapter_vendor = $RequiredVendor;
            actual_adapter = (Get-SpikeDField $Checked.start 'adapter'); actual_dpi_percent = (Get-SpikeDField $Checked.start 'actual_dpi_percent');
            samples = @($Checked.samples); input_source = 'windows-sendinput'; exit_code = $ExitCode;
            machine_status = $(if ($Failures.Count) { 'FAIL' } else { 'PASS' }); failures = $Failures;
            raw_log = 'probe.jsonl'; raw_log_sha256 = (Get-FileHash -Algorithm SHA256 $Raw).Hash;
            stderr_log = 'stderr.log'; stderr_log_sha256 = (Get-FileHash -Algorithm SHA256 $Stderr).Hash;
            human_observation = $Observation;
            hardware = @{ os_caption = $Os.Caption; os_build = $Os.BuildNumber; gpu_driver = $Drivers; refresh_rate = @($Drivers | ForEach-Object { $_.CurrentRefreshRate }); power_note = $RecordedPowerNote; active_power_scheme = $PowerScheme; battery = $Battery }
        }
        Write-SpikeDJson $Cell (Join-Path $Directory 'cell.json')
        $Cells += @{ cell_path = "cells/$CellId/cell.json"; machine_status = $Cell.machine_status; failures = $Failures }
        Write-Host "Machine status: $($Cell.machine_status); human gate: pending unless expressly observed. Evidence: $Directory"
        if ($AbortSession) { throw "Collection stopped after bounded process cleanup failed. The failed cell and raw logs were preserved in $Directory" }
    }
}
$Session = @{
    format_name = 'afterglow-spike-d-collection-session'; format_version = 2; minimum_reader_version = 2;
    session_id = $SessionId; requested_gpu_class = $GpuClass; requested_dpi_percent = $DpiPercent; cells = $Cells;
    status = $(if (@($Cells | Where-Object { $_.machine_status -eq 'FAIL' }).Count) { 'FAIL' } else { 'PENDING' });
    reason = 'Final acceptance requires complete strict validation and direct human observations. Prior attempts remain preserved.'
}
New-Item -ItemType Directory -Path (Join-Path $OutputDirectory 'sessions') -Force | Out-Null
Write-SpikeDJson $Session (Join-Path $OutputDirectory "sessions/$SessionId.json")
