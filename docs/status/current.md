# Current handoff — 2026-10-30

## Upstream adoption after 0.50.4

Baseline HEAD is `32cc2609fcc7b89626dfa82798f0b96957ee82be` (Release 0.50.4).
The user requested current Host, Platform and Shared Tooling adoption. The batch
is complete with focused checks; draft **0.51.0** notes explain the changed
consumer build constraint. Workspace/package versions remain **0.50.4**.
Changes are unstaged/uncommitted. No release, publication, live effect, sibling
mutation or artifact cleanup was performed.

Cargo requirements select Host Artifacts/FS/Process and Testkit **0.70.5**, and
Platform **0.9.0**. The release editor also selects **0.70.5**. The incoming lock
already selected Host/Testkit 0.70.5; those packages and unrelated selections are
preserved. Its exact bytes remain at `/tmp/backup-adopt054-incoming.lock`;
`-lock-review.txt` records only the added Platform 0.9 package and affected edges.
Candid **0.10.39**, Agent **0.49.3**, external Principal **0.1.5**, native management
DTO **0.11.0**, PocketIC **16.1.0** and Rust floor **1.91.0** remain selected.
The frozen manifest and lock hashes are in `/tmp/backup-adopt054-graph.sha256`.

Shared **0.27.11**, revision
`62edfbd43087a718bf0283e9040676a49211976d`, is adopted through its committed
exporter: **131 payloads**, eleven updates, five additions and no removals.
GitHub main independently supplied the revision; a clean isolated checkout and
trusted source comparison verify every exported byte and executable mode.
All six selected registry package archives match their sparse-index checksums.
[The review](../shared-tooling-review.json) records exact sources and evidence.

## Compatibility and maintained contracts

Production normal/build graphs use only Platform 0.9. Testkit retains Platform
0.8.3 in development; native Principal, management and simulator types keep their
upstream identities. Independent consumers prove production features without
that development graph. Core remains Agent-free.

Platform 0.9's package-wide executor singleton guard changes supported consumer
dependency composition even on native hosts. An actual isolated Cargo fixture
rejects the old registry executor alongside default-disabled Platform 0.9:
`/tmp/backup-adopt054-executor-composition/refusal.log`. Consumers selecting an
old executor or a different new Platform runtime identity must align dependencies.
This is the reason for minor notes; Backup enables no executor/runtime feature.
Native DTOs, Agent identities, public signatures, record/wire bytes and all
original spending, custody and recovery rules remain unchanged. No product Rust
source changes were required.

Shared input checkpointing and independent Cargo workspace admission retain
their actual consumer callers. Publication Make dispatch remains opt-in; Backup
keeps its existing publication and exact-resume adapters. No path-based Rust
module wiring exists. Seven narrow shared-fixture dead-code allowances retain
their used/unused binary scope; an expectation would fail when the helper is used.
Unfulfilled expectations remain denied.

## Focused qualification

Strict Core/Agent Clippy passes for all targets/features with warnings denied.
Actual Rust **1.91.0** independent Core and Agent consumers pass with original
graph admission and public Host error probes; evidence is
`target/msrv-consumers.QOZKZ5`.

All **154 selected Core unit cases**, **13 public codec/artifact/source cases**,
**four Agent unit cases** and **16 HTTP cases** pass: **187 native tests**.
All **nine selected authenticated PocketIC cases** pass: seven independent
normal-consumer journeys, complete Agent transfer/upload/same-ID restore and
wrong trusted root. Cold replay remains effect-free; process death, failed
retention and partial output retain original pending authority without reissue.
Independent build evidence is `/tmp/ic-backup-agent-independent-build-2428378-0`.
Original fixtures are `/tmp/ic-backup-agent-pocketic-2428378-*`, including complete
cold restore `-9`, load death `-11` and operator failures `-13`, `-15`, `-17`.

Logs use `/tmp/backup-adopt054-*`: `-clippy.log`, `-consumers.log`,
`-core-unit.log`, `-core-public.log`, `-agent-native.log`, `-agent-pocketic.log`,
`-tooling.log`, `-testkit-routing.log` and `-source-checks.log`.
Selected tool setup and offline admission pass. Consumer input-drift/refusal and
retained failure evidence pass. Release preparation/refusal and selected-source
metadata/real-index cases pass with editor 0.70.5; originals remain in
`target/shared-release-tests.rCBmhJ` and `target/shared-release-tests.dplp36`.
Changed shell lint, new Perl syntax, formatting, dependency inheritance/boundaries,
repository files/licenses, snapshot integrity and local documentation checks pass.
No full gate, fresh native macOS matrix, registry-only release acceptance,
installed Canic migration or live application qualification was run.

## Retained obligations and evidence

[Implementation contracts](../implementation-contracts.md),
[Platform ownership](../ic-platform.md) and the
[operator map](../canic-adoption.md#reusable-operator-composition) remain canonical.
Application authority, external obligations, original effects and guarded terminal
release remain integration-owned. This adoption completes no Canic live operation.

The [previous committed handoff](https://github.com/dragginzgame/ic-backup/blob/32cc2609fcc7b89626dfa82798f0b96957ee82be/docs/status/current.md)
retains the 0.50.4 cleanup, exact frozen inputs and earlier qualification/custody
history. Preserve all `/tmp/backup-*`, `/tmp/ic-backup-agent-independent-build-*`,
`/tmp/ic-backup-agent-pocketic-*`, `/tmp/ic-backup-pocketic-*`, original Canic
journals, `target/msrv-consumers.*`, `target/shared-release-tests.*`, validation
failure logs and old/failed tool bundles. Historical passes keep their original
source/graph/host scope; no retained data or evidence is disposable by this summary.
