# 0001 — Phase 0 toolchain and boundaries

- Status: accepted
- Date: 2026-10-05
- SPEC.md sections: 8, 9, 146; D-003, D-005, D-017
- Scope: bootstrap only; no architecture deviation

## Decision

Use a Cargo workspace with the thirteen specified subsystem crates and separate
Builder/Viewer binary packages. Libraries are deliberately empty until their
implementation phase. Both binary scaffolds exit unsuccessfully with an explanation.
Builder and Viewer have no shared application dependency; the Viewer does not depend
on the creator-side packager.

Pin Rust 1.90.0 with rustfmt and Clippy and use edition 2024. Pin Node 24.19.0 and
npm 11.9.0. Use one committed npm workspace lock for the Svelte/TypeScript frontend.
The frontend is only a mount/test harness with a disabled Build control. It contains
no authoring implementation, renderer, preview provider or release controls.
Tauri integration belongs to the Builder implementation phase; winit/wgpu and
security-sensitive integrations are deferred to their specified phases/gates.

Pin runtime-template metadata to the Viewer package version, label it `scaffold`,
and supply no EXE template. This is not a production packaging artifact.

CI runs bootstrap Rust tests/lints and frontend checks on Linux and Windows x64.
No Windows-specific platform behavior or mandatory spike is claimed validated.

## Consequences

No crypto, TLS, drand, NTS, PE or GPU production dependency has been selected.
Unsafe code is forbidden by default; a later native API integration may require a
documented, narrowly scoped allowance and tests. No license is asserted for this
repository; packages are non-publishable.

All mandatory spikes remain unresolved. Phase 0 checks and exact commands are
documented in `docs/development.md`. Phase 1 follows before the Phase 2 spikes.

## Validation evidence

Local Linux x64 validation on 2026-10-05 used Rust 1.90.0, Node 24.19.0, npm 11.9.0
and Python 3.12.14:

- `cargo test --locked` and `cargo test --locked --workspace --all-targets`: pass;
  two binary integration tests, with unit-test module skeletons in all libraries.
- `python3 scripts/check_bootstrap.py`: four checks pass.
- `npm ci` followed by `npm test`: clean locked install and two frontend tests pass.
- `cargo fmt --all -- --check` and Clippy with `-D warnings`: pass.
- `npm run format:check`, `npm run check` and `npm run lint`: pass; Svelte reports
  zero errors and warnings.
- `npm run build`: pass.
- `npm audit --audit-level=moderate`: zero reported vulnerabilities. This is a
  registry advisory check, not an audit endorsement.

Windows x64 is configured in CI but was not executed in this Linux session.
No mandatory spike or Windows-specific integration was validated.
