# Spike A: timelock interoperability

Historical test runner:

```sh
python spikes/timelock/build_and_check.py
```

This builds the exact pinned Rust candidate and official Go reference, then runs
nine historical/differential/negative tests. The payload is public synthetic data.
No fixture private key or manufactured beacon exists. The Rust wrapper verifies
the exact pinned chain and round, BLS signature and randomness before invoking the
candidate's decryption; it writes plaintext only after successful authentication.

For a real future-round test on a permitted network:

```sh
python spikes/run_live.py timelock
```

This reads the centralized relay candidate, compares live chain metadata against
the historical pins, verifies a live latest beacon, encrypts ten rounds ahead in
both implementations, requires target-round HTTP 404 and pre-round decryption
failure, then waits at most 120 seconds for an exact verifiable target beacon.
Only cryptographic verification/decryption authorizes the test's success. A
monotonic clock bounds the wait and does not authorize release. Endpoint or pin
changes require review of external assumptions; the runner never adopts them.

Historical checks and the two pure live-input tests passed locally on Linux.
The live relay is blocked by this workspace's proxy policy; Windows execution
must be collected through CI. See the [evidence record](../../docs/decisions/spike-a-timelock.md).

The Go reference is pinned to `v1.2.1-0.20260923175943-3ea7fbb59e85`
(`3ea7fbb59e85d00b0d9b6b2554e9652d5766811b`). Its normal build uses the public
module proxy and checksum database. In this workspace, a proxy redirect for
`github.com/drand/drand/v2@v2.1.2` was policy-blocked. The local test instead used
an external, temporary Go workspace replacement pointing to an official GitHub
checkout of that exact tag (`9a1af8b9700bc542aa2abe378b666eec925bdc1f`).
No replacement path or relaxed checksum/TLS setting is committed. This fallback
must be disclosed when reproducing the local evidence.
