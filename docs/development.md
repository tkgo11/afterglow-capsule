# Development

The repository has the Phase 0 foundation and Phase 1 format/schema core.
Follow SPEC.md in order. Neither binary scaffold is a working application.

## Toolchains

- Rust: install rustup; `rust-toolchain.toml` pins Rust, rustfmt and Clippy.
- Node: use `.node-version` (24.19.0), with npm 11.9.0.
- Python: 3.11 or newer, standard library only, for bootstrap topology checks.
- Windows Rust uses the x64 MSVC toolchain and Visual Studio C++ build tools.

From the repository root:

```sh
npm ci
cargo test --locked --workspace --all-targets
python scripts/check_bootstrap.py
npm test
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
npm run format:check
npm run check
npm run lint
npm run build
cargo check --manifest-path fuzz/Cargo.toml --locked --bins
cargo fmt --manifest-path fuzz/Cargo.toml --all -- --check
```

These commands cover the Phase 0/1 checks. Plain `cargo test` also runs
the workspace. `npm test` runs the Builder frontend tests in non-watch mode.
CI executes the same checks on Ubuntu and Windows x64. Local Linux checks do not
establish Windows spike acceptance; consult the actual Windows CI run.

`npm run format` formats editable source and new documentation. The five input
documents are excluded from Prettier to preserve canonical requirements.
`cargo fmt --all` formats Rust. Update and commit both lockfiles intentionally.

`npm run dev --workspace @afterglow/builder-ui` opens the frontend scaffold for
development. The native Builder/Viewer entry points currently explain their
unavailability and exit with failure; there is no production release path.

## Boundaries and phase gates

`apps/builder` and `packages/builder-ui` are creator-side only. `apps/viewer` is
recipient-side only. `viewer-runtime/template` holds pinned metadata for the future
Windows runtime template, not a generated EXE. Crate responsibilities are described
in their module documentation.

`ag-schema`, `ag-project` and `ag-capsule` now provide versioned models, metadata
validation and bounded header parsing. Their READMEs document the exact formats.
The four runnable parser fuzz harnesses are described in `fuzz/README.md`.
Compilation is checked in CI; instrumented fuzzing requires cargo-fuzz/nightly.

The test directories reserve the required subsystem matrices. Implement real tests
alongside each feature; scaffolds are not a claim of completed security coverage.
`scripts/check_bootstrap.py` checks workspace membership, application dependency
separation and version pins. It does not verify release cryptography.

Use `docs/decisions` for decisions and spike evidence. Revalidate external facts in
`docs/external-assumptions.md` at the relevant gates. Do not add private fixtures,
recovery credentials or signing keys to the repository.
