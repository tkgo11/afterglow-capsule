# 0002 — Phase 1 format and schema core

- Status: accepted
- Date: 2026-10-05
- SPEC.md sections: 11–19, 26–27, 58–65, 143, 147
- Scope: Phase 1; no architectural deviation

## Decisions

Use bounded serde JSON models with explicit format/reader versions, random
UUIDv4 identities (UUIDv7 accepted on input), standard RFC3339/UTC dates and IANA
timezone validation. Unknown safe optional fields are ignored; executable content
nodes, unknown critical enum variants and incompatible versions are rejected.

Use typed declarative blocks and schema field/visibility definitions. Keep editable
creator documents separate from the recipient manifest. Public metadata has no
private title/source filename fields. Required private JSON is referenced by opaque
object ID. Public/private extents are validated separately against their stores.

Use a 128-byte fixed little-endian capsule header with minimum-reader version in
the reserved space. Details are in `crates/ag-capsule/README.md`. Reject overflow,
overlaps, unsupported versions/flags, unknown reserved bytes and missing required
sections. No digest/crypto protection is claimed by structural validation alone.

Keep exact integer target-round calculation in metadata validation. This uses only
explicit requested/genesis instants and a period; no runtime clock or unlock path
is introduced. Production profile trust remains gated by Spike A.

Set bounded reader limits: 16 MiB JSON, 1024 custom fields, 100,000 project/object
records, 10,000 blocks per entry, 16 allowlisted relays. Capsule size defaults to
1 GiB with an explicit expert size override; all structural checks remain mandatory.
These are implementation resource limits, not weaker cryptographic rules.

## Validation

Round-trip and rejection tests cover generic documents/content, all field types,
reader/version compatibility, UUIDs, required private manifest objects, relay
metadata, exact fractional round boundaries, long release horizons, duplicate IDs,
nonce prefixes, object protection/extents, wrong-project references and capsule
truncation/overflow/overlaps. Fuzz harnesses target the header, manifest, object
table and creator project document; their separate lockfile is committed.

Windows execution, live chain identity, Go/Rust interoperability, NTS, PE signing
and GPU evidence remain Phase 2 obligations. No production crypto or platform
integration has been added.
