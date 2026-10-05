# Spike B — NTS/time evidence on Windows

Status: **BLOCKED — local failure tests only**. Review date: 2026-10-05.
Canonical acceptance: SPEC.md §22 and §148.

## Evaluated stack and environment

Linux x64, Rust 1.90.0; isolated `rkik-nts 1.4.0`, `tokio 1.52.3`, locked
`rustls 0.23.45`, `tokio-rustls 0.26.6`, `ring 0.17.14` and `aes-siv 0.7.0`.
The candidate compiles on the pinned toolchain, but maintainability, Windows
operation and security suitability have not been established. No production
time adapter or fallback is selected. `rcgen 0.14.7` is test-only.

Alternative metadata was inspected: `ntp-proto 1.9.0` exposes an unstable internal
API; `ntp_usg-client 5.0.0` requires a newer Rust toolchain. Those observations do
not establish that no maintainable integration exists. See
[dependencies](../dependencies.md) for licenses/upstreams/security status and
[external assumptions](../external-assumptions.md) for operator candidate provenance.

## Commands and observed results

```sh
cargo test --manifest-path spikes/Cargo.toml --locked -p afterglow-spike-b
```

All four local tests passed. Zero timeout and verification-disable configuration
are rejected; an actual stalled loopback TLS peer times out; an actual untrusted
loopback TLS certificate produces `UnknownIssuer` before key exchange. No
authenticated connection or KE metadata is retained on failure. Disposable test
certificate material stays in memory, is not installed as a trust root, and does
not change the candidate client's verifier.

The [CLI](../../spikes/nts/src/main.rs) performs standard verified NTS-KE,
requires an authenticated NTP result and reports RTT. Provider observations have
bounded waits and fail independently. It is excluded from production and has no
release/CEK access. Verification disabling and key logging features are excluded.

## Acceptance matrix

| SPEC §22 requirement         | Evidence                                                                          |
| ---------------------------- | --------------------------------------------------------------------------------- |
| NTS key exchange             | Live result pending                                                               |
| NTS-authenticated NTP        | Live result pending                                                               |
| RTT measurement              | CLI prepared; live measurement pending                                            |
| Timeout handling             | Actual stalled loopback peer test passed                                          |
| Certificate validation       | Actual untrusted certificate rejection passed; public operator validation pending |
| Independent provider results | Both live results and provenance review pending                                   |
| Windows execution            | Workflow prepared; no observed result yet                                         |

This workspace has no external TCP/UDP grants for NTS. Cloudflare's hostname was
confirmed in public documentation source; Netnod's candidate needs official
confirmation because its documentation site was policy-blocked. Run the
centralized candidates on Windows with permitted NTS-KE and authenticated NTP
transport, preserve both results and confirm distinct operators before acceptance.

**The gate remains open.** If maintainable Windows NTS proves unavailable, document
that evidence as SPEC permits. Time evidence may degrade; the pinned-round
timelock invariant must remain unchanged. This experiment does not authorize
silently substituting local time, unauthenticated NTP or HTTPS time for release.
