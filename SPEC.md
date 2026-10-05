# AFTERGLOW
## Canonical AI-Implementable Production Specification

**Version:** 3.0  
**Specification status:** Canonical / implementation-ready  
**Last external-assumption review:** 2026-10-05  
**Primary audience:** AI coding agents, software engineers, security reviewers, designers, QA  
**Recipient platform:** Windows 10/11 x64  
**Authoring product:** AFTERGLOW Builder  
**Recipient product:** Project-specific AFTERGLOW Viewer  
**Final recipient distribution artifact:** one `.exe` file  
**Viewer language/runtime:** Rust + `winit` + `wgpu` + WGSL  
**Builder stack:** Tauri 2 + Rust + Svelte + TypeScript  
**Default visual system:** Temporal Glass  
**Default motion system:** Temporal Motion  
**Default release model:** client-only drand timelock + multi-source time evidence  
**Project-owned backend:** prohibited for the default architecture  

---

# 0. AI AGENT OPERATING CONTRACT

This section is normative.

An AI coding agent implementing AFTERGLOW **MUST read and follow this section before writing code**.

## 0.1 Normative words

The words below have precise meanings:

- **MUST / MUST NOT** — mandatory requirement.
- **SHOULD / SHOULD NOT** — expected requirement; deviation requires a documented reason.
- **MAY** — optional.
- **BLOCKER** — implementation must not proceed past the affected gate until resolved.
- **SPIKE** — short technical proof used to validate an uncertain dependency or platform behavior.
- **DEFAULT** — binding unless the project configuration explicitly overrides it.

## 0.2 Source-of-truth order

If two statements conflict, use this priority:

1. **Security Invariants**
2. **Canonical Decision Register**
3. **Requirement IDs**
4. **Data format and state-machine definitions**
5. **Acceptance criteria**
6. **UI/design defaults**
7. **Examples**
8. **Non-normative explanatory prose**

Do not infer a different architecture from an example.

## 0.3 Implementation behavior

The implementation agent MUST:

1. Preserve the generic nature of the engine.
2. Keep domain-specific wording in presets/configuration only.
3. Keep the final Viewer free of Builder/editor functionality.
4. Preserve one-file recipient distribution.
5. Fail closed on cryptographic validation errors.
6. Never replace the timelock release gate with a local-clock check.
7. Never invent a custom cryptographic primitive.
8. Never add an AFTERGLOW-owned cloud service unless this specification is intentionally revised.
9. Never add arbitrary JavaScript/plugin execution to generated Viewer projects.
10. Never place private content in plaintext Viewer resources.
11. Never mutate a code-signed final EXE after signing.
12. Treat accessibility fallbacks as product behavior, not optional polish.
13. Add tests with every security-sensitive feature.
14. Prefer a simple correct implementation over a clever fragile one.
15. Record any deliberate deviation in `docs/decisions/`.

## 0.4 How to handle uncertainty

If this specification defines a default, use it.

Do not ask for clarification merely because another implementation is possible.

If a dependency or platform behavior is uncertain and materially affects security or packaging, create a **SPIKE** and validate it before proceeding.

The four mandatory spikes are defined in Part III.

## 0.5 Definition of “AI-friendly”

This specification is written so an implementation agent can answer:

- What must be built?
- What must never be built?
- What is the default?
- Which details are configurable?
- Which state owns a behavior?
- What data format crosses subsystem boundaries?
- What failure behavior is required?
- Which tests prove completion?
- What is security-critical?
- What can be deferred?

If an implementation choice is not defined and is not security-sensitive, prefer the smallest maintainable solution consistent with the specification.

---

# 1. PRODUCT MISSION

AFTERGLOW is a **general-purpose future-release archive framework**.

It lets creators use **AFTERGLOW Builder** to define a project containing messages, people, photos, memories, and other declarative content, then produce a single native Windows Viewer executable that reveals protected content at a future release time.

AFTERGLOW is not intrinsically a broadcast-club product.

Broadcast Club, Graduation, Family, Friends, Team Farewell, Anniversary, Personal Future Letter, Class Archive, and similar concepts are **presets** built on the same generic core.

The intended result is:

```text
Generic Engine
+
Project Configuration
+
Project Content
=
Personal Final Artifact
```

The final artifact must feel project-specific even though the engine is generic.

---

# 2. DELIVERABLES

There are two distinct deliverable classes.

## 2.1 Creator-side deliverables

The creators MAY retain:

```text
AFTERGLOW Builder
project workspace
source assets
creator-side build report
optional external recovery package
source code
CI artifacts
```

These are not recipient requirements.

## 2.2 Recipient-side deliverable

The recipient MUST be able to receive exactly one file:

```text
ProjectName.exe
```

Example:

```text
AFTERGLOW.exe
ForOurJuniors.exe
OpenIn2035.exe
FamilyTimeCapsule.exe
```

No sibling data directory is required.

No `.json`, `.dll`, `.pak`, message directory, project database, or configuration file may be required beside the Viewer.

### Important definition

“One-file distribution” means **the distributed project consists of one EXE**.

Windows, GPU drivers, certificate stores, SmartScreen, shader caches, OS caches, or optional post-release user-state created by Windows do not violate this distribution requirement.

---

# 3. CORE EXPERIENCE

The default emotional arc is:

```text
Time
  ↓
Opening
  ↓
Content
  ↓
Memory
```

For a people-centered project:

```text
Time
  ↓
Opening
  ↓
People
  ↓
Memory
```

## 3.1 Pre-release

The Viewer primarily communicates waiting.

It may show:

- project identity,
- project introduction,
- release date,
- D-day,
- precise countdown,
- approved public contributor previews,
- time-integrity status,
- project information.

Protected content remains encrypted.

## 3.2 Release

At release:

```text
target drand round becomes available
        ↓
beacon fetched
        ↓
beacon cryptographically verified
        ↓
timelock envelope decrypted
        ↓
CEK recovered
        ↓
private objects authenticated
        ↓
release state becomes READY
        ↓
ceremony plays
        ↓
recipient deliberately enters archive
```

The visual countdown reaching zero is **not sufficient** to unlock content.

## 3.3 Post-release

The Viewer becomes an archive.

Depending on Builder configuration, the user may browse:

- contributors,
- entries,
- letters,
- galleries,
- timelines,
- chapters,
- credits,
- project memories.

---

# 4. CANONICAL DECISION REGISTER

These decisions are binding unless this document is intentionally versioned.

| ID | Decision |
|---|---|
| D-001 | AFTERGLOW Core is domain-neutral. Domain language exists only in project configuration and presets. |
| D-002 | Recipient distribution is one Windows EXE. |
| D-003 | Builder and Viewer are separate applications. |
| D-004 | Viewer is native Rust using `winit` + `wgpu`; no Chromium/Electron runtime. |
| D-005 | Builder uses Tauri 2 + Rust + Svelte/TypeScript unless a documented future decision replaces it. |
| D-006 | Default architecture has no AFTERGLOW-owned release backend. |
| D-007 | Release authorization is based on a build-time-pinned future drand round, not the local wall clock. |
| D-008 | Current production timelock network profile is drand Quicknet; network identity is pinned by chain parameters, not trusted by hostname alone. |
| D-009 | Any Rust timelock implementation MUST pass differential interoperability tests against the official drand Go `tlock` reference before production use. |
| D-010 | Multi-source network time is auxiliary evidence for countdown/UX and anomaly detection; it is not the final cryptographic release gate. |
| D-011 | Private objects use per-object keys derived from a random 256-bit master CEK and chunked AES-256-GCM. |
| D-012 | Project capsule is inserted into a PE `RCDATA` resource before Authenticode signing. Do not use an unsigned PE overlay as the canonical capsule location. |
| D-013 | Viewer customization is declarative. Arbitrary project scripts/plugins are prohibited in v1. |
| D-014 | Default security profile does not embed a recovery key in the recipient EXE. |
| D-015 | Creator-held external recovery is supported as an optional resilience mechanism. |
| D-016 | Embedded “Heritage Recovery” is advanced and explicitly weaker against expert reverse engineering. |
| D-017 | Builder preview may simulate time/release state, but preview hooks MUST NOT exist in production Viewer release logic. |
| D-018 | All PE resource mutation, icon mutation, version-info mutation, and capsule injection occur before final signing. |
| D-019 | Viewer networking is allowlisted and never uploads project content. |
| D-020 | Every serialized format has an explicit version and migration policy. |
| D-021 | Private media is decrypted on demand; the Viewer does not decrypt the whole project into a filesystem directory. |
| D-022 | Default post-release persistence is Strict Portable Mode: no recovered CEK is persisted. |
| D-023 | Video is optional and not required for Viewer v1 completion. |
| D-024 | Text and still-image projects are first-class and MUST work without media codecs beyond image decoding. |
| D-025 | Accessibility settings override decorative visual effects. |

---

# 5. SECURITY INVARIANTS

The following are non-negotiable.

## SEC-I-001 — No local-clock-only unlock

A user setting Windows to a future date MUST NOT be sufficient to decrypt protected content.

## SEC-I-002 — No plaintext CEK in recipient EXE

The CEK MUST NOT be embedded as plaintext or reversibly obfuscated plaintext in the standard security profiles.

## SEC-I-003 — No plaintext private content

Protected message text, private images, private audio, and protected metadata MUST NOT be embedded in plaintext resources.

## SEC-I-004 — Exact target round

The build process MUST compute and pin one target timelock round.

The Viewer MUST request/decrypt against that target round.

It MUST NOT choose “whatever current round matches my local clock.”

## SEC-I-005 — Beacon verification

A relay response MUST be cryptographically verified against the pinned network identity before being used.

HTTPS transport alone is insufficient.

## SEC-I-006 — Fail closed

If beacon verification, timelock decryption, AEAD authentication, capsule parsing, or version validation fails, protected content MUST remain locked.

## SEC-I-007 — Preview isolation

Builder simulation flags, debug unlocks, fake clocks, mock beacons, and test keys MUST NOT be compiled into production release logic.

## SEC-I-008 — No custom cryptography

Do not invent custom block ciphers, KDFs, signature algorithms, or time-lock schemes.

## SEC-I-009 — Sign after mutation

If Authenticode is used, no capsule/resource mutation may occur after signing.

## SEC-I-010 — No false authenticity claim

An embedded project signature whose trust root is also freely replaceable inside the same unsigned EXE MUST NOT be described as attacker-resistant authenticity.

---

# 6. THREAT MODEL

## 6.1 Assets

Primary protected assets:

- private text,
- private images,
- private audio/video,
- CEK,
- protected metadata,
- unreleased contributor entries.

Integrity-sensitive assets:

- release timestamp,
- target drand round,
- project ID,
- public manifest,
- theme/configuration,
- executable itself.

## 6.2 Adversary classes

### A0 — Accidental user

Can:

- copy files,
- lose network,
- change timezone accidentally,
- encounter corrupt storage.

### A1 — Curious user

Can:

- change Windows clock,
- inspect EXE resources,
- run `strings`,
- use common archive/resource tools,
- monitor ordinary network traffic.

### A2 — Skilled local analyst

Can:

- disassemble native code,
- patch conditional branches,
- debug the process,
- dump memory,
- modify executable bytes.

### A3 — Expert reverse engineer

Can:

- deeply analyze the Viewer,
- reconstruct fallback logic,
- patch runtime code,
- inspect cryptographic integration.

### A4 — Infrastructure-level adversary

Can:

- block services,
- spoof unauthenticated NTP,
- operate malicious HTTP infrastructure,
- attempt MITM.

## 6.3 Required protection

AFTERGLOW MUST strongly resist A0 and A1.

The standard timelock design SHOULD make early decryption meaningfully difficult for A2 because the future beacon is not yet available.

No claim is made that a client-only system can defeat every A3 attack if an early-recovery secret is embedded.

Beacon verification and NTS SHOULD reduce trust in unauthenticated network transport against A4.

## 6.4 Explicit non-guarantees

AFTERGLOW does not guarantee:

- screenshot prevention after release,
- plaintext invisibility in process memory after release,
- protection if the timelock network threshold is malicious,
- post-quantum secrecy,
- eternal availability of third-party networks,
- perfect DRM,
- recovery from every future platform change.

---

# 7. TRUST MODEL

## 7.1 Trusted

At build time:

- creator machine,
- Builder binary,
- approved dependencies,
- project source content,
- OS cryptographic RNG.

At Viewer runtime:

- validated embedded capsule structure,
- pinned timelock network identity,
- cryptographically verified drand beacon,
- AES-GCM authentication result.

## 7.2 Not inherently trusted

- Windows wall clock,
- timezone,
- relay hostname alone,
- plain NTP response,
- public Wi-Fi,
- arbitrary HTTPS `Date` header,
- user-edited project binary,
- embedded public key that has no external trust anchor.

---

# PART I — REPOSITORY AND PRODUCT ARCHITECTURE

# 8. REPOSITORY LAYOUT

Use:

```text
afterglow/
│
├── apps/
│   ├── builder/
│   └── viewer/
│
├── crates/
│   ├── ag-schema/
│   ├── ag-project/
│   ├── ag-capsule/
│   ├── ag-object-store/
│   ├── ag-crypto/
│   ├── ag-timelock/
│   ├── ag-time/
│   ├── ag-theme/
│   ├── ag-motion/
│   ├── ag-render/
│   ├── ag-validation/
│   ├── ag-pe-packager/
│   └── ag-windows/
│
├── packages/
│   └── builder-ui/
│
├── viewer-runtime/
│   └── template/
│
├── shaders/
│   ├── glass.wgsl
│   ├── aurora.wgsl
│   ├── blur.wgsl
│   └── transitions.wgsl
│
├── presets/
├── test-vectors/
├── tests/
│   ├── interoperability/
│   ├── security/
│   ├── capsule/
│   ├── ui/
│   └── e2e/
│
├── docs/
│   ├── decisions/
│   ├── threat-model/
│   └── external-assumptions/
│
├── Cargo.toml
├── Cargo.lock
└── README.md
```

---

# 9. DEPENDENCY POLICY

## 9.1 Locking

Production builds MUST use:

- committed `Cargo.lock`,
- committed JavaScript package lock,
- pinned runtime-template version.

## 9.2 Security-sensitive dependencies

Security-sensitive dependencies include:

- AES-GCM,
- HKDF/SHA-256,
- Argon2id if used,
- timelock/drand libraries,
- TLS,
- PE packaging/signing integration.

For each, document:

```text
name
version
license
upstream repository
security status
reason selected
```

## 9.3 Timelock dependency rule

The current Rust ecosystem includes third-party drand/tlock implementations.

Because the official drand documentation identifies the Go `drand/tlock` project as the reference maintained implementation and Rust alternatives may have different audit/platform status, the production choice MUST pass the Timelock Interoperability Spike before adoption.

Do not silently replace the reference algorithm.

---

# 10. PRODUCT COMPONENTS

```text
AFTERGLOW Builder
      │
      ├── Project editor
      ├── Asset manager
      ├── Preview engine
      ├── Validation
      ├── Crypto packager
      ├── Timelock packager
      ├── PE resource injector
      └── Optional code signer
      │
      ▼
Project-specific Viewer EXE
      │
      ├── Capsule loader
      ├── Release engine
      ├── Time evidence
      ├── Object decryptor
      ├── Renderer
      ├── Accessibility
      └── Archive navigation
```

---

# PART II — SERIALIZED FORMATS

# 11. VERSIONING RULE

Every persistent format MUST contain:

```text
format_name
format_version
minimum_reader_version
```

Unknown major versions MUST be rejected.

Unknown optional fields in the same major version SHOULD be ignored when safe.

Migrations happen in Builder, not in the recipient Viewer where possible.

---

# 12. PROJECT WORKSPACE

Editable workspace:

```text
MyProject.afterglow/
│
├── project.json
├── terminology.json
├── theme.json
├── motion.json
├── release.json
├── structure.json
├── contributors/
├── entries/
├── assets/
├── locales/
└── .builder/
```

Builder MAY also export:

```text
MyProject.agproject
```

as a compressed transport backup.

The Viewer never consumes an editable workspace directly.

---

# 13. PROJECT IDENTIFIERS

Use random UUIDv4 or UUIDv7-style stable IDs.

Required:

```text
project_id
build_id
contributor_id
entry_id
object_id
```

Do not derive IDs from names.

---

# 14. GENERIC TERMINOLOGY

Core UI MUST reference semantic terms.

Default:

```json
{
  "contributor_singular": "Contributor",
  "contributor_plural": "Contributors",
  "recipient_singular": "Recipient",
  "recipient_plural": "Recipients",
  "entry_singular": "Message",
  "entry_plural": "Messages",
  "role_label": "Role",
  "archive_label": "Archive",
  "open_action": "Open",
  "locked_label": "Locked"
}
```

A Broadcast Club preset changes configuration, not code.

---

# 15. PROJECT MANIFEST

The capsule contains a public manifest.

Example shape:

```json
{
  "format_name": "afterglow-project",
  "format_version": 1,
  "project_id": "uuid",
  "build_id": "uuid",
  "viewer_min_version": "1.0.0",
  "identity": {
    "title": "AFTERGLOW",
    "subtitle": "A Letter to the Future"
  },
  "release": {
    "utc": "2030-03-01T15:00:00Z",
    "timezone": "Asia/Seoul",
    "target_round": 0,
    "timelock_profile": "quicknet-v1"
  },
  "layout": {
    "pre_release": "countdown-default",
    "post_release": "constellation"
  },
  "theme_id": "temporal-glass-dark",
  "objects": []
}
```

The example `target_round: 0` is placeholder-only; production Build MUST write the computed round.

---

# 16. OBJECT MODEL

All embedded content becomes objects.

Object classes:

```text
public-json
public-image
public-audio
public-font
private-json
private-image
private-audio
private-video
private-binary
```

Private object IDs are opaque.

A private object's human title MAY live only inside the encrypted JSON object if leaking it pre-release is undesirable.

---

# 17. PRIVATE OBJECT CRYPTO FORMAT

Do not encrypt one huge monolithic archive as a single AES-GCM operation.

Private objects are encrypted independently.

Benefits:

- on-demand decryption,
- bounded memory,
- partial media loading,
- smaller corruption blast radius,
- simpler object caching.

## 17.1 Master CEK

Builder generates:

```text
CEK = 32 cryptographically random bytes
```

using OS CSPRNG.

## 17.2 Per-object key derivation

For private object `object_id`:

```text
object_key =
HKDF-SHA256(
  IKM  = CEK,
  salt = project_id_bytes,
  info = "AFTERGLOW/object/v1/" || object_id
)
```

Output:

```text
32 bytes
```

## 17.3 Compression

Compress each object before encryption when beneficial.

Defaults:

```text
JSON/text    → Zstandard
PNG/JPEG/WebP → usually no secondary compression
Audio/video  → no secondary compression unless format benefits
```

Builder decides based on content type and measured result.

Do not increase size merely to satisfy a compression rule.

## 17.4 Chunking

Default encrypted chunk size:

```text
1 MiB
```

For each object:

```text
nonce_prefix = random 8 bytes
nonce = nonce_prefix || chunk_index_u32_be
```

AES-GCM nonce length:

```text
12 bytes
```

Builder MUST reject an object requiring more than `u32::MAX` chunks.

## 17.5 AAD

Each chunk's Associated Authenticated Data MUST bind:

```text
format_version
project_id
build_id
object_id
chunk_index
compression_method
```

## 17.6 Chunk record

Logical record:

```text
chunk_index
plaintext_length
ciphertext_length
ciphertext
gcm_tag
```

Exact binary field sizes MUST be documented in `ag-object-store`.

---

# 18. PUBLIC OBJECT INTEGRITY

Public objects are not secret.

For accidental corruption detection, each object has SHA-256 in the manifest.

Do not claim this alone prevents malicious replacement.

When the final EXE is Authenticode-signed and the capsule is inside a signed PE resource, executable signature verification provides a stronger publisher-level integrity boundary.

---

# 19. PROJECT CAPSULE

The project capsule is one versioned binary blob embedded as `RCDATA`.

Recommended resource identity:

```text
Type: RT_RCDATA
Name: AGLOW_CAPSULE
Language: neutral
```

## 19.1 Capsule header

Use a fixed little-endian header.

Minimum logical fields:

```text
magic[8]             = "AGCAPS1\0"
format_version_u16
header_size_u16
flags_u32
manifest_offset_u64
manifest_length_u64
public_store_offset_u64
public_store_length_u64
private_store_offset_u64
private_store_length_u64
tlock_offset_u64
tlock_length_u64
digest_table_offset_u64
digest_table_length_u64
reserved...
```

All offsets are relative to the beginning of the capsule.

Validate every arithmetic operation against overflow.

## 19.2 Size policy

Recommended v1 Builder policy:

```text
<= 512 MiB   normal
512 MiB–1 GiB warning
> 1 GiB      blocked by default unless expert override
```

This is a product reliability policy, not a claim about the maximum theoretical PE size.

Avoid giant video-heavy capsules.

---

# 20. WHY THE CAPSULE MUST BE A PE RESOURCE

The canonical Builder MUST mutate the precompiled Viewer template with Windows resource APIs before signing.

Do not use an appended EXE overlay as the canonical security boundary.

Reason:

Windows Authenticode PE hashing excludes some data regions, and data past the final PE section can fall outside the signed image digest.

Therefore:

```text
copy runtime template
↓
update icon/version resources
↓
insert AGLOW_CAPSULE RCDATA
↓
close resource update
↓
verify resource
↓
sign final PE
↓
verify signature
↓
NEVER MODIFY AGAIN
```

---

# PART III — MANDATORY TECHNICAL SPIKES

These spikes are BLOCKERS before production implementation.

# 21. SPIKE A — TIMelock INTEROPERABILITY

## Goal

Prove Windows Rust Viewer compatibility with drand Quicknet timelock ciphertext generated from the official/reference ecosystem.

## Required tests

1. Generate future-round test ciphertext with official Go `drand/tlock`.
2. Decrypt after target round with the selected Rust implementation.
3. Generate ciphertext with Rust implementation.
4. Decrypt with Go reference.
5. Reject pre-round decryption.
6. Reject modified ciphertext.
7. Reject beacon from wrong chain.
8. Reject invalid signature.
9. Run on Windows x64 CI.
10. Record dependency versions.

## Acceptance

All tests pass.

If they do not pass, timelock integration remains BLOCKED.

Do not write a new cryptographic scheme merely to unblock the project.

---

# 22. SPIKE B — NTS / TIME EVIDENCE ON WINDOWS

## Goal

Prove that the selected Rust time stack can obtain authenticated time evidence on Windows from at least two independent NTS operators.

Preferred defaults:

```text
Cloudflare NTS
Netnod NTS
```

Advisory additional sources:

```text
NIST NTP
Google Public NTP (separate leap-smear bucket)
Windows wall clock
```

## Acceptance

The spike must demonstrate:

- NTS key exchange,
- NTS-authenticated NTP response,
- RTT measurement,
- timeout handling,
- certificate validation,
- independent provider results,
- Windows execution.

If no maintainable Windows NTS integration is available, document the limitation and preserve the release security invariant: timelock remains the release gate.

Time evidence may degrade without weakening timelock.

---

# 23. SPIKE C — PE RESOURCE INJECTION + SIGNING

## Goal

Prove Builder can produce a one-file Viewer without recompiling Rust.

Procedure:

1. Copy precompiled Viewer template.
2. Insert test `RCDATA`.
3. Replace app icon/version info.
4. Load resource from runtime.
5. Sign EXE.
6. Verify signature.
7. Verify capsule remains readable.
8. Mutate resource after signing and confirm signature verification fails.
9. Test on clean Windows VM.

## Acceptance

All steps pass.

---

# 24. SPIKE D — WGPU TEMPORAL GLASS

## Goal

Prove default glass effects can maintain a stable target frame rate on representative integrated and discrete GPUs.

Test:

- 960×640
- 1440×900
- 1920×1080
- 100%, 150%, 200% DPI
- integrated GPU
- discrete GPU
- Reduced Transparency
- low-quality fallback

Acceptance:

- input remains responsive,
- 60 FPS target is achievable on reference hardware or effects degrade automatically,
- no mandatory content depends on the glass effect.

---

# PART IV — RELEASE AND TIMelock

# 25. CURRENT TIMelock NETWORK BASELINE

As of the external review date, drand documents Quicknet as the production unchained network used for timelock encryption.

Current chain hash:

```text
52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971
```

Documented period:

```text
3 seconds
```

The project MUST NOT trust the string `"quicknet"` alone.

A network profile contains:

```text
profile_id
chain_hash
public_key
genesis_time
period
scheme_id
relay list
```

Before shipping a production Builder release, maintainers MUST re-check these external assumptions against official drand data.

---

# 26. TARGET ROUND COMPUTATION

Builder converts the chosen release UTC instant to the **first scheduled round at or after the release instant**.

Let:

```text
G = genesis UTC instant of round 1
P = period
T = requested release UTC
```

Then:

```text
delta = T - G
target_round = ceil(delta / P) + 1
```

for `T >= G`.

The scheduled target-round time is:

```text
G + (target_round - 1) * P
```

Builder MUST display:

```text
Requested release
Effective timelock round
Scheduled round time
Maximum nominal rounding delay (< one period)
```

This is important because a 3-second beacon period means release may occur slightly after the requested wall-clock instant.

---

# 27. BUILD-TIME PINNING

The build embeds:

```text
requested_release_utc
target_round
chain_hash
public_key
genesis_time
period
scheme_id
relay allowlist
```

The Viewer MUST use the embedded target round.

The Viewer MUST NOT recompute target round from the local clock.

---

# 28. RELAY STRATEGY

Embed multiple relays.

Current examples may include official/public League of Entropy endpoints.

The implementation SHOULD race multiple relays with sensible delays and use the first **valid cryptographically verified** response.

Do not accept the first HTTP 200 without verification.

Network errors are not cryptographic failures.

---

# 29. RELEASE ALGORITHM

Pseudo-code:

```text
if archive_already_ready_in_memory:
    return READY

target = capsule.release.target_round

responses = fetch_target_round_from_allowlisted_relays(target)

for response in responses:
    if verify_beacon(response, pinned_network_identity):
        beacon = response
        break

if no valid beacon:
    return WAITING_FOR_RELEASE_MATERIAL

cek = timelock_decrypt(capsule.tlock_envelope, beacon)

if cek invalid:
    return LOCKED_ERROR

verify/decrypt required private manifest object

if authentication fails:
    zeroize cek
    return INTEGRITY_ERROR

return READY
```

No local date comparison grants cryptographic access.

---

# 30. EARLY REQUEST BEHAVIOR

It is safe for the Viewer to request the exact target round before release.

Before the target round exists, a valid target-round beacon is unavailable.

Therefore the Viewer MAY attempt target-round retrieval:

- near release,
- after local estimated zero,
- when the user presses Retry,
- when a network reconnect event occurs.

Rate-limit requests.

---

# 31. TIME EVIDENCE IS SEPARATE

The countdown time engine and the release engine are intentionally separate.

```text
Time evidence
→ UX / countdown / anomaly detection

Timelock target beacon
→ actual release capability
```

If all NTP/NTS sources are unavailable but a valid target-round drand beacon exists, the Viewer MAY release.

If local/network time says “future” but the target beacon does not exist, the Viewer MUST remain locked.

---

# 32. TIMelock LIBRARY REQUIREMENT

The Viewer SHOULD use a Rust implementation for native integration.

However, production adoption requires Spike A.

Current candidate families include:

- `tlock-rs` / `tlock_age`
- `drand-rs` client components

These are not automatically trusted merely because they are written in Rust.

The Go `drand/tlock` implementation is the interoperability reference for testing.

---

# PART V — TIME EVIDENCE ENGINE

# 33. PURPOSE

Time evidence provides:

- accurate countdown,
- clock-jump detection,
- confidence status,
- release-stage UI changes,
- user diagnostics.

It does not hold the decryption secret.

---

# 34. PROVIDER CLASSES

## Tier A — Authenticated unsmeared UTC evidence

Preferred:

```text
Cloudflare NTS
Netnod NTS
```

## Tier B — Reputable unsmeared NTP evidence

Example:

```text
NIST Internet Time Service
```

Unauthenticated NTP receives lower trust.

## Tier C — Separate smeared-time evidence

Example:

```text
Google Public NTP
```

Google leap-smeared time MUST NOT be naively averaged with unsmeared UTC during a smear interval.

## Tier D — Local evidence

```text
Windows wall clock
monotonic performance counter
```

---

# 35. TIME OBSERVATION MODEL

```rust
struct TimeObservation {
    provider_id: ProviderId,
    operator_id: OperatorId,
    timescale: TimeScale,
    authenticated: bool,
    estimated_utc: SystemTime,
    uncertainty: Duration,
    rtt: Duration,
    monotonic_received_at: Instant,
}
```

`operator_id` exists so multiple hostnames run by one operator are not counted as independent organizations.

---

# 36. CONSENSUS RULE

Recommended algorithm:

1. Query multiple operators.
2. Discard malformed responses.
3. Calculate a conservative interval per response using RTT/uncertainty.
4. Group by compatible timescale.
5. Prefer authenticated unsmeared UTC evidence.
6. Identify mutually intersecting intervals.
7. Flag outliers.
8. Produce:
   - trusted midpoint,
   - trusted uncertainty,
   - confidence class.
9. Anchor subsequent short-term countdown progression to monotonic time.
10. Revalidate after sleep/resume or a large wall-clock discontinuity.

---

# 37. CONFIDENCE CLASSES

```text
VERIFIED
  >= 2 independent authenticated compatible sources agree

GOOD
  one authenticated source + supporting independent source

DEGRADED
  only unauthenticated reputable sources agree

LOCAL_ONLY
  network unavailable; estimate based on last trusted anchor/local clock

DISAGREEMENT
  trustworthy observations conflict
```

The UI should not expose these technical names unless expanded.

User-facing examples:

```text
Time verified
Time partially verified
Offline estimate
Time sources disagree
```

---

# 38. QUERY FREQUENCY

Default:

```text
on launch
on network reconnect
on resume
after large wall-clock change
periodically at low frequency
more frequently near release
```

Do not query every second.

Suggested baseline:

```text
>24 h remaining: every 30–60 min
1–24 h: every 5–15 min
<1 h: every 1–5 min
<1 min: do not hammer time services; rely on monotonic anchor
```

The exact values may be tuned.

---

# 39. LEAP HANDLING

Do not mix Google smeared time into unsmeared consensus near a leap event.

The provider model MUST include timescale/leap behavior.

When uncertain, exclude smeared sources from the primary UTC consensus and use them only as advisory evidence.

---

# PART VI — CRYPTOGRAPHIC OBJECT STORE

# 40. PRIVATE OBJECT ACCESS

The Viewer decrypts only required private objects.

Example:

```text
open contributor
↓
decrypt profile JSON
↓
render metadata
↓
decrypt portrait if private
↓
decrypt message blocks as needed
```

Do not unpack the private project to a normal directory.

---

# 41. KEY LIFETIME

The CEK SHOULD exist in process memory only after legitimate release.

Per-object keys are derived on demand.

Use zeroization where supported for:

- CEK buffers,
- derived object keys,
- temporary plaintext buffers.

Do not claim zeroization can defeat every debugger or OS memory copy.

---

# 42. STRICT PORTABLE MODE

Default post-release behavior:

- do not persist CEK,
- do not create project sidecar files,
- reacquire already-public target beacon on later launches.

After release the beacon is public, so reacquisition does not weaken secrecy.

Internet is required again on later launches unless a local persistence option is enabled.

---

# 43. OPTIONAL POST-RELEASE PERSISTENCE

Builder MAY enable:

```text
Windows Protected Cache
```

If enabled:

- protect recovered CEK or equivalent state with Windows DPAPI,
- store under user profile,
- clearly explain that runtime state exists outside the EXE.

This mode violates strict runtime portability but not one-file distribution.

Default: OFF.

---

# 44. EXTERNAL CREATOR RECOVERY

Default Balanced profile SHOULD support an optional creator-side recovery artifact that is **not distributed with the recipient EXE**.

Possible artifact:

```text
ProjectName.agrecovery
```

It contains a recovery-wrapped CEK.

Recommended protection:

```text
Argon2id(passphrase)
↓
256-bit KEK
↓
AES-256-GCM wrap CEK
```

The recovery artifact may be stored separately by the creator.

It can be used if the public timelock infrastructure permanently fails.

Possession of recovery credentials may allow early decryption; this is a creator trust decision.

---

# 45. EMBEDDED HERITAGE RECOVERY

Advanced only.

If enabled, Builder MUST display a strong warning:

> Embedded recovery improves eventual self-contained recoverability but weakens resistance to expert reverse engineering because all recovery logic/material required by the fallback must exist in the recipient EXE.

Do not label it “cryptographic timelock backup.”

Default: OFF.

---

# PART VII — AUTHENTICODE AND BINARY PACKAGING

# 46. PACKAGING PIPELINE

Canonical order:

```text
1. Freeze project snapshot
2. Validate project
3. Optimize assets
4. Generate public objects
5. Generate private objects
6. Generate CEK
7. Encrypt private objects
8. Generate timelock CEK envelope
9. Serialize project capsule
10. Copy Viewer Runtime Template
11. Apply project icon/version resources
12. Insert AGLOW_CAPSULE RCDATA
13. Close resource update
14. Reopen EXE and verify capsule byte-for-byte
15. Run plaintext leak scan
16. Run executable smoke test
17. Optional Authenticode signing
18. Verify Authenticode
19. Compute final SHA-256
20. Generate Build Report
```

Nothing may mutate the EXE after step 17 except operations explicitly permitted by the signing mechanism, such as supported timestamping as part of signing.

---

# 47. PE RESOURCE IMPLEMENTATION

Builder should use Windows APIs equivalent to:

```text
BeginUpdateResourceW
UpdateResourceW
EndUpdateResourceW
```

Viewer loads `AGLOW_CAPSULE` using the normal PE resource-loading APIs.

The packager MUST verify that the exact bytes read from the produced PE match the generated capsule hash before signing.

---

# 48. CODE SIGNING

Authenticode is recommended for public distribution.

Builder supports:

```text
Unsigned
External SignTool integration
Configured signing provider
```

Private signing credentials MUST NOT be stored in project configuration.

Signing happens after all resource mutation.

---

# 49. SMARTSCREEN REALITY

A valid code signature:

- identifies a publisher,
- allows reputation to accumulate,
- improves provenance,

but does not guarantee a new binary will never show a SmartScreen warning.

Do not promise “signed = no warning.”

---

# PART VIII — BUILDER PRODUCT SPECIFICATION

# 50. BUILDER MISSION

AFTERGLOW Builder is a no-code/low-code project authoring environment.

A non-programmer should be able to create a polished project without editing source code.

Builder combines:

```text
content editor
+
design system editor
+
release configurator
+
security packager
+
preview environment
```

---

# 51. BUILDER MODES

## Owner Mode

Full project authority.

## Contributor Mode

Restricted authoring for one contributor assignment.

Contributor Mode MUST NOT expose:

- other private entries,
- project-wide recovery material,
- signing credentials,
- release-security controls,
- raw CEK.

---

# 52. BUILDER SHELL

Recommended desktop layout:

```text
┌────────────────────────────────────────────────────────────────────┐
│ AFTERGLOW Builder  Project       Preview ▾  Validate   Build       │
├──────────────┬──────────────────────────────────┬──────────────────┤
│ Project nav  │                                  │ Inspector        │
│              │            Canvas                │                  │
│ Overview     │                                  │ Contextual       │
│ Identity     │                                  │ controls         │
│ Terms        │                                  │                  │
│ Contributors │                                  │                  │
│ Content      │                                  │                  │
│ Experience   │                                  │                  │
│ Theme        │                                  │                  │
│ Motion       │                                  │                  │
│ Release      │                                  │                  │
│ Security     │                                  │                  │
│ Build        │                                  │                  │
├──────────────┴──────────────────────────────────┴──────────────────┤
│ Problems 0   Warnings 1                    Autosaved 15:43         │
└────────────────────────────────────────────────────────────────────┘
```

---

# 53. BUILDER NAVIGATION

Primary sections:

```text
Overview
Identity
Terminology
Schema
Contributors
Content
Structure
Experience
Theme
Motion
Accessibility
Localization
Release
Security
Preview
Diagnostics
Build
```

---

# 54. NEW PROJECT WIZARD

Steps:

1. Choose preset.
2. Enter project name.
3. Choose language.
4. Choose project shape.
5. Choose release date/time.
6. Choose visual preset.
7. Choose contributor model.
8. Create.

Use safe defaults.

Do not require users to understand cryptography in the wizard.

---

# 55. PRESETS

Required initial presets:

```text
Blank
Broadcast Club
Graduation
Friends
Family Time Capsule
Team Farewell
Anniversary
Personal Future Letter
Class Archive
Community Archive
Custom Multi-Contributor
```

Presets are configuration bundles.

No preset may require special Viewer source code.

---

# 56. IDENTITY EDITOR

Fields:

```text
project display name
subtitle
short introduction
creator/group credit
project period
logo
symbol
window title
EXE filename
app icon
pre-release tagline
release headline
post-release headline
```

---

# 57. TERMINOLOGY EDITOR

Example:

```text
Contributor singular   [ Member          ]
Contributor plural     [ Members         ]
Recipient singular     [ Junior          ]
Recipient plural       [ Juniors         ]
Entry singular         [ Letter          ]
Entry plural           [ Letters         ]
Role label             [ Broadcast Role  ]
Archive label          [ Our Club        ]
Open action            [ Open the Letter ]
```

All built-in Viewer copy references semantic tokens.

---

# 58. CUSTOM SCHEMA EDITOR

Supported field types:

```text
short text
long text
number
date
date range
single select
multi-select
boolean
image
tag list
URL
hidden ID
```

Visibility:

```text
builder-only
public pre-release
public post-release
private encrypted
metadata only
```

---

# 59. CONTRIBUTOR MODEL

Contributor statuses:

```text
EMPTY
DRAFT
SUBMITTED
NEEDS_REVIEW
APPROVED
LOCKED_FOR_BUILD
```

Builder supports:

- grid,
- table,
- reorder,
- tags,
- bulk field updates,
- completeness indicator,
- validation indicator.

---

# 60. CONTRIBUTOR SLOT

A slot defines required content.

Example:

```json
{
  "slot_type": "person-letter",
  "profile_fields": [
    "display_name",
    "role",
    "portrait"
  ],
  "required_blocks": [
    "message_title",
    "rich_text"
  ],
  "limits": {
    "images": 8,
    "audio_seconds": 180,
    "video_seconds": 0
  }
}
```

---

# 61. OFFLINE MULTI-AUTHOR WORKFLOW

No server is required.

```text
Owner
↓
Export assignment
↓
Alice.aginvite
↓
Contributor Builder Mode
↓
write/import
↓
Export submission
↓
Alice.agentry
↓
Owner imports
```

Both package types include:

```text
project_id
slot_id
schema_version
submission_token
```

A package from another project MUST be rejected.

---

# 62. CONTRIBUTOR PRIVACY

Default workflow assumes the Project Owner can review imported content.

A “sealed local package” may protect against casual file browsing, but must not be marketed as cryptographic privacy from the Owner if the Owner ultimately holds the project packaging authority.

---

# 63. CONTENT BLOCK SYSTEM

Required blocks:

```text
Heading
Paragraph
Quote
Signature
Divider
Image
Image Pair
Gallery
Caption
Timeline
Callout
Date Stamp
Credits
Spacer
Link
```

SHOULD:

```text
Audio
```

MAY / feature-flagged:

```text
Short Video
```

No arbitrary HTML/JavaScript.

---

# 64. RICH TEXT

Allowed inline formatting:

```text
bold
italic
optional underline
theme emphasis
link
soft break
```

Typography remains theme-controlled.

Do not let individual authors set arbitrary fonts and 70 different text sizes.

---

# 65. EXTERNAL LINKS

A Link block MUST:

- display the destination domain,
- open via the system browser,
- not fetch remote HTML into the Viewer,
- not execute remote script.

This improves privacy, security, and preservation.

---

# 66. ASSET MANAGER

Support:

```text
portrait
photo
logo
icon
audio
video
background
decorative texture
font
```

Show:

```text
dimensions
original size
optimized size
metadata state
privacy state
usage count
```

Actions:

```text
Optimize
Crop
Focal point
Replace
Remove metadata
Find usages
Delete
```

---

# 67. IMAGE IMPORT

Default:

1. decode with bounded resource limits,
2. correct orientation,
3. detect metadata,
4. strip GPS by default,
5. create preview proxy,
6. preserve project-managed source,
7. create optimized build variants.

---

# 68. STRUCTURE EDITOR

Page types:

```text
Pre-Release Home
About
Collection
Contributor Detail
Entry Detail
Gallery
Timeline
Credits
Technical Status
Static Page
```

Builder controls which are enabled.

---

# 69. COLLECTION LAYOUTS

Required:

```text
Constellation
Portrait Grid
Timeline
Chapters
Carousel
Single Entry
```

Collection visuals may be organic, but keyboard/focus order remains deterministic.

---

# 70. EXPERIENCE EDITOR

Configurable:

```text
show intro
show contributor previews
show exact release date
show seconds
show time-integrity badge
require ENTER after ceremony
show About
show Technical Status
allow full screen
post-release landing page
```

---

# 71. PRE-RELEASE DESIGNER

Components:

```text
Wordmark
Tagline
Introduction
D-Day
Precise countdown
Orbital progress
Contributor previews
Release date
Time badge
About
Settings
```

Allow reordering within safe layout slots.

Do not provide unrestricted absolute positioning in v1.

---

# 72. RELEASE CEREMONY PRESETS

Required:

```text
Glass Thaw
Light Reveal
Quiet Fade
Orbit Resolve
Minimal
```

Customizable:

```text
headline
secondary copy
duration intensity
sound
warmth shift
spectral accent
ENTER label
```

---

# 73. THEME STUDIO

Sections:

```text
Palette
Typography
Materials
Background
Geometry
Spacing
Icons
Photography
Contrast
```

Modes:

```text
Basic
Advanced
```

---

# 74. THEME PRESETS

```text
Temporal Glass — Dark
Temporal Glass — Pearl
Y2K Aurora
Quiet Archive
Midnight Chrome
Warm Memory
Minimal Future
Custom
```

---

# 75. SEMANTIC COLOR TOKENS

```text
background
background_secondary
surface
text_primary
text_secondary
accent_primary
accent_secondary
success
warning
error
glass_tint
glass_edge
focus
```

Default palette:

| Token | Value |
|---|---|
| Space | `#070915` |
| Midnight | `#0E1630` |
| Deep Blue | `#101A35` |
| Pearl | `#F7F8FF` |
| Mist | `#D7D8E8` |
| Aqua | `#7BE7FF` |
| Violet | `#B19CFF` |
| Pink | `#FF9DE6` |
| Mint | `#80F2C2` |
| Amber | `#FFD27A` |
| Rose | `#FF8098` |

Builder MUST run contrast checks.

---

# 76. TYPOGRAPHY ROLES

```text
Brand
Display
Countdown
UI
Body
Signature
Technical
```

Countdown font SHOULD support tabular figures.

Korean long-form text MUST prioritize readability.

---

# 77. FONT POLICY

Custom fonts require creator confirmation that redistribution is permitted.

Builder MUST provide a fallback font stack.

Missing custom font must not break the project.

---

# 78. GLASS MATERIAL EDITOR

Materials:

```text
Glass/Clear
Glass/Regular
```

Parameters:

```text
tint
opacity
blur
refraction
edge highlight
specular
dispersion
noise
```

Safe defaults limit exaggerated effects.

---

# 79. GLASS DEFAULT LIMITS

| Property | Default maximum |
|---|---:|
| card scale | `1.015` |
| pointer translation | `2 px` |
| refraction shift | `3 px` |
| RGB split | `< 1 px` |
| hover highlight gain | `18%` |

---

# 80. BACKGROUND STUDIO

Layers:

```text
Solid
Gradient
Aurora
Static image
Grain
Orbital geometry
Sparse particles
```

Builder SHOULD cap simultaneous animated layers.

---

# 81. MOTION STUDIO

Motion classes:

```text
Reactive
Spatial
Ambient
Ceremonial
```

Presets:

```text
Calm
Standard
Expressive
Custom
```

---

# 82. MOTION TOKENS

| Token | Default |
|---|---:|
| press-in | `70 ms` |
| press-out | `100 ms` |
| hover | `120 ms` |
| fast | `167 ms` |
| standard | `250 ms` |
| card | `333 ms` |
| back | `400 ms` |
| media-open | `500 ms` |
| forward | `600 ms` |
| app-enter | `750 ms` |
| collection-enter | `800 ms` |
| ceremony | `2700 ms` |
| ambient cycle | `25–40 s` |

---

# 83. EASING TOKENS

Defaults:

```text
enter:
cubic-bezier(0, 0, 0, 1)

move:
cubic-bezier(.55, .55, 0, 1)

exit:
fast acceleration + fade

ambient:
smooth sine/ease-in-out
```

Avoid arbitrary per-component physics.

---

# 84. ACCESSIBILITY STUDIO

Builder exposes:

```text
Reduced Motion preview
Reduced Transparency preview
High Contrast preview
text scaling
focus style
keyboard test
sound defaults
caption rules
```

Accessibility warnings are part of Diagnostics.

---

# 85. LOCALIZATION

Support project locales such as:

```text
ko-KR
en-US
ja-JP
```

Built-in UI labels are localizable.

Contributor-authored text is not automatically translated unless the owner supplies translations.

---

# 86. RELEASE EDITOR

Required fields:

```text
release date
release local time
IANA timezone
resolved UTC
timelock network profile
effective target round
time provider profile
offline behavior
recovery profile
```

Builder displays both local and UTC.

Example:

```text
Local
2030-03-02 00:00 Asia/Seoul

UTC
2030-03-01 15:00:00Z
```

---

# 87. RELEASE CHANGE WARNING

After a security build exists, changing release time invalidates:

- target round,
- timelock envelope,
- build identity.

Builder must require a rebuild.

---

# 88. SECURITY PROFILES

## Strict

```text
timelock
no embedded recovery
no persisted post-release CEK
```

## Balanced — DEFAULT

```text
timelock
optional creator-held external recovery package
no embedded recovery
no persisted post-release CEK
```

## Heritage Embedded — ADVANCED

```text
timelock
embedded delayed fallback
explicit reverse-engineering warning
```

## Custom

Expert-only.

---

# 89. PREVIEW MATRIX

Builder MUST preview:

```text
first launch
normal pre-release
7 days
24 hours
1 hour
60 seconds
10 seconds
release verification
release ceremony
post-release home
entry detail
gallery
offline
time disagreement
invalid/corrupt project mock
Reduced Motion
Reduced Transparency
High Contrast
low GPU
960×640
1440×900
1920×1080
200% DPI
```

---

# 90. TIME-TRAVEL PREVIEW

Preview state is injected into the renderer through a mock state provider.

Production release code MUST NOT contain a “preview unlock” environment variable, CLI switch, registry key, magic date, or hidden keyboard shortcut.

---

# 91. AUTOSAVE

Builder MUST:

- save debounced edits,
- use atomic file replacement for manifests,
- retain periodic recovery snapshots,
- expose last-save status.

---

# 92. UNDO / REDO

Support:

```text
Ctrl+Z
Ctrl+Shift+Z
```

for normal content/theme operations.

---

# 93. DIAGNOSTICS

Severity:

```text
ERROR
HIGH_WARNING
WARNING
INFO
```

Build-blocking examples:

- invalid release time,
- unresolved schema error,
- duplicate nonce,
- timelock generation failure,
- missing required object,
- capsule serialization failure,
- runtime template mismatch.

High warnings:

- unsigned EXE,
- release horizon very long,
- no recovery option,
- contrast issue,
- huge project,
- custom font rights not confirmed.

---

# PART IX — VIEWER UX

# 94. VIEWER STATE MACHINE

```text
BOOT
  ↓
PRE_RELEASE
  ↓
RELEASE_MATERIAL_CHECK
  ├─ unavailable → PRE_RELEASE/WAITING
  └─ valid
       ↓
UNLOCKING
       ↓
AUTHENTICATING
       ↓
READY_FOR_CEREMONY
       ↓
CEREMONY
       ↓
POST_RELEASE_HOME
       ↓
ENTRY_DETAIL
       ↓
MEDIA_VIEWER
```

Error states overlay or return to locked safe state.

---

# 95. BOOT

Boot sequence:

1. Load RCDATA capsule.
2. Check magic/version/bounds.
3. Verify section digests.
4. Parse public manifest.
5. Initialize theme.
6. Initialize accessibility state.
7. Initialize GPU.
8. Begin time evidence.
9. Render first usable frame.

Do not wait for network before rendering the pre-release UI.

---

# 96. PRE-RELEASE DEFAULT UI

```text
PROJECT WORDMARK

PROJECT TAGLINE

D - 928

12 : 06 : 41 : 29
DAYS  HOURS  MIN  SEC

Short introduction

[optional public contributor previews]

● Time verified
```

---

# 97. COUNTDOWN HIERARCHY

More than 24 hours:

```text
D-day dominant
```

Under 24 hours:

```text
precise clock dominant
```

Under 60 seconds:

```text
seconds gain strongest emphasis
```

Do not become a loud New-Year countdown unless explicitly configured.

---

# 98. ORBITAL TIME INDICATOR

The default orbit represents overall project progress.

It does not continuously spin.

A moving dot corresponds to actual long-term progress.

---

# 99. TIME STATUS

Main surface:

```text
● Time verified
```

Expanded:

```text
TIME INTEGRITY

Status
Verified

Last check
...

Independent sources
...

Local offset
...

Technical details
```

Do not force recipients to understand NTS/drand to use the project.

---

# 100. RELEASE WAITING STATE

If visual countdown has reached zero but target beacon is unavailable:

```text
The release time has arrived.

AFTERGLOW is waiting for verifiable release data.

[ Try again ]
```

Do not show an alarming “security failure” for ordinary network delay.

---

# 101. POST-RELEASE LAYOUTS

Renderer must support:

```text
Constellation
Portrait Grid
Timeline
Chapters
Carousel
Single Entry
```

---

# 102. CONSTELLATION

Default people-centered behavior:

- stable positions,
- faint relationships/lines,
- no constant floating,
- deterministic keyboard navigation,
- visible focus,
- no overlapping interactive hit targets.

---

# 103. ENTRY DETAIL

Generic:

```text
← Back

[Hero visual]

Entry title
Contributor metadata

Content blocks

Optional media

Signature / credits
```

Long text must use a stable reading surface.

---

# 104. READING WIDTH

Recommended:

```text
680–780 px
```

Avoid full-monitor-width paragraphs.

---

# PART X — TEMPORAL GLASS

# 105. DESIGN PHILOSOPHY

Default design mix:

```text
40% modern minimal Windows clarity
30% Liquid Glass
20% restrained Y2K futurism
10% archive / scrapbook cues
```

Y2K is expressed through material and optimism, not retro imitation.

---

# 106. VISUAL LAYERS

## Atmosphere

- dark field,
- subtle aurora,
- sparse geometry,
- restrained grain.

## Content

- readable,
- stable,
- mostly opaque.

## Glass Controls

- navigation,
- status,
- action capsules,
- overlays.

---

# 107. GLASS SHADER PIPELINE

```text
scene sample
↓
edge-biased refraction
↓
diffusion
↓
luminance adaptation
↓
tint
↓
inner edge
↓
specular
↓
subtle spectral dispersion
↓
foreground
```

---

# 108. DO NOT USE GLASS FOR

- long letter surfaces,
- every nested panel,
- raw photos,
- critical text with insufficient contrast.

---

# 109. WINDOW BEHAVIOR

Preserve Windows expectations:

- resize,
- minimize,
- maximize,
- close,
- DPI,
- keyboard,
- Snap where feasible.

Custom visuals must not break basic desktop behavior.

---

# PART XI — TEMPORAL MOTION

# 110. MOTION PRINCIPLE

Every motion is one of:

```text
Reactive
Spatial
Ambient
Ceremonial
```

Anything else requires justification.

---

# 111. APP ENTRY

Default total:

```text
~750 ms
```

```text
0–250 ms     background
120–450 ms   wordmark
250–600 ms   countdown
400–750 ms   secondary controls
```

The interface becomes interactive as soon as the state is ready.

---

# 112. AMBIENT AURORA

```text
cycle: 25–40 s
travel: 2–5% viewport
```

Pause/reduce when:

- minimized,
- inactive,
- Battery Saver,
- Reduced Motion,
- low GPU tier.

---

# 113. HOVER

Default:

```text
120–167 ms
scale <= 1.012
small highlight increase
```

Avoid large lifts and card flips.

---

# 114. PRESS

```text
press-in: 70 ms to ~0.992
release: 100 ms to 1.000
```

Action dispatch does not wait for animation completion.

---

# 115. COUNTDOWN DIGITS

Normal second:

```text
instant replacement
+ optional 80 ms opacity settle
```

No flip clock by default.

---

# 116. CONNECTED TRANSITION

If a portrait/image exists in both views, animate the same visual identity.

Forward:

```text
~600 ms
```

Back:

```text
~400 ms
```

---

# 117. RELEASE BUILD-UP

## 7 days

Subtle atmosphere increase.

## 24 hours

Precise countdown gains prominence.

## 1 hour

Seconds become clearer.

## 60 seconds

Ambient motion decreases.

## 10 seconds

Subtle luminance pulse per second.

---

# 118. RELEASE VERIFICATION LOADER

Use the project/orbit symbol.

Default revolution:

```text
1.6–2.0 s
```

Linear rotation is acceptable for indefinite progress.

---

# 119. GLASS THAW

Default flagship ceremony:

```text
~2700 ms
```

| Time | Event |
|---:|---|
| `0–300 ms` | countdown freezes |
| `200–700 ms` | secondary UI fades |
| `400–1100 ms` | refraction intensifies |
| `700–1400 ms` | frost dissolves |
| `1000–1700 ms` | spectral light crosses |
| `1300–1900 ms` | glass clears |
| `1600–2200 ms` | atmosphere warms |
| `1900–2500 ms` | project symbol resolves |
| `2200–2700 ms` | release text appears |

Default:

```text
The time has come.

It is time to open what was left here.

[ ENTER ]
```

---

# 120. CEREMONY PROHIBITIONS

Default ceremony MUST NOT include:

- screen shake,
- white flash,
- glass explosion,
- confetti,
- fireworks,
- aggressive zoom,
- high-frequency flashing.

---

# PART XII — ACCESSIBILITY

# 121. WINDOWS PREFERENCE INTEGRATION

Viewer MUST respond to system preferences for:

- animations,
- advanced/transparency effects.

On Windows, use the relevant `UISettings`/platform APIs or equivalent stable WinRT access.

---

# 122. REDUCED MOTION

Normal:

```text
spatial movement
```

Reduced:

```text
crossfade / short opacity transition
```

Normal Glass Thaw becomes:

```text
short fade
release message
ENTER
```

Meaning remains intact.

---

# 123. REDUCED TRANSPARENCY

Replace dynamic glass with:

- opaque/tinted surfaces,
- stronger borders,
- no refraction,
- no dispersion,
- reduced blur.

---

# 124. HIGH CONTRAST

No essential meaning may depend on:

- transparency,
- subtle glow,
- color alone.

---

# 125. KEYBOARD

Minimum:

```text
Tab
Shift+Tab
Arrow keys
Enter
Space
Escape
Alt+F4
```

Do not trap focus.

---

# 126. FOCUS

Default:

```text
2 px solid bright outline
3 px separation
optional subtle cyan glow
```

Glow alone is insufficient.

---

# PART XIII — PERFORMANCE

# 127. FRAME TARGET

Primary:

```text
stable 60 FPS
```

Support high-refresh displays where possible.

Animation time is based on elapsed time, not frame number.

---

# 128. GPU-FRIENDLY ANIMATION

Prefer:

```text
transform
opacity
shader uniforms
UV
color
mask
refraction
highlight
```

Avoid every-frame:

```text
text reflow
image decode
large layout rebuild
CPU rasterization
```

---

# 129. ADAPTIVE QUALITY

Suggested:

| Average frame time | Mode |
|---|---|
| `<12 ms` | Ultra |
| `12–16.7 ms` | Full |
| `16.7–25 ms` | Reduced FX |
| `>25 ms` | Static Glass |

Reduction order:

```text
dispersion
refraction samples
blur radius
aurora animation
ambient decoration
```

Priority:

```text
Interaction
> Navigation
> Ceremony correctness
> Glass physics
> Ambient
```

---

# 130. IDLE BEHAVIOR

Minimized:

- stop ambient animation,
- reduce/stop frame production,
- stop frequent polling.

Inactive:

- reduce ambient effects.

The app may remain open for days.

---

# PART XIV — PRIVACY AND NETWORKING

# 131. NO TELEMETRY BY DEFAULT

Viewer MUST NOT send:

- project ID,
- contributor names,
- message text,
- media,
- recipient behavior,
- analytics.

Default telemetry:

```text
NONE
```

---

# 132. NETWORK ALLOWLIST

Viewer networking is limited to configured release/time providers and intentional user-opened links.

No arbitrary project-configured background HTTP requests.

---

# 133. NETWORK PRIVACY

Relay/time operators may observe the user's IP.

The Viewer SHOULD:

- send no project content,
- avoid unique project-identifying query parameters,
- use a generic runtime user agent if one is required,
- fetch only required public timing/beacon data.

---

# 134. FACE / PERSONAL DATA

Builder SHOULD remind creators:

- obtain permission,
- define intended audience,
- strip GPS metadata,
- avoid unnecessary personal details.

---

# PART XV — BUILD VALIDATION

# 135. BUILD PIPELINE REQUIREMENT IDS

## AG-BLD-001

Builder MUST generate a frozen immutable project snapshot before packaging.

Acceptance:
editing after Build starts cannot mutate the in-progress snapshot.

## AG-BLD-002

Builder MUST validate every required schema.

## AG-BLD-003

Builder MUST generate CEK from OS CSPRNG.

## AG-BLD-004

Builder MUST encrypt every private object before inserting the capsule.

## AG-BLD-005

Builder MUST generate timelock protection for the CEK.

## AG-BLD-006

Builder MUST insert the capsule as `RCDATA`.

## AG-BLD-007

Builder MUST read the generated resource back and compare SHA-256 before signing.

## AG-BLD-008

Builder MUST run plaintext leak checks.

## AG-BLD-009

Builder MUST sign only after all PE mutations.

## AG-BLD-010

Builder MUST compute final EXE SHA-256 after signing.

---

# 136. PLAINTEXT LEAK SCAN

Check final EXE for:

- known private message excerpts,
- private source filenames,
- private JSON field values,
- private asset magic/byte samples where practical,
- CEK test value,
- build secrets.

This is a safety net, not a proof.

---

# 137. BUILD REPORT

Creator-side report:

```text
Project
Build ID
Viewer version
Schema version
Release UTC
Target round
Timelock chain hash
Security profile
Recovery profile
Object counts
Private bytes
Public bytes
Capsule SHA-256
Final EXE SHA-256
Authenticode status
Warnings
```

---

# PART XVI — TESTING

# 138. CRYPTO TESTS

MUST include:

- CEK generation length/entropy source smoke,
- per-object key derivation deterministic test vector,
- nonce uniqueness,
- chunk reorder rejection,
- chunk modification rejection,
- wrong object ID AAD rejection,
- wrong project ID rejection,
- truncation rejection,
- corrupted GCM tag rejection.

---

# 139. TIMelock TESTS

MUST include:

- Go reference → Rust decrypt,
- Rust → Go reference decrypt,
- too-early behavior,
- wrong chain,
- wrong round,
- modified timelock ciphertext,
- invalid beacon,
- relay failure,
- multiple-relay race,
- Windows CI.

---

# 140. TIME TESTS

- authenticated source agrees,
- one operator unavailable,
- source outlier,
- high RTT,
- local clock +1 day,
- local clock -1 day,
- timezone changed,
- resume,
- network reconnect,
- Google smear source excluded from unsmeared consensus where required,
- no network.

---

# 141. PACKAGING TESTS

- capsule inserted,
- icon applied,
- resource loaded,
- SHA matches,
- signed build verifies,
- post-sign resource mutation breaks signature,
- clean machine launch,
- one-file distribution.

---

# 142. VIEWER SECURITY TESTS

- `strings` does not reveal private messages,
- resource extractor sees ciphertext for private objects,
- local future clock does not unlock without target beacon,
- branch patching cannot manufacture valid tlock beacon,
- malformed capsule fails safely,
- integer overflow/fuzz tests for capsule parser.

---

# 143. FUZZING

Fuzz:

```text
capsule header parser
manifest parser
object table
encrypted chunk reader
network response parser
timelock response decoder
```

Parser code handling untrusted network or binary data is high priority.

---

# 144. UI TESTS

Test:

```text
960×640
1280×720
1440×900
1920×1080
ultrawide
100%
125%
150%
175%
200% DPI
60 Hz
120 Hz
144 Hz
keyboard only
Reduced Motion
Reduced Transparency
High Contrast
integrated GPU
remote desktop
```

---

# 145. BUILDER TESTS

- every preset creates valid project,
- terminology fully changes Viewer copy,
- custom schema,
- assignment export/import,
- wrong-project submission rejected,
- autosave restore,
- undo/redo,
- theme preview,
- motion preview,
- accessibility preview,
- build with/without signing,
- corrupted asset validation,
- runtime-template version mismatch.

---

# PART XVII — IMPLEMENTATION PLAN FOR AI AGENTS

# 146. PHASE 0 — BOOTSTRAP

Create:

- workspace,
- crates,
- CI,
- formatting,
- linting,
- unit-test skeleton,
- decision-log structure.

Exit:

```text
cargo test
builder frontend test
format/lint
```

all pass.

---

# 147. PHASE 1 — FORMAT AND SCHEMA CORE

Implement:

```text
ag-schema
ag-project
capsule header structs
version validation
project manifest
object metadata
```

Do not implement UI first.

Exit:

- round-trip serialization,
- invalid-version tests,
- fuzz harness skeleton.

---

# 148. PHASE 2 — MANDATORY SPIKES

Complete Spike A–D.

Do not proceed to production crypto integration until blockers are resolved.

Store results under:

```text
docs/decisions/spike-*.md
```

---

# 149. PHASE 3 — OBJECT CRYPTO

Implement:

- CEK generation,
- HKDF per-object keys,
- chunked AES-256-GCM,
- object compression,
- in-memory/on-demand decryption,
- zeroization where practical.

Exit:

all crypto test vectors pass.

---

# 150. PHASE 4 — TIMelock ADAPTER

Create interface:

```rust
trait TimelockEngine {
    fn lock_cek(...);
    fn unlock_cek(...);
    fn verify_network_profile(...);
}
```

Use the Spike A validated implementation.

Builder and Viewer share compatibility tests.

---

# 151. PHASE 5 — CAPSULE PACKAGER

Implement:

- capsule serialization,
- RCDATA insertion,
- resource readback,
- SHA validation,
- icon/version mutation,
- signing hook.

Exit:

clean Windows VM loads project-specific Viewer.

---

# 152. PHASE 6 — VIEWER STATE ENGINE

Implement:

```text
Boot
PreRelease
ReleaseMaterialCheck
Unlocking
Authenticating
ReadyForCeremony
Ceremony
PostRelease
```

No visual polish required yet.

Exit:

state-machine integration tests.

---

# 153. PHASE 7 — TIME ENGINE

Implement:

- provider adapter abstraction,
- NTS validated adapters from Spike B,
- advisory NTP adapters,
- confidence engine,
- monotonic anchor,
- resume/network events.

Timelock release remains independent.

---

# 154. PHASE 8 — BASIC VIEWER UI

Implement:

- window,
- text,
- countdown,
- collection,
- entry detail,
- image rendering,
- keyboard navigation,
- accessibility fallbacks.

Use flat fallback materials first.

---

# 155. PHASE 9 — TEMPORAL GLASS + MOTION

Add:

- glass shader,
- aurora,
- responsive material,
- motion tokens,
- connected transitions,
- Glass Thaw,
- adaptive quality.

Do not regress accessibility.

---

# 156. PHASE 10 — BUILDER MVP

Implement:

- project wizard,
- identity,
- terminology,
- contributor schema,
- content blocks,
- assets,
- preview,
- release,
- build.

---

# 157. PHASE 11 — BUILDER ADVANCED

Implement:

- Theme Studio,
- Motion Studio,
- contributor packages,
- accessibility previews,
- localization,
- diagnostics,
- signing integration,
- build report.

---

# 158. PHASE 12 — HARDENING

Complete:

- fuzzing,
- clean-machine tests,
- SmartScreen/signing documentation,
- antivirus false-positive checks,
- performance tests,
- leak scan,
- dependency review,
- privacy review.

---

# 159. PR / TASK SIZE GUIDANCE

AI agents SHOULD avoid one giant “implement AFTERGLOW” patch.

Preferred task scope:

```text
one crate
one state-machine slice
one Builder screen family
one security primitive integration
one renderer feature
one test matrix area
```

Every security-sensitive PR includes tests.

---

# PART XVIII — FAILURE BEHAVIOR

# 160. FAILURE TABLE

| Failure | Required behavior |
|---|---|
| no network before release | show offline estimate; remain locked |
| target beacon unavailable | wait/retry; remain locked |
| one drand relay down | try other relays |
| invalid beacon | reject; continue trying valid relays |
| NTS unavailable | degrade time confidence; do not weaken timelock |
| time sources disagree | show degraded/disagreement state; target beacon remains authority |
| capsule malformed | refuse project load |
| GCM tag failure | do not display partial plaintext |
| GPU effect failure | use static fallback |
| custom font failure | use fallback font |
| audio decoder failure | text/images remain usable |
| code signature absent | app may run; Builder warns |
| signing failed during Build | Build not marked final |
| unsupported capsule major version | show compatibility error |
| external recovery unavailable | normal timelock path remains |

---

# PART XIX — DEFAULT COPY

# 161. GENERIC PRE-RELEASE COPY

Default:

```text
A LETTER TO THE FUTURE

Something has been left here for you.

It is not time to open it yet.
```

Builder should customize this for the project.

---

# 162. RELEASE COPY

Default:

```text
The time has come.

It is time to open what was left here.

ENTER
```

---

# 163. NETWORK DELAY COPY

```text
The release time has arrived.

AFTERGLOW is waiting for verifiable release data.

Try again
```

---

# 164. OFFLINE COPY

```text
Time cannot be fully verified right now.

The countdown is using the last trusted time estimate.
```

---

# PART XX — GENERIC PRESET EXAMPLES

# 165. BROADCAST CLUB

Configuration only:

```text
Contributor → Broadcast Club Member
Recipient → Junior
Role → Broadcast Role
Entry → Letter
Collection → Constellation
Theme → Temporal Glass Dark
Ceremony → Glass Thaw
```

No special Viewer code.

---

# 166. FAMILY

```text
Contributor → Family Member
Entry → Letter
Collection → Portrait Grid
Theme → Warm Memory
Ceremony → Quiet Fade
```

---

# 167. PERSONAL FUTURE LETTER

```text
single contributor
single entry
no collection page
optional gallery
Theme → Minimal Future
Ceremony → Orbit Resolve
```

---

# PART XXI — RELEASE CHECKLIST

# 168. CREATOR CHECKLIST

- [ ] Project identity final.
- [ ] Terminology reviewed.
- [ ] Release local time and UTC reviewed.
- [ ] Effective target round reviewed.
- [ ] All contributors approved.
- [ ] Public/private visibility reviewed.
- [ ] GPS/metadata reviewed.
- [ ] Font rights reviewed.
- [ ] Theme contrast passes.
- [ ] Keyboard preview passes.
- [ ] Reduced Motion preview passes.
- [ ] Reduced Transparency preview passes.
- [ ] Security profile understood.
- [ ] Recovery policy understood.
- [ ] Private object encryption succeeds.
- [ ] Timelock envelope generation succeeds.
- [ ] Capsule readback hash matches.
- [ ] Plaintext leak scan passes.
- [ ] Clean-machine Viewer test passes.
- [ ] Optional Authenticode succeeds.
- [ ] Final SHA-256 recorded.
- [ ] Creator-side backup stored separately.

---

# 169. DEFINITION OF DONE — BUILDER

Builder is complete when:

1. A non-programmer can make a generic project.
2. Domain wording is configurable.
3. Presets do not fork core behavior.
4. Custom contributor fields work.
5. Multi-author offline import/export works.
6. Structured content blocks work.
7. Theme and motion are safely configurable.
8. Release time resolves to a pinned target round.
9. Security profile explains tradeoffs.
10. Preview covers all critical states.
11. Accessibility variants are visible in preview.
12. Diagnostics block unsafe builds.
13. A final EXE can be generated without requiring the user to install Rust.
14. PE resource injection is verified.
15. Signing occurs after resource insertion.
16. Final Viewer contains no Builder UI.

---

# 170. DEFINITION OF DONE — VIEWER

Viewer is complete when:

1. One-file distribution works.
2. The Viewer loads its embedded RCDATA capsule.
3. Pre-release UI works without network.
4. Network time evidence updates confidence.
5. A future Windows clock alone cannot unlock.
6. Exact target-round timelock release works.
7. Beacon signatures are verified.
8. Invalid beacons are rejected.
9. CEK is not present in plaintext before release.
10. Private resources do not appear in plaintext EXE scans.
11. Private object GCM failures fail closed.
12. Release ceremony occurs only after verified unlock.
13. Post-release navigation is accessible.
14. Reduced Motion works.
15. Reduced Transparency works.
16. High-contrast fallback works.
17. 60 FPS target or graceful degradation works.
18. Minimized idle behavior is efficient.
19. No telemetry is sent by default.
20. No Builder/admin functionality exists.

---

# PART XXII — IMPROVEMENTS OVER SPECIFICATION V2

This section explains why v3 exists.

## IMP-001 — Requirements are now normative

V2 mixed design ideas, requirements, and suggestions.

V3 separates:

```text
MUST
SHOULD
MAY
```

and provides requirement IDs and exit criteria.

## IMP-002 — Timelock and countdown are fully separated

V2 still left room for an implementation to accidentally treat trusted network time as the release gate.

V3 pins the exact target drand round at build time.

## IMP-003 — Round computation is explicit

The release timestamp → target round formula is defined.

## IMP-004 — Timelock implementation risk is acknowledged

A Rust library is not assumed secure merely because it exists.

Differential tests against the official Go reference are mandatory.

## IMP-005 — PE capsule placement is corrected

V2 did not specify a safe canonical binary placement.

V3 uses `RCDATA` before signing and prohibits relying on an unsigned overlay.

## IMP-006 — Authenticode semantics are more accurate

V3 distinguishes:

- publisher authenticity,
- AEAD private-object integrity,
- simple corruption hashes.

It does not overclaim an internally replaceable signing key.

## IMP-007 — Whole-archive AES-GCM was replaced

A giant single encrypted blob can create unnecessary memory/streaming problems.

V3 uses per-object keys and chunked AES-256-GCM.

## IMP-008 — Recovery profiles are more honest

Balanced recovery is now external/creator-held by default.

Embedded recovery is explicitly marked weaker.

## IMP-009 — Viewer networking is privacy-scoped

V3 prohibits analytics and arbitrary background URLs.

## IMP-010 — Builder-to-Viewer packaging is concrete

The resource injection/signing order is defined.

## IMP-011 — Format versioning is mandatory

Project, capsule, and object formats have explicit version policy.

## IMP-012 — Mandatory technical spikes are added

Critical uncertain integrations must be proven before the project grows around them.

## IMP-013 — Arbitrary scripts are prohibited

This keeps the generated Viewer security model understandable.

## IMP-014 — Scope is controlled

Text and images are mandatory.

Audio is recommended.

Video is optional.

This prevents the first implementation from being blocked by full multimedia complexity.

## IMP-015 — AI implementation phases are explicit

An agent can now build AFTERGLOW in vertical stages with measurable exits.

---

# PART XXIII — EXTERNAL ASSUMPTIONS AND REFERENCE BASELINE

These references should be re-checked before a major release of AFTERGLOW Builder.

## 171. drand timelock

Official timelock overview:

- https://docs.drand.love/docs/timelock-encryption/

Official HTTP API:

- https://docs.drand.love/developer/http-api/
- https://docs.drand.love/developer/API-v2/drand-http-api/

Reference `tlock` implementation:

- https://github.com/drand/tlock

Current documented production Quicknet chain hash at review time:

```text
52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971
```

The project MUST revalidate live chain parameters before shipping.

## 172. Rust drand/tlock candidates

Third-party Rust implementations used for evaluation:

- https://github.com/thibmeu/tlock-rs
- https://github.com/thibmeu/drand-rs

Do not interpret listing here as an audit endorsement.

## 173. Network Time Security

Cloudflare NTS:

- https://developers.cloudflare.com/time-services/nts/

Netnod NTS:

- https://www.netnod.se/netnod-time/how-to-use-nts

NTS is defined by RFC 8915.

## 174. Other time references

NIST Internet Time Service:

- https://www.nist.gov/pml/time-and-frequency-division/time-distribution/internet-time-service-its

Google Public NTP / leap smear:

- https://developers.google.com/time
- https://developers.google.com/time/faq
- https://developers.google.com/time/smear

Google Public NTP does not support NTS and uses leap smear; do not naively mix it with unsmeared NTP during a leap-smear interval.

## 175. Windows accessibility effects

Microsoft Windows UI animation/transparency settings:

- https://learn.microsoft.com/windows/apps/develop/composition/composition-tailoring
- https://learn.microsoft.com/uwp/api/windows.ui.viewmanagement.uisettings.animationsenabled

## 176. PE resources and signing

PE resource update APIs:

- https://learn.microsoft.com/windows/win32/api/winbase/nf-winbase-beginupdateresourcew
- https://learn.microsoft.com/windows/win32/api/winbase/nf-winbase-updateresourcew
- https://learn.microsoft.com/windows/win32/api/winbase/nf-winbase-endupdateresourcew

PE/AuthentiCode behavior:

- https://learn.microsoft.com/windows/win32/secbp/understanding-pe-signatures
- https://learn.microsoft.com/windows/win32/debug/pe-format

SignTool:

- https://learn.microsoft.com/windows/win32/seccrypto/signtool

SmartScreen reputation:

- https://learn.microsoft.com/windows/apps/package-and-deploy/smartscreen-reputation

---

# 177. FINAL ARCHITECTURE

```text
                         AFTERGLOW
                             │
        ┌────────────────────┴────────────────────┐
        │                                         │
 AFTERGLOW Builder                        Native Viewer Runtime
        │                                         │
 Identity                                      Capsule loader
 Terminology                                   Release engine
 Schema                                        Time evidence
 Contributors                                  Timelock verification
 Content                                       Object decryptor
 Theme                                         Temporal Glass
 Motion                                        Temporal Motion
 Release                                       Accessibility
 Security                                      Archive renderer
        │                                         │
        └────────────────────┬────────────────────┘
                             │
                    Versioned Project Capsule
                             │
                   PE RT_RCDATA insertion
                             │
                   Optional Authenticode
                             │
                             ▼
                       ProjectName.exe
```

---

# 178. FINAL ONE-LINE SPECIFICATION

> **AFTERGLOW is a generic Builder-and-Viewer framework that lets creators define a future-release archive, packages protected content into a versioned encrypted project capsule, pins release to a verifiable future drand round, embeds the capsule inside a native Windows Viewer executable, and presents the result through a customizable, accessible Temporal Glass experience without requiring a project-owned release server.**

---

# 179. FINAL IMPLEMENTATION RULE

When choosing between two approaches, prefer the one that better satisfies this order:

```text
correct release semantics
> data confidentiality/integrity
> recoverability
> long-term reliability
> accessibility
> interaction clarity
> performance
> visual fidelity
> decorative complexity
```

Never reverse this order merely to make the demo look more impressive.
