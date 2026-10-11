# Metrics 0.6.1 consumer adoption

Extend the existing undated 0.18.0 batch. Published Metrics 0.6.1 supplies the
single prepared-byte histogram aggregate; the public projection and re-export use
`MeasurementAggregate`, with no latest byte value or duplicate accumulator.
Duration lanes retain ordered `MeasurementSummary`. No persisted record changes.

Admit each duration/byte conversion only when it fits u64. Exclude an
unrepresentable sample independently rather than making a clipped cap exact.
Valid zero/repeated byte work remains sampled even if its duration cannot fit.
Shared overflow flags distinguish exact caps from actual accumulated overflow;
shared means/intervals retain their own arithmetic owner.

The selected registry archive matches Cargo.lock. Published 0.6.1 Rust files
match 0.6.0; the aggregate API break happened in that minor release. Retain incoming
Host 0.13/Testkit 0.33.3 and committed Shared Tooling 0.3.11. Metrics has no enabled
features, and the independent normal consumer excludes simulator dependencies.
Detailed source/graph/log evidence stays under ignored `target/metrics-061`.

Six focused metric cases and a Rust 1.88 public consumer pass. All 16 `make ci`
targets pass, including strict Clippy, actual simulator journeys, independent
MSRV consumers and exact paired-package builds. Technical source/graph inputs
remain identical before and after the gate. The detailed registered passing
cases and logs are hash-bound by [the compact proof](qualification.json). Current candidate native macOS qualification remains
pending; local Linux checks cannot establish that acceptance. No Git/release,
publication, production IC, sibling edit or evidence cleanup is performed.

Coordination: [Backup #38](https://github.com/dragginzgame/ic-backup/issues/38),
[Metrics #57](https://github.com/dragginzgame/ic-metrics/issues/57).
