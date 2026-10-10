# NTS authenticated metadata integration boundary

Status: OPEN integration issue, not a change to SPEC.md or a reversal of Spike B.

Spike B proved the required TLS, NTS-KE, authenticated response, RTT, failures and
Windows behavior. Its evaluated `rkik-nts 1.4.0` high-level `TimeSnapshot` includes
network/local time, RTT, authenticated status, stratum and reference ID. Published
source inspection shows it omits the authenticated NTP header's root delay,
root dispersion, precision and leap indicator. The internal response parser sees
the raw header and authenticates it as NTS associated data, but those protocol
modules are private and the public result does not preserve those fields.

RTT alone is not a complete bound on the source's UTC uncertainty. Choosing an
arbitrary constant and describing it as measured uncertainty would weaken the
meaning of the time-evidence model. No such production adapter was added.

The isolated time-consensus/anchor implementation requires an honest uncertainty
input and keeps time evidence entirely separate from cryptographic release.
Preparation tests do not manufacture provider precision or accept Phase 7.

Before production provider integration, validate a maintainable API that exposes
the authenticated metadata, or another correctly authenticated and bounded
adapter. Any dependency/API change requires renewed Windows NTS positive and
negative tests and a recorded small spike. Do not disable TLS verification, fork
cryptographic primitives or reinterpret authentication as UTC accuracy.

The registry currently publishes no newer patch release for the evaluated
candidate. This is an implementation/API limitation, not a contradiction that
makes the specification impossible. SPEC.md stays unchanged. C/D physical gates
remain separately PENDING and their prior evidence is preserved.
