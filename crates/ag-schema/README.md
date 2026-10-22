# Schema core v1

`Document<T>` is the versioned Builder document envelope:

```json
{
  "format_name": "afterglow-terminology",
  "format_version": 1,
  "minimum_reader_version": 1,
  "data": {}
}
```

Public `Manifest` fields use the top-level shape in SPEC.md §15 and add its
mandatory minimum-reader version and pinned network identity (§11, §27).
`private_manifest_id` names a required private JSON object. Object metadata exposes
opaque identities, storage extents, compression and integrity information. It has
no private title or source filename field. Public SHA-256 is a 64-character hex
string for corruption detection, not an authenticity claim.

Object offsets are relative to the corresponding public or private store.
`validate_object_store_bounds` binds those extents to actual store lengths.
Private nonce prefixes are eight public bytes. Chunk sizes/counts are metadata;
actual authenticated chunk records follow in Phase 3.

Parsing is limited to 16 MiB of JSON with serde_json's default recursion bound.
Unknown optional fields are dropped when safe; unknown enum variants, missing
required fields, duplicate known fields and incompatible versions are rejected.
Schema callers must use the expected format name supplied by their own reader,
not an untrusted document's declared name.

Network profile validation checks structure only. It does not prove chain
identity, validate a BLS signature or enable decryption. No real chain parameters
or relays are copied into this crate. Test networks are synthetic and exist only
in integration tests. Target-round computation is integer build-time metadata
logic and never uses a runtime/local clock to authorize release.

Links are declarative HTTP(S) destinations without credentials; other schemes are
rejected. Literal text is not HTML. Rich text contains theme-controlled formatting,
and no script, arbitrary font/size, plugin or executable content node exists.
Optional video blocks are deferred; all required text and image block types exist.
