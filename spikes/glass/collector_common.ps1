# SPIKE ONLY. Pure protocol checks are also exercised with synthetic unit fixtures.
Set-StrictMode -Version Latest
function Get-SpikeDField {
    param($Object, [string]$Name)
    if ($null -eq $Object) { return $null }
    if ($Object -is [Collections.IDictionary]) { return $Object[$Name] }
    $Property = $Object.PSObject.Properties[$Name]
    if ($null -ne $Property) { return $Property.Value }
    return $null
}
function Test-SpikeDFiniteNumber {
    param($Value, [switch]$AllowZero)
    if ($null -eq $Value -or $Value -is [string] -or $Value -is [bool]) { return $false }
    try { $Number = [double]$Value } catch { return $false }
    return -not [double]::IsNaN($Number) -and -not [double]::IsInfinity($Number) -and (($Number -gt 0) -or ($AllowZero -and $Number -eq 0))
}
function Test-SpikeDCount {
    param($Value, [switch]$AllowZero)
    return (Test-SpikeDFiniteNumber $Value -AllowZero:$AllowZero) -and ([math]::Floor([double]$Value) -eq [double]$Value)
}
function Test-SpikeDProtocol {
    param([object[]]$Records, [string]$GpuClass, [int]$DpiPercent, [int]$Width, [int]$Height, [string]$Mode, [int]$ExitCode)
    $Failures = New-Object 'Collections.Generic.List[string]'
    $ExpectedType = if ($GpuClass -eq 'integrated') { 'IntegratedGpu' } else { 'DiscreteGpu' }
    if ($ExitCode -ne 0) { $Failures.Add("probe exit code $ExitCode") }
    $State = 'before-start'
    $InventorySeen = $false
    foreach ($Record in $Records) {
        $Event = Get-SpikeDField $Record 'event'
        switch ($Event) {
            'adapter_inventory' { if ($State -ne 'before-start' -or $InventorySeen) { $Failures.Add('adapter inventory order invalid') }; $InventorySeen = $true }
            'start' { if ($State -ne 'before-start') { $Failures.Add('start event order invalid') }; $State = 'sampling' }
            'sample' { if ($State -ne 'sampling') { $Failures.Add('sample event order invalid') } }
            'complete' { if ($State -ne 'sampling') { $Failures.Add('complete event order invalid') }; $State = 'complete' }
            'failure' { }
            default { $Failures.Add('unknown event kind') }
        }
        if ((Get-SpikeDField $Record 'format_name') -ne 'afterglow-spike-d-event' -or (Get-SpikeDField $Record 'format_version') -ne 2 -or (Get-SpikeDField $Record 'minimum_reader_version') -ne 2) { $Failures.Add('unsupported or missing event protocol version') }
        if ((Get-SpikeDField $Record 'event') -eq 'failure') { $Failures.Add('probe emitted failure: ' + ($Record | ConvertTo-Json -Compress -Depth 10)) }
    }
    $Starts = @($Records | Where-Object { (Get-SpikeDField $_ 'event') -eq 'start' })
    $Samples = @($Records | Where-Object { (Get-SpikeDField $_ 'event') -eq 'sample' })
    $Completed = @($Records | Where-Object { (Get-SpikeDField $_ 'event') -eq 'complete' })
    $Start = if ($Starts.Count -eq 1) { $Starts[0] } else { $null }
    if ($Starts.Count -ne 1) { $Failures.Add('exactly one start event required') }
    if ($Samples.Count -ne 3) { $Failures.Add('exactly three sample windows required') }
    if ($Completed.Count -ne 1 -or (Get-SpikeDField $Completed[0] 'status') -ne 'PASS' -or (Get-SpikeDField $Completed[0] 'windows_collected') -ne 3) { $Failures.Add('successful complete event required') }
    if ((Get-SpikeDField $Start 'drive_input') -ne $true) { $Failures.Add('requested Windows input-driving mode was not recorded') }
    if ((Get-SpikeDField $Start 'requested_dpi_percent') -ne $DpiPercent) { $Failures.Add('probe requested DPI mismatch') }
    if ((Get-SpikeDField $Start 'requested_gpu_class') -ne $GpuClass) { $Failures.Add('probe requested GPU class mismatch') }
    $Adapter = Get-SpikeDField $Start 'adapter'
    if ((Get-SpikeDField $Adapter 'device_type') -ne $ExpectedType) { $Failures.Add('actual adapter class mismatch') }
    foreach ($Field in @('name', 'vendor', 'device', 'backend', 'driver', 'driver_info')) {
        if ($null -eq (Get-SpikeDField $Adapter $Field)) { $Failures.Add("adapter metadata missing: $Field") }
    }
    $Measured = @($Start) + $Samples
    foreach ($Record in $Measured) {
        if ((Get-SpikeDField $Record 'native_dpi') -ne ($DpiPercent * 96 / 100) -or (Get-SpikeDField $Record 'dpi_awareness') -ne 'PerMonitorAware') { $Failures.Add('native DPI or per-monitor DPI-awareness mismatch') }
        if (-not (Test-SpikeDFiniteNumber (Get-SpikeDField $Record 'actual_dpi_percent')) -or (Get-SpikeDField $Record 'actual_dpi_percent') -ne $DpiPercent) { $Failures.Add('actual DPI does not match requested DPI') }
        if (-not (Test-SpikeDCount (Get-SpikeDField $Record 'width')) -or -not (Test-SpikeDCount (Get-SpikeDField $Record 'height')) -or (Get-SpikeDField $Record 'width') -ne $Width -or (Get-SpikeDField $Record 'height') -ne $Height) { $Failures.Add('actual client dimensions do not match requested resolution') }
    }
    if ((Get-SpikeDField $Start 'requested_mode') -ne $Mode) { $Failures.Add('requested effect mode mismatch') }
    $Index = 1
    $ExpectedMode = $Mode
    foreach ($Sample in $Samples) {
        if ((Get-SpikeDField $Sample 'index') -ne $Index) { $Failures.Add('sample windows must be ordered 1, 2, 3') }
        $Index++
        if ((Get-SpikeDField $Sample 'frames') -ne 300) { $Failures.Add('each sample window requires exactly 300 frames') }
        foreach ($Field in @('interval_avg_ms', 'interval_p95_ms', 'input_ack_avg_ms', 'input_ack_p95_ms', 'event_to_present_avg_ms', 'event_to_present_p95_ms')) {
            if (-not (Test-SpikeDFiniteNumber (Get-SpikeDField $Sample $Field) -AllowZero:($Field -like '*ack*' -or $Field -like 'event_to_present*'))) { $Failures.Add("missing or invalid timing: $Field") }
        }
        foreach ($Field in @('pointer_events', 'input_ack_count', 'event_to_present_count')) {
            if (-not (Test-SpikeDCount (Get-SpikeDField $Sample $Field))) { $Failures.Add("received/acknowledged input required in every window: $Field") }
        }
        if (-not (Test-SpikeDCount (Get-SpikeDField $Sample 'keyboard_events'))) { $Failures.Add('keyboard event counter missing or invalid') }
        if ((Get-SpikeDField $Sample 'event_to_present_count') -ne ((Get-SpikeDField $Sample 'pointer_events') + (Get-SpikeDField $Sample 'keyboard_events'))) { $Failures.Add('input-to-presentation count does not match actual received events') }
        if ((Get-SpikeDField $Sample 'input_ack_count') -gt ((Get-SpikeDField $Sample 'pointer_events') + (Get-SpikeDField $Sample 'keyboard_events'))) { $Failures.Add('input acknowledgments exceed actual received events') }
        $ActualMode = Get-SpikeDField $Sample 'mode'
        $NextMode = Get-SpikeDField $Sample 'next_mode'
        if ($ActualMode -ne $ExpectedMode) { $Failures.Add('sample effect mode differs from requested mode or previous automatic transition') }
        $ExpectedMode = $NextMode
        if ($ActualMode -notin @('full', 'reduced', 'opaque', 'static') -or $NextMode -notin @('full', 'reduced', 'opaque', 'static')) { $Failures.Add('unknown effect mode') }
        if ($Mode -eq 'opaque' -and ($ActualMode -ne 'opaque' -or $NextMode -ne 'opaque')) { $Failures.Add('Reduced Transparency preference changed') }
        if ($Mode -eq 'reduced' -and ($ActualMode -eq 'full' -or $NextMode -eq 'full')) { $Failures.Add('low-quality request upgraded to full') }
        $Average = Get-SpikeDField $Sample 'interval_avg_ms'
        if ((Test-SpikeDFiniteNumber $Average) -and $Average -gt 16.7 -and $ActualMode -eq 'full' -and $NextMode -eq 'full') { $Failures.Add('slow full effects failed to degrade automatically') }
        if ((Test-SpikeDFiniteNumber $Average) -and $Average -gt 25 -and $ActualMode -eq 'reduced' -and $NextMode -eq 'reduced') { $Failures.Add('slow reduced effects failed to become static') }
    }
    return [pscustomobject]@{ failures = @($Failures.ToArray()); start = $Start; samples = $Samples; machine_status = $(if ($Failures.Count) { 'FAIL' } else { 'PASS' }) }
}
function Write-SpikeDJson {
    param($Value, [string]$Path)
    $Json = $Value | ConvertTo-Json -Depth 30
    [IO.File]::WriteAllText($Path, $Json, (New-Object Text.UTF8Encoding($false)))
}
function Read-SpikeDRecords {
    param([string]$Path)
    $Records = @()
    foreach ($Line in [IO.File]::ReadAllLines($Path)) {
        if (-not [string]::IsNullOrWhiteSpace($Line)) { $Records += ($Line | ConvertFrom-Json) }
    }
    return $Records
}
