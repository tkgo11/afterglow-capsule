# Prepared exact-round timelock adapter

Preparation only; production integration and Phase 4 acceptance remain gated.
The library uses the exact `tlock_age 0.0.10` and `drand_core 0.0.19` path tested
against the official Go reference in Spike A. No new timelock primitive is used.

The central chain metadata is extracted at build time from the reviewed external
assumptions. Only chain metadata enters the non-test library; historical beacon
fixtures are test-only. Profiles must match hash, public key, genesis, period and
the supported RFC9380 scheme. Shipping still requires current official profile
revalidation; historical metadata does not complete that review.

The engine owns immutable validated release metadata. It checks the envelope's
exact round and chain, verifies the beacon's BLS signature and SHA-256 randomness,
and only then recovers exactly 32 bytes into a zeroizing CEK. Failed or truncated
decryption returns no secret. A verified-beacon capability has no public
constructor. Time evidence and wall clocks are absent from release authorization.

Configured relay URLs contain only the pinned chain and round, with no project
identity. The bounded async relay race accepts the first cryptographically valid
beacon, tolerates failure/absence/invalid replies, and reports waiting otherwise.
Transport is an explicit interface; a production HTTP adapter remains unadopted
until integration/review. Test transports exist only under `cfg(test)`.

`check_reference.py` exercises both directions with public 32-byte CEKs against
the pinned Go wrapper, plus mutation/truncation failures. The `compatibility`
example is an excluded development harness, never a production Viewer command.
Historical tests neither replace the actual future-round Spike A evidence nor
accept the incomplete overall Phase 2 gate.

Local Linux differential tests passed with the previously disclosed exact-tag Go
workspace fallback: the normal module ZIP redirect was policy-blocked, so
`drand/v2 v2.1.2` was read from its official tag at
`9a1af8b9700bc542aa2abe378b666eec925bdc1f` in an ignored external workspace.
Committed Go module/sum files and normal TLS/checksum settings were unchanged.
CI uses the normal proxy path and no workspace replacement.
