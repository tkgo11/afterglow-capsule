# Spike B: Windows NTS candidate

The isolated CLI accepts exactly two distinct operator hostnames. It performs
NTS key exchange, requires an authenticated NTP result, and reports RTT and UTC
epoch timing evidence. It grants no release capability. TLS verification remains
enabled; dangerous configuration and TLS key logging features are excluded.

```sh
cargo test --manifest-path spikes/Cargo.toml --locked -p afterglow-spike-b
python spikes/run_live.py nts
python spikes/nts/review_operator_docs.py
```

Four local tests passed: invalid zero timeout, prohibited verification disabling,
an actual stalled loopback TLS peer, and an actual untrusted loopback certificate.
Certificates are ephemeral test material kept in memory; they are never installed
as trust roots. The same four tests passed on Windows; successful native live
observations and operator provenance are separately recorded below.

Live inputs come from the candidate block in `docs/external-assumptions.md`.
Windows CI retrieved official Netnod documentation and the candidate was updated
to its documented hostname. Both documented independent operators then passed
certificate-verified NTS-KE/authenticated NTP and matching official provenance
review on Windows. Two different hostnames alone do not establish independence. The workspace has no
external TCP/UDP grants for NTS. Run the live experiment on native Windows with
permitted NTS-KE TCP and authenticated NTP UDP access, preserve both provider
results, and record OS/library versions and operator provenance. See the
[evidence record](../../docs/decisions/spike-b-nts.md).

The provenance helper collects the exact candidate's mention from each official
documentation URL, records a public excerpt/source/hash, and fails when a host is
absent or the document cannot be reached. It never changes endpoints or promotes
that result to release authority. Review the collected provenance alongside both
authenticated NTS observations before considering the gate satisfied.
