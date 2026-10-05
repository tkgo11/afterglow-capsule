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

## Phase 2 isolated candidates — not production selections

All dependencies below are confined to the excluded `spikes/` workspace or its
Go wrapper. Production Builder/Viewer dependency closures do not include them.
Exact registry artifacts and transitive versions are committed in the separate
Cargo/Go locks. Published crate license/repository metadata and the official Go
reference source were inspected. **Security status for every entry is unreviewed:
no audit endorsement or completed advisory assessment is claimed.** Passing the
listed test subset establishes only that evidence; it does not establish
maintainability, platform support or overall cryptographic security. Production
adoption requires the mandatory gate results and further dependency review.

| Candidate      | Exact evaluated version              | License                  | Upstream                                | Selection rationale / evidence limit                                          |
| -------------- | ------------------------------------ | ------------------------ | --------------------------------------- | ----------------------------------------------------------------------------- |
| Go toolchain   | 1.27.1                               | BSD-3-Clause             | https://go.dev/                         | Official reference compiler; Linux archive SHA-256 verified                   |
| drand/tlock    | v1.2.1-0.20260923175943-3ea7fbb59e85 | MIT OR Apache-2.0        | https://github.com/drand/tlock          | Official differential reference; historical Go/Rust tests passed              |
| drand/drand/v2 | 2.1.2                                | MIT                      | https://github.com/drand/drand          | Reference's pinned dependency; local exact-tag workspace fallback disclosed   |
| tlock_age      | 0.0.10                               | MIT                      | https://github.com/thibmeu/tlock-rs     | RFC9380 timelock/age candidate; historical tests passed, live/Windows pending |
| tlock          | 0.0.10                               | MIT                      | https://github.com/thibmeu/tlock-rs     | Candidate transitive timelock implementation; not custom crypto               |
| age            | 0.11.5                               | MIT OR Apache-2.0        | https://github.com/str4d/rage           | Candidate transitive ciphertext format; not a replacement release gate        |
| drand_core     | 0.0.19                               | MIT                      | https://github.com/thibmeu/drand-rs     | Pinned-chain/round BLS/randomness checks in isolated wrapper                  |
| rkik-nts       | 1.4.0                                | MIT                      | https://github.com/aguacero7/rkik-nts   | Toolchain-compatible NTS candidate; local TLS failure tests only              |
| tokio          | 1.52.3                               | MIT                      | https://github.com/tokio-rs/tokio       | Bounded async network experiments                                             |
| rustls         | 0.23.45                              | Apache-2.0 OR ISC OR MIT | https://github.com/rustls/rustls        | Candidate TLS verifier/transport; no bypass configured                        |
| tokio-rustls   | 0.26.6                               | MIT OR Apache-2.0        | https://github.com/rustls/tokio-rustls  | Candidate TLS integration and local test server                               |
| ring           | 0.17.14                              | Apache-2.0 AND ISC       | https://github.com/briansmith/ring      | Candidate TLS crypto provider; transitive, not an audit endorsement           |
| aes-siv        | 0.7.0                                | Apache-2.0 OR MIT        | https://github.com/RustCrypto/AEADs     | Candidate NTS AEAD dependency; not the capsule cipher selection               |
| rcgen          | 0.14.7                               | MIT OR Apache-2.0        | https://github.com/rustls/rcgen         | Test-only ephemeral loopback TLS certificate generation                       |
| windows-sys    | 0.61.2                               | MIT OR Apache-2.0        | https://github.com/microsoft/windows-rs | Native PE resource reader/injector probe; Windows execution pending           |
| wgpu           | 27.0.1                               | MIT OR Apache-2.0        | https://github.com/gfx-rs/wgpu          | Toolchain-compatible windowed glass probe; reference hardware pending         |
| winit          | 0.30.13                              | Apache-2.0               | https://github.com/rust-windowing/winit | Native experiment window/input/DPI; runtime validation pending                |
| pollster       | 0.4.0                                | MIT OR Apache-2.0        | https://github.com/zesterer/pollster    | Startup-only adapter/device future executor                                   |
| naga           | 27.0.3                               | MIT OR Apache-2.0        | https://github.com/gfx-rs/wgpu          | Test-only WGSL parse/validation; hardware evidence not inferred               |

No dangerous TLS configuration, TLS key logging, fabricated beacon or development
release path is enabled in production. No production cryptographic or platform
adoption follows from these candidate records. See the four
[spike decisions](decisions/README.md) for the outstanding acceptance evidence.
