# AFTERGLOW

> Build memories that open in the future.

AFTERGLOW is a general-purpose framework for creating **single-file Windows archives that unlock at a future time**.

Creators use **AFTERGLOW Builder** to configure identity, terminology, contributors, content, design, motion, release policy, and security. Builder then packages the project into one native recipient-facing executable:

```text
ProjectName.exe
```

The recipient does not need a project folder, database, account, or AFTERGLOW-owned backend.

## Core architecture

```text
AFTERGLOW Builder
        │
        ├─ Identity / Terminology
        ├─ Contributors / Content
        ├─ Theme / Motion
        ├─ Release / Security
        └─ Preview / Validation
        │
        ▼
Versioned encrypted Project Capsule
        +
Generic native Viewer Runtime
        │
        ▼
Windows PE resource injection
        │
        ▼
Optional Authenticode signing
        │
        ▼
ProjectName.exe
```

## What makes AFTERGLOW different

- **One-file recipient distribution**
- **Generic Builder** instead of a domain-specific app
- **Client-only delayed release** with no project-owned release server
- **Pinned future drand round** as the cryptographic release gate
- **Multi-source time evidence** for countdown accuracy and anomaly detection
- **Per-object authenticated encryption**
- **Native Rust Viewer** using `winit`, `wgpu`, and WGSL
- **Temporal Glass** visual system
- **Temporal Motion** animation system
- **Reduced Motion / Reduced Transparency / keyboard accessibility**
- **No telemetry by default**

## Important security model

The local Windows clock does **not** authorize release.

At build time, the chosen release instant is mapped to a specific future drand round. The Viewer must obtain and cryptographically verify that exact round before the protected archive can be unlocked.

Network time sources are used for countdown quality and confidence only.

AFTERGLOW does **not** claim perfect DRM. After legitimate release, plaintext must exist in memory to be displayed. Embedded emergency-recovery modes may also weaken resistance to expert reverse engineering.

See [SECURITY.md](SECURITY.md) and [SPEC.md](SPEC.md).

## Project status

**Pre-alpha / Phase 1 format and schema core.**

The repository includes the Rust workspace, subsystem and application scaffolds,
Builder frontend test harness, and Linux/Windows CI. Phase 1 adds versioned generic
schema/project models, public manifest and object metadata, bounded capsule header
parsing, and parser fuzz harnesses. The application scaffolds cannot build or open archives.

See [docs/development.md](docs/development.md) for toolchains and validation commands
and [docs/decisions/](docs/decisions/README.md) for decisions and mandatory spike status.

Implementation must follow the phases and mandatory technical spikes in [SPEC.md](SPEC.md).

## Documentation

- [SPEC.md](SPEC.md) — canonical implementation specification and source of truth
- [AGENTS.md](AGENTS.md) — instructions for AI coding agents
- [SECURITY.md](SECURITY.md) — threat model, security boundaries, and reporting guidance
- [docs/external-assumptions.md](docs/external-assumptions.md) — time-sensitive external dependencies that must be revalidated

## Planned products

### AFTERGLOW Builder

Creator-side authoring application.

Recommended stack:

```text
Tauri 2
Rust backend
Svelte + TypeScript frontend
```

### AFTERGLOW Viewer

Recipient-facing native Windows runtime.

Recommended stack:

```text
Rust
winit
wgpu
WGSL
```

The final Viewer contains no Builder/admin UI.

## Example use cases

AFTERGLOW Core is domain-neutral. Builder presets may include:

- Broadcast Club
- Graduation
- Friends
- Family Time Capsule
- Team Farewell
- Anniversary
- Personal Future Letter
- Class Archive
- Community Archive

Presets change configuration, not core Viewer code.

## Design principle

Before release, **time** is the subject.

After release, **the project content** becomes the subject.

The default visual system combines modern Windows clarity, restrained Y2K futurism, Liquid Glass materials, and archival/memory cues.

## Development rule

When two implementation choices conflict, prefer:

```text
correct release semantics
> confidentiality and integrity
> recoverability
> long-term reliability
> accessibility
> interaction clarity
> performance
> visual fidelity
> decorative complexity
```

## License

A repository license has not been selected yet. Until a license is added, do not assume permission to redistribute or reuse the code.
