# Current handoff — 2026-11-15

Work lives at `/home/adam/projects/ic-backup`, branch `main`; the former legacy
local path was absent. Frozen legacy remains read-only at `dragginzgame/ic-backup-legacy`.
No original journals, bundles or obligations were imported, migrated or cleaned up.

The 0.101.2 provider/snapshot checkpoint was pushed and remotely verified at
[ec681f2](https://github.com/dragginzgame/ic-backup/commit/ec681f21506db0482b2cf3843f44593c56347994).
The user selected GitHub delivery of retained-result recovery, local observation
certificate authentication and the Platform/Tooling refresh. Pending 0.101.3
notes cover these compatible changes; product manifests remain 0.101.2. All
packages remain `publish = false`; no tag, publication or deployment is selected.

## Implemented scope and safeguards

The [inventory contract](../architecture/local-inventory.md) owns private stores,
bounded listings/latest backup per target, manifest-bound verification and held
source reads. Integrity does not authenticate capture or grant restore authority.
The [capture contract](../architecture/capture-intent.md) owns immutable plans,
durable claims and bounded chained ledgers. Uncertain writes invalidate handles;
complete uncertain frames remain spent. Partial frames, orphan evidence and
missing owners/locks refuse without repair or fresh authority.

`advance_status` checks stopped state, approved module and original controller.
`advance_snapshot` authenticates the original list, admits capacity and takes one
new snapshot. Each call retains exact requests, separate original admissions and
reservations within cumulative budgets. Prepared continuation never resigns;
pending continuation invokes no provider; settled replay authenticates locally.
Inspection is not authentication or completion. No replacement or cleanup exists.

`settle_capture_result` records surviving original certified status/list/take
replies under original trust/timing and spending. Identical settled replay is
effect-free; replacement bytes refuse. Rejection and policy-invalid replies may
be retained but never become policy success. Settlement grants no application
authority and advances no dependent work. Orphans/partial tails still refuse.

`observe_capture_certificate` locally authenticates original signed requests and
certificates under a separate explicit observation window and combined byte cap.
It distinguishes received/processing/done/absent/pruned without refunds or retry
permission. It creates no observation receipt, spends no budget and performs no
lookup. Later certificates do not widen original settlement timing.

`DirectCaptureProvider` composes bounded external sign-only and HTTP submission
for each call. Route/network/root, signer/guard protocols and historical timing
remain bound to original authority; expiry and spending never renew. Direct
Ed25519/secp256k1 are supported; delegated/other identities refuse. Integrations
own signer/process custody, exclusive snapshot custody and continuous application
maintenance. Backup owns no keys or Canic runtime.

The [transfer contract](../architecture/capture-transfer.md) owns bounded pure
metadata/data decoding, schedules and upload/readback comparison. Remote lookup,
download, completion writing, full bundle-evidence admission, upload/load/restore
and CLI remain unfinished. A remote snapshot is not a complete local backup.

## Dependencies and qualification

Platform 0.33.7/Types 0.20.4 and Tooling 0.44.1 are adopted in both workspaces.
Platform manifest minimums are 0.33.7; Tooling has exact inherited disabled markers.
The authenticated exporter retains nine entrypoints/51 files. The selected changes
are the baseline and LOC reporter; omitted depth now matches `-d 0`. Backup has no
caller of the removed Python evidence API. Compiler/tool pins and Testkit wiring
are unchanged. [Setup](../development/setup.md) owns provenance and commands.

Host FS/Artifacts/HTTP remain 0.79.0, Tokio 1.54.1 and Rust/Cargo 1.100.0.
Reqwest remains behind Host HTTP; there are no sibling patches or source edits.
Private qualification retains Testkit 0.106.0/PocketIC 16.1 and its exact
`thiserror` 2.0.18 constraint; product retains 2.0.21. Other selections are unchanged.

Current adoption checks pass: 93 capture/recovery tests, 17 documentation examples,
strict product/private Clippy, warning-denying Rustdoc, both actual IC fixtures,
tools/snapshot admission, formatting, documentation links, dependency listing and
equal default/`-d 0` LOC output. Logs are under
`target/evidence/platform-tooling-refresh/`. Earlier evidence remains under
`target/evidence/snapshot-capture/`, `target/evidence/capture-settlement/` and
`target/evidence/capture-observation/`.

The actual IC fixtures cover normal status/list/take and a surviving real take
reply withheld before settlement, recovered after server shutdown and guard
revocation with exactly three signing/submission reservations. Prior snapshots,
module, stopped state and original evidence are preserved. This is provider-boundary
interruption, not physical power loss or lookup of a never-received reply. Failed
and successful fixture bundles remain under `qualification/target/fixtures/`.
The broader product suite, live read-state, external signer processes, Canic fences,
trusted HTTPS sessions, hosted macOS and full backup/restore remain unqualified.

## Next work

Platform 0.33.7 supplies the strict read-state reader requested by
[Platform #167](https://github.com/dragginzgame/ic-platform/issues/167), now closed.
Its raw-input bound is not a total work/allocation quota or consumer authentication.
Implement separate durable signing/lookup spending, exact-path/signature/binding
admission and observed-result retention, then qualify actual read-state ingress.
Snapshot download and verified local completion remain subsequent work. Never
repeat a take because its reply was lost.

Canic owns membership, concurrency, routing, release identity, maintenance,
external settlement and reopening; [Canic #1](https://github.com/dragginzgame/canic/issues/1)
covers adapters and executor/fenced recovery. V1 remains one already-stopped
canister, local bundles and same-release restore to the same existing ID, left
stopped. Blob storage remains v2.
