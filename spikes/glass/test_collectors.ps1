# PURE UNIT TESTS ONLY: synthetic JSON fixtures, not physical validation evidence.
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
. (Join-Path $PSScriptRoot 'collector_common.ps1')
foreach ($Path in @('collector_common.ps1', 'collect_matrix.ps1', 'run_all_gpu_validation.ps1', 'test_collectors.ps1', 'review_gpu_evidence.ps1')) {
    $Tokens = $null
    $Errors = $null
    [Management.Automation.Language.Parser]::ParseFile((Join-Path $PSScriptRoot $Path), [ref]$Tokens, [ref]$Errors) | Out-Null
    if ($Errors.Count) { throw ($Errors | Out-String) }
}
function New-SyntheticProtocolFixture {
    $Header = @{ format_name = 'afterglow-spike-d-event'; format_version = 2; minimum_reader_version = 2 }
    $Start = $Header.Clone()
    $Start.event = 'start'; $Start.requested_gpu_class = 'integrated'; $Start.requested_dpi_percent = 150; $Start.actual_dpi_percent = 150
    $Start.native_dpi = 144; $Start.dpi_awareness = 'PerMonitorAware'
    $Start.width = 960; $Start.height = 640; $Start.requested_mode = 'full'; $Start.drive_input = $true
    $Start.adapter = @{ name = 'SYNTHETIC UNIT FIXTURE'; vendor = 32902; device = 1; backend = 'Dx12'; driver = ''; driver_info = ''; device_type = 'IntegratedGpu' }
    $Records = @($Start)
    foreach ($Index in @(1, 2, 3)) {
        $Sample = $Header.Clone()
        $Sample.event = 'sample'; $Sample.index = $Index; $Sample.frames = 300; $Sample.actual_dpi_percent = 150
        $Sample.native_dpi = 144; $Sample.dpi_awareness = 'PerMonitorAware'
        $Sample.width = 960; $Sample.height = 640; $Sample.mode = 'full'; $Sample.next_mode = 'full'
        $Sample.interval_avg_ms = 16.6; $Sample.interval_p95_ms = 16.7; $Sample.pointer_events = 90; $Sample.keyboard_events = 90
        $Sample.input_ack_count = 90; $Sample.input_ack_avg_ms = 0.1; $Sample.input_ack_p95_ms = 0.2
        $Sample.event_to_present_count = 180; $Sample.event_to_present_avg_ms = 0.3; $Sample.event_to_present_p95_ms = 0.4
        $Records += $Sample
    }
    $Complete = $Header.Clone()
    $Complete.event = 'complete'; $Complete.status = 'PASS'; $Complete.windows_collected = 3; $Complete.reason = $null
    return @($Records) + @($Complete)
}
function Invoke-SyntheticCheck {
    param([object[]]$Records, [int]$ExitCode = 0)
    return Test-SpikeDProtocol -Records $Records -GpuClass integrated -DpiPercent 150 -Width 960 -Height 640 -Mode full -ExitCode $ExitCode
}
$Tests = 0
function Assert-Rejected {
    param([object[]]$Records, [string]$Description, [int]$ExitCode = 0)
    $Checked = Invoke-SyntheticCheck $Records $ExitCode
    if ($Checked.machine_status -ne 'FAIL' -or -not $Checked.failures.Count) { throw "Was not rejected: $Description" }
    $script:Tests++
}
$Valid = Invoke-SyntheticCheck (New-SyntheticProtocolFixture)
if ($Valid.machine_status -ne 'PASS' -or $Valid.failures.Count) { throw ('Pure valid protocol fixture failed: ' + ($Valid | ConvertTo-Json -Depth 10)) }
$Tests++
$Fixture = New-SyntheticProtocolFixture; $Fixture[0].adapter.device_type = 'DiscreteGpu'; Assert-Rejected $Fixture 'wrong actual GPU'
$Fixture = New-SyntheticProtocolFixture; $Fixture[0].requested_gpu_class = 'discrete'; Assert-Rejected $Fixture 'wrong requested GPU'
$Fixture = New-SyntheticProtocolFixture; $Fixture[0].actual_dpi_percent = 100; Assert-Rejected $Fixture 'wrong initial DPI'
$Fixture = New-SyntheticProtocolFixture; $Fixture[2].actual_dpi_percent = 200; Assert-Rejected $Fixture 'DPI change during sample'
$Fixture = New-SyntheticProtocolFixture; $Fixture[1].pointer_events = 0; Assert-Rejected $Fixture 'zero actual received pointer events'
$Fixture = New-SyntheticProtocolFixture; $Fixture[1].input_ack_count = 0; Assert-Rejected $Fixture 'input not acknowledged'
$Fixture = New-SyntheticProtocolFixture; $Fixture[1].event_to_present_count = 0; Assert-Rejected $Fixture 'no event to presentation samples'
$Fixture = New-SyntheticProtocolFixture; $Fixture[1].frames = 299; Assert-Rejected $Fixture 'partial sample window'
$Fixture = New-SyntheticProtocolFixture; $Fixture = @($Fixture[0], $Fixture[1], $Fixture[2], $Fixture[4]); Assert-Rejected $Fixture 'missing sample window'
$Fixture = New-SyntheticProtocolFixture; $Fixture[2].index = 1; Assert-Rejected $Fixture 'reordered sample windows'
$Fixture = New-SyntheticProtocolFixture; $Fixture[1].interval_avg_ms = [double]::NaN; Assert-Rejected $Fixture 'NaN cadence'
$Fixture = New-SyntheticProtocolFixture; $Fixture[1].event_to_present_p95_ms = $null; Assert-Rejected $Fixture 'missing latency'
$Fixture = New-SyntheticProtocolFixture; $Fixture[1].width = 1440; Assert-Rejected $Fixture 'wrong client dimensions'
$Fixture = New-SyntheticProtocolFixture; $Fixture[0].format_version = 1; Assert-Rejected $Fixture 'obsolete protocol'
$Fixture = New-SyntheticProtocolFixture; Assert-Rejected $Fixture 'nonzero process exit' 1
$Fixture = New-SyntheticProtocolFixture; $Fixture[1].interval_avg_ms = 30; Assert-Rejected $Fixture 'slow full effects without automatic degradation'
$Fixture = New-SyntheticProtocolFixture; $Fixture += @{ format_name = 'afterglow-spike-d-event'; format_version = 2; minimum_reader_version = 2; event = 'failure'; reason = 'synthetic failure fixture' }; Assert-Rejected $Fixture 'failure event never hidden'
$Fixture = New-SyntheticProtocolFixture; $Fixture = @($Fixture[0..3]); Assert-Rejected $Fixture 'missing complete event'
$Fixture = New-SyntheticProtocolFixture; $Fixture[1].input_ack_avg_ms = 0; $Fixture[1].event_to_present_avg_ms = 0
if ((Invoke-SyntheticCheck $Fixture).machine_status -ne 'PASS') { throw 'Measured zero latency should be representable, not fabricated into a positive value' }
$Tests++
$Fixture = New-SyntheticProtocolFixture; $Fixture[1].keyboard_events = 0; Assert-Rejected $Fixture 'zero actual keyboard events'
$Fixture = New-SyntheticProtocolFixture; $Fixture[0].native_dpi = 96; Assert-Rejected $Fixture 'native DPI mismatch'
$Fixture = New-SyntheticProtocolFixture; $Fixture[1].dpi_awareness = 'Unaware'; Assert-Rejected $Fixture 'DPI virtualization'
$Fixture = New-SyntheticProtocolFixture; $Fixture[1].event_to_present_count = 181; Assert-Rejected $Fixture 'received input accounting mismatch'
$Fixture = New-SyntheticProtocolFixture; $Fixture[1].input_ack_count = 181; Assert-Rejected $Fixture 'input acknowledgment accounting mismatch'
$Fixture = New-SyntheticProtocolFixture; $Fixture[1].mode = 'opaque'; Assert-Rejected $Fixture 'effect mode differs from requested cell'
$Fixture = New-SyntheticProtocolFixture; $Fixture[1].next_mode = 'reduced'; Assert-Rejected $Fixture 'next sample hides mode transition'
$Fixture = New-SyntheticProtocolFixture; $Fixture += @{format_name='afterglow-spike-d-event';format_version=2;minimum_reader_version=2;event='unknown'}; Assert-Rejected $Fixture 'unknown event kind'
$Fixture = New-SyntheticProtocolFixture; $Fixture = @($Fixture[1], $Fixture[0], $Fixture[2], $Fixture[3], $Fixture[4]); Assert-Rejected $Fixture 'sample before start'
$Fixture = New-SyntheticProtocolFixture; $Fixture = @($Fixture[0], $Fixture[1], $Fixture[2], $Fixture[4], $Fixture[3]); Assert-Rejected $Fixture 'sample after complete'
$Fixture = New-SyntheticProtocolFixture; $Fixture[1].pointer_events = 90.5; Assert-Rejected $Fixture 'fractional received event counter'
$RunnerSource = Get-Content -Raw (Join-Path $PSScriptRoot 'run_all_gpu_validation.ps1')
if ($RunnerSource -notmatch "event -eq 'adapter_inventory'") { throw 'Adapter inventory event protocol differs from the Rust probe' }
$Tests++
$Fixture = New-SyntheticProtocolFixture; $Fixture[1].interval_p95_ms = 50; Assert-Rejected $Fixture 'bad p95 cadence hidden by fast average'
$Fixture = New-SyntheticProtocolFixture; $Fixture[1].interval_p95_ms = 20; Assert-Rejected $Fixture 'p95 requires reduced effects'
$Fixture = New-SyntheticProtocolFixture; $Fixture[1].interval_avg_ms = 30; $Fixture[1].next_mode = 'static'; $Fixture[2].mode = 'static'; $Fixture[2].next_mode = 'static'; $Fixture[3].mode = 'static'; $Fixture[3].next_mode = 'static'
if ((Invoke-SyntheticCheck $Fixture).machine_status -ne 'PASS') { throw 'Correct conservative degradation fixture was rejected' }
$Tests++
$Fixture = New-SyntheticProtocolFixture
if ((Test-SpikeDProtocol -Records $Fixture -GpuClass integrated -DpiPercent 150 -Width 960 -Height 640 -Mode full -ExitCode 0 -RequiredVendor 4318).machine_status -ne 'FAIL') { throw 'Wrong requested hybrid vendor was accepted' }
$Tests++
if (Test-SpikeDObservationMayBeCompleted @{responsive_input=$false;foreground_preserved=$null}) { throw 'Failed input observation may not be relabelled PASS' }
$Tests++
if (Test-SpikeDObservationMayBeCompleted @{responsive_input=$null;foreground_preserved=$false}) { throw 'Failed foreground observation may not be relabelled PASS' }
$Tests++
if (-not (Test-SpikeDObservationMayBeCompleted @{responsive_input=$null;foreground_preserved=$null})) { throw 'An actually observed pending cell may be completed by explicit human review' }
$Tests++
Write-Output "$Tests pure collector protocol tests passed. No physical evidence was generated."
