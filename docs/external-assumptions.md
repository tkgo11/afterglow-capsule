# External Assumptions

> This file contains **time-sensitive external facts**.  
> Revalidate it before every security-sensitive release.

**Last reviewed:** 2026-10-05

Permanent architecture belongs in [SPEC.md](../SPEC.md). This file exists so current network parameters, endpoints, library status, and service behavior are not mistaken for eternal invariants.

## drand / timelock baseline

Current intended production network profile:

```text
Network: Quicknet
Architecture: unchained
Nominal period: 3 seconds
Chain hash:
52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971
```

Before shipping:

- fetch/verify current official chain information,
- confirm the public key and scheme ID,
- confirm genesis time/period,
- confirm supported public relay endpoints,
- update test vectors,
- rerun Go ↔ Rust interoperability tests.

Official references:

- https://docs.drand.love/docs/timelock-encryption/
- https://docs.drand.love/developer/http-api/
- https://docs.drand.love/developer/API-v2/drand-http-api/
- https://github.com/drand/tlock

## Rust timelock/drand candidates

Candidates currently worth evaluating:

- https://github.com/thibmeu/tlock-rs
- https://github.com/thibmeu/drand-rs

Listing a library here is **not an audit endorsement**.

Production adoption is blocked until it passes the interoperability spike in SPEC.md.

## Time evidence

Preferred authenticated NTS operators:

- Cloudflare NTS
- Netnod NTS

Additional/reference sources:

- NIST Internet Time Service
- Google Public NTP
- Windows local wall clock + monotonic clock

References:

- https://developers.cloudflare.com/time-services/nts/
- https://www.netnod.se/netnod-time/how-to-use-nts
- https://www.nist.gov/pml/time-and-frequency-division/time-distribution/internet-time-service-its
- https://developers.google.com/time
- https://developers.google.com/time/faq
- https://developers.google.com/time/smear

### Google caveat

Google Public NTP uses leap smear and does not provide NTS.

Do not naïvely average Google smeared time with unsmeared UTC during a smear interval.

## Windows PE resources and Authenticode

The canonical packaging approach is:

```text
resource/icon/version mutation
→ insert AGLOW_CAPSULE as RCDATA
→ verify readback
→ optional Authenticode signing
→ no further mutation
```

References:

- https://learn.microsoft.com/windows/win32/api/winbase/nf-winbase-beginupdateresourcew
- https://learn.microsoft.com/windows/win32/api/winbase/nf-winbase-updateresourcew
- https://learn.microsoft.com/windows/win32/api/winbase/nf-winbase-endupdateresourcew
- https://learn.microsoft.com/windows/win32/secbp/understanding-pe-signatures
- https://learn.microsoft.com/windows/win32/debug/pe-format
- https://learn.microsoft.com/windows/win32/seccrypto/signtool
- https://learn.microsoft.com/windows/apps/package-and-deploy/smartscreen-reputation

## Windows accessibility assumptions

Viewer should integrate with current Windows animation/transparency preferences and provide its own explicit reduced-effects fallbacks.

References:

- https://learn.microsoft.com/windows/apps/develop/composition/composition-tailoring
- https://learn.microsoft.com/uwp/api/windows.ui.viewmanagement.uisettings.animationsenabled

## Revalidation checklist

Before a security-sensitive release:

- [ ] drand chain parameters verified from official sources
- [ ] relay list refreshed
- [ ] Go/Rust tlock interoperability tests rerun
- [ ] selected Rust crates reviewed for maintenance/security changes
- [ ] NTS provider behavior retested on Windows
- [ ] Windows signing/tooling behavior retested
- [ ] SmartScreen documentation assumptions reviewed
- [ ] this file's review date updated
