# Prepared encrypted-object format v1

This is the isolated candidate for `ag-object-store` (SPEC §17.6), not an accepted
production format. Serialized name: `afterglow-encrypted-object`; its fixed magic
identifies that name. Format version and minimum reader version are both 1.
Unknown versions, compression methods, flags, invalid UUIDs and trailing bytes
are rejected. All integer fields are little-endian unless specified otherwise.

| Offset | Size | Field                                                        |
| ------ | ---- | ------------------------------------------------------------ |
| 0      | 8    | `AGOBJ1\0\0` magic                                           |
| 8      | 2    | format version                                               |
| 10     | 2    | minimum reader version                                       |
| 12     | 2    | header size (96)                                             |
| 14     | 1    | compression: 0 none, 1 Zstandard                             |
| 15     | 1    | flags (zero)                                                 |
| 16     | 16   | project UUID bytes in RFC 4122 order                         |
| 32     | 16   | build UUID bytes                                             |
| 48     | 16   | opaque object UUID bytes                                     |
| 64     | 8    | random nonce prefix                                          |
| 72     | 4    | encoded plaintext bytes per chunk (default 1 MiB, max 4 MiB) |
| 76     | 4    | chunk count (at least one, including empty objects)          |
| 80     | 8    | original, decompressed plaintext length                      |
| 88     | 8    | encoded/compressed plaintext length                          |

Each chunk follows directly after the preceding chunk:

| Size              | Field                                                               |
| ----------------- | ------------------------------------------------------------------- |
| 4                 | zero-based chunk index                                              |
| 4                 | encoded plaintext length of this chunk                              |
| 4                 | ciphertext length, excluding tag; must equal chunk plaintext length |
| ciphertext length | ciphertext                                                          |
| 16                | AES-256-GCM authentication tag                                      |

The nonce is the header's 8-byte prefix followed by the chunk index as **big-endian
u32**. A build encryptor owns one frozen project/build and rejects repeated object
IDs or nonce prefixes. Retry a failed build with a fresh encryptor/CEK/build ID.

Per-object HKDF info is ASCII `AFTERGLOW/object/v1/` followed by the object's 16
UUID bytes; salt is the project's 16 UUID bytes, IKM is the 32-byte CEK. Output is
32 bytes. The build ID is authenticated in AAD rather than changing this KDF.

AAD is the exact 96-byte header followed by the exact 12-byte chunk record. It
binds all SPEC-required fields, plus nonce prefix, object lengths, total chunk
count and chunk size. Changing a count/length to disguise final-chunk truncation
invalidates authentication on remaining chunks.

Parse every record and exact total length before decryption. Authenticate every
chunk before returning any plaintext. For compressed objects authenticate before
decoding, allow one Zstandard frame, cap the window at 8 MiB and bound decoded
bytes to the authenticated original length. Concatenated/trailing compressed
frames are rejected. Compression is retained only when its measured output is
smaller. JPEG/PNG/WebP and audio/video default to no secondary compression.

Default reader limits: 1 GiB encoded store, 512 MiB original/encoded plaintext,
65,536 chunks; explicit limits can be tightened. The v1 protocol also rejects
more than u32::MAX chunks. These are memory limits, not claims about larger PE or
media support. Whole-object in-memory decoding is presently provided; authenticated
partial-media caching and product integration remain unaccepted work.
