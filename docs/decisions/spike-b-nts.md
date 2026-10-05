# Spike B — NTS/time evidence on Windows

Status: **PASSED — native Windows, independent providers and failure evidence**.
Review date: 2026-10-05.
Canonical acceptance: SPEC.md §22 and §148.

## Evaluated stack and environment

Linux x64, Rust 1.90.0; isolated `rkik-nts 1.4.0`, `tokio 1.52.3`, locked
`rustls 0.23.45`, `tokio-rustls 0.26.6`, `ring 0.17.14` and `aes-siv 0.7.0`.
The candidate compiles on the pinned toolchain and executes on Windows. The
technical acceptance criteria pass; overall security suitability is not inferred. No production
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

All four local tests also passed in native Windows CI. Zero timeout and verification-disable configuration
are rejected; an actual stalled loopback TLS peer times out; an actual untrusted
loopback TLS certificate produces `UnknownIssuer` before key exchange. No
authenticated connection or KE metadata is retained on failure. Disposable test
certificate material stays in memory, is not installed as a trust root, and does
not change the candidate client's verifier.

Windows Server 2025 10.0.26100 x64, Rust 1.90.0, obtained verified NTS-KE and
NTS-authenticated NTP results from both initial candidates. RTTs were 3,394 and
117,472 microseconds in the first run. The operator documentation check subsequently
found the original Netnod alias absent from its official documentation. The central
candidate was updated to the officially documented hostname and retested in the
successful rerun below.
Evidence: [initial live results](https://github.com/tkgo11/afterglow-capsule/actions/runs/37288622769/job/111693473097)
and [official documentation review](https://github.com/tkgo11/afterglow-capsule/actions/runs/37289490311/job/111696272208).

The refreshed-provider rerun passed:
[native Windows live/provenance evidence](https://github.com/tkgo11/afterglow-capsule/actions/runs/37290256379/job/111698735744).
Both documented operators completed NTS-KE with normal certificate verification
and returned NTS-authenticated NTP. RTTs were 18,157 and 127,917 microseconds.
The public documentation excerpts explicitly associate each selected hostname
with its respective operator; they are different operators, not two aliases of
one service. Document hashes and excerpts are preserved in the CI log, and actual
hostnames/URLs remain centralized in external assumptions.

Maintainability basis for this spike: the published 1.4.0 changelog records active
2026-10-03 fixes for Windows toolchain compatibility, bounded cookie storage and
TLS security. Its Rustls floor is 0.23.45, which the official RustSec advisory
`RUSTSEC-2026-0285` lists as patched. The evaluated lock uses that patched version.
This review and platform evidence are a basis for the candidate experiment, not
a comprehensive advisory scan or third-party audit.

The [CLI](../../spikes/nts/src/main.rs) performs standard verified NTS-KE,
requires an authenticated NTP result and reports RTT. Provider observations have
bounded waits and fail independently. It is excluded from production and has no
release/CEK access. Verification disabling and key logging features are excluded.

## Acceptance matrix

| SPEC §22 requirement         | Evidence                                                                                           |
| ---------------------------- | -------------------------------------------------------------------------------------------------- |
| NTS key exchange             | Passed for both documented independent operators on Windows                                        |
| NTS-authenticated NTP        | Passed for both documented independent operators on Windows                                        |
| RTT measurement              | Recorded for both documented operators                                                             |
| Timeout handling             | Actual stalled loopback peer tests passed on Linux and Windows                                     |
| Certificate validation       | Untrusted certificate rejected on Linux/Windows; public TLS verified for both documented operators |
| Independent provider results | Both authenticated results and both official provenance excerpts confirmed                         |
| Windows execution            | Four failure tests and documented-provider live/provenance results passed                          |

This workspace has no external TCP/UDP grants for NTS. Cloudflare's hostname was
confirmed in public documentation source; native Windows fetched Netnod's official
documentation and exposed a stale candidate alias. The refreshed centralized
candidate has now passed native Windows NTS and matching documentation review.

**Spike B's technical acceptance passes.** Phase 2 remains blocked by the other
unresolved gates. Time evidence may degrade; the pinned-round timelock invariant
must remain unchanged. This experiment does not authorize substituting local
time, unauthenticated NTP or HTTPS time for release.
