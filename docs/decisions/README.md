# Decision log

SPEC.md remains canonical. An ADR records a routine implementation decision or a
discovered issue; it does not authorize weakening a requirement.

Use `NNNN-short-name.md` for decisions and `spike-a-*.md` through `spike-d-*.md`
for mandatory spike results. Start from [the template](template.md).

Each spike result must record dependency versions, commands, environment, positive
and negative evidence, acceptance criteria, and unresolved blockers. Scheduling a
CI job is not evidence that it passed.

| Gate                                      | Current status                                                            | Outstanding evidence                                                       |
| ----------------------------------------- | ------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| [A — Timelock](spike-a-timelock.md)       | PASSED; historical, real future-round and Windows tests                   | None for the required spike; production dependency review remains separate |
| [B — Windows NTS](spike-b-nts.md)         | PASSED; authenticated independent operators, provenance, Windows failures | None for the technical spike; production security review remains separate  |
| [C — PE resources/signing](spike-c-pe.md) | BLOCKED; native replacement/signing/readback/mutation rejection passed    | Shell icon; clean recipient VM                                             |
| [D — Temporal Glass](spike-d-glass.md)    | BLOCKED; Linux/Windows shader/policy/lint tests passed                    | All 54 physical GPU resolution/DPI/effect/interaction cells                |

Phases 0 and 1 are implemented and tested on Linux and Windows. Phase 2 is in
progress; A and B pass, while C and D remain blocked. SPEC.md §148 says:
"Do not proceed to production crypto integration until blockers are resolved."
Production Phases 3–12 remain pending. Scheduling CI is not a passing result.
The [manual validation protocol](../../spikes/manual/README.md) explains the
remaining clean recipient VM and physical GPU evidence.

At spike source commit `f797ac893c67bfe6e2f36fdc647b3ddd7be9228d`, all
[workspace checks](https://github.com/tkgo11/afterglow-capsule/actions/runs/37292698532)
and all five [spike jobs](https://github.com/tkgo11/afterglow-capsule/actions/runs/37292698552)
passed. That includes Linux/Windows core/frontend checks, fuzz harness compilation,
historical and live timelock, NTS/provenance, native PE automation and Windows GPU
probe compilation. Hosted success still closes neither C's recipient/shell
checks nor D's physical hardware matrix.

Version 2 evidence tooling and immutable retry review are documented in
[0003 — Phase 2 evidence](0003-phase2-evidence.md). Its automated validator
requires independently reviewed provenance and genuine complete physical evidence.
Old version 1 bundles remain historical evidence only.
