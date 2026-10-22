# Prepared release-owned Viewer state

This library is isolated preparation for Phase 6, not a production Viewer.
It provides the complete named state sequence, entry/media navigation, bounded
state history and locked errors. The UI cannot set states or retrieve the CEK.

Capsule load validates structure, versions, section digests, metadata and the
exact-round envelope. Boot can immediately enter PreRelease without network.
Only a cryptographically verified exact-round beacon can begin unlock. The
required private archive index must decrypt/authenticate, parse its version and
match project/build/object bindings before ReadyForCeremony. Any failure drops
secret state. Clock/countdown values are not release inputs.

Ceremony requires ReadyForCeremony, explicit completion and deliberate archive
entry. Archive navigation is unavailable beforehand. Entry and image bytes are
decrypted on demand in memory. Media must be referenced by the current entry and
have an image class. Public images use already-verified public-store bytes.
Invalid AEAD or private document format clears the CEK and returns a locked error.
No recovered key is persisted or returned to UI code.

Private index and entry JSON use the existing versioned `Document<T>` envelope:

- `afterglow-private-archive`: project/build IDs; contributor ID/profile-object
  pairs; entry ID/contributor ID/entry-object triples. Counts, references and
  uniqueness are validated. Human titles remain inside encrypted entry objects.
- `afterglow-private-entry`: title and generic existing `ContentBlock` values.
  Blocks retain schema validation, including allowed links and no script nodes.

Owned entry text is redacted in Debug and cleared on drop where practical.
Returned image bytes use zeroizing buffers. UI/decoder/OS copies are outside that
guarantee and require their own lifetime controls at integration.

Integration tests build actual encrypted capsules and verify state ordering,
navigation, rate limiting, unavailable material, verified wrong-round rejection,
private version/project mismatch and maliciously rehashed AEAD corruption.
Fixtures and historical beacons exist only in tests. Renderer, accessibility,
Windows resource loading and production phase acceptance remain separate work.
