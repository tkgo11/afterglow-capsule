# Prepared capsule assembly and loading

Isolated Phase 5 portable software preparation. Native PE resource mutation,
project icon/version changes, Authenticode and clean-machine acceptance are not
promoted by this package. No production Builder or Viewer uses it yet.

The existing 128-byte `ag-capsule` header is preserved. Serialization places the
manifest, public store, private store, timelock envelope and digest table in that
order, contiguously. Empty public store uses offset/length zero. There are no
unclaimed gaps or trailing bytes. Capsule limits remain explicit and checked.

The fixed digest table identifies `afterglow-capsule-digests` with magic
`AGDIG1\0\0`. Its 16-byte header contains magic, little-endian u16 format/reader
versions (1/1) and little-endian u32 record count (5). Each 40-byte record contains
an ordered u8 kind, seven zero reserved bytes and a 32-byte SHA-256. Kinds 0–4
hash the exact capsule header, manifest, public store, private store and timelock
envelope. Empty public store hashes the empty byte string. The digest table is
exactly 216 bytes; its record types, size and order are validated.

These hashes detect corruption, not attacker-resistant authenticity. Unsigned
replaceable hashes are not a substitute for Authenticode or AES-GCM.

Loading verifies all section digests, manifest versions/minimum Viewer version,
object identities/extents/coverage, public-object hashes, private record metadata
and exact-round envelope. It returns encrypted private objects; loading grants
no release capability. Private titles/filenames are not object table fields.

The consuming assembler owns a fresh CEK and an immutable project/build binding.
Private-class bytes cannot enter its public-store API. It encrypts private
objects, wraps only the CEK with the prepared interoperable timelock adapter,
serializes and reparses the exact result, and scans provided private needles and
the actual CEK before returning bytes. No CEK is serialized in plaintext or
returned with the capsule. Full final-EXE leak scans and PE readback/sign-last
integration remain required later; a capsule-only scan cannot substitute for them.

In-memory object decryption uses authenticated headers/counts and zeroizing
buffers. Product asset optimization, source filename/field-value needle
generation, immutable authoring snapshots, executable smoke tests and creator
build reports remain separate work; this package does not claim their completion.
