# Creator project documents

`ProjectDocument` is a versioned, creator-side in-memory/transport model named
`afterglow-workspace`. It is separate from the recipient `afterglow-project`
manifest. It aggregates versioned terminology, custom schema, requested release,
contributors and entries. Each nested document must have its own matching name,
version and minimum reader version.

An editable release has no target round; build-time pinning happens when a frozen
snapshot is packaged. Project, contributor and entry IDs are random stable UUIDs.
Foreign-project records, duplicate identities and dangling contributor references
are rejected. Draft/empty contributors may omit required fields, but supplied
values remain typed; submitted/reviewed/approved/build-locked records must satisfy
all required fields.

This phase defines models and JSON boundaries. Filesystem layout, asset import,
atomic autosave, immutable build snapshots and invite/submission packages follow
in their specified phases. The Viewer must never load these editable documents.
