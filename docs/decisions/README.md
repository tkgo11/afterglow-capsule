# Decision log

SPEC.md remains canonical. An ADR records a routine implementation decision or a
discovered issue; it does not authorize weakening a requirement.

Use `NNNN-short-name.md` for decisions and `spike-a-*.md` through `spike-d-*.md`
for mandatory spike results. Start from [the template](template.md).

Each spike result must record dependency versions, commands, environment, positive
and negative evidence, acceptance criteria, and unresolved blockers. Scheduling a
CI job is not evidence that it passed.

| Gate                          | Phase 0 status                               | Required evidence                                                                         |
| ----------------------------- | -------------------------------------------- | ----------------------------------------------------------------------------------------- |
| A — Timelock interoperability | Not started; production integration blocked  | Go ↔ Rust, rejection cases, Windows x64                                                   |
| B — Windows NTS               | Not started; platform assumption unvalidated | Two independent operators, authentication, RTT, certificate and failure cases, Windows    |
| C — PE resources/signing      | Not started; production packaging blocked    | Injection/readback, icon/version, signing, post-sign mutation rejection, clean Windows VM |
| D — Temporal Glass            | Not started; GPU assumption unvalidated      | Hardware, resolution/DPI, responsiveness and fallback matrix                              |

Phase 1 is the next implementation phase. No spike has been completed by bootstrap.
