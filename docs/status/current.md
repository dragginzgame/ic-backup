# Current handoff — 2026-10-31

## Current dependency adoption

Baseline HEAD is `2c34f91ce7fbca0277f719f36fd84b37ace98562`
(Release **0.52.1**). The pending batch adopts production Platform **0.12.0**
alongside the earlier Host Artifacts/FS/Process, Testkit and release editor
**0.72.1** adoption. Draft **0.52.2** notes remain a compatible patch: Backup
exposes the unchanged native external IC types, while Platform's owned Principal
is internal. Workspace/package versions remain **0.52.1**.
Changes are unstaged/uncommitted. No release, publication, live operation,
sibling mutation or artifact cleanup was performed.

Platform adoption adds only the **0.12.0** facade package and adjusts dependency
edges. Testkit **0.72.1** still requires Platform **0.11.1** in the development
graph; it has no Agent feature after the split. Independent normal consumers
resolve only Platform **0.12.0**. No macro/runtime-guard package or executor/gzip
feature is enabled by either normal consumer. See [the boundary](../ic-platform.md).

Existing Host/Testkit, Log **0.4.35**, WAT **1.263.0**, Wast **263.0.0** and
wasm-encoder/parser **0.263.0** selections are preserved. Candid **0.10.39**,
Agent **0.49.3**, external Principal **0.1.5**, native management DTO **0.11.0**,
PocketIC **16.1.0** and Rust floor **1.91.0** remain unchanged. Shared **0.27.11**,
revision `62edfbd43087a718bf0283e9040676a49211976d`, retains all **131 payloads**.

The incoming lock/manifest/diff and prior handoff are retained at
`/tmp/backup-platform012-incoming.lock`, `-incoming.toml`, `-incoming.patch` and
`-prior-status.md`. Exact adoption deltas, package identities and frozen hashes
are in `-lock-review.json`, `-package-identities.json` and `-graph.sha256`.

## Provenance and compatibility

The published index selects **0.12.0**. Facade, macro and runtime-guard archives
match registry checksums and source revision
`7d9927f49320b06922fcd4dc745f8229f5a371fc`; only the facade enters Backup's graph.
Proof and original indexes are `/tmp/backup-platform012-registry-proof.json`
and `-index-*.jsonl`; verified archives/source remain under the same prefix.

All **89 Rust files** in the facade archive are identical to **0.11.1**.
The published manifest updates its own version, optional macro/runtime coupling
and optional gzip's Host requirement from 0.71 to 0.72. Backup selects neither
gzip nor runtime; public Candid/management/Agent suppliers and codec behavior
remain. Platform's owned Principal has a new dependency identity internally;
Backup's public boundaries retain external principal/DTO/SDK identities.
Review: `-source-review.json` and `-manifests.diff`.
No Backup Rust, record/wire format, spending or recovery changes were required.

The earlier Tooling review remains in `/tmp/backup-adopt0721-*`: all five archives
match registry checksums and their **293 Rust files** match 0.72.0. Its original
incoming lock/diff and prior handoff remain retained. No sibling/path override
supplies either adoption.

## Focused qualification

Both independent consumers pass on actual Rust **1.91.0**, including public Host
error probes and exact normal-graph admission. Core is Agent-free; both exclude
Testkit/PocketIC/WAT and select only Platform **0.12.0** with the intended features.
Evidence: `target/msrv-consumers.762XEr` and
`/tmp/backup-platform012-consumer-features.json`.

Strict Clippy passes for both packages, all targets/features, with warnings denied.
All **103 selected native cases** pass: **72 Core unit**, **11 public codec/
identity/selection** and **four Agent unit plus 16 HTTP** cases. These include
principal normalization, independent wire/hash goldens, codec/bounds rejection,
empty upload/data behavior and original pending/cancellation accounting.

Both selected authenticated independent PocketIC journeys pass: cold capture,
transfer, same-ID restore/start and effect-free terminal replay; process death
after authenticated load retains the pending original without redispatch.
Independent candidate graph/build evidence:
`/tmp/ic-backup-agent-independent-build-3306690-0`.
Original fixture roots: `/tmp/ic-backup-agent-pocketic-3306690-1` and `-3`.

Locked fetch, offline metadata, selected Testkit/server/editor admission,
formatting, dependency inheritance/boundaries, regular-file/license checks,
Shared integrity, documentation links and diff checks pass. Logs use
`/tmp/backup-platform012-*`: `-update.log`, `-fetch.log`, `-consumers.log`,
`-clippy.log`, `-core-unit.log`, `-core-public.log`, `-agent-native.log`,
`-agent-pocketic.log`, `-tools-check.log`, `-source-checks.log` and `-final-checks.log`.
The unchanged Tooling routing/release-preparation evidence remains at
`/tmp/backup-adopt0721-testkit-routing.log`, `-release-preparation.log` and
`target/shared-release-tests.StpDk3`; these are prior executions, not new runs.

No full CI gate, full MSRV/macOS matrix, registry-only release acceptance,
installed Canic migration or live application qualification was run.

## Retained obligations and evidence

[Implementation contracts](../implementation-contracts.md) and the
[operator map](../canic-adoption.md#reusable-operator-composition) remain canonical.
Application fencing, external obligations, pending effects, original custody and
guarded terminal release remain integration-owned.

The [previous committed handoff](https://github.com/dragginzgame/ic-backup/blob/2c34f91ce7fbca0277f719f36fd84b37ace98562/docs/status/current.md)
preserves cleanup and earlier qualification/custody history. Historical passes
retain their original source/graph/host scope. Preserve all `/tmp/backup-*`,
`/tmp/ic-backup-agent-independent-build-*`, `/tmp/ic-backup-agent-pocketic-*`,
`/tmp/ic-backup-pocketic-*`, original Canic journals, `target/msrv-consumers.*`,
`target/shared-release-tests.*`, validation failure logs and old/failed tool
bundles. This summary authorizes no disposal.
