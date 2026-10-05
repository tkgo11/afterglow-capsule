# Dependency selection record

Phase 0 selects no security-sensitive production dependency. In particular, no
AES-GCM, HKDF/SHA-256, Argon2id, timelock/drand, TLS or PE/signing implementation has
been adopted. All mandatory spikes remain open.

Before selecting any such dependency, add its name, exact version, license,
upstream repository, security status and selection rationale here (SPEC.md §9.2).
Use exact versions in manifests and commit lockfile changes. Record interoperability
and platform evidence in the corresponding spike decision. Listing a candidate in
external assumptions is not an audit endorsement.

The npm lock records exact frontend and test/lint tooling versions and integrity
hashes. These dependencies currently support only the Builder scaffold and its
development checks. They are not shipped in the native recipient Viewer.
