# Clean Windows client validation in cloud

Research date: 2026-10-07. Status: **PENDING**. This is feasibility research,
not Spike C acceptance and not a change to SPEC.md §23 or §148.

The current hosted test machine is Windows Server 2025 with developer tooling.
It cannot qualify as the clean Windows 10/11 recipient guest. The collected host
inventory reports Hyper-V features and `Get-VM`; absence of Windows Sandbox alone
does not establish that nested Hyper-V is unavailable. The read-only
`spikes/manual/probe_nested_windows.ps1` records processor virtualization flags,
VM management service state, actual `Get-VMHost`/`Get-VM` query results, and free
disk space. It always reports PENDING because none of these queries proves a
clean guest was booted or tested.

The separate `spikes/manual/probe_nested_boot.ps1` can test execution rather than
infer capability from flags. It creates one uniquely named, exact-ID-owned
generation-2 VM with 512 MiB RAM, no VHD, OS, boot media or network switch. A
bounded background job attempts to start its firmware; AVAILABLE requires the
actual owned VM to reach Running. A failed actual start records UNAVAILABLE for
that allocated host attempt; setup failures remain NOT_TESTED. The probe stops
and removes only its exact owned VM and scratch tree, preserving cleanup failures
in the report and failing CI if cleanup cannot be verified. It makes no host
feature, trust, network, display or reboot changes. This is an empty firmware
capability probe, **not** installation or execution of a clean Windows client.
Its overall clean-recipient status always remains PENDING, even if firmware runs.

The supported Hyper-V operations used by this isolated diagnostic are documented
by Microsoft: [New-VM with `-NoVHD`](https://learn.microsoft.com/en-us/powershell/module/hyper-v/new-vm?view=windowsserver2025-ps),
[Start-VM with an exact VM object](https://learn.microsoft.com/en-us/powershell/module/hyper-v/start-vm?view=windowsserver2025-ps)
and [Remove-VM with an exact VM object](https://learn.microsoft.com/en-us/powershell/module/hyper-v/remove-vm).

Microsoft documents nested Hyper-V as a supported test/evaluation mechanism.
CPU virtualization extensions must be exposed by the outer host; installing a
module in the runner does not expose them. The outer physical-host operation
is outside this workspace's control. An older official runner-images maintainer
response identified nested support on larger runners, rather than standard
runners; that 2024 statement does not prove the capability of today's allocated
runner. Current larger-runner documentation requires configured access and
billing, neither of which is assumed here.

Sources:

- [Microsoft nested virtualization prerequisites and outer-host configuration](https://learn.microsoft.com/en-us/windows-server/virtualization/hyper-v/enable-nested-virtualization)
- [Official runner-images discussion, 2024 maintainer response](https://github.com/actions/runner-images/discussions/9285)
- [Current GitHub larger-runner specifications and billing/access requirements](https://docs.github.com/en/actions/reference/runners/larger-runners)

Official clean client installation media is available. Microsoft's
[Windows 11 Enterprise Evaluation download page](https://www.microsoft.com/en-us/evalcenter/download-windows-11-enterprise)
currently lists an x64 English (United States) ISO through
`https://go.microsoft.com/fwlink/?LinkId=2382600&clcid=0x409&country=us&culture=en-us`.
The observed redirect is:

```text
https://software-static.download.prss.microsoft.com/dbazure/26300.9457.260913-1737.26h2_ge_release_svc_refresh_CLIENTENTERPRISEEVAL_OEMRET_x64FRE_en-us.iso
```

[Microsoft support KB5130867, published 2026-09-29](https://support.microsoft.com/en-us/servicing/os/windows/docs/2026/09/verify-the-authenticity-of-a-windows-11-enterprise-evaluation-iso-file)
advertises this SHA-256 for Windows 11 Enterprise version 26H2 Evaluation x64
EN-US DVD9:

```text
BC3F24086EBADC94489066B5AD78089E2CF5C3491E90E790BB81A2B199C10E38
```

The official pages and redirect were read during research. The ISO was **not
downloaded or hash-verified**. The redirect is an observation, not an independently
verified media pin. Before use, re-read the official download/verification pages,
download actual media and verify its advertised hash. The evaluation page
specifies registration, a 90-day evaluation and Microsoft account installation
guidelines. No registration, account sign-in or license acceptance was performed.
Applicable evaluation terms must be reviewed and satisfied before installation.
Do not substitute an unofficial preactivated image, developer appliance or
Windows Server container for a clean client VM. The former official developer-VM
download URL currently redirects to general development documentation and does
not supply a clean recipient image.

The smallest credible cloud path, if the allocated host can genuinely boot a
nested guest and evaluation prerequisites are satisfied, is:

1. Install fresh verified official client media in a dedicated generation-2 guest
   with supported Secure Boot, virtual TPM, at least 4 GiB RAM, two virtual
   processors and a 64 GiB disk. Preserve image hash, installation record and VM
   configuration as provenance; install no development tools in the guest.
2. Transfer only the public Spike C executable, expected manifest and collector
   inputs. Run the existing clean-recipient collector **inside the guest**,
   preserving inventory, resource readback, actual one-file execution and Shell
   icon PNG. Hyper-V PowerShell Direct can transfer files/run guest commands
   without a guest network server, once the guest has a configured user profile
   and valid guest credentials.
3. Inspect the actual guest Shell icon image and record the human observation.
   Run the strict evidence validator with separately reviewed artifact provenance.
   A successful host workflow or nested firmware boot is not a Spike C PASS.

Sources:

- [Supported Windows 11 VM configuration](https://learn.microsoft.com/en-us/windows/whats-new/windows-11-requirements)
- [PowerShell Direct requirements, guest commands and file transfer](https://learn.microsoft.com/en-us/windows-server/virtualization/hyper-v/powershell-direct)
- [Former developer VM download URL, now redirected](https://developer.microsoft.com/en-us/windows/downloads/virtual-machines/)

No compliant client guest has yet been executed from Codex Cloud. This is a
specific missing execution/provenance requirement, not a claim that all cloud
Windows virtualization is impossible. The existing supported local Windows
Sandbox collector remains the shorter path when Sandbox is already enabled:
it launches a fresh actual client VM and keeps cleanliness and visual gates intact.

## Observed cloud prerequisites

The [native read-only diagnostic](https://github.com/tkgo11/afterglow-capsule/actions/runs/37495124322/job/112377589885)
passed on source `7b6b54619efee06620c11e9ac6a07ae65845e443`, with its downloaded
[artifact](https://github.com/tkgo11/afterglow-capsule/actions/runs/37495124322/artifacts/11426883267)
ZIP SHA-256 `632f124d863b640fda02bbd017ddf5dbb23666b448460ed1a60c4fa659bf75d4`
independently verified. The allocated Windows Server 2025 VM reported four vCPUs,
16 GiB RAM, a running vmms service, successful Get-VMHost, zero existing guests,
and approximately 147 GiB free on D:. WindowsSandbox.exe was absent. The three
processor virtualization flags were false. Those flags are not interpreted alone
as proof that child execution is impossible; actual firmware execution is tested
separately by the narrowly owned probe above. No client media or guest execution
was represented by this diagnostic report.

The subsequent [actual firmware execution job](https://github.com/tkgo11/afterglow-capsule/actions/runs/37496787894/job/112383261479)
passed on source `deaab7dbc48cdb072943990241071a621af150f4`. Its downloaded
[artifact](https://github.com/tkgo11/afterglow-capsule/actions/runs/37496787894/artifacts/11428365671)
ZIP SHA-256 `e041a2b56d86b567b40e4f51d8e3122c2d3f216aaa861595eba93585937e034f`
was independently verified. The generation-2, empty 512 MiB VM actually reached
Running; `boot_capability` was AVAILABLE, and strict owned-VM cleanup was PASS
with no diagnostic or cleanup problems. Thus this allocated standard hosted
runner can execute nested firmware. No Windows client was installed or run:
`has_os`, `has_media`, `has_vhd` and `qualifies_clean_recipient` all remained false,
and the clean-recipient result remained PENDING. A properly licensed/registered
client image, supported guest installation and configured guest account are still
needed for the cloud recipient route, followed by the existing guest collector
and honest Shell-icon observation.
