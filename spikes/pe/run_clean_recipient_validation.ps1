# SPIKE ONLY. WinPS 5.1 x64; Windows built-ins only; no admin or trust/policy changes.
# Extract the complete bundle, supply genuine vm-provenance.json, and run this script.
# machine_status covers machine checks; shell_icon_observation remains human-only.
param(
    [string]$ArtifactDirectory = $PSScriptRoot,
    [string]$Report = (Join-Path $ArtifactDirectory 'spike-c-clean-vm-result.json')
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
. (Join-Path $PSScriptRoot 'collector_logic.ps1')
$Failures = New-Object 'Collections.Generic.List[string]'
$Pending = New-Object 'Collections.Generic.List[string]'
$ReportPath = [IO.Path]::GetFullPath($Report)
$ReportFolder = Split-Path $ReportPath -Parent
New-Item -ItemType Directory -Path $ReportFolder -Force | Out-Null
if (Test-Path -LiteralPath $ReportPath) {
    # Preserve previous failed/pending results and their referenced files for audit.
    $History = Join-Path $ReportFolder ('prior-c-evidence-' + [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ') + '-' + [guid]::NewGuid())
    New-Item -ItemType Directory -Path $History | Out-Null
    foreach ($Name in @([IO.Path]::GetFileName($ReportPath), 'shell-icon.png', 'vm-provenance.json', 'SpikeC-Standalone.exe', 'spike-c-expected.json')) {
        $Prior = Join-Path $ReportFolder $Name
        if (Test-Path -LiteralPath $Prior -PathType Leaf) { Copy-Item -LiteralPath $Prior -Destination (Join-Path $History $Name) }
    }
}
$Result = [ordered]@{
    format_name = 'afterglow-spike-c-clean-vm-result'; format_version = 2; minimum_reader_version = 2;
    provenance = $null; exe_sha256 = $null; resource_sha256 = @{}; file_version = $null; architecture = $null;
    one_file_execution = 'pending'; environment = $null; powershell = $PSVersionTable.PSVersion.ToString();
    vm_provenance_file = 'vm-provenance.json'; vm_provenance_sha256 = $null; clean_recipient_vm_attested = $false;
    shell_icon_file = 'shell-icon.png'; shell_icon_sha256 = $null;
    shell_icon_observation = @{correct = $null; note = $null};
    authenticode_status = $null; authenticode_status_message = $null; signer_subject = $null;
    machine_status = 'PENDING'; failures = @(); pending = @()
}
$Scratch = $null
try {
    $ArtifactDirectory = [IO.Path]::GetFullPath($ArtifactDirectory)
    # A supplied provenance file is evidence, never an exemption from actual inventory.
    $VmFile = Join-Path $ArtifactDirectory 'vm-provenance.json'
    if (-not (Test-Path -LiteralPath $VmFile -PathType Leaf)) {
        $Pending.Add('Supply vm-provenance.json from the genuine clean Windows VM installation.')
    } else {
        try {
            $Vm = Get-Content -LiteralPath $VmFile -Raw | ConvertFrom-Json
            $VmProblems = @(Test-SpikeCVmProvenance $Vm)
            foreach ($Problem in $VmProblems) { $Failures.Add($Problem) }
            $Result.clean_recipient_vm_attested = ($VmProblems.Count -eq 0)
            $Result.vm_provenance_sha256 = (Get-FileHash -LiteralPath $VmFile -Algorithm SHA256).Hash.ToLowerInvariant()
            if ([IO.Path]::GetFullPath($VmFile) -ne [IO.Path]::GetFullPath((Join-Path $ReportFolder 'vm-provenance.json'))) {
                Copy-Item -LiteralPath $VmFile -Destination (Join-Path $ReportFolder 'vm-provenance.json') -Force
            }
        } catch { $Failures.Add('Malformed VM provenance: ' + $_.Exception.Message) }
    }
    $Os = Get-CimInstance Win32_OperatingSystem
    $Machine = Get-CimInstance Win32_ComputerSystem
    $Processors = @(Get-CimInstance Win32_Processor)
    $NativeX64 = ($Processors.Count -gt 0 -and @($Processors | Where-Object { $_.Architecture -ne 9 }).Count -eq 0)
    $Tools = New-Object 'Collections.Generic.List[object]'
    $Commands = @('cargo','rustc','rustup','go','node','npm','python','python3','py','signtool','cl','cmake','msbuild','dotnet','gcc','git')
    foreach ($Name in $Commands) {
        foreach ($Command in @(Get-Command $Name -CommandType Application -All -ErrorAction SilentlyContinue)) {
            # Stock Windows python execution aliases are not an installed interpreter.
            if ($Name -match '^python' -and $Command.Source -match '\\Microsoft\\WindowsApps\\python[^\\]*\.exe$') { continue }
            $Tools.Add(@{kind='executable'; name=$Name; path=$Command.Source})
        }
    }
    $Directories = @()
    foreach ($Base in @($env:ProgramFiles, ${env:ProgramFiles(x86)}, $env:LOCALAPPDATA)) {
        if ($Base) { foreach ($Name in @('Microsoft Visual Studio','Windows Kits','Microsoft SDKs','Python','Programs\Python','Go','nodejs','Rust','Microsoft\VisualStudio','Android\Sdk','JetBrains','dotnet\sdk')) { $Directories += (Join-Path $Base $Name) } }
    }
    foreach ($Name in @('.cargo','.rustup','go','.nvm')) { $Directories += (Join-Path $env:USERPROFILE $Name) }
    foreach ($Path in $Directories) { if (Test-Path -LiteralPath $Path) { $Tools.Add(@{kind='directory';name=[IO.Path]::GetFileName($Path);path=$Path}) } }
    foreach ($Registry in @('HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*','HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*','HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*')) {
        foreach ($Package in @(Get-ItemProperty $Registry -ErrorAction SilentlyContinue)) {
            $NameProperty = $Package.PSObject.Properties['DisplayName']
            if ($NameProperty -and $NameProperty.Value -match 'Visual Studio|Windows.*(SDK|Software Development Kit)|Python|Rust|Node\.js|Golang|Go Programming|CMake|MinGW|LLVM|JetBrains|\.NET.*SDK|Git for Windows|Microsoft Build Tools') {
                $Tools.Add(@{kind='installed_package';name=[string]$NameProperty.Value;path=$Package.PSPath})
            }
        }
    }
    foreach ($Path in @('HKLM:\SOFTWARE\Microsoft\Microsoft SDKs\Windows','HKLM:\SOFTWARE\WOW6432Node\Microsoft\Microsoft SDKs\Windows','HKLM:\SOFTWARE\Microsoft\VisualStudio\Setup','HKLM:\SOFTWARE\WOW6432Node\Microsoft\VisualStudio\Setup')) {
        if (Test-Path $Path) { $Tools.Add(@{kind='registry';name='SDK/VisualStudio';path=$Path}) }
    }
    $Hosted = ($env:GITHUB_ACTIONS -eq 'true' -or $env:TF_BUILD -eq 'True' -or [bool]$env:RUNNER_TRACKING_ID -or [bool]$env:AGENT_ID)
    $Environment = [ordered]@{
        os_caption = $Os.Caption; os_build = $Os.BuildNumber; os_version = $Os.Version; os_architecture = $Os.OSArchitecture;
        os_native_architecture = $(if ($NativeX64) { 'AMD64' } else { 'unsupported' });
        product_type = [int]$Os.ProductType; process_64_bit = [Environment]::Is64BitProcess; developer_tools = @($Tools.ToArray());
        hosted_ci_detected = [bool]$Hosted; manufacturer = $Machine.Manufacturer; model = $Machine.Model;
        virtual_machine_detected = [bool]($Machine.Model -match 'Virtual|VMware|KVM|QEMU|VirtualBox|HVM|Parallels|Bochs' -or $Machine.Manufacturer -match 'VMware|Xen|QEMU|innotek|Parallels'); qualifies_clean = $false
    }
    $EnvironmentProblems = @(Test-SpikeCEnvironment ([pscustomobject]$Environment))
    foreach ($Problem in $EnvironmentProblems) { $Failures.Add($Problem) }
    $Environment.qualifies_clean = ($EnvironmentProblems.Count -eq 0 -and $Result.clean_recipient_vm_attested)
    $Result.environment = $Environment
    $Expected = Get-Content -LiteralPath (Join-Path $ArtifactDirectory 'spike-c-expected.json') -Raw | ConvertFrom-Json
    $MetadataProblems = @(Test-SpikeCMetadata $Expected)
    if ($MetadataProblems.Count -gt 0) { throw ($MetadataProblems -join ' ') }
    $Result.provenance = $Expected.provenance
    $Exe = Join-Path $ArtifactDirectory 'SpikeC-Standalone.exe'
    $Result.exe_sha256 = (Get-FileHash -LiteralPath $Exe -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($Result.exe_sha256 -ne $Expected.signed_exe_sha256) { throw 'Signed EXE checksum differs from workflow manifest.' }
    # Fixture executables are small; reject unexpected large inputs before allocating.
    if ((Get-Item -LiteralPath $Exe).Length -gt 16777216) { throw 'Test executable exceeds the 16 MiB bounded fixture limit.' }
    $Result.architecture = Get-SpikeCArchitecture ([IO.File]::ReadAllBytes($Exe))
    $Signature = Get-AuthenticodeSignature -LiteralPath $Exe
    $Result.authenticode_status = $Signature.Status.ToString()
    $Result.authenticode_status_message = $Signature.StatusMessage
    if (-not $Signature.SignerCertificate) { throw 'Embedded disposable test signer is absent.' }
    $Result.signer_subject = $Signature.SignerCertificate.Subject
    # Public disposable self-signing root was removed by CI; no trust changes here.
    Add-Type -AssemblyName System.Drawing
    if (-not ('AfterglowSpikeCNative' -as [type])) {
        Add-Type -Path (Join-Path $PSScriptRoot 'native_collector.cs') -ReferencedAssemblies @('System', 'System.Core', 'System.Drawing')
    }
    $Scratch = Join-Path ([IO.Path]::GetTempPath()) ('afterglow-clean-recipient-' + [guid]::NewGuid())
    New-Item -ItemType Directory -Path $Scratch | Out-Null
    if (@(Get-ChildItem -LiteralPath $Scratch -Force).Count -ne 0) { throw 'New scratch directory is not empty.' }
    $Standalone = Join-Path $Scratch 'SpikeC-Standalone.exe'
    Copy-Item -LiteralPath $Exe -Destination $Standalone
    foreach ($Kind in @('capsule','icon','group-icon','version')) {
        $Bytes = [AfterglowSpikeCNative]::ReadResource($Standalone, $Scratch, $Kind)
        $Digest = Get-SpikeCSha256 $Bytes
        if ($Digest -ne $Expected.resource_sha256.$Kind) { throw "Embedded resource checksum differs for $Kind." }
        $Result.resource_sha256[$Kind] = $Digest
    }
    $Version = [Diagnostics.FileVersionInfo]::GetVersionInfo($Standalone)
    $Result.file_version = '{0}.{1}.{2}.{3}' -f $Version.FileMajorPart,$Version.FileMinorPart,$Version.FileBuildPart,$Version.FilePrivatePart
    if ($Result.file_version -ne '2.0.0.0') { throw 'Native Windows version differs from 2.0.0.0.' }
    if ((Get-FileHash -LiteralPath $Standalone -Algorithm SHA256).Hash.ToLowerInvariant() -ne $Result.exe_sha256) { throw 'Standalone execution changed EXE bytes.' }
    $Contents = @(Get-ChildItem -LiteralPath $Scratch -Force)
    if ($Contents.Count -ne 1 -or $Contents[0].Name -ne 'SpikeC-Standalone.exe') { throw 'Standalone execution generated unexpected project sidecars.' }
    $Result.one_file_execution = 'passed'
    # Verification inputs/evidence copies do not become recipient runtime sidecars.
    # The executable was already exercised alone in a newly empty directory.
    foreach ($Name in @('SpikeC-Standalone.exe','spike-c-expected.json')) {
        $Source = Join-Path $ArtifactDirectory $Name
        $Destination = Join-Path $ReportFolder $Name
        if ([IO.Path]::GetFullPath($Source) -ne [IO.Path]::GetFullPath($Destination)) { Copy-Item -LiteralPath $Source -Destination $Destination -Force }
    }

    $Png = Join-Path $ReportFolder 'shell-icon.png'
    [AfterglowSpikeCNative]::SaveShellIcon($Standalone, $Png)
    $Result.shell_icon_sha256 = (Get-FileHash -LiteralPath $Png -Algorithm SHA256).Hash.ToLowerInvariant()
    $Pending.Add('Inspect shell-icon.png/Explorer icon against the cyan square with white diagonal and fill shell_icon_observation.correct and note in the report.')
} catch {
    $Failures.Add($_.Exception.Message)
} finally {
    if ($Scratch -and (Test-Path -LiteralPath $Scratch)) {
        try { Remove-Item -LiteralPath $Scratch -Recurse -Force } catch { $Failures.Add('Scratch cleanup failed: ' + $_.Exception.Message) }
    }
    $Result.failures = @($Failures.ToArray())
    $Result.pending = @($Pending.ToArray())
    if ($Failures.Count -gt 0) { $Result.machine_status = 'FAIL' }
    elseif (-not $Result.clean_recipient_vm_attested) { $Result.machine_status = 'PENDING' }
    else { $Result.machine_status = 'PASS' }
    $Result | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $ReportPath -Encoding UTF8
    Write-Output ($Result.machine_status + ': canonical collector report ' + $ReportPath)
    foreach ($Reason in $Failures) { Write-Output ('FAIL: ' + $Reason) }
    foreach ($Reason in $Pending) { Write-Output ('PENDING: ' + $Reason) }
}
if ($Failures.Count -gt 0) { exit 1 }
if ($Result.machine_status -eq 'PENDING') { exit 2 }
exit 0
