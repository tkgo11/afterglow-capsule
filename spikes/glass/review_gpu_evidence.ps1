# SPIKE ONLY. Review original physical observations and resolve retries explicitly.
# This never runs a GPU probe, invents observations or deletes original evidence.
param(
    [string]$ArtifactDirectory = $PSScriptRoot,
    [string]$OutputDirectory = (Join-Path $PSScriptRoot 'results/d'),
    [switch]$ObservationsOnly,
    [switch]$SelectionsOnly,
    [string]$TrustedProvenance
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
. (Join-Path $PSScriptRoot 'collector_common.ps1')
if ($ObservationsOnly -and $SelectionsOnly) { throw 'Choose at most one review scope' }
$Expected = Read-SpikeDBoundedJson (Join-Path $OutputDirectory 'spike-d-expected.json')
if ($Expected.format_name -ne 'afterglow-spike-d-evidence' -or $Expected.format_version -ne 2 -or $Expected.minimum_reader_version -ne 2) { throw 'Only current version 2 evidence can be reviewed; preserve obsolete evidence separately.' }
if ((Get-FileHash -Algorithm SHA256 (Join-Path $OutputDirectory 'SpikeD-Glass.exe')).Hash -ne $Expected.exe_sha256) { throw 'Evidence executable checksum differs' }
$ReviewId = [guid]::NewGuid().ToString()
$ReviewDirectory = Join-Path $OutputDirectory "reviews/$ReviewId"
New-Item -ItemType Directory -Path $ReviewDirectory -Force | Out-Null
$Attempts = @()
foreach ($File in @(Get-ChildItem (Join-Path $OutputDirectory 'cells/*/cell.json') -ErrorAction SilentlyContinue)) {
    if (($File.Attributes -band [IO.FileAttributes]::ReparsePoint) -or ($File.Directory.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Review refuses evidence symlinks/reparse points' }
    $Cell = Read-SpikeDBoundedJson $File.FullName
    if ($Cell.format_name -ne 'afterglow-spike-d-cell-result' -or $Cell.format_version -ne 2 -or $Cell.minimum_reader_version -ne 2 -or $Cell.cell_id -ne $File.Directory.Name -or $Cell.exe_sha256 -ne $Expected.exe_sha256) { throw 'Cell identity/version/executable differs; no observation was rewritten.' }
    if ($Cell.cell_id -notmatch '^[A-Za-z0-9_-]{1,128}$') { throw 'Unsafe or oversized cell identity' }
    foreach ($Kind in @('raw_log', 'stderr_log')) {
        $Name = Get-SpikeDField $Cell $Kind
        if ($Name -notin @('probe.jsonl', 'stderr.log')) { throw 'Unsafe raw evidence path' }
        $LogPath = Join-Path $File.Directory.FullName $Name
        if ((Get-Item -LiteralPath $LogPath).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Review refuses raw log reparse points' }
        if ((Get-FileHash -Algorithm SHA256 -LiteralPath $LogPath).Hash -ne (Get-SpikeDField $Cell ($Kind + '_sha256'))) { throw 'Raw evidence checksum differs; no human observation is rewritten' }
    }
    $Coordinate = "$($Cell.requested_gpu_class)/$($Cell.requested_dpi_percent)/$($Cell.width)x$($Cell.height)/$($Cell.requested_mode)"
    if ($Attempts.Count -ge 4096) { throw 'Too many evidence attempts for bounded review' }
    $Attempts += [pscustomobject]@{ coordinate = $Coordinate; path = "cells/$($Cell.cell_id)/cell.json"; file = $File.FullName; cell = $Cell }
}
if (-not $Attempts.Count) { throw 'No original physical cells exist to review' }
$SelectionPath = Join-Path $OutputDirectory 'matrix-selection.json'
$PriorSelection = $null
if (Test-Path $SelectionPath) {
    $PriorSelection = Read-SpikeDBoundedJson $SelectionPath
    if ($PriorSelection.format_name -ne 'afterglow-spike-d-matrix-selection' -or $PriorSelection.format_version -ne 2 -or $PriorSelection.minimum_reader_version -ne 2) { throw 'Unsupported selection document; preserved without rewriting.' }
}
$Actions = @()
if (-not $ObservationsOnly) {
    $Selections = @()
    $Exclusions = @()
    foreach ($Group in @($Attempts | Group-Object coordinate | Sort-Object Name)) {
        $Candidates = @($Group.Group | Sort-Object path)
        if ($Candidates.Count -eq 1) { $Selections += @{ cell_path = $Candidates[0].path }; continue }
        Write-Host "Repeated coordinate: $($Group.Name). Every original attempt remains on disk."
        for ($Index = 0; $Index -lt $Candidates.Count; $Index++) {
            $Candidate = $Candidates[$Index]
            Write-Host "$($Index + 1): $($Candidate.path), machine=$($Candidate.cell.machine_status), failures=$($Candidate.cell.failures -join '; ')"
        }
        $Answer = Read-Host 'Enter the number of the explicitly reviewed attempt to select, or pending (no default)'
        $Chosen = 0
        if (-not [int]::TryParse($Answer, [ref]$Chosen) -or $Chosen -lt 1 -or $Chosen -gt $Candidates.Count) {
            # Preserve any prior explicit decision, but do not resolve new unknown attempts.
            if ($null -ne $PriorSelection) {
                foreach ($Selection in $PriorSelection.selections) { if ($Selection.cell_path -in @($Candidates.path)) { $Selections += @{ cell_path = $Selection.cell_path } } }
                foreach ($Exclusion in $PriorSelection.exclusions) { if ($Exclusion.cell_path -in @($Candidates.path)) { $Exclusions += @{ cell_path = $Exclusion.cell_path; reason = $Exclusion.reason } } }
            }
            $Actions += @{ coordinate = $Group.Name; action = 'retry decision remains pending; any prior explicit decisions preserved' }
            continue
        }
        $Selections += @{ cell_path = $Candidates[$Chosen - 1].path }
        for ($Index = 0; $Index -lt $Candidates.Count; $Index++) {
            if ($Index -eq $Chosen - 1) { continue }
            $Reason = Read-Host "Explain excluding $($Candidates[$Index].path); original failure/logs remain visible. Blank means unresolved"
            if ($Reason.Length -gt 4096) { throw 'Exclusion explanation exceeds 4096 character bound' }
            if (-not [string]::IsNullOrWhiteSpace($Reason)) { $Exclusions += @{ cell_path = $Candidates[$Index].path; reason = $Reason }; $Actions += @{ cell_path = $Candidates[$Index].path; action = 'explicit exclusion'; reason = $Reason } }
        }
        $Actions += @{ cell_path = $Candidates[$Chosen - 1].path; action = 'explicit retry selection' }
    }
    if (Test-Path $SelectionPath) { Copy-Item $SelectionPath (Join-Path $ReviewDirectory 'matrix-selection-before.json') }
    Write-SpikeDJson @{
        format_name = 'afterglow-spike-d-matrix-selection'; format_version = 2; minimum_reader_version = 2;
        selections = $Selections; exclusions = $Exclusions; review_id = $ReviewId
    } $SelectionPath
}
if (-not $SelectionsOnly) {
    Write-Host 'Only directly observed original cells may be confirmed. Logs or synthetic-input counters alone cannot establish human perceived response.'
    $SelectedPaths = @()
    if (Test-Path $SelectionPath) { $CurrentSelection = Read-SpikeDBoundedJson $SelectionPath; $SelectedPaths = @($CurrentSelection.selections | ForEach-Object { $_.cell_path }) }
    foreach ($Attempt in $Attempts | Sort-Object coordinate, path) {
        if ($SelectedPaths.Count -and $Attempt.path -notin $SelectedPaths) { continue }
        $Cell = $Attempt.cell
        if (-not (Test-SpikeDObservationMayBeCompleted $Cell.human_observation)) { Write-Host "Preserved failed original human observation for $($Attempt.path); collect a new physical attempt to demonstrate a correction."; continue }
        if ($null -ne $Cell.human_observation.responsive_input -and $null -ne $Cell.human_observation.foreground_preserved) { continue }
        Write-Host "$($Attempt.coordinate), original attempt $($Cell.cell_id), machine=$($Cell.machine_status)"
        $Answer = Read-Host 'Did you directly observe responsive input AND preserved opaque foreground/pointer feedback in this original cell? yes / input-failed / foreground-failed / both-failed / pending'
        if ($Answer -notin @('yes', 'input-failed', 'foreground-failed', 'both-failed')) { continue }
        $BeforeHash = (Get-FileHash -Algorithm SHA256 $Attempt.file).Hash
        $Backup = Join-Path $ReviewDirectory ($Cell.cell_id + '-cell-before.json')
        Copy-Item $Attempt.file $Backup
        if ($Answer -eq 'yes') {
            $Cell.human_observation.responsive_input = $true
            $Cell.human_observation.foreground_preserved = $true
            $Cell.human_observation.note = 'Human separately reviewed their directly observed original cell; logs alone were not used to attest responsiveness.'
        } else {
            if ($Answer -in @('input-failed', 'both-failed')) { $Cell.human_observation.responsive_input = $false }
            if ($Answer -in @('foreground-failed', 'both-failed')) { $Cell.human_observation.foreground_preserved = $false }
            $Cell.human_observation.note = Read-Host 'Describe the directly observed failure'
            if ($Cell.human_observation.note.Length -gt 4096) { throw 'Human observation note exceeds 4096 character bound' }
        }
        Write-SpikeDJson $Cell $Attempt.file
        $Actions += @{ cell_path = $Attempt.path; action = 'human observation review'; answer = $Answer; prior_cell_sha256 = $BeforeHash; prior_cell_file = ($Cell.cell_id + '-cell-before.json') }
    }
}
Write-SpikeDJson @{
    format_name = 'afterglow-spike-d-human-review'; format_version = 2; minimum_reader_version = 2;
    review_id = $ReviewId; captured_utc = [DateTime]::UtcNow.ToString('o'); actions = $Actions;
    status = 'PENDING'; reason = 'Review records observations and explicit exclusions; strict independently anchored validation still decides acceptance.'
} (Join-Path $ReviewDirectory 'review.json')
$Validator = Join-Path $ArtifactDirectory 'EvidenceValidator.exe'
if (-not (Test-Path $Validator)) { throw 'EvidenceValidator.exe is missing from the self-contained bundle' }
$Arguments = @('validate-spike-d', ('"' + $OutputDirectory + '"'))
if ($TrustedProvenance) { $Arguments += @('--trusted-provenance', ('"' + $TrustedProvenance + '"')) }
$Summary = Join-Path $ReviewDirectory 'validation.json'
$Process = Start-Process -FilePath $Validator -ArgumentList $Arguments -NoNewWindow -PassThru -RedirectStandardOutput $Summary -RedirectStandardError (Join-Path $ReviewDirectory 'validator.stderr.log')
try {
    if (-not $Process.WaitForExit(60000)) { $Process.Kill(); if (-not $Process.WaitForExit(5000)) { throw 'Validator termination cleanup timed out' }; throw 'Evidence validator timed out' }
    $Code = $Process.ExitCode
} finally { $Process.Dispose() }
Get-Content -Raw $Summary | Write-Output
Write-Host "Original records preserved in $ReviewDirectory. Strict validation exit $Code (0 PASS, 1 FAIL, 2 PENDING)."
exit $Code
