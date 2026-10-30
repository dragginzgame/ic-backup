# Current handoff — 2026-10-30

## Current upstream adoption

Baseline HEAD remains `6d4e675c9e083f4c92fc8ba4595086c9d8f3a536`
(Release 0.51.1). The user requested current tooling, Platform and related
dependencies, continuing the uncommitted 0.71/0.10 adoption. The complete pending
batch now selects **IC Tooling 0.72.0** and **Platform 0.11.1** with draft
**0.52.0** notes. Workspace/package versions remain **0.51.1**.
Changes are unstaged/uncommitted; no release, publication, live operation,
sibling mutation or artifact cleanup was performed.

Host Artifacts/FS/Process, Testkit and the release editor select **0.72.0**.
Production and development now share one Platform **0.11.1**; the previous
Platform 0.9/0.10 selections are removed. Unrelated locked packages are unchanged;
`/tmp/backup-adopt072-lock-review.txt` records the exact selected-family delta.
Candid **0.10.39**, Agent **0.49.3**, external Principal **0.1.5**, native
management DTO **0.11.0**, PocketIC **16.1.0** and Rust floor **1.91.0** remain.

The incoming lock, complete dirty diff and prior handoff are retained at
`/tmp/backup-adopt072-incoming.lock`, `-incoming.patch` and `-prior-status.md`.
The last preserves the 0.71 batch's original **187 native/nine simulator** results,
source graph, artifacts and evidence paths; they are not new 0.72 executions.
The manifest/lock frozen hashes are in `/tmp/backup-adopt072-graph.sha256`.
An initial update selector referred to Platform 0.10 after it had left the
resolved graph; its diagnostic remains in `-cargo-update.log`. Subsequent
locked metadata, fetch and graph admission pass without unrelated reselection.

## Provenance and compatibility

The published sparse index selects the current versions. All six package archives
match their checksums in `/tmp/backup-adopt072-registry-proof.log`; package/source
identities are in `-registry-sources.jsonl`. Platform source is
`1ed69142b0337311632db540cb70e40bbf059ee6`; Tooling source is
`8511880f26030ca70628e1c3a42d97e43d11ac37`. No sibling/path override supplies them.

Host errors now have **0.72** Rust identity. Consumers exchanging Backup's public
artifact/filesystem errors must align direct Host dependencies. Fields, source
chains, record/wire bytes and recovery behavior remain unchanged. The pending
minor notes describe this actual consumer requirement.

Tooling's selected Rust implementation directories are unchanged from 0.71;
`/tmp/backup-adopt072-tooling-rust.diff` is empty. Platform removes unused optional
Wire schema modules/features; native Principal/Account representation, Candid,
Agent and management behavior remain. Backup does not select the removed schemas
or Testkit's optional Wire calls, so no direct Wire dependency is added.
The [Platform boundary](../ic-platform.md) records current ownership and features.
No Backup Rust source, transport, runtime, persisted format, spending or recovery
transition changes were required.

Shared **0.27.11**, revision
`62edfbd43087a718bf0283e9040676a49211976d`, is independently still current on
GitHub main. Its reviewed **131 payloads** are unchanged and pass local integrity;
[the original adoption review](../shared-tooling-review.json) retains its scope.
No producer-wide suites, new gates or maintenance schedules were activated.

## Focused qualification

Strict Clippy passes for both packages, all targets/features, with warnings denied.
Actual Rust **1.91.0** independent Core and Agent consumers pass public Host
conversion probes and exact normal-graph admission; Core remains Agent-free,
and both exclude Testkit/PocketIC/WAT. Evidence: `target/msrv-consumers.XAPYPD`.

All **10 inventory/plan unit**, **13 public codec/artifact/source**, **four Agent
unit** and **16 HTTP** cases pass: **43 selected native tests**.
Both selected authenticated independent journeys pass: complete cold capture,
transfer, same-ID restore/start and effect-free terminal replay; process death
after authenticated load preserves the pending original without redispatch.
Independent build: `/tmp/ic-backup-agent-independent-build-3496677-0`.
Original fixture roots: `/tmp/ic-backup-agent-pocketic-3496677-1` and `-3`.

Logs use `/tmp/backup-adopt072-*`: `-clippy.log`, `-consumers.log`,
`-core-unit.log`, `-core-public.log`, `-agent-native.log`, `-agent-pocketic.log`,
`-setup.log`, `-testkit-routing.log`, `-source-checks.log` and `-doc-links.log`.
Selected tools are prepared and admitted offline. Testkit routing/refusal and
release metadata preparation/refusal pass; release fixtures remain in
`target/shared-release-tests.v55ojY`. Formatting, dependency inheritance/boundaries,
regular-file/license checks, Shared integrity, documentation links and diff
checks pass. No full gate, fresh macOS matrix, registry-only release acceptance,
installed Canic migration or live application qualification was run.

## Retained obligations and evidence

[Implementation contracts](../implementation-contracts.md) and the
[operator map](../canic-adoption.md#reusable-operator-composition) remain canonical.
Application fencing, external obligations, pending effects, original custody and
guarded terminal release remain integration-owned.

The [previous committed handoff](https://github.com/dragginzgame/ic-backup/blob/6d4e675c9e083f4c92fc8ba4595086c9d8f3a536/docs/status/current.md)
and retained prior-status file above preserve earlier qualification/custody history.
Preserve all `/tmp/backup-*`, `/tmp/ic-backup-agent-independent-build-*`,
`/tmp/ic-backup-agent-pocketic-*`, `/tmp/ic-backup-pocketic-*`, original Canic
journals, `target/msrv-consumers.*`, `target/shared-release-tests.*`, validation
failure logs and old/failed tool bundles. Historical passes retain their original
source/graph/host scope; this summary authorizes no disposal.
