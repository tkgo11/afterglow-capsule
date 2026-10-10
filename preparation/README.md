# Unaccepted implementation preparation

This excluded workspace holds reviewable software preparation while Phase 2 C/D
requires physical evidence. The latest user instruction defers tasks requiring
the user's machine or observations. It does not manufacture evidence or accept
any production phase. SPEC.md is unchanged.

Neither production Builder nor Viewer may depend on these packages until the
mandatory evidence gate passes. Tests here establish the tested subsystem
behavior only. They do not establish clean-machine packaging, physical GPU
performance, final product integration or release readiness.

Prepare independent slices in specification order. Do not implement an uncertain
platform assumption as an accepted production dependency. Promotion requires a
genuine Phase 2 PASS, dependency review and the applicable integration tests.

Run the prepared software checks:

```sh
cargo test --manifest-path preparation/Cargo.toml --locked --workspace --all-targets
cargo fmt --manifest-path preparation/Cargo.toml --all -- --check
cargo clippy --manifest-path preparation/Cargo.toml --locked --workspace --all-targets -- -D warnings
```

## Phase 3 preparation

`object-crypto` uses OS-generated 256-bit CEKs, HKDF-SHA256 per-object keys,
chunked AES-256-GCM and beneficial Zstandard compression. Objects are decrypted
on demand in memory. No bytes are returned before every chunk authenticates.
Temporary secret/plaintext buffers use zeroizing wrappers where practical; this
does not guarantee erasure of allocator, OS, codec or debugger copies.

`object-store` documents and reads the [bounded binary record](object-store/README.md).
Its fixed public test vector was independently generated with Python cryptography
50.0.0. Vector keys are public fixtures, never production runtime inputs.

## Phase 4 preparation

The [timelock adapter](timelock/README.md) checks the exact pinned round and
reviewed chain parameters, verifies beacons locally and decrypts only a 32-byte
CEK. Its relay race has no time-evidence authorization path. Production HTTP
transport adoption and application integration remain gated.

## Phase 5 portable preparation

The [capsule assembler and reader](capsule/README.md) preserve the existing header,
validate contiguous object stores and section/public-object digests, check the
exact-round envelope and scan supplied private plaintext needles plus the CEK.
PE injection, final EXE scanning/signing and clean-machine acceptance are deferred.
