# AFTERGLOW — Canonical Implementation Specification

**Version:** 3.1 (repository edition)  
**Status:** Canonical / implementation-ready  
**Audience:** AI coding agents, engineers, security reviewers, designers, QA  
**Recipient platform:** Windows 10/11 x64  
**Final recipient artifact:** one project-specific `.exe`  
**Project-owned release backend:** prohibited by default

---

## 1. Normative language

- **MUST / MUST NOT** — mandatory.
- **SHOULD / SHOULD NOT** — expected; deviation requires a documented reason.
- **MAY** — optional.
- **BLOCKER** — implementation must stop at the affected gate.
- **SPIKE** — short proof used to validate an uncertain technical dependency.

If requirements conflict, use this priority:

1. Security invariants
2. Canonical decisions
3. Requirement IDs
4. Data formats and state machines
5. Acceptance criteria
6. UI/design defaults
7. Examples

---

## 2. Product definition

AFTERGLOW is a generic Builder-and-Viewer framework for future-release archives.

```text
Generic Engine
+
Project Configuration
+
Project Content
=
Personal Final Artifact
```

The engine MUST remain domain-neutral. Terms such as “Broadcast Club”, “Junior”, “Family Member”, or “Graduate” belong in presets/project configuration, never core logic.

### Creator-side product

**AFTERGLOW Builder**

Responsibilities:

- identity and terminology
- custom contributor schema
- contributor/content management
- assets
- layout
- visual theme
- motion
- accessibility
- localization
- release configuration
- security profile
- preview
- validation
- packaging
- optional Authenticode signing

### Recipient-side product

**AFTERGLOW Viewer**

Responsibilities:

- load embedded project capsule
- show pre-release experience
- collect time evidence
- obtain and verify exact target drand beacon
- unlock the CEK
- decrypt private objects on demand
- play configured release ceremony
- render post-release archive

The Viewer MUST contain no editor or administrator UI.

---

## 3. Canonical decisions

| ID | Decision |
|---|---|
| D-001 | Core is domain-neutral; presets are configuration only. |
| D-002 | Recipient distribution is one Windows EXE. |
| D-003 | Builder and Viewer are separate products. |
| D-004 | Viewer is native Rust using `winit` + `wgpu`; no Electron/Chromium runtime. |
| D-005 | Builder uses Tauri 2 + Rust + Svelte/TypeScript unless replaced by a documented ADR. |
| D-006 | No AFTERGLOW-owned release backend in the default architecture. |
| D-007 | Release authority is a build-time-pinned future drand round, not the local clock. |
| D-008 | Timelock network identity is pinned by chain parameters, not hostname alone. |
| D-009 | Rust timelock code must pass differential interoperability tests against the official/reference Go `drand/tlock` ecosystem before production use. |
| D-010 | Network time is auxiliary evidence, never the final decryption gate. |
| D-011 | Private content uses per-object keys derived from a random 256-bit master CEK and chunked AES-256-GCM. |
| D-012 | Project capsule is inserted into PE `RCDATA` before Authenticode signing. |
| D-013 | Viewer customization is declarative; arbitrary project scripts are prohibited in v1. |
| D-014 | Default security profiles do not embed a recovery secret in the recipient EXE. |
| D-015 | Optional creator-held external recovery is supported. |
| D-016 | Embedded recovery is advanced and explicitly weaker against expert reverse engineering. |
| D-017 | Preview/mock release controls must not exist in production release logic. |
| D-018 | All PE mutation occurs before final signing. |
| D-019 | Viewer networking is allowlisted and never uploads project content. |
| D-020 | Every serialized format is explicitly versioned. |
| D-021 | Private media is decrypted on demand, never unpacked as a normal project directory. |
| D-022 | Default post-release mode persists no CEK. |
| D-023 | Video is optional for v1. |
| D-024 | Text + still-image projects are mandatory first-class use cases. |
| D-025 | Accessibility overrides decorative effects. |

---

## 4. Security invariants

### SEC-I-001 — No local-clock-only unlock

Changing Windows to a future date MUST NOT unlock protected content.

### SEC-I-002 — No plaintext CEK

The recipient EXE MUST NOT contain the CEK as plaintext or simple reversible obfuscation in standard profiles.

### SEC-I-003 — No plaintext private content

Protected text/media/metadata MUST NOT be embedded as plaintext Viewer resources.

### SEC-I-004 — Exact target round

Builder MUST compute one release target round. Viewer MUST request that round; it must not derive a new round from the local clock.

### SEC-I-005 — Beacon verification

Relay data MUST be cryptographically verified against the pinned network identity. HTTPS alone is insufficient.

### SEC-I-006 — Fail closed

Beacon, timelock, capsule, version, or AEAD validation failure MUST keep protected content locked.

### SEC-I-007 — Preview isolation

No production unlock path may depend on environment variables, CLI switches, registry flags, debug keys, magic dates, or hidden shortcuts.

### SEC-I-008 — No custom cryptography

Use maintained standard cryptographic constructions. Do not invent a cipher, KDF, signature scheme, or proprietary timelock.

### SEC-I-009 — Sign after mutation

If Authenticode is used, no resource/capsule/icon/version mutation may occur after signing.

### SEC-I-010 — Honest authenticity claims

An internal signature does not become attacker-resistant merely because its verification key is stored in the same modifiable EXE.

---

## 5. Threat model

AFTERGLOW MUST strongly resist:

- accidental misuse
- Windows clock changes
- casual resource extraction
- `strings`/common static inspection
- ordinary network failures
- malicious/invalid relay responses

AFTERGLOW does not guarantee:

- screenshot prevention after release
- plaintext invisibility in process memory after release
- perfect DRM
- resistance to every expert reverse-engineering technique
- eternal availability of third-party infrastructure
- post-quantum secrecy

---

## 6. Repository architecture

```text
afterglow-capsule/
│
├── apps/
│   ├── builder/
│   └── viewer/
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
├── packages/
│   └── builder-ui/
├── viewer-runtime/
├── shaders/
├── presets/
├── test-vectors/
├── tests/
├── docs/
│   ├── decisions/
│   └── external-assumptions.md
├── Cargo.toml
└── Cargo.lock
```

Do not create empty directories solely to match this tree; create them when the corresponding phase begins.

---

## 7. Formats and versioning

Every persistent format MUST contain:

```text
format_name
format_version
minimum_reader_version
```

Unknown major versions must be rejected.

Core stable IDs:

```text
project_id
build_id
contributor_id
entry_id
object_id
```

IDs must not be derived from display names.

---

## 8. Generic terminology

Core UI references semantic labels.

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

Presets override values only.

---

## 9. Private object cryptography

### Master key

Builder generates a random 32-byte CEK using the OS CSPRNG.

### Per-object key

```text
HKDF-SHA256(
  IKM  = CEK,
  salt = project_id,
  info = "AFTERGLOW/object/v1/" || object_id
)
```

Output: 32 bytes.

### Compression

Compress before encryption only when useful.

- JSON/text: Zstandard by default
- already-compressed images/audio/video: usually no extra compression

### Chunking

Default encrypted chunk size:

```text
1 MiB
```

For each object:

```text
nonce_prefix = random 8 bytes
nonce = nonce_prefix || chunk_index_u32_be
```

AES-GCM nonce = 12 bytes.

AAD binds:

```text
format_version
project_id
build_id
object_id
chunk_index
compression_method
```

No nonce reuse with the same key is permitted.

---

## 10. Project capsule

Canonical capsule location:

```text
PE resource type: RT_RCDATA
resource name: AGLOW_CAPSULE
```

The capsule is a versioned binary blob containing:

- public manifest
- public object store
- encrypted private object store
- timelock envelope
- digests
- project configuration

Recommended v1 size policy:

- <= 512 MiB: normal
- 512 MiB–1 GiB: warning
- > 1 GiB: blocked unless expert override

This is a reliability policy, not a theoretical PE limit.

---

## 11. Why not an EXE overlay

The canonical capsule MUST NOT rely on blindly appending bytes after the final PE section.

Packaging order:

```text
copy runtime template
↓
apply icon/version resources
↓
insert AGLOW_CAPSULE RCDATA
↓
close resource update
↓
read resource back and verify
↓
optional Authenticode signing
↓
verify signature
↓
do not mutate again
```

---

## 12. Mandatory technical spikes

These are BLOCKERS before production implementation.

### SPIKE A — Timelock interoperability

Prove the chosen Rust implementation interoperates with the official/reference Go `drand/tlock` ecosystem.

Required:

1. Go-generated ciphertext → Rust decrypt
2. Rust-generated ciphertext → Go decrypt
3. reject early decryption
4. reject wrong chain
5. reject wrong round
6. reject modified ciphertext
7. reject invalid beacon
8. Windows x64 CI

### SPIKE B — NTS on Windows

Prove authenticated network-time evidence from at least two independent NTS operators on Windows.

Preferred:

- Cloudflare
- Netnod

If NTS integration is unavailable, document degradation. Do not weaken the timelock gate.

### SPIKE C — PE resource injection + signing

Prove:

1. copy runtime template
2. insert RCDATA
3. replace icon/version info
4. load capsule at runtime
5. sign
6. verify signature
7. mutate after signing and confirm signature invalidates
8. clean Windows VM run

### SPIKE D — Temporal Glass performance

Prove the default `wgpu` effects degrade gracefully and remain interactive across representative GPUs/DPI values.

---

## 13. Release model

Builder resolves the user-selected local time to UTC.

Then it maps that UTC instant to the **first scheduled drand round at or after the requested instant**.

Given:

```text
G = genesis time of round 1
P = period
T = requested release UTC
```

```text
target_round = ceil((T - G) / P) + 1
```

Builder embeds:

- requested release UTC
- effective target round
- scheduled target-round time
- chain hash
- public key/network profile
- period/genesis metadata
- allowlisted relays

Viewer never recomputes the target round from the local clock.

---

## 14. Release algorithm

```text
target = embedded target round

fetch target round from allowlisted relays

for each response:
    if beacon verifies against pinned network identity:
        use it
        break

if no valid beacon:
    remain locked / waiting

decrypt timelock CEK envelope

if failure:
    remain locked

authenticate/decrypt required private manifest object

if failure:
    zeroize temporary key material
    remain locked with integrity error

state = READY_FOR_CEREMONY
```

Time evidence does not grant access.

---

## 15. Time evidence engine

Purpose:

- countdown
- confidence state
- clock anomaly detection
- UI stage timing

Preferred classes:

- authenticated unsmeared NTS: Cloudflare, Netnod
- reputable additional UTC evidence: NIST
- smeared advisory source: Google Public NTP
- local evidence: Windows wall clock + monotonic clock

Do not naively average Google leap-smear time with unsmeared UTC during smear intervals.

Confidence classes:

```text
VERIFIED
GOOD
DEGRADED
LOCAL_ONLY
DISAGREEMENT
```

Suggested refresh:

- >24 h: 30–60 min
- 1–24 h: 5–15 min
- <1 h: 1–5 min
- <1 min: use monotonic anchor; do not hammer services

---

## 16. Recovery profiles

### Strict

- timelock
- no embedded recovery
- no persisted CEK

### Balanced — default

- timelock
- optional creator-held external recovery package
- no embedded recovery
- no persisted CEK

### Heritage Embedded — advanced

- timelock
- embedded delayed fallback
- explicit warning that expert reverse engineering resistance is reduced

Creator-held recovery must never be implied to prevent the creator from opening early.

---

## 17. Builder

Recommended stack:

```text
Tauri 2
Rust backend
Svelte + TypeScript
```

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

Builder must support presets without forking Viewer logic.

Initial presets:

- Blank
- Broadcast Club
- Graduation
- Friends
- Family Time Capsule
- Team Farewell
- Anniversary
- Personal Future Letter
- Class Archive
- Community Archive
- Custom Multi-Contributor

---

## 18. Contributor workflow

Offline multi-author flow:

```text
Owner exports assignment
↓
Name.aginvite
↓
Contributor Mode
↓
Contributor writes/imports
↓
Name.agentry
↓
Owner imports
```

Submission packages must include project/slot identity so cross-project imports are rejected.

Default model assumes the Project Owner can review imported content.

---

## 19. Content model

Required v1 blocks:

- Heading
- Paragraph
- Quote
- Signature
- Divider
- Image
- Image Pair
- Gallery
- Caption
- Timeline
- Callout
- Date Stamp
- Credits
- Spacer
- Link

Audio SHOULD be supported.

Video MAY be feature-flagged/deferred.

Arbitrary HTML/JavaScript is prohibited.

External links open in the system browser and do not load remote HTML in the Viewer.

---

## 20. Builder theme system

Default design language: **Temporal Glass**.

Default composition:

```text
40% modern Windows clarity
30% Liquid Glass
20% restrained Y2K futurism
10% archive/memory cues
```

Three visual layers:

1. Atmosphere
2. Content
3. Glass controls

Long reading surfaces should be stable and mostly opaque.

Default semantic colors include:

```text
Space      #070915
Midnight   #0E1630
Deep Blue  #101A35
Pearl      #F7F8FF
Mist       #D7D8E8
Aqua       #7BE7FF
Violet     #B19CFF
Pink       #FF9DE6
Mint       #80F2C2
Amber      #FFD27A
Rose       #FF8098
```

---

## 21. Glass defaults

Materials:

- Glass/Clear
- Glass/Regular

Safe maxima:

| Property | Default max |
|---|---:|
| card scale | 1.015 |
| pointer translation | 2 px |
| refraction | 3 px |
| RGB split | < 1 px |
| hover highlight gain | 18% |

Glass is primarily for controls, not long-form content.

---

## 22. Temporal Motion

Motion classes:

- Reactive
- Spatial
- Ambient
- Ceremonial

Default tokens:

| Token | Duration |
|---|---:|
| press-in | 70 ms |
| press-out | 100 ms |
| hover | 120 ms |
| fast | 167 ms |
| standard | 250 ms |
| card | 333 ms |
| back | 400 ms |
| media-open | 500 ms |
| forward | 600 ms |
| app-enter | 750 ms |
| collection-enter | 800 ms |
| ceremony | 2700 ms |
| ambient cycle | 25–40 s |

Default ceremony: **Glass Thaw**.

No default confetti, fireworks, screen shake, glass explosion, white flash, or high-frequency flashing.

---

## 23. Accessibility

Viewer MUST support:

- keyboard navigation
- visible focus
- Reduced Motion
- Reduced Transparency
- High Contrast-compatible fallback
- text scaling
- sound controls

Minimum keys:

```text
Tab
Shift+Tab
Arrow keys
Enter
Space
Escape
Alt+F4
```

Reduced Motion replaces spatial travel with crossfades/short opacity transitions.

Reduced Transparency removes dynamic refraction/dispersion and strengthens opaque surfaces.

---

## 24. Viewer state machine

```text
BOOT
↓
PRE_RELEASE
↓
RELEASE_MATERIAL_CHECK
├─ unavailable → waiting / locked
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

At countdown zero, the UI may say:

```text
The release time has arrived.

AFTERGLOW is waiting for verifiable release data.
```

Do not unlock merely because countdown reached zero.

---

## 25. Performance

Target:

```text
stable 60 FPS
```

Suggested quality tiers:

| Average frame time | Mode |
|---|---|
| <12 ms | Ultra |
| 12–16.7 ms | Full |
| 16.7–25 ms | Reduced FX |
| >25 ms | Static Glass |

Reduce in order:

1. chromatic dispersion
2. refraction sampling
3. blur
4. aurora
5. ambient decoration

Never sacrifice interaction correctness first.

When minimized, pause/reduce rendering and polling.

---

## 26. Privacy and networking

Viewer telemetry default:

```text
NONE
```

Viewer may contact only:

- configured time providers
- configured drand relays
- user-activated external links via system browser

It must not upload:

- messages
- contributor names
- photos
- project content
- analytics

Strip GPS metadata from imported photos by default.

---

## 27. Build pipeline

Canonical order:

```text
1. Freeze project snapshot
2. Validate schemas
3. Optimize assets
4. Generate public objects
5. Generate private objects
6. Generate CEK
7. Encrypt private objects
8. Generate timelock CEK envelope
9. Serialize capsule
10. Copy Viewer Runtime Template
11. Apply icon/version resources
12. Insert AGLOW_CAPSULE RCDATA
13. Read back and verify capsule bytes
14. Plaintext leak scan
15. Viewer smoke test
16. Optional Authenticode signing
17. Verify Authenticode
18. Compute final EXE SHA-256
19. Generate creator-side Build Report
```

Nothing mutates the EXE after signing.

---

## 28. Build report

Must include at least:

```text
Project
Build ID
Viewer version
Schema version
Release UTC
Target round
Timelock chain/profile
Security profile
Recovery profile
Object counts
Capsule SHA-256
Final EXE SHA-256
Authenticode status
Warnings
```

---

## 29. Mandatory tests

### Cryptography

- per-object key derivation
- nonce uniqueness
- tamper rejection
- chunk reorder rejection
- truncation rejection
- wrong AAD rejection

### Timelock

- Go ↔ Rust interoperability
- early round rejection
- wrong chain/round rejection
- invalid beacon rejection
- relay failover
- Windows CI

### Packaging

- RCDATA insert/read
- icon/version update
- signing
- post-sign mutation invalidation
- clean-machine launch
- one-file distribution

### Time

- provider outage
- source disagreement
- large local clock jumps
- timezone change
- sleep/resume
- offline
- leap-smear handling

### UI

- 960×640 through 1920×1080+
- 100–200% DPI
- 60/120/144 Hz
- keyboard-only
- Reduced Motion
- Reduced Transparency
- High Contrast
- integrated GPU

### Fuzzing

Prioritize:

- capsule parser
- manifest parser
- object table
- encrypted chunk reader
- network response parser
- timelock response decoder

---

## 30. Implementation phases

### Phase 0 — Bootstrap

Workspace, CI, formatting, linting, test skeleton, ADR directory.

### Phase 1 — Schemas/formats

`ag-schema`, `ag-project`, capsule structs, version rules.

### Phase 2 — Mandatory spikes

Complete Spike A–D before production integrations.

### Phase 3 — Object crypto

CEK, HKDF, chunked AES-GCM, compression, on-demand decrypt.

### Phase 4 — Timelock adapter

Shared Builder/Viewer interface using the validated implementation.

### Phase 5 — Capsule packager

Serialization, RCDATA, icon/version mutation, signing hook.

### Phase 6 — Viewer state engine

Release state machine without visual polish.

### Phase 7 — Time evidence

Providers, confidence, monotonic anchor, system events.

### Phase 8 — Basic Viewer UI

Countdown, collection, detail, images, keyboard, accessibility.

### Phase 9 — Temporal Glass + Motion

Shaders, motion tokens, connected transitions, Glass Thaw, adaptive quality.

### Phase 10 — Builder MVP

Wizard, identity, terminology, contributors, content, assets, preview, release, build.

### Phase 11 — Builder advanced

Theme/Motion Studios, contributor packages, localization, diagnostics, signing.

### Phase 12 — Hardening

Fuzzing, clean-machine tests, performance, leak scan, dependency/privacy review.

Do not submit one giant “implement AFTERGLOW” change. Prefer one subsystem or vertical slice per PR.

---

## 31. Definition of done — Builder

Builder is complete when:

1. A non-programmer can create a generic project.
2. Domain terminology is configurable.
3. Presets do not fork core behavior.
4. Custom contributor fields work.
5. Offline multi-author packages work.
6. Structured content works.
7. Theme and motion are safely configurable.
8. Release time resolves to a pinned target round.
9. Security/recovery tradeoffs are explicit.
10. Critical states are previewable.
11. Accessibility variants are previewable.
12. Diagnostics block unsafe builds.
13. Final EXE generation does not require the user to install Rust.
14. Resource injection is verified.
15. Signing occurs last.
16. Final Viewer contains no Builder UI.

---

## 32. Definition of done — Viewer

Viewer is complete when:

1. One-file distribution works.
2. Embedded RCDATA capsule loads.
3. Pre-release UI works offline.
4. Time evidence updates confidence.
5. Future local clock alone cannot unlock.
6. Exact target-round timelock release works.
7. Beacons are cryptographically verified.
8. Invalid beacons are rejected.
9. CEK/private content are not plaintext before release.
10. AEAD failures fail closed.
11. Ceremony occurs only after verified unlock.
12. Post-release navigation is accessible.
13. Reduced Motion works.
14. Reduced Transparency works.
15. High-contrast fallback works.
16. Performance degrades gracefully.
17. Minimized idle behavior is efficient.
18. No telemetry is sent by default.
19. No Builder/admin functionality exists.

---

## 33. External assumptions

Time-sensitive values such as drand chain parameters, relay endpoints, candidate Rust crates, and third-party service behavior do **not** belong in permanent architecture prose.

They live in:

[docs/external-assumptions.md](docs/external-assumptions.md)

That file must be revalidated before security-sensitive releases.

---

## 34. Final implementation priority

When choosing between two approaches, prefer:

```text
correct release semantics
> confidentiality/integrity
> recoverability
> long-term reliability
> accessibility
> interaction clarity
> performance
> visual fidelity
> decorative complexity
```
