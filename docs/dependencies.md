# Dependency selection record

Phase 1 selects serialization, identifier and metadata utilities, but no
security-sensitive crypto/platform production dependency. In particular, no
AES-GCM, HKDF/SHA-256, Argon2id, timelock/drand, TLS or PE/signing implementation has
been adopted. All mandatory spikes remain open.

Before selecting any such dependency, add its name, exact version, license,
upstream repository, security status and selection rationale here (SPEC.md §9.2).
Use exact versions in manifests and commit lockfile changes. Record interoperability
and platform evidence in the corresponding spike decision. Listing a candidate in
external assumptions is not an audit endorsement.

## Phase 1 selection record

All entries below are pinned and compile/test with Rust 1.90.0. Upstream crate
metadata/license was inspected. This is not an audit endorsement; no security
review of third-party timelock, NTS or Windows integrations is inferred.

| Name          | Version | License                      | Upstream                                | Selection rationale                                       |
| ------------- | ------- | ---------------------------- | --------------------------------------- | --------------------------------------------------------- |
| serde         | 1.0.229 | MIT OR Apache-2.0            | https://github.com/serde-rs/serde       | Typed versioned document serialization                    |
| serde_json    | 1.0.151 | MIT OR Apache-2.0            | https://github.com/serde-rs/json        | Bounded JSON parsing; default depth limit retained        |
| uuid          | 1.27.0  | Apache-2.0 OR MIT            | https://github.com/uuid-rs/uuid         | OS-random UUIDv4 identifiers; v4/v7 validation            |
| chrono        | 0.4.45  | MIT OR Apache-2.0            | https://github.com/chronotope/chrono    | UTC/date parsing; runtime wall-clock feature disabled     |
| chrono-tz     | 0.10.4  | MIT OR Apache-2.0            | https://github.com/chronotope/chrono-tz | IANA timezone validation                                  |
| semver        | 1.0.28  | MIT OR Apache-2.0            | https://github.com/dtolnay/semver       | Viewer minimum-version parsing                            |
| thiserror     | 2.0.21  | MIT OR Apache-2.0            | https://github.com/dtolnay/thiserror    | Explicit parser/compatibility error diagnostics           |
| url           | 2.5.8   | MIT OR Apache-2.0            | https://github.com/servo/rust-url       | Standard URL parsing; scheme/credential validation        |
| libfuzzer-sys | 0.4.13  | (MIT OR Apache-2.0) AND NCSA | https://github.com/rust-fuzz/libfuzzer  | Development-only fuzz harness runtime; separate workspace |

The npm lock records exact frontend and test/lint tooling versions and integrity
hashes. These dependencies currently support only the Builder scaffold and its
development checks. They are not shipped in the native recipient Viewer.
