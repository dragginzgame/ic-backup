# Current handoff — 2026-10-26

## Released source and ownership

Released Backup is **0.44.2**, `33702a4`. Core owns bounded records, guarded
persistence, reconciliation and original-plan coordination. Agent owns reserved
signed transport. Applications own current authority, membership, release binding,
whole-unit fencing and external obligations. Host owns generic IO/process/hash,
Platform external IC identities/SDKs, and Testkit simulator assets/processes.
[Implementation contracts](../implementation-contracts.md) retain product obligations.

The selected graph uses Host/Testkit and the Build editor **0.68.7**, Platform
**0.6.18**, Agent 0.49.3, management DTO 0.11.0 and PocketIC 16.1.0. Shared Tooling
is the reviewed **0.26.20** snapshot. Package versions remain 0.44.2.

## Pending 0.44.3

The incoming private journal admission cleanup remains unchanged: one helper
admits exact authority, pending mutation and absence of recovery in that order.
Mutation, acquisition, transfer-read and upload requests preserve public typed
errors, constructor checks and rejection order. Its earlier thirty focused tests
and public-error assertions remain at `/tmp/backup-pending-admission-*.log`.
No new record, public API, effect, spending or recovery mode is introduced.

Shared's generic checksum/archive suites and release-runner matrix are retired
through its canonical exporter. Backup retains actual CI/evidence callers,
Testkit setup ordering, product gates and native-host CI. Shared owns its generic
matrices upstream. Current release-adapter assertions remain local.

Normal releases now start fresh from clean committed metadata without reading
historical plans. Only the selected candidate's stale plan/staging/PR receipt is
superseded; other plans, metadata backups and failure evidence remain retained.
Backup archives earlier validation bytes before fresh proof. Exact saved recovery
uses `release-resume VERSION=X.Y.Z` and still requires the original receipt.
[Release guidance](../releasing.md) owns the command contract; no release is run.

## Published upstream adoption

Manifest minimums and the contributor release editor select published Tooling
**0.68.7**, source `7ae301d00353e1e27e94300d492e47f9736941c3`, and Platform
**0.6.18**, source `98a2a8aa9b015c730ef222941b387674a729b0ab`. Registry index,
archive SHA-256 and committed source comparisons are retained in
`/tmp/backup-adopt-published.json`. Dirty sibling Platform work is excluded.
Shared's committed **0.26.20** source is
`a7f85d45d4212d4c39225f58597580560a333550`; its clean checkout matched observed
GitHub main. [The review](../shared-tooling-review.json) binds selection and evidence.

The incoming lock already selected these libraries and remains byte-identical:
SHA-256 `350624a872db50c087b688c26170f76146099c54e60b972bfdb27c10ebccd78e`.
Host FS consolidates name admission; Testkit consolidates startup/assets; Build
consolidates metadata editing. Platform's added optional BLAKE3 feature is not
selected. Existing features, public external identities, wire/storage bytes and
product Rust source are unchanged. No new dependency patch or sibling input is used.
Canonical setup prepared the selected CLIs and reused authenticated PocketIC
assets, retaining previous tools and failed evidence. Offline admission passes.

## Focused Linux qualification

Both affected libraries pass strict all-target/all-feature Clippy. Artifact
checksum/publication, download integrity and local restore-source integration
checks pass. Both independent Rust 1.91.0 consumers pass, including public Host
error identities; evidence is at `target/msrv-consumers.Dz7z4d/`.
Root and independent locked metadata pass. Normal consumers select singleton
Platform 0.6.18 and Host 0.68.7 without CLI, simulator, WAT, Wire-schema or CDK
runtime features; Core stays Agent-free. Graph evidence: `/tmp/backup-adopt-graphs/`.

The connected Core capture/restore/terminal-replay case passes with retained
fixture `/tmp/ic-backup-pocketic-3016097-0/`. Both authenticated independent Agent
journeys pass: cold capture/restore/application acceptance and terminal replay,
and death after load preserving original pending spending without redispatch.
Build/graph evidence: `/tmp/ic-backup-agent-independent-build-3141305-0/`;
application fixtures: `/tmp/ic-backup-agent-pocketic-3141305-{1,2}/`.

Backup's release-adapter suite passes, including explicit lost-commit recovery,
exact historical receipts, fresh patch/minor attempts leaving earlier delivery
uncertain, failed-gate retention and real private-index admission. Evidence:
`target/shared-release-tests.DrvLEW/` and `/tmp/backup-adopt-release.log`.
Actual CI/evidence caller fixtures, focused ShellCheck, snapshot integrity,
formatting and local documentation links pass. Logs use `/tmp/backup-adopt-*`.
The initial sandbox-only setup and local-port failures remain retained alongside
successful authorized retries; they supply no passing product evidence.

## Limits and remaining integration

Native macOS, installed Canic/IC, new package-archive/registry-only acceptance,
Shared's owner matrix and full local CI were not run. Host-consumer graphs do not
qualify deployed Wasm bytes; deployment binding remains application-owned.
Agent still contains crypto-bigint 0.5.5 transitively; this adoption does not resolve
that existing cryptographic dependency concern. The [Canic guide](../canic-adoption.md)
retains application responsibilities before retiring its existing restore engine.
No staging, commit, push, release, publication, live IC effect or sibling edit occurs.
