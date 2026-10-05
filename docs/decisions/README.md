# Decision log

SPEC.md remains canonical. An ADR records a routine implementation decision or a
discovered issue; it does not authorize weakening a requirement.

Use `NNNN-short-name.md` for decisions and `spike-a-*.md` through `spike-d-*.md`
for mandatory spike results. Start from [the template](template.md).

Each spike result must record dependency versions, commands, environment, positive
and negative evidence, acceptance criteria, and unresolved blockers. Scheduling a
CI job is not evidence that it passed.

| Gate                                      | Current status                                              | Outstanding evidence                                                        |
| ----------------------------------------- | ----------------------------------------------------------- | --------------------------------------------------------------------------- |
| [A — Timelock](spike-a-timelock.md)       | BLOCKED; historical differential/rejection tests passed     | Real future-round after-release interoperability; Windows x64               |
| [B — Windows NTS](spike-b-nts.md)         | BLOCKED; real local TLS failure tests passed                | Two live authenticated independent providers, RTT, Windows, provider review |
| [C — PE resources/signing](spike-c-pe.md) | BLOCKED; Windows API cross-check and fixtures passed        | Native injection/readback/signing/mutation; shell icon; clean recipient VM  |
| [D — Temporal Glass](spike-d-glass.md)    | BLOCKED; shader/policy tests and Windows cross-check passed | Integrated/discrete GPU resolution/DPI/effect/interaction matrix            |

Phases 0 and 1 are locally implemented and tested. Phase 2 is in progress; no
mandatory spike has met its full acceptance criterion. SPEC.md §148 says:
"Do not proceed to production crypto integration until blockers are resolved."
Production Phases 3–12 remain pending. Scheduling CI is not a passing result.
