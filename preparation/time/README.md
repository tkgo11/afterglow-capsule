# Prepared auxiliary time evidence

Phase 7 pure-logic preparation only. This package has no release, timelock,
object-crypto or Viewer dependency, and cannot return a CEK or authorize opening.
It does not claim a production NTS/advisory network adapter.

A registry owns reviewed provider/operator identities, authentication capability
and timescale. Samples with spoofed metadata, duplicate provider IDs, excessive
RTT/uncertainty, future monotonic receipt or stale receipt are rejected.
Independent operators, not hostnames, count toward confidence. Authenticated
aliases cannot be sharpened by unauthenticated same-operator observations.

Intervals include reported uncertainty and half RTT, propagated to one monotonic
instant. A shared intersection is required. Two independent authenticated UTC
operators yield Verified; one authenticated plus independent support yields Good;
two independent advisory UTC operators yield Degraded. Conflicting authenticated
evidence yields Disagreement and cannot replace the trusted anchor.

An isolated source does not satisfy the specified confidence quorum. Its sample
remains advisory; countdown uses the last accepted anchor or the local estimate
with LocalOnly confidence. Google smeared time is always kept out of the primary
unsmeared UTC consensus, including when smear scheduling is unknown.

Monotonic progression adds a conservative 50 ppm drift allowance. Wall-clock
jumps in either direction trigger a refresh without shifting the trusted
countdown. Resume invalidates the anchor; reconnect schedules refresh. Polling is
45 minutes far from release/minimized, 10 minutes within a day, two minutes within
an hour, and at most once per minute in the final minute. No one-second polling.

Tests cover quorums, aliases, authenticated disagreement, advisory outliers,
excessive RTT/stale/spoofed samples, leap-smear exclusion, both one-day clock jumps,
offline anchoring, resume/reconnect and quiet polling. The engine accepts UTC and
monotonic values; presentation timezone is outside its input model.

The [NTS metadata integration issue](../../docs/decisions/0006-nts-metadata-boundary.md)
records why the Spike B high-level result cannot yet be promoted as a complete
uncertainty-aware provider. Authentication success alone must not become an
invented precision claim. Production adapter integration remains pending.
