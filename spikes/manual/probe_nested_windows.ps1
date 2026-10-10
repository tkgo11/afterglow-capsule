# SPIKE ONLY. Read-only host capability research, never clean-recipient evidence.
# No VM creation/start, installation, license acceptance, account login or reboot.
param(
    [string]$Report = 'nested-windows-feasibility.json',
    [switch]$SelfTest
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Get-NestedWindowsResearch {
    [ordered]@{
        reviewed_date = '2026-10-07'
        evaluation_page = 'https://www.microsoft.com/en-us/evalcenter/download-windows-11-enterprise'
        official_download_link = 'https://go.microsoft.com/fwlink/?LinkId=2382600&clcid=0x409&country=us&culture=en-us'
        observed_download_redirect = 'https://software-static.download.prss.microsoft.com/dbazure/26300.9457.260913-1737.26h2_ge_release_svc_refresh_CLIENTENTERPRISEEVAL_OEMRET_x64FRE_en-us.iso'
        hash_source = 'https://support.microsoft.com/en-us/servicing/os/windows/docs/2026/09/verify-the-authenticity-of-a-windows-11-enterprise-evaluation-iso-file'
        hash_source_kb = '5130867'
        image_description = 'Windows 11 Enterprise version 26H2 Evaluation x64 EN-US DVD9'
        advertised_iso_sha256 = 'bc3f24086ebadc94489066b5ad78089e2cf5c3491e90e790bb81a2b199c10e38'
        iso_downloaded_or_verified = $false
        terms_accepted = $false
        evaluation_requirements = @(
            'Microsoft evaluation page specifies registration and a 90-day evaluation.'
            'Review and comply with applicable evaluation terms before installation; no terms have been accepted by this probe.'
            'Published installation guidelines specify Microsoft account sign-in; no account or registration is supplied by this probe.'
            'Guest must satisfy supported Windows 11 VM requirements: generation 2, Secure Boot, virtual TPM, at least 4 GiB RAM, 2 virtual processors and 64 GiB disk.'
        )
        sources = @(
            'https://learn.microsoft.com/en-us/windows-server/virtualization/hyper-v/enable-nested-virtualization'
            'https://learn.microsoft.com/en-us/windows/whats-new/windows-11-requirements'
            'https://learn.microsoft.com/en-us/windows-server/virtualization/hyper-v/powershell-direct'
            'https://docs.github.com/en/actions/reference/runners/larger-runners'
        )
    }
}

if ($SelfTest) {
    $Tokens = $null
    $Errors = $null
    $Ast = [Management.Automation.Language.Parser]::ParseFile($PSCommandPath, [ref]$Tokens, [ref]$Errors)
    if ($Errors.Count -ne 0) { throw 'Read-only diagnostic script has PowerShell parser errors.' }
    # Guard the reviewed diagnostic against accidental infrastructure mutations.
    $Forbidden = @('New-VM','Start-VM','Stop-VM','Remove-VM','Set-VM','Set-VMProcessor','New-VHD','Mount-VHD','Install-WindowsFeature','Enable-WindowsOptionalFeature','New-VMSwitch','New-NetNat','Restart-Computer','Set-ExecutionPolicy','Invoke-Expression')
    foreach ($Command in $Ast.FindAll({param($Node) $Node -is [Management.Automation.Language.CommandAst]}, $true)) {
        if ($Forbidden -contains $Command.GetCommandName()) { throw ('Infrastructure mutation in read-only probe: ' + $Command.GetCommandName()) }
    }
    $Research = Get-NestedWindowsResearch
    if ($Research.advertised_iso_sha256 -notmatch '^[a-f0-9]{64}$' -or $Research.iso_downloaded_or_verified -or $Research.terms_accepted) { throw 'Research provenance must distinguish advertised values from verified media/accepted terms.' }
    Write-Output 'PASS: parser, read-only command guard, and unverified evaluation-media provenance.'
    exit 0
}

$Errors = New-Object 'Collections.Generic.List[string]'
$Os = $null
$Computer = $null
$Processors = @()
$Drives = @()
$Vmms = $null
$VmHost = $null
$GuestCount = $null
try {
    $ActualOs = Get-CimInstance Win32_OperatingSystem
    $Os = @{caption=$ActualOs.Caption;version=$ActualOs.Version;build=$ActualOs.BuildNumber;product_type=[int]$ActualOs.ProductType}
} catch { $Errors.Add('OS inventory: ' + $_.Exception.Message) }
try {
    $ActualComputer = Get-CimInstance Win32_ComputerSystem
    $Computer = @{manufacturer=$ActualComputer.Manufacturer;model=$ActualComputer.Model;hypervisor_present=[bool]$ActualComputer.HypervisorPresent;total_physical_memory=$ActualComputer.TotalPhysicalMemory}
} catch { $Errors.Add('Computer inventory: ' + $_.Exception.Message) }
try {
    $Processors = @(Get-CimInstance Win32_Processor | ForEach-Object {
        @{name=$_.Name;architecture=$_.Architecture;logical_processors=$_.NumberOfLogicalProcessors;virtualization_firmware_enabled=$_.VirtualizationFirmwareEnabled;vm_monitor_mode_extensions=$_.VMMonitorModeExtensions;second_level_address_translation_extensions=$_.SecondLevelAddressTranslationExtensions}
    })
} catch { $Errors.Add('Processor inventory: ' + $_.Exception.Message) }
try {
    $Drives = @(Get-CimInstance Win32_LogicalDisk -Filter 'DriveType=3' | ForEach-Object { @{device=$_.DeviceID;size_bytes=$_.Size;free_bytes=$_.FreeSpace} })
} catch { $Errors.Add('Disk inventory: ' + $_.Exception.Message) }
try {
    $Service = Get-Service -Name vmms -ErrorAction Stop
    $Vmms = @{name=$Service.Name;status=$Service.Status.ToString()}
} catch { $Errors.Add('Hyper-V vmms service: ' + $_.Exception.Message) }
$Commands = @{}
foreach ($Name in @('Get-VMHost','Get-VM','New-VM','Start-VM','New-PSSession','WindowsSandbox.exe')) {
    $Commands[$Name] = [bool](Get-Command $Name -ErrorAction SilentlyContinue)
}
if ($Commands['Get-VMHost']) {
    try {
        $HostInfo = Get-VMHost -ErrorAction Stop
        $VmHost = @{query_succeeded=$true;logical_processor_count=$HostInfo.LogicalProcessorCount;memory_capacity=$HostInfo.MemoryCapacity;virtual_machine_path=$HostInfo.VirtualMachinePath;virtual_hard_disk_path=$HostInfo.VirtualHardDiskPath}
    } catch { $Errors.Add('Get-VMHost: ' + $_.Exception.Message) }
}
if ($Commands['Get-VM']) {
    try { $GuestCount = @(Get-VM -ErrorAction Stop).Count } catch { $Errors.Add('Get-VM: ' + $_.Exception.Message) }
}
[ordered]@{
    format_name = 'afterglow-nested-windows-feasibility'; format_version = 1; minimum_reader_version = 1
    collected_utc = [DateTime]::UtcNow.ToString('o'); operating_system = $Os; computer = $Computer
    processors = $Processors; disks = $Drives; vmms_service = $Vmms; commands_available = $Commands
    vm_host = $VmHost; existing_guest_count = $GuestCount; diagnostic_errors = @($Errors.ToArray())
    research = Get-NestedWindowsResearch
    status = 'PENDING'; qualifies_clean_recipient = $false; nested_guest_boot_tested = $false
    reasons = @(
        'Module, processor flags, host query and service availability alone do not prove a nested guest can boot.'
        'No clean Windows client guest was installed or exercised; no guest provenance, collector report or visual observation exists.'
    )
    operations = 'Read-only host queries and report writing only; no media download, license acceptance, account login, feature changes, VM provisioning, guest execution or reboot.'
} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $Report -Encoding UTF8
Write-Output ('PENDING: nested Windows capability diagnostics saved to ' + $Report)
