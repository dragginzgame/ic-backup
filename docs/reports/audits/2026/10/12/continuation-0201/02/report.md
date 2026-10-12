# Shared Tooling 0.8.0 and current locked graph

Released Backup 0.20.0 is `2e9dbc4ccaae7822d07afcca40e39c1e1ae9c4b2`.
The compatible uncommitted draft remains 0.20.1 with package versions 0.20.0.
The preceding [0.6.6 phase](../01/report.md) retains its exact Testkit 0.34.2
gate and earlier failed attempts. A subsequent external lock change selected
Testkit 0.34.3; this review uses that newer graph.

Canonically export committed Shared Tooling 0.8.0
`045a6f22a9de497bfb3af90cb2571957384806f2` from a clean isolated source.
The preview and export retain the reviewed 102-file payload. The v1 manifest
records those existing paths as initial entrypoints; no payload is retired.
Four selected files update: the snapshot verifier, snapshot-adoption guide,
Make-interface guide and snapshot-drift task. Future narrowing requires a
separate reviewed selection of directly used roots and derived companions.
The snapshot verifier passes on the exact committed source. Common Cargo
tool selection stays at `cargo-sort-derives` 0.14.1 from the previous phase;
literal `$crate` direct and conditional derive probes passed there. Backup has
no affected product macros or default derive-sort invocation.

The incoming lock retains all four Host 0.14.5 crates and Metrics 0.7.1. Its
only change since the passing 0.6.6 phase is Testkit 0.34.2 to 0.34.3. The
new Testkit archive matches its lockfile checksum and committed producer source
`c78b02be422018067b48f6aaa4c20fb0d6f95ad2`; no Rust file changes from
0.34.2. Explicit CLI/server setup and offline admission pass. A full 0.34.3
gate under Shared 0.6.6 passed through release, hooks and formatting, then
stopped at a transient foreign shared-store installation lock during Testkit
admission. The lock cleared, admission passed, and the original failure remains
retained. No lock or source rollback occurred.

Every registered full `make ci` target passes on the final unchanged source
and Testkit 0.34.3 lock inputs, including strict Clippy, both real simulator
suites, Rust 1.88 independent consumers and paired package archives. The root
manifest, lockfile, snapshot, other technical files, real index and HEAD stay
unchanged across that gate. [Compact qualification evidence](qualification.json)
records exact targets and input identities. Detailed logs and canonical
preview/export evidence stay under ignored `target/continuation-0201/`.

[Host 0.14.5 producer CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38225165549)
passes Linux and both native macOS hosts. [Shared Tooling 0.8.0 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38229670802)
passes lint/Linux but both macOS jobs were cancelled. [Testkit 0.34.3 CI](https://github.com/dragginzgame/ic-testkit/actions/runs/38231005016)
passes Linux jobs with most macOS jobs queued at final inspection.
[Released Backup 0.20.0 CI](https://github.com/dragginzgame/ic-backup/actions/runs/38225541731)
passes Linux with both macOS jobs running. This uncommitted 0.20.1 candidate
has no hosted/native result, so producer and consumer native acceptance remain
separate. Earlier reports retain their own source and graph identities.
Coordination: [Backup #36](https://github.com/dragginzgame/ic-backup/issues/36),
[Shared #131](https://github.com/dragginzgame/shared-tooling/issues/131).
