# Current handoff — 2026-10-21

## Current boundary

Core owns backup records, bounded spending, guarded persistence, reconciliation
and coordination. Agent owns reserved signed transport. Platform owns external
IC types/codecs/SDKs. IC Tooling owns Host, Build and Testkit: generic host
IO/process custody, release metadata rendering and simulator assets/processes.
Applications retain fresh authority, consistency, fencing and external-effect
settlement. The [implementation contracts](../implementation-contracts.md) remain
mandatory; no local Rust, storage, wire or recovery implementation changes occur.

## Current batch and compatibility

Task-entry HEAD is **`1ef6227` / 0.36.1**, with the pending Platform 0.6 adoption
and its qualified evidence already present. Preserve that adoption. Host
Artifacts/FS/Process, Testkit and the separately installed Build release editor
now select published **IC Tooling 0.63.0**. Package versions remain **0.36.1**;
combined pending notes select **0.37.0** because public Host Rust identities change.
The maintainer committed the implementation as `4b93cf6` during qualification.
A concurrent maintainer release was observed; its completion is not verified.
Final handoff/provenance edits remain uncommitted; no release is retried here.

All five registry archives match published checksums and actual Cargo source
cache bytes. They record source `c342bb281dc82ff8878c26b5f5449972e9f5341d`.
Committed migration guidance is checked against those archives. Host's three Rust
implementations are byte-identical to 0.62.1. Consumers sharing
`PersistenceError::Publication` / Host FS `NamedWriteError` or the Host Artifacts
`DirectoryChecksumError` conversion must align direct Host dependencies to **0.63**.
Independent consumer checks compile those public identities. Publication phases,
byte framing, bounds, journals and recovery obligations remain unchanged.

Build and Testkit binaries require **`cli`**. Both Make setup/check recipes and
CI reuse commands select it explicitly; fixtures refuse missing selection.
Run `make install-tools` after adopting this batch. Both new tools are prepared
and pass offline admission. Library dependencies keep `cli` disabled; no tool
binary or CLI dependency enters the product graph. Prior installations and failed
evidence stay retained. Testkit still owns authenticated PocketIC **16.1.0** assets,
managed readiness/cleanup and the offline library check; no reset/retry policy is adopted.

Removed upstream Testkit Candid-call and benchmark error APIs have no Backup
consumers. Backup already uses raw management ingress with its Platform codecs
and bounded product admission. Testkit's library no longer introduces Clap or
IC Metrics, and the qualified graph converges on **Platform 0.6.0**.
The workspace lock drops the old Platform identity and those unused packages;
retained package versions/checksums, including Candid/derive **0.10.39** and Agent
SDK **0.49.2**, are unchanged in that qualified graph. A later concurrent lock
update selects **Platform 0.6.1**; it is preserved but not qualified by this batch.

## Contributor tooling and retained history

Shared Tooling remains **0.25.9**, revision
`85aded191d85ac4a8c0e5deab207f6be18895862`, with 119 canonical payloads. Required
setup orders common host/Cargo tools, locked fetch, Testkit and the release editor.
Offline checks admit required selections. Optional IC executables remain outside
aggregate setup, CI reuse/PATH and release preflight. Manual commands and retained
bundles remain available. See [development](../development.md),
[release preparation](../releasing.md) and [Shared guidance](../shared-tooling.md).

Complete adoption records remain in the [Shared archive](archive-shared-tooling-2026-10-21.md)
and [Platform archive](archive-platform-adoption-2026-10-21.md). Manifests, the
snapshot and this handoff own current selections/evidence. Exact external principal
**0.1.5** and management DTO **0.11.0** constraints remain mandatory.

## Qualification and remaining limits

All **14 focused targets** pass: offline tools, snapshot, direct dependency
boundaries/inheritance, formatting, early strict Clippy, all-target compilation,
tests, Rustdoc, Rust **1.91** independent consumers, exact package archives,
shell, consumer tooling and links. All **638 native tests** pass, including
**38 Core + 12 Agent PocketIC** and **14 HTTP** cases. All **90 consumer release
cases** pass against the actual new editor. The actual revised CI shell body
reuses exact physical host/formatter/Testkit selections in a second consumer
offline, refusing downloads. No current-repository Git/release effects occur.

Current evidence: `/tmp/backup-tooling-adoption-20261021-0omaduks/`, including
selected registry archives/metadata, source comparisons, `lock-delta.json`,
`validation/validation.Onjrrz/timings.tsv`, `install-tools.log`,
`ci-reuse-result.json`, `ci-reuse-evidence/`, `release-adapter.log`, eight
normal/build target trees and task-entry files under `before/`. Independent
consumers remain at `target/msrv-consumers.4BzR1Q/`, exact archives at
`target/package-check.SjvwLY/`; the release registry is recorded in
[fixture ownership](../release-fixture-ownership.json).

Production trees for Linux, Intel/ARM macOS and Wasm keep Core Agent-free and
exclude simulator, parser/CDK, WAT, CLI and opt-in filesystem backends. The complete
workspace graph contains one Host and Platform identity and no Clap or Metrics.
The direct gate checks aliases and normal/build/dev/target declarations, not
transitive features or deployed bytes. Native/Wasm Platform identity probes from
`/tmp/backup-platform-06-20261021-r9dxe81w/platform-identity/` retain their original
scope for Platform 0.6.0, not the later 0.6.1 update. Candid remains required for
management/SDK contracts; PocketIC's development DTO identity remains separate.
No full local `make ci`, new native macOS, installed-IC, hosted artifact delivery,
release or publication is claimed. Earlier failures and receipts stay retained.

## Remaining application acceptance

Complete acceptance still requires integration-owned current membership/control/read
authority, whole-selection fencing, retained source and external-obligation custody,
and terminal evidence. Local all-Applied checkpoints do not establish those facts.
See [application admission](../implementation-contracts.md#application-admission-and-fencing).
Earlier Canic observations retain their identities; see
[the October 19 archive](archive-2026-10-19.md).
