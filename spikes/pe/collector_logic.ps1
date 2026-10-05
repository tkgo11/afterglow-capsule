# SPIKE ONLY: pure, testable evidence checks. Fixtures in tests are not VM evidence.
Set-StrictMode -Version Latest
function Test-SpikeCMetadata($Metadata) {
    $Reasons = New-Object 'Collections.Generic.List[string]'
    try {
        if ($Metadata.format_name -ne 'afterglow-spike-c-evidence' -or $Metadata.format_version -ne 2 -or $Metadata.minimum_reader_version -ne 2) { $Reasons.Add('Unsupported evidence metadata; v1 artifacts are obsolete.') }
        if ($Metadata.signed_exe_sha256 -notmatch '^[a-fA-F0-9]{64}$') { $Reasons.Add('Invalid expected EXE SHA-256.') }
        if ($Metadata.file_version -ne '2.0.0.0') { $Reasons.Add('Expected native version must be 2.0.0.0.') }
        foreach ($Kind in @('capsule', 'icon', 'group-icon', 'version')) {
            if ($Metadata.resource_sha256.$Kind -notmatch '^[a-fA-F0-9]{64}$') { $Reasons.Add("Invalid expected $Kind SHA-256.") }
        }
        $P = $Metadata.provenance
        if ($P.repository -ne 'tkgo11/afterglow-capsule' -or $P.source_commit -notmatch '^[a-fA-F0-9]{40}$' -or $P.workflow_path -ne '.github/workflows/spikes.yml') { $Reasons.Add('Invalid repository/source/workflow provenance.') }
        foreach ($Field in @('workflow_run_id', 'workflow_run_attempt')) {
            $Value = $P.$Field
            if (($Value -isnot [int] -and $Value -isnot [long]) -or $Value -le 0) { $Reasons.Add("Invalid integer $Field provenance.") }
        }
    } catch { $Reasons.Add('Required evidence metadata is absent or malformed: ' + $_.Exception.Message) }
    return $Reasons.ToArray()
}
function Test-SpikeCVmProvenance($Vm) {
    $Reasons = New-Object 'Collections.Generic.List[string]'
    try {
        if ($Vm.format_name -ne 'afterglow-spike-c-vm-provenance' -or $Vm.format_version -ne 2 -or $Vm.minimum_reader_version -ne 2) { $Reasons.Add('Unsupported VM provenance format; version 2 is required.') }
        if ($Vm.installation_kind -ne 'clean-windows-vm') { $Reasons.Add('VM installation_kind must be clean-windows-vm.') }
        if ($Vm.clean_recipient_attested -isnot [bool] -or $Vm.clean_recipient_attested -ne $true) { $Reasons.Add('Genuine clean-recipient attestation is required.') }
        foreach ($Field in @('origin', 'base_image', 'note')) {
            if ($Vm.$Field -isnot [string] -or [string]::IsNullOrWhiteSpace($Vm.$Field)) { $Reasons.Add("VM provenance $Field is required.") }
        }
    } catch { $Reasons.Add('VM provenance is malformed: ' + $_.Exception.Message) }
    return $Reasons.ToArray()
}
function Test-SpikeCEnvironment($Environment) {
    $Reasons = New-Object 'Collections.Generic.List[string]'
    try {
        if ($Environment.product_type -ne 1 -or $Environment.os_caption -notmatch 'Windows (10|11)\b') { $Reasons.Add('Recipient must be Windows 10/11 client, not Server.') }
        if ($Environment.os_native_architecture -ne 'AMD64') { $Reasons.Add('Native Windows architecture must be AMD64, not ARM emulation.') }
        if ($Environment.process_64_bit -ne $true -or $Environment.os_architecture -notmatch '64') { $Reasons.Add('Windows x64 and a 64-bit PowerShell process are required.') }
        if ($Environment.hosted_ci_detected -ne $false) { $Reasons.Add('Hosted CI is not a clean recipient VM.') }
        if (@($Environment.developer_tools).Count -gt 0) { $Reasons.Add('Developer tooling is installed; use a fresh clean recipient VM.') }
        if ($Environment.virtual_machine_detected -ne $true) { $Reasons.Add('Windows machine does not report recognized VM hardware provenance.') }
    } catch { $Reasons.Add('Environment inventory is incomplete: ' + $_.Exception.Message) }
    return $Reasons.ToArray()
}
function Get-SpikeCSha256([byte[]]$Bytes) {
    $Hasher = [Security.Cryptography.SHA256]::Create()
    try { return [BitConverter]::ToString($Hasher.ComputeHash($Bytes)).Replace('-', '').ToLowerInvariant() }
    finally { $Hasher.Dispose() }
}
function Get-SpikeCArchitecture([byte[]]$Bytes) {
    if ($Bytes.Length -lt 64 -or $Bytes[0] -ne 0x4d -or $Bytes[1] -ne 0x5a) { throw 'Invalid DOS/PE header.' }
    $Offset = [BitConverter]::ToUInt32($Bytes, 60)
    if ($Offset -gt ($Bytes.Length - 26) -or [BitConverter]::ToUInt32($Bytes, [int]$Offset) -ne 0x4550) { throw 'Invalid bounded PE signature.' }
    if ([BitConverter]::ToUInt16($Bytes, [int]$Offset + 4) -ne 0x8664 -or [BitConverter]::ToUInt16($Bytes, [int]$Offset + 24) -ne 0x20b) { throw 'Expected x86_64 PE32+ architecture.' }
    return 'x86_64'
}

function New-SpikeCSandboxConfiguration([string]$InputDirectory,[string]$OutputDirectory) {
    $EscapedInput = [Security.SecurityElement]::Escape($InputDirectory)
    $EscapedOutput = [Security.SecurityElement]::Escape($OutputDirectory)
    # RemoteSigned applies to this disposable guest process only. It is a supported
    # scoped policy, not Bypass/Unrestricted; Group Policy restrictions remain effective.
    return @"
<Configuration>
  <VGpu>Disable</VGpu>
  <Networking>Disable</Networking>
  <ClipboardRedirection>Disable</ClipboardRedirection>
  <PrinterRedirection>Disable</PrinterRedirection>
  <MappedFolders>
    <MappedFolder><HostFolder>$EscapedInput</HostFolder><SandboxFolder>C:\afterglow-input</SandboxFolder><ReadOnly>true</ReadOnly></MappedFolder>
    <MappedFolder><HostFolder>$EscapedOutput</HostFolder><SandboxFolder>C:\afterglow-output</SandboxFolder><ReadOnly>false</ReadOnly></MappedFolder>
  </MappedFolders>
  <LogonCommand><Command>powershell.exe -NoProfile -ExecutionPolicy RemoteSigned -Command &quot;&amp; 'C:\afterglow-input\run_clean_recipient_validation.ps1' -ArtifactDirectory 'C:\afterglow-input' -Report 'C:\afterglow-output\spike-c-clean-vm-result.json' *&gt; 'C:\afterglow-output\collector-console.log'&quot;</Command></LogonCommand>
</Configuration>
"@
}
