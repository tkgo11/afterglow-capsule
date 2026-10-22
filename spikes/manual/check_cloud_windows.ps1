# Read-only feasibility evidence. A disposable developer runner is not a clean VM.
param([string]$Report = "cloud-windows-feasibility.json")
$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
$Os = Get-CimInstance Win32_OperatingSystem
$Computer = Get-CimInstance Win32_ComputerSystem
$Tools = @()
foreach ($Name in @("cargo", "rustc", "python", "node", "go", "signtool", "WindowsSandbox", "Get-VM")) {
    $Commands = @(Get-Command $Name -ErrorAction SilentlyContinue)
    foreach ($Command in $Commands) { $Tools += @{ name = $Name; available = $true; path = $Command.Source } }
}
$Features = @(Get-CimInstance Win32_OptionalFeature | Where-Object { $_.Name -match "Hyper-V|Containers-DisposableClientVM" } | ForEach-Object {
    @{ name = $_.Name; install_state = $_.InstallState }
})
$Reasons = @("Hosted runner contains development tooling and has no clean recipient VM provenance.")
if ($Os.ProductType -ne 1) { $Reasons += "Hosted Windows Server is not the Windows 10/11 client recipient baseline." }
if (-not (Get-Command WindowsSandbox.exe -ErrorAction SilentlyContinue)) { $Reasons += "Windows Sandbox executable is unavailable." }
@{
    format_name = "afterglow-cloud-windows-feasibility"; format_version = 1; minimum_reader_version = 1;
    os_caption = $Os.Caption; os_version = $Os.Version; os_build = $Os.BuildNumber; product_type = $Os.ProductType;
    machine_model = $Computer.Model; hypervisor_present = $Computer.HypervisorPresent;
    developer_tools = $Tools; optional_features = $Features;
    qualifies_clean_recipient = $false; status = "PENDING"; reasons = $Reasons;
    operations = "read-only; no feature installation, VM provisioning, credentials, trust changes or reboot"
} | ConvertTo-Json -Depth 6 | Set-Content -Encoding UTF8 $Report
Write-Output "Clean recipient gate remains PENDING; feasibility evidence saved to $Report."
