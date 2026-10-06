# Dependency selection record

Phase 1 selects serialization, identifier and metadata utilities, but no
security-sensitive crypto/platform production dependency. In particular, no
AES-GCM, HKDF/SHA-256, Argon2id, timelock/drand, TLS or PE/signing implementation has
been adopted. Spikes A/B passed; C/D still block production integration.

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
reference source were inspected. **Overall security status is not established:
no audit endorsement or comprehensive security assessment is claimed.** The NTS
candidate's maintenance record and the specific Rustls advisory below were reviewed.
Passing the listed tests establishes that evidence; it does not establish overall
cryptographic security. Production
adoption requires the mandatory gate results and further dependency review.

| Candidate      | Exact evaluated version              | License                  | Upstream                                | Selection rationale / evidence limit                                                    |
| -------------- | ------------------------------------ | ------------------------ | --------------------------------------- | --------------------------------------------------------------------------------------- |
| Go toolchain   | 1.27.1                               | BSD-3-Clause             | https://go.dev/                         | Official reference compiler; Linux archive SHA-256 verified                             |
| drand/tlock    | v1.2.1-0.20260923175943-3ea7fbb59e85 | MIT OR Apache-2.0        | https://github.com/drand/tlock          | Official reference; historical and real future-round Go/Rust tests passed on Windows    |
| drand/drand/v2 | 2.1.2                                | MIT                      | https://github.com/drand/drand          | Reference's pinned dependency; local exact-tag workspace fallback disclosed             |
| tlock_age      | 0.0.10                               | MIT                      | https://github.com/thibmeu/tlock-rs     | RFC9380 candidate; all required Spike A interoperability/rejection/Windows tests passed |
| tlock          | 0.0.10                               | MIT                      | https://github.com/thibmeu/tlock-rs     | Candidate transitive timelock implementation; not custom crypto                         |
| age            | 0.11.5                               | MIT OR Apache-2.0        | https://github.com/str4d/rage           | Candidate transitive ciphertext format; not a replacement release gate                  |
| drand_core     | 0.0.19                               | MIT                      | https://github.com/thibmeu/drand-rs     | Pinned-chain/round BLS/randomness checks in isolated wrapper                            |
| rkik-nts       | 1.4.0                                | MIT                      | https://github.com/aguacero7/rkik-nts   | Verified independent documented providers and real failure tests on Windows             |
| tokio          | 1.52.3                               | MIT                      | https://github.com/tokio-rs/tokio       | Bounded async network experiments                                                       |
| rustls         | 0.23.45                              | Apache-2.0 OR ISC OR MIT | https://github.com/rustls/rustls        | Candidate TLS verifier/transport; no bypass configured                                  |
| tokio-rustls   | 0.26.6                               | MIT OR Apache-2.0        | https://github.com/rustls/tokio-rustls  | Candidate TLS integration and local test server                                         |
| ring           | 0.17.14                              | Apache-2.0 AND ISC       | https://github.com/briansmith/ring      | Candidate TLS crypto provider; transitive, not an audit endorsement                     |
| aes-siv        | 0.7.0                                | Apache-2.0 OR MIT        | https://github.com/RustCrypto/AEADs     | Candidate NTS AEAD dependency; not the capsule cipher selection                         |
| rcgen          | 0.14.7                               | MIT OR Apache-2.0        | https://github.com/rustls/rcgen         | Test-only ephemeral loopback TLS certificate generation                                 |
| windows-sys    | 0.61.2                               | MIT OR Apache-2.0        | https://github.com/microsoft/windows-rs | Native signing/readback/mutation rejection passed; clean VM/shell pending               |
| wgpu           | 27.0.1                               | MIT OR Apache-2.0        | https://github.com/gfx-rs/wgpu          | Toolchain-compatible windowed glass probe; reference hardware pending                   |
| winit          | 0.30.13                              | Apache-2.0               | https://github.com/rust-windowing/winit | Native experiment window/input/DPI; runtime validation pending                          |
| pollster       | 0.4.0                                | MIT OR Apache-2.0        | https://github.com/zesterer/pollster    | Startup-only adapter/device future executor                                             |
| naga           | 27.0.3                               | MIT OR Apache-2.0        | https://github.com/gfx-rs/wgpu          | Test-only WGSL parse/validation; hardware evidence not inferred                         |

The published `rkik-nts 1.4.0` changelog records 2026-10-03 Windows compatibility,
bounded-cookie and TLS-security fixes. The official RustSec
[RUSTSEC-2026-0285](https://rustsec.org/advisories/RUSTSEC-2026-0285.html) advisory
lists Rustls >=0.23.45 as patched; the candidate locks 0.23.45. This specific review
was originally limited to that advisory. The current lockfile advisory scans
below remain a separate, limited review and do not establish overall security.

No dangerous TLS configuration, TLS key logging, fabricated beacon or development
release path is enabled in production. No production cryptographic or platform
adoption follows from these candidate records. See the four
[spike decisions](decisions/README.md) for the outstanding acceptance evidence.

## Phase 2 evidence and fuzz development tooling

The isolated evidence validator reuses pinned serde/serde_json and sha2 0.10.9
(MIT OR Apache-2.0, https://github.com/RustCrypto/hashes) for duplicate-rejecting
bounded JSON and artifact SHA-256 consistency. sha2 was already locked in the
spike dependency graph. No production crypto dependency is adopted. Windows
Shell/SendInput/DPI collection reuses windows-sys 0.61.2; the native helper uses
stock Windows .NET Framework/System.Drawing, not a distributed third-party DLL.

Instrumented development-only parser smoke uses `cargo-fuzz 0.13.2`
(MIT OR Apache-2.0, https://github.com/rust-fuzz/cargo-fuzz) with
`nightly-2025-09-15` (Rust compiler 1.92 nightly). Exact install uses `--locked`;
production remains Rust 1.90.0. These development tools were executed and source/
license provenance reviewed, not comprehensively security audited.

## Existing lockfile advisory checks

On 2026-10-07, `cargo-audit 0.22.2` (Apache-2.0 OR MIT,
https://github.com/rustsec/rustsec, Rust >=1.88) scanned the root, isolated spike
and fuzz lockfiles against the official RustSec database revision
`ef6173cbc5c50ec8166f9a5b28f07834144373ee` (2026-10-03; 1,290 advisories).
The graphs contained 82, 434 and 77 dependencies respectively. No published
vulnerability was reported, with no ignored advisory, platform or severity
filter. All lock hashes remained unchanged. npm 11.9.0 reported zero
vulnerabilities across its 248 locked dependencies, including development tools.
The [recorded review](decisions/dependency-advisory-review.json) preserves exact
lock/report hashes, database revision and the informational warning.

RustSec [RUSTSEC-2026-0173](https://rustsec.org/advisories/RUSTSEC-2026-0173.html)
reports unmaintained `proc-macro-error2 2.0.1`, reached only through Spike A's
compile-time localization chain: `tlock_age 0.0.10 -> age 0.11.5 ->
i18n-embed-fl 0.9.4`. Registry checks found no age 0.11.6 or i18n-embed-fl 0.9.5.
The published newer lines (age 0.12.1, i18n-embed-fl 0.10.1) fall outside the
current parents' age 0.11/i18n-embed-fl 0.9 constraints. The warning is retained
visibly; changing or forking the validated crypto dependency path solely for this
informational warning is not adopted. Resolve/review maintenance before
production adoption and rerun mandatory interoperability tests for any change.

CI now scans all three Rust graphs against one freshly checked-out official
database revision and preserves raw reports, hashes and warning IDs. Known
vulnerabilities, unsoundness warnings, ignored/filtered reports, malformed output,
changed lockfiles and scanner failures fail the job. Eight failure regressions
check the report handling. npm checks include all severity levels and preserve
their raw result. These advisory checks are development tooling: they neither
ship in Viewer nor establish a comprehensive dependency audit or Phase 2 PASS.
