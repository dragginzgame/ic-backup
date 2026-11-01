# Current handoff — 2026-11-01

## Published Platform and Testkit adoption

The workspace now requires Platform **0.17.0** and Testkit **0.77.0**, resolving
both from crates.io. Their shared native PocketIC client is the maintained
**ic-platform-pocket-ic 0.17.0** package; canonical owned Principal comes from
**Types 0.16.0**. This resolves the previous published Testkit/client mismatch.
The former blocked attempt and original handoff are archived under this batch's
`before/`; candidate-only proofs retain their original scope.

Official metadata and checksum-verified old/new archives are retained in
`/tmp/backup-platform-testkit-adopt.j3pankrp/`. Testkit's Rust source is identical
to **0.76.2**. Platform's consumed Candid, management, external Principal and Agent
facade files are unchanged from **0.16.0**. Types changes concern optional
self-authentication; Backup selects only `principal-external`. Principal admission,
identity/hash behavior, external SDK types and management wire contracts retain
their existing behavior. No product Rust migration or public owned Principal API
change is required. Own versions remain **0.53.5**; the compatible **0.53.6** draft
records the dependency batch. Changes remain unstaged/uncommitted.

Only the required Platform/Types/Testkit supplier entries changed in the incoming
lockfile: the old registry PocketIC client is replaced by the owned client, and
the target-dependent owned Runtime **0.17.1** entry is added. Scoped online Cargo
updates and locked fetch preserved every unrelated selected package. Host
Artifacts/FS/Process stay at the incoming **0.73.6** selections under **0.73.5**
requirements; arithmetic-only Metrics stays **0.24.6**. Optional Wire remains
unselected. Runtime is absent from both production host graphs.

Independent normal and published-consumer admission now rejects the maintained
PocketIC package by resolved identity alongside Testkit, registry PocketIC and
WAT. Existing fixture cases exercise the new identity, including renamed edges.
No vendored Shared files or runtime/product policies were changed.

## Fresh focused qualification

Evidence is `/tmp/backup-platform-testkit-adopt.j3pankrp/`: incoming copies/diff,
registry metadata and verified archives, source/manifest comparisons, full locked
metadata, production trees, graph proofs, frozen inputs and command logs.

- Strict both-package all-target/all-feature Clippy passes (`clippy.log`).
- Formatting, Cargo inheritance/pins, direct IC dependency boundaries and local
  documentation links pass (`source-checks.log`, final handoff checks). The actual
  Linux normal/build trees retain one Platform/Types/Candid/management/Principal
  identity, with Agent only in Agent; simulator/runtime/Wire/Metrics/WAT/CLI
  packages are excluded (`production-graph-proof.json`). The reviewed snapshot
  has no generic graph helper; its failed attempted invocation is retained and
  actual locked Cargo observations supply the explicit proof instead.
- Both graph-admission fixture sections pass (`graph-fixtures.log`); this runs the
  affected native and published consumer cases without unrelated tooling suites.
- Both independent normal consumers pass on actual Rust **1.91.0**
  (`msrv-consumers.log`, `target/msrv-consumers.g5qufG/`). Core also checks public
  Host checksum/publication error identities. This is not a full MSRV matrix.
- All **12 selected Core contract tests** pass (`core-contract-tests.log`), covering
  canonical principal aliases, exact request bytes, inventory/authority hashes,
  malformed declarations and admission bounds.
- Both real Core PocketIC cases pass (`core-pocketic.log`): original same-ID
  capture/restore/effect-free replay and resume at allocation/load/start. Their
  retained roots are `/tmp/ic-backup-pocketic-3442151-{0,1}`.
- The independent normal Agent consumer passes real authenticated cold capture,
  restore acceptance and terminal replay (`agent-pocketic.log`). Its retained
  roots are `/tmp/ic-backup-agent-pocketic-3463653-1` and
  `/tmp/ic-backup-agent-independent-build-3463653-0`. Together with the Core
  cases, all **15 selected tests** pass on the published dependency graph.

The exact locked Testkit **0.77.0** CLI is prepared and its offline server check
passes. The first setup attempt timed out behind an active shared installer for
another consumer; its failure log is retained. After that owner finished, retry
reused its admitted cache. The authenticated PocketIC server remains **16.1.0**;
old/failed bundles and original fixture evidence remain retained. Both simulator
suites authenticate assets through the selected Testkit library before startup.
Frozen Rust/manifest/lock/graph inputs match the passing validation.

No full repository CI/release gate, normalized registry package acceptance, fresh
native macOS matrix, installed-IC or live Canic qualification was run. Focused
native consumers and actual simulator cases qualify this graph; they do not grant
generic application or live effect authority.

## Preserved baseline and evidence

The user-released source baseline remains **0.53.5**, commit `74c9a0e`. The four
previous simplifications and their **74 focused passes** retain their original
proof under `/tmp/backup-four-cleanup.4kn9mu5z/`. The prior Host/Testkit receipt is
`/tmp/backup-host-testkit-update.vxhtct1w/`; prior Platform candidate work is
`/tmp/backup-platform-adopt.oex0l5i7/`, with its upstream draft PR #306. Those
historical states are not current publication or adoption blockers.

The explicitly delegated **0.53.4** delivery remains
`/tmp/backup-platform-testkit-deploy.44nom2x5/`, including independently verified
registry/VCS identities at `959b2d8`. No new release/publication was requested or
executed here. Existing release receipts retain their original scope.

Other identities stay Candid **0.10.39**, native Agent **0.49.3**, external
Principal **0.1.5**, native management DTO **0.11.0** and Build editor **0.73.1**.
Shared remains `143a9ab`; its snapshot and payloads were not refreshed. Earlier
fixture, failure, custody and recovery evidence stays recorded in the archived
incoming handoff. Preserve original journals, referenced artifacts, simulator/
consumer roots and old/failed bundles. No staging, commits, publication, sibling
mutation, live IC effects or cleanup occurred in this batch.

[Implementation contracts](../implementation-contracts.md),
[the Platform boundary](../ic-platform.md) and
[the operator map](../canic-adoption.md#reusable-operator-composition) remain
canonical.
