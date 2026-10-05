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

### Phase 2 historical interoperability baseline

The following **historical** public material was cross-checked against the official
Go `drand/tlock` source at commit
`3ea7fbb59e85d00b0d9b6b2554e9652d5766811b` (`tlock_test.go`,
`TestDecryptText`) and the Rust candidate's Quicknet public key. It is centralized
here so spike programs do not copy chain parameters into source files.

The fixture itself does **not** revalidate current live chain information. Local
relay access returned HTTP 403 under this workspace's network policy on 2026-10-05.
Native Windows CI subsequently verified current chain metadata and real future-round
interoperability, as recorded below and in the Spike A decision. These experimental
pins are not automatically promoted to production defaults; revalidate before shipping.

<!-- BEGIN AFTERGLOW HISTORICAL QUICKNET FIXTURE -->
```json
{
  "format_name": "afterglow-spike-a-public-fixture",
  "format_version": 1,
  "minimum_reader_version": 1,
  "chain": {
    "public_key": "83cf0f2896adee7eb8b5f01fcad3912212c437e0073e911fb90022d3e760183c8c4b450b6a0a6c3ac6a5776a2d1064510d1fec758c921cc22b0e17e63aaf4bcb5ed66304de9cf809bd274ca73bab4af5a6e9c76a4bc09e76eae8991ef5ece45a",
    "period": 3,
    "genesis_time": 1692803367,
    "hash": "52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971",
    "groupHash": "f477d5c89f21a17c863a7f937c6a6d15859414d2be09cd448d4279af331c5d3e",
    "schemeID": "bls-unchained-g1-rfc9380",
    "metadata": { "beaconID": "quicknet" }
  },
  "historical_beacon": {
    "round": 12040883,
    "signature": "929906c959032ab363c9f26570d215d66f5c06cb0c44fe508c12bb5839f04ec895bb6868e5b9ff13ab289bdb5266b394"
  },
  "additional_quicknet_beacon": {
    "round": 1000,
    "signature": "b44679b9a59af2ec876b1a6b1ad52ea9b1615fc3982b19576350f93447cb1125e342b73a8dd2bacbe47e4b6b63ed5e39"
  },
  "historical_foreign_beacon": {
    "round": 1000,
    "signature": "b09eacd45767c4d58306b98901ad0d6086e2663766f3a4ec71d00cf26f0f49eaf248abc7151c60cf419c4e8b37e80412"
  }
}
```
<!-- END AFTERGLOW HISTORICAL QUICKNET FIXTURE -->

The additional round-1000 Quicknet and foreign Fastnet signatures come from the
Rust candidate source at commit `717a5d1d91c7182a5e32f291e65583ab8729edbc`,
`tlock/src/lib.rs`, `test_pk_g2_sig_g1`, under the corresponding RFC9380 feature
branches. They are public historical rejection-test inputs, not production profiles.

## Time evidence

Preferred authenticated NTS operators:

- Cloudflare NTS
- Netnod NTS

### Isolated Phase 2 endpoint candidates

These are experimental inputs, not production defaults or a completed shipping
review. The drand relay is listed by the pinned official Go reference; Cloudflare's
hostname was confirmed in its public documentation source on 2026-10-05. The
original Netnod hostname was listed by the selected NTS candidate's README.
Windows CI fetched Netnod's official documentation on 2026-10-05 and found that
the original alias is no longer listed; the candidate below is now the documented
hostname. That site and the drand
relay returned proxy-policy HTTP 403 in this workspace. Neither NTS operator has
been reached from this Linux environment. Passing TLS and authenticated NTP tests
alone does not establish operator independence or close the shipping review checklist.

Windows Server 2025 x64 CI on 2026-10-05 did reach the pinned drand relay: its
chain metadata matched the fixture and its latest beacon passed BLS verification.
A requested future round returned HTTP 425 (Too Early), so spike polling must
recognize that response as well as 404. Neither status authorizes release. The
first live run stopped at this scheduling mismatch. The same Windows run obtained NTS-authenticated responses
from both candidates with RTTs of 3,394 and 117,472 microseconds respectively.
That run used the original Netnod alias; the documented hostname was retested below.
Evidence: https://github.com/tkgo11/afterglow-capsule/actions/runs/37288622769/job/111693473097

A subsequent Windows run confirmed target-round absence and pre-round rejection,
then received HTTP 500 while polling. Transient server failures may be retried
with a bounded budget, but must never be interpreted as future-round absence or
successful release evidence. The same run fetched both official NTS documentation
pages, which exposed the stale Netnod alias above.
Evidence: https://github.com/tkgo11/afterglow-capsule/actions/runs/37289490311/job/111696272208

The corrected native Windows run passed both real future-round interoperability
directions at target round `32796465`, with pre-round rejection and exact pinned
chain/round BLS verification. Both currently documented NTS operators completed
certificate-verified NTS-KE and authenticated NTP, with RTTs of 18,157 and 127,917
microseconds. The official documentation collector preserved source excerpts and
hashes explicitly associating each selected hostname with its distinct operator.
All live/provenance steps passed; this closes the required A/B technical evidence
without closing the other Phase 2 gates or the shipping review checklist.
Evidence: https://github.com/tkgo11/afterglow-capsule/actions/runs/37290256379/job/111698735744

Sources:

- https://github.com/drand/tlock/blob/3ea7fbb59e85d00b0d9b6b2554e9652d5766811b/README.md
- https://github.com/cloudflare/cloudflare-docs/blob/production/src/content/docs/time-services/nts.mdx
- https://github.com/aguacero7/rkik-nts (published 1.4.0 README)

<!-- BEGIN AFTERGLOW SPIKE ENDPOINT CANDIDATES -->

```json
{
  "format_name": "afterglow-spike-endpoint-candidates",
  "format_version": 1,
  "minimum_reader_version": 1,
  "drand_relay": "https://api.drand.sh",
  "nts_operators": [
    {
      "operator": "Cloudflare",
      "host": "time.cloudflare.com",
      "documentation_url": "https://developers.cloudflare.com/time-services/nts/"
    },
    {
      "operator": "Netnod",
      "host": "nts.netnod.se",
      "documentation_url": "https://www.netnod.se/netnod-time/how-to-use-nts"
    }
  ]
}
```

<!-- END AFTERGLOW SPIKE ENDPOINT CANDIDATES -->

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

The isolated native Windows signing experiment passed sign/verify, exact resource
readback after signing and disposable post-sign mutation rejection on 2026-10-05.
Its ephemeral test certificate is nonexportable and removed with its temporary
public machine trust root. This does not select production publisher trust or
establish SmartScreen reputation. Same-ID stock icon/version replacement also
passed in the strengthened rerun. Explorer icon and clean recipient VM evidence
remain outstanding in the Spike C decision; a hosted developer runner is not a
clean recipient VM.

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
