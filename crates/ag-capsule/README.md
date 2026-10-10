# Capsule header v1

All numeric fields are little-endian. The fixed header is 128 bytes. This format
choice fills the reserved space allowed by SPEC.md §19.1; the format name is
identified by the eight-byte magic, not stored as a second redundant string.

| Offset | Size | Field                                 |
| ------ | ---- | ------------------------------------- |
| 0      | 8    | `AGCAPS1\0`                           |
| 8      | 2    | format version, currently 1           |
| 10     | 2    | header size, exactly 128              |
| 12     | 4    | flags, currently zero                 |
| 16     | 16   | manifest offset and length (u64 each) |
| 32     | 16   | public store offset and length        |
| 48     | 16   | private store offset and length       |
| 64     | 16   | timelock envelope offset and length   |
| 80     | 16   | digest table offset and length        |
| 96     | 2    | minimum reader version, currently 1   |
| 98     | 30   | reserved, must be zero                |

Offsets are relative to capsule byte zero. Nonempty sections must be outside the
header, inside the capsule, and mutually nonoverlapping. Only the public store
may be empty; its canonical empty extent is `(0, 0)`. The encrypted private
manifest, timelock envelope, public manifest and digest table are required even for
a small project. Unknown versions, reader requirements, flags, or nonzero reserved
bytes fail closed. Every offset/length addition and range conversion is checked.

Default read limit: 1 GiB. An explicit caller-supplied expert size limit changes
only this bound; it does not relax format, extent, version or overlap validation.
The Builder's warning policy above 512 MiB is implemented with packaging later.

Phase 1 encodes only the header and parses structure. It does not generate a full
capsule, verify section digests, implement PE loading, or authenticate/decrypt
objects. Those are required later and must never be replaced by this parser.
