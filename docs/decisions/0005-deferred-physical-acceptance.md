# User-deferred physical acceptance and isolated preparation

The user requested: "Complete all Phases. Skip Phases that require user."

Physical-only execution and human observations are deferred. Existing failed and
pending evidence stays intact. The strict Phase 2 validator and trusted artifact
provenance remain unchanged, and Phase 2 C/D remains PENDING.

SPEC §148 still prevents production crypto integration before the mandatory
spikes pass. The previously authorized preparation of independent later-phase
code therefore occurs in an excluded `preparation/` workspace. Production
Builder/Viewer dependency closures cannot include it. This allows cloud-executable
software work without accepting gated phases or depending on unvalidated GPU or
clean-recipient assumptions.

Prepared code is tested and reviewed one subsystem at a time in specification
order. Preparation is not a production phase exit. After genuine evidence passes,
each subsystem must be promoted with its required integration and acceptance
tests; no script may turn absent physical evidence into PASS or enable a release
bypass. User-supplied evidence is untrusted input until validated and reviewed.

This decision changes neither SPEC.md nor any security invariant. No Viewer
unlock path, platform-validation exception or product distribution claim is
introduced by deferring the physical work.
