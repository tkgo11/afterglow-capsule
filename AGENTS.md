# AGENTS.md

This repository is designed to be implemented with AI coding agents as well as human contributors.

## Before changing architecture

Read, in order:

1. [SPEC.md](SPEC.md)
2. [SECURITY.md](SECURITY.md)
3. [docs/external-assumptions.md](docs/external-assumptions.md) when working on drand, time, Windows signing, or other external integrations

The canonical specification wins over examples, comments, or convenience.

## Non-negotiable rules

1. Keep AFTERGLOW Core domain-neutral.
2. Do not add an AFTERGLOW-owned release backend.
3. Do not replace the pinned drand-round release gate with local time, NTP, or HTTPS time.
4. Do not embed private content or the CEK in plaintext.
5. Do not invent custom cryptography.
6. Do not add arbitrary project scripts/plugins to the generated Viewer.
7. Do not include Builder/admin functionality in the final Viewer.
8. Do not mutate a signed EXE after signing.
9. Do not add telemetry by default.
10. Do not introduce hidden preview/debug unlock paths in production.
11. Security-sensitive changes require tests.
12. Accessibility fallbacks are mandatory product behavior.

## Work order

Follow the implementation phases in [SPEC.md](SPEC.md).

The mandatory technical spikes are blockers. Do not build production architecture on top of an unvalidated timelock, NTS, PE-packaging, or GPU assumption.

## Scope discipline

Prefer one subsystem or vertical slice per change.

Good scopes:

- one crate
- one parser/format
- one release-state slice
- one Builder screen family
- one crypto integration
- one renderer feature
- one test matrix area

Avoid giant “implement the whole project” patches.

## Required behavior for uncertainty

If SPEC.md defines a default, use it.

If a material platform/security assumption is uncertain:

1. create a short spike,
2. record the result under `docs/decisions/`,
3. update `docs/external-assumptions.md` if the fact is time-sensitive,
4. then implement production code.

Do not silently substitute a different security model.

## Tests

At minimum:

- unit tests for pure logic
- integration tests for subsystem boundaries
- regression tests for security-sensitive bugs
- fuzzing for untrusted binary/network parsers where specified
- Windows CI for Windows-specific paths

## Documentation updates

Update SPEC.md only when product/architecture requirements intentionally change.

Use an ADR under `docs/decisions/` for deliberate architectural deviations.

Do not copy time-sensitive chain hashes/endpoints into multiple files; keep them centralized in `docs/external-assumptions.md`.

## Completion rule

A feature is not complete merely because the happy path works.

It is complete only when:

- failure behavior is defined,
- required accessibility fallback exists,
- required tests pass,
- diagnostics are understandable,
- security invariants remain true.
