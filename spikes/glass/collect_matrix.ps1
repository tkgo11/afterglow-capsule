# SPIKE ONLY. Collect nine cells on one physical GPU at one actual Windows DPI.
# Logs and exit codes are observations, not automatic acceptance of performance.
param(
    [Parameter(Mandatory=$true)][ValidateSet("integrated", "discrete")][string]$GpuClass,
    [Parameter(Mandatory=$true)][ValidateSet(100, 150, 200)][int]$DpiPercent,
    [string]$ArtifactDirectory = $PSScriptRoot,
    [string]$OutputDirectory = (Join-Path $PSScriptRoot "results")
)
$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
$Exe = Join-Path $ArtifactDirectory "SpikeD-Glass.exe"
$Expected = Get-Content -Raw (Join-Path $ArtifactDirectory "spike-d-expected.json") | ConvertFrom-Json
if ($Expected.format_name -ne "afterglow-spike-d-evidence" -or $Expected.format_version -ne 1 -or $Expected.minimum_reader_version -ne 1) { throw "Unsupported evidence metadata" }
if ((Get-FileHash -Algorithm SHA256 $Exe).Hash -ne $Expected.exe_sha256) { throw "GPU probe checksum differs" }
$Session = Join-Path $OutputDirectory ("$GpuClass-$DpiPercent-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $Session -Force | Out-Null
$Cells = @()
foreach ($Size in @(@(960, 640), @(1440, 900), @(1920, 1080))) {
    foreach ($Mode in @("full", "opaque", "reduced")) {
        $Cell = "$($Size[0])x$($Size[1])-$Mode"
        Write-Output "$Cell : warm up, keep the window focused, interact, collect at least three 300-frame windows, then Escape."
        $Process = Start-Process -FilePath $Exe -ArgumentList @($Size[0], $Size[1], $Mode) -PassThru -Wait -NoNewWindow -RedirectStandardOutput (Join-Path $Session "$Cell.stdout.log") -RedirectStandardError (Join-Path $Session "$Cell.stderr.log")
        $Cells += @{
            width = $Size[0]; height = $Size[1]; mode = $Mode; exit_code = $Process.ExitCode;
            log_prefix = $Cell; responsive_input = $null; foreground_preserved = $null;
            performance_or_automatic_degradation = $null; acceptance_review = "pending"
        }
        $Process.Dispose()
    }
}
@{
    format_name = "afterglow-spike-d-observations"; format_version = 1; minimum_reader_version = 1;
    windows = [Environment]::OSVersion.VersionString; exe_sha256 = $Expected.exe_sha256;
    requested_gpu_class = $GpuClass; requested_dpi_percent = $DpiPercent;
    actual_adapter_and_dpi = "must be verified from every probe log";
    hardware_driver_refresh_power_notes = $null; cells = $Cells; acceptance_review = "pending"
} | ConvertTo-Json -Depth 6 | Set-Content -Encoding UTF8 (Join-Path $Session "observations.json")
Write-Output "Evidence collected in $Session. Repeat for each GPU/DPI combination; review logs and fill human observations."
