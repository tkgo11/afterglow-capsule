# SPIKE ONLY: isolated empty Gen2 firmware capability test, not a Windows guest.
# Creates exactly one GUID-owned VM, 512 MiB, no disk, OS, network switch or media.
param(
    [string]$Report = 'nested-boot-feasibility.json',
    [switch]$SelfTest
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Test-OwnedNestedVm {
    param($Vm, [guid]$Id, [string]$Name, [string]$Scratch)
    if (-not $Vm -or $Id -eq [guid]::Empty -or $Vm.Id -ne $Id -or $Vm.Name -cne $Name) { return $false }
    if ($Name -notmatch '^afterglow-firmware-[a-f0-9]{32}$') { return $false }
    $Root = [IO.Path]::GetFullPath($Scratch).TrimEnd([IO.Path]::DirectorySeparatorChar)
    $VmPath = [IO.Path]::GetFullPath($Vm.Path).TrimEnd([IO.Path]::DirectorySeparatorChar)
    return ($VmPath.Equals($Root, [StringComparison]::OrdinalIgnoreCase) -or $VmPath.StartsWith($Root + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase))
}

if ($SelfTest) {
    $Tokens = $null
    $ParseErrors = $null
    $Ast = [Management.Automation.Language.Parser]::ParseFile($PSCommandPath, [ref]$Tokens, [ref]$ParseErrors)
    if ($ParseErrors.Count -ne 0) { throw 'Firmware diagnostic has PowerShell parser errors.' }
    $Forbidden = @('Install-WindowsFeature','Enable-WindowsOptionalFeature','Set-ExecutionPolicy','Restart-Computer','Stop-Service','Restart-Service','New-VMSwitch','Set-VMSwitch','New-NetNat','Set-VMProcessor','Set-VMHost','New-VHD','Mount-VHD','Add-VMDvdDrive','Invoke-WebRequest','Invoke-Expression')
    foreach ($Command in $Ast.FindAll({param($Node) $Node -is [Management.Automation.Language.CommandAst]}, $true)) {
        $CommandName = $Command.GetCommandName()
        if ($CommandName) { $CommandName = ($CommandName -split '\\')[-1] }
        if ($Forbidden -contains $CommandName) { throw ('Forbidden global/media operation: ' + $CommandName) }
        if ($CommandName -in @('Start-VM','Stop-VM','Remove-VM')) {
            $Parameters = @($Command.CommandElements | Where-Object { $_ -is [Management.Automation.Language.CommandParameterAst] } | ForEach-Object { $_.ParameterName })
            if ($Parameters -notcontains 'VM' -or $Parameters -contains 'Name') { throw 'VM mutations must use an explicitly validated VM object, never name wildcards.' }
            for ($Index = 1; $Index -lt $Command.CommandElements.Count; $Index++) {
                $Element = $Command.CommandElements[$Index]
                if ($Element -is [Management.Automation.Language.CommandParameterAst] -and $Element.ParameterName -eq 'VM') {
                    $Argument = $Command.CommandElements[$Index + 1]
                    if ($Argument -isnot [Management.Automation.Language.VariableExpressionAst] -or $Argument.VariablePath.UserPath -cne 'Vm') { throw 'VM mutation must use the checked local Vm object.' }
                }
            }
        }
        if ($CommandName -eq 'Remove-Item') {
            $Parameters = @($Command.CommandElements | Where-Object { $_ -is [Management.Automation.Language.CommandParameterAst] } | ForEach-Object { $_.ParameterName })
            if ($Parameters -notcontains 'LiteralPath' -or $Parameters -contains 'Path') { throw 'Scratch cleanup must use one literal owned path.' }
            for ($Index = 1; $Index -lt $Command.CommandElements.Count; $Index++) {
                $Element = $Command.CommandElements[$Index]
                if ($Element -is [Management.Automation.Language.CommandParameterAst] -and $Element.ParameterName -eq 'LiteralPath') {
                    $Argument = $Command.CommandElements[$Index + 1]
                    if ($Argument -isnot [Management.Automation.Language.VariableExpressionAst] -or $Argument.VariablePath.UserPath -cne 'Scratch') { throw 'Scratch cleanup cannot remove arbitrary paths.' }
                }
            }
        }
        if ($CommandName -eq 'Get-VM') {
            $Parameters = @($Command.CommandElements | Where-Object { $_ -is [Management.Automation.Language.CommandParameterAst] } | ForEach-Object { $_.ParameterName })
            if ($Parameters -notcontains 'Id' -and $Parameters -notcontains 'Name') { throw 'Firmware diagnostic must not enumerate or mutate arbitrary existing VMs.' }
        }
    }
    $TestId = [guid]::NewGuid()
    $Name = 'afterglow-firmware-' + $TestId.ToString('N')
    $Root = Join-Path ([IO.Path]::GetTempPath()) $Name
    $Fixture = [pscustomobject]@{Id=$TestId;Name=$Name;Path=$Root}
    if (-not (Test-OwnedNestedVm $Fixture $TestId $Name $Root)) { throw 'Exact owned VM rejected.' }
    if (Test-OwnedNestedVm $Fixture ([guid]::NewGuid()) $Name $Root) { throw 'Foreign VM ID accepted.' }
    if (Test-OwnedNestedVm $Fixture $TestId 'existing-vm' $Root) { throw 'Foreign VM name accepted.' }
    $Fixture.Path = $Root + '-other'
    if (Test-OwnedNestedVm $Fixture $TestId $Name $Root) { throw 'Sibling scratch-prefix path accepted.' }
    $Fixture.Path = Join-Path $Root 'child'
    if (-not (Test-OwnedNestedVm $Fixture $TestId $Name $Root)) { throw 'Owned VM child directory rejected.' }
    Write-Output 'PASS: parser, infrastructure-operation guard and exact ID/name/path ownership cases.'
    exit 0
}

$RunId = [guid]::NewGuid()
$Name = 'afterglow-firmware-' + $RunId.ToString('N')
$Scratch = Join-Path ([IO.Path]::GetTempPath()) $Name
$OwnedId = [guid]::Empty
$Created = $false
$StartJob = $null
$OperationJob = $null
$Problems = New-Object 'Collections.Generic.List[string]'
$CleanupProblems = New-Object 'Collections.Generic.List[string]'
$Result = [ordered]@{
    format_name='afterglow-nested-boot-feasibility';format_version=1;minimum_reader_version=1
    collected_utc=[DateTime]::UtcNow.ToString('o');status='PENDING';qualifies_clean_recipient=$false
    run_id=$RunId.ToString();vm_id=$null;vm_name=$Name;generation=2;memory_bytes=536870912
    has_vhd=$false;has_os=$false;has_media=$false;has_network_switch=$false
    start_timeout_seconds=45;start_attempted=$false;observed_vm_state=$null;boot_capability='NOT_TESTED'
    cleanup_status='PENDING';problems=@();cleanup_problems=@()
    operations='One new GUID-owned empty firmware VM only; no Windows image, license acceptance, account, host trust/display/network/feature/reboot changes.'
    limitation='Running proves nested firmware VM execution only; no clean Windows guest or Spike C evidence exists.'
}

if (Test-Path -LiteralPath $Report) { throw 'Report already exists; supply a new path to preserve earlier capability evidence.' }
try {
    Import-Module Hyper-V -ErrorAction Stop
    if (Test-Path -LiteralPath $Scratch) { throw 'GUID scratch path already exists; refusing to reuse it.' }
    if (Hyper-V\Get-VM -Name $Name -ErrorAction SilentlyContinue) { throw 'GUID VM name already exists; refusing to use it.' }
    New-Item -ItemType Directory -Path $Scratch | Out-Null
    $Created = $true
    $Vm = Hyper-V\New-VM -Name $Name -Generation 2 -MemoryStartupBytes 512MB -NoVHD -Path $Scratch -ErrorAction Stop
    $OwnedId = [guid]$Vm.Id
    $Result.vm_id = $OwnedId.ToString()
    if (-not (Test-OwnedNestedVm $Vm $OwnedId $Name $Scratch)) { throw 'Created VM does not match exact ID/name/scratch ownership.' }
    if (@(Hyper-V\Get-VMHardDiskDrive -VM $Vm).Count -ne 0) { throw 'Unexpected disk attached to empty firmware VM.' }
    if (@(Hyper-V\Get-VMDvdDrive -VM $Vm | Where-Object { $_.Path }).Count -ne 0) { throw 'Unexpected media attached to empty firmware VM.' }
    if (@(Hyper-V\Get-VMNetworkAdapter -VM $Vm | Where-Object { $_.SwitchName }).Count -ne 0) { throw 'Unexpected network switch attached to empty firmware VM.' }
    $StartJob = Start-Job -ArgumentList $OwnedId.ToString(),$Name,$Scratch -ScriptBlock {
        param($ExpectedId,$ExpectedName,$ExpectedScratch)
        $ErrorActionPreference = 'Stop'
        Import-Module Hyper-V -ErrorAction Stop
        $Vm = Hyper-V\Get-VM -Id ([guid]$ExpectedId) -ErrorAction Stop
        $Root = [IO.Path]::GetFullPath($ExpectedScratch).TrimEnd([IO.Path]::DirectorySeparatorChar)
        $Path = [IO.Path]::GetFullPath($Vm.Path).TrimEnd([IO.Path]::DirectorySeparatorChar)
        if ($Vm.Id -ne [guid]$ExpectedId -or $Vm.Name -cne $ExpectedName -or -not ($Path.Equals($Root,[StringComparison]::OrdinalIgnoreCase) -or $Path.StartsWith($Root+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase))) { throw 'Child start refused foreign VM.' }
        [pscustomobject]@{event='start_attempt';vm_id=$Vm.Id.ToString()}
        Hyper-V\Start-VM -VM $Vm -ErrorAction Stop
        $Observed = Hyper-V\Get-VM -Id ([guid]$ExpectedId) -ErrorAction Stop
        [pscustomobject]@{event='start_observed';vm_id=$Observed.Id.ToString();state=$Observed.State.ToString()}
    }
    $Finished = Wait-Job -Job $StartJob -Timeout 45
    $StartErrors = @()
    $Output = @(Receive-Job -Job $StartJob -ErrorAction SilentlyContinue -ErrorVariable +StartErrors)
    foreach ($Item in $Output) {
        if ($Item.event -eq 'start_attempt' -and $Item.vm_id -eq $OwnedId.ToString()) { $Result.start_attempted = $true }
        if ($Item.event -eq 'start_observed' -and $Item.vm_id -eq $OwnedId.ToString()) { $Result.observed_vm_state = $Item.state }
    }
    foreach ($Problem in $StartErrors) { $Problems.Add($Problem.ToString()) }
    if (-not $Finished) { $Problems.Add('Actual nested start did not complete within the 45-second bound.'); Stop-Job -Job $StartJob }
    if ($Result.start_attempted) {
        if ($Finished -and $StartJob.State -eq 'Completed' -and $Result.observed_vm_state -eq 'Running') { $Result.boot_capability = 'AVAILABLE' }
        else { $Result.boot_capability = 'UNAVAILABLE' }
    }
} catch { $Problems.Add($_.Exception.Message) }
finally {
    if ($StartJob) {
        try {
            if ($StartJob.State -in @('Running','NotStarted')) { Stop-Job -Job $StartJob }
            Remove-Job -Job $StartJob -Force
        } catch { $CleanupProblems.Add('Start job cleanup: ' + $_.Exception.Message) }
    }
    # A failed New-VM may leave a registration. Recover only the exact unique
    # name inside our newly created scratch tree; never touch a preexisting VM.
    if ($Created -and $OwnedId -eq [guid]::Empty) {
        try {
            $Partial = Hyper-V\Get-VM -Name $Name -ErrorAction SilentlyContinue
            if ($Partial) {
                if (-not (Test-OwnedNestedVm $Partial ([guid]$Partial.Id) $Name $Scratch)) { throw 'Partial registration ownership could not be verified.' }
                $OwnedId = [guid]$Partial.Id
                $Result.vm_id = $OwnedId.ToString()
            }
        } catch { $CleanupProblems.Add('Partial VM inventory: ' + $_.Exception.Message) }
    }
    if ($OwnedId -ne [guid]::Empty) {
        try {
            $Owned = Hyper-V\Get-VM -Id $OwnedId -ErrorAction Stop
            if (-not (Test-OwnedNestedVm $Owned $OwnedId $Name $Scratch)) { throw 'Cleanup refused foreign VM ID/name/path.' }
            $OperationJob = Start-Job -ArgumentList $OwnedId.ToString(),$Name,$Scratch -ScriptBlock {
                param($ExpectedId,$ExpectedName,$ExpectedScratch)
                $ErrorActionPreference = 'Stop'
                Import-Module Hyper-V -ErrorAction Stop
                $Vm = Hyper-V\Get-VM -Id ([guid]$ExpectedId) -ErrorAction Stop
                $Root = [IO.Path]::GetFullPath($ExpectedScratch).TrimEnd([IO.Path]::DirectorySeparatorChar)
                $Path = [IO.Path]::GetFullPath($Vm.Path).TrimEnd([IO.Path]::DirectorySeparatorChar)
                if ($Vm.Id -ne [guid]$ExpectedId -or $Vm.Name -cne $ExpectedName -or -not ($Path.Equals($Root,[StringComparison]::OrdinalIgnoreCase) -or $Path.StartsWith($Root+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase))) { throw 'Child cleanup refused foreign VM.' }
                if ($Vm.State.ToString() -ne 'Off') { Hyper-V\Stop-VM -VM $Vm -TurnOff -Force -ErrorAction Stop }
                $Vm = Hyper-V\Get-VM -Id ([guid]$ExpectedId) -ErrorAction Stop
                if ($Vm.State.ToString() -ne 'Off') { throw 'Owned firmware VM is still running.' }
                Hyper-V\Remove-VM -VM $Vm -Force -ErrorAction Stop
            }
            if (-not (Wait-Job -Job $OperationJob -Timeout 45)) { throw 'Owned VM stop/removal exceeded the 45-second cleanup bound.' }
            Receive-Job -Job $OperationJob -ErrorAction Stop | Out-Null
            if ($OperationJob.State -ne 'Completed') { throw 'Owned VM cleanup job did not complete successfully.' }
            if (Hyper-V\Get-VM -Id $OwnedId -ErrorAction SilentlyContinue) { throw 'Owned VM registration still exists after removal.' }
        } catch { $CleanupProblems.Add('Owned VM cleanup: ' + $_.Exception.Message) }
        finally {
            if ($OperationJob) {
                try {
                    if ($OperationJob.State -in @('Running','NotStarted')) { Stop-Job -Job $OperationJob }
                    Remove-Job -Job $OperationJob -Force
                } catch { $CleanupProblems.Add('Cleanup job removal: ' + $_.Exception.Message) }
            }
        }
    }
    # Preserve files on cleanup failure; deleting live VM configuration is unsafe.
    if ($Created -and $CleanupProblems.Count -eq 0) {
        try {
            Remove-Item -LiteralPath $Scratch -Recurse -Force
            if (Test-Path -LiteralPath $Scratch) { throw 'Owned scratch directory remains.' }
        } catch { $CleanupProblems.Add('Owned scratch cleanup: ' + $_.Exception.Message) }
    }
    $Result.problems = @($Problems.ToArray())
    $Result.cleanup_problems = @($CleanupProblems.ToArray())
    $Result.cleanup_status = $(if ($CleanupProblems.Count -eq 0) { 'PASS' } else { 'FAIL' })
    $Result | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath $Report -Encoding UTF8
    Write-Output ('PENDING: nested firmware boot capability ' + $Result.boot_capability + '; cleanup ' + $Result.cleanup_status + '; report ' + $Report)
}
if ($CleanupProblems.Count -gt 0) { exit 1 }
exit 0
