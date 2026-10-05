# Pure fixture tests only: never physical clean-VM evidence.
param([string]$ArtifactDirectory)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
. (Join-Path $PSScriptRoot 'collector_logic.ps1')
function Assert([bool]$Condition,[string]$Reason) { if (-not $Condition) { throw $Reason } }
$Vm = [pscustomobject]@{ format_name='afterglow-spike-c-vm-provenance';format_version=2;minimum_reader_version=2; installation_kind='clean-windows-vm';clean_recipient_attested=$true;origin='fixture only';base_image='fixture only';note='not physical evidence' }
Assert (@(Test-SpikeCVmProvenance $Vm).Count -eq 0) 'Complete pure VM fixture rejected'
$Vm.clean_recipient_attested = 'true'
Assert (@(Test-SpikeCVmProvenance $Vm).Count -gt 0) 'String attestation incorrectly accepted'
Assert (@(Test-SpikeCVmProvenance ([pscustomobject]@{})).Count -gt 0) 'Missing provenance incorrectly accepted'
$Environment = [pscustomobject]@{os_native_architecture='AMD64';product_type=1;os_caption='Microsoft Windows 11 Pro';process_64_bit=$true;os_architecture='64-bit';hosted_ci_detected=$false;developer_tools=@();virtual_machine_detected=$true}
Assert (@(Test-SpikeCEnvironment $Environment).Count -eq 0) 'Complete pure clean environment fixture rejected'
$Environment.hosted_ci_detected=$true
Assert (@(Test-SpikeCEnvironment $Environment).Count -gt 0) 'Hosted runner incorrectly accepted'
$Environment.hosted_ci_detected=$false
$Environment.developer_tools=@(@{kind='directory';path='C:\fixture-sdk'})
Assert (@(Test-SpikeCEnvironment $Environment).Count -gt 0) 'SDK inventory incorrectly accepted'
$Environment.developer_tools=@()
$Environment.product_type=3
Assert (@(Test-SpikeCEnvironment $Environment).Count -gt 0) 'Windows Server incorrectly accepted'
$Metadata = [pscustomobject]@{format_name='afterglow-spike-c-evidence';format_version=2;minimum_reader_version=2;signed_exe_sha256=('a'*64);resource_sha256=[pscustomobject]@{capsule=('b'*64);icon=('c'*64);'group-icon'=('d'*64);version=('e'*64)};file_version='2.0.0.0';provenance=[pscustomobject]@{repository='tkgo11/afterglow-capsule';source_commit=('1'*40);workflow_path='.github/workflows/spikes.yml';workflow_run_id=[long]1;workflow_run_attempt=1}}
Assert (@(Test-SpikeCMetadata $Metadata).Count -eq 0) 'Complete pure metadata fixture rejected'
$Metadata.signed_exe_sha256='invalid'
Assert (@(Test-SpikeCMetadata $Metadata).Count -gt 0) 'Malformed expected checksum accepted'
$Metadata.signed_exe_sha256=('a'*64)
$Metadata.format_version=1
Assert (@(Test-SpikeCMetadata $Metadata).Count -gt 0) 'Obsolete v1 evidence incorrectly accepted'
Assert ((Get-SpikeCSha256 ([Text.Encoding]::UTF8.GetBytes('abc'))) -eq 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad') 'SHA256 known answer differs'
[byte[]]$Pe = New-Object byte[] 128
$Pe[0]=0x4d;$Pe[1]=0x5a;$Pe[60]=64;$Pe[64]=0x50;$Pe[65]=0x45;$Pe[68]=0x64;$Pe[69]=0x86;$Pe[88]=0x0b;$Pe[89]=2
Assert ((Get-SpikeCArchitecture $Pe) -eq 'x86_64') 'Bounded x64 PE fixture rejected'
$Pe[69]=0
$Rejected=$false
try { Get-SpikeCArchitecture $Pe | Out-Null } catch { $Rejected=$true }
Assert $Rejected 'Wrong architecture incorrectly accepted'
Write-Output 'Spike C pure collector checks passed; these fixtures are not clean-VM evidence.'

if ($ArtifactDirectory) {
    # Actual hosted CI smoke: machine/resource extraction executes but cleanliness
    # must fail. Never use this smoke as clean-recipient acceptance evidence.
    $Scratch = Join-Path ([IO.Path]::GetTempPath()) ('afterglow-ci-collector-smoke-' + [guid]::NewGuid())
    New-Item -ItemType Directory -Path $Scratch | Out-Null
    try {
        $Report = Join-Path $Scratch 'spike-c-clean-vm-result.json'
        & powershell.exe -NoProfile -File (Join-Path $PSScriptRoot 'run_clean_recipient_validation.ps1') -ArtifactDirectory $ArtifactDirectory -Report $Report
        Assert ($LASTEXITCODE -eq 1) 'Hosted CI collector did not fail cleanliness'
        $Observed = Get-Content -LiteralPath $Report -Raw | ConvertFrom-Json
        Assert ($Observed.machine_status -eq 'FAIL') 'Hosted CI collector falsely passed'
        Assert ($Observed.environment.qualifies_clean -eq $false) 'Hosted CI qualifies as clean'
        Assert ($Observed.one_file_execution -eq 'passed') 'Actual standalone resource extraction failed'
        Assert ($Observed.file_version -eq '2.0.0.0') 'Actual native version readback failed'
        Assert ((Test-Path (Join-Path $Scratch 'shell-icon.png'))) 'Actual Shell icon extraction failed'
        Assert ($Observed.shell_icon_sha256 -match '^[a-f0-9]{64}$') 'Actual shell icon hash missing'
        Assert ($null -eq $Observed.shell_icon_observation.correct) 'Hosted CI manufactured visual observation'
        Write-Output 'Hosted CI actual extraction smoke passed; cleanliness correctly FAIL.'
        $global:LASTEXITCODE = 0
    } finally { Remove-Item -LiteralPath $Scratch -Recurse -Force }
}
