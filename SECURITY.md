# Security Policy

AFTERGLOW is a client-only future-release archive framework. Its security goals are deliberately narrower than DRM.

## Security goals

The standard architecture is designed to resist:

- changing the local Windows clock to unlock early,
- casual resource extraction,
- plaintext string searches,
- simple inspection of embedded private assets,
- invalid or spoofed release-relay responses,
- accidental corruption of protected objects.

The standard architecture uses:

- a build-time-pinned future drand round as the release gate,
- local verification of the target beacon,
- a random 256-bit content-encryption key,
- per-object keys derived with HKDF-SHA256,
- chunked AES-256-GCM for private content,
- optional Authenticode for publisher provenance.

## Important non-guarantees

AFTERGLOW does **not** guarantee:

- perfect DRM,
- screenshot prevention after release,
- plaintext invisibility in memory after legitimate decryption,
- protection against every expert reverse engineer,
- eternal availability of third-party public infrastructure,
- post-quantum secrecy,
- safety if a creator deliberately retains an early recovery secret.

The optional embedded Heritage Recovery mode intentionally weakens expert reverse-engineering resistance because recovery material must exist in the recipient EXE.

## Critical invariants

A security issue is high impact if it allows any of the following:

- protected content decrypts before the pinned target round exists,
- changing local time alone unlocks protected content,
- the CEK is recoverable as plaintext from a normal production EXE,
- protected content is packaged as plaintext,
- an invalid drand beacon is accepted,
- AES-GCM authentication failure still reveals content,
- Builder preview/debug state can unlock a production Viewer,
- a signed executable is mutated after signing without invalidating the build process.

See [SPEC.md](SPEC.md) for the complete security invariants.

## Recovery

The default Balanced profile may create a **creator-held external recovery artifact**. It is not distributed with the recipient EXE.

Anyone possessing sufficient recovery credentials may be able to recover content independently of the public timelock path. This is a creator trust/resilience tradeoff, not a stronger timelock.

## Code signing

Authenticode improves publisher provenance and allows reputation to accumulate. It does not guarantee that Microsoft SmartScreen will never warn about a newly distributed binary.

## Reporting vulnerabilities

Until a dedicated private security-reporting channel is configured, **do not publish a working exploit, private test content, or secret material in a public GitHub issue**.

For non-sensitive security hardening ideas, a normal issue is acceptable.

Repository maintainers should enable GitHub Private Vulnerability Reporting before the first public security-sensitive release.

## Supported versions

The project is currently **pre-alpha**. No production security support window is promised yet.

This policy should be updated before the first tagged public release.
