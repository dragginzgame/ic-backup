# IC Backup

In 0.103.0, commitment arguments and results use
`ic_backup::store::BackupDigest`. Replace the former Host `Sha256Digest` import;
`compute`, `from_bytes`, `as_bytes` and canonical lowercase parsing/display remain
available on the new Backup type. Filesystem and byte errors now use Host 0.80
variants; publication distinguishes an unpublished attempt from visible but
unconfirmed cleanup/durability. This is a Rust API change, with no stored-format
change or migration of existing evidence.

An experimental Unix host library for capturing one stopped Internet Computer
canister and restoring it to the same canister, network and release. Restore
verifies metadata and every uploaded byte before loading, then returns
`LoadedStopped`. Application acceptance and restarting remain caller-owned.

The current implementation uses a host signer that is an actual controller of
the target. Canic integration is required for the intended managed application
product and is still unfinished. A host controlling a manager canister does not
thereby control that manager's children. No manager route is implemented here.

This incompatible restart replaces legacy `ic-backup`/`ic-backup-agent` APIs.
Version 0.103.0 remains an experimental restart release. Rust 1.100 or later is
required. Native checks target Linux x86-64 and macOS ARM64/x86-64;
actual-IC qualification currently runs locally on Linux using PocketIC.

## Required integration

- Supply a Tokio runtime with IO and timers for network operations.
- Configure `BackupHttpTransport` with an HTTPS endpoint, exact network identity,
  independently provisioned root key and finite timeout. Plain HTTP is limited
  to literal loopback addresses. Backup does not discover a trusted root.
- Implement `BackupSigner`: sign only the exact requested call or observation
  within its deadline, envelope limit and reserved signing/network allowance.
  The direct route supports Ed25519 and secp256k1 identities.
- Implement `BackupApplicationGuard`: admit the actual capture or restore action
  under an original continuous maintenance witness. Stopped state alone is not
  proof that payments or other external obligations can safely be rewound.
- Provide a private store directory, exact target/release/module identities,
  immutable approvals and finite operation limits. Preserve them for recovery.

Configure one `BackupConnection` from the transport, signer and guard. Plan and
approve capture with `connection.plan`, then call `store.create_capture` once.
Drive `operation.advance` until `Complete` or `NeedsAttention`. Each advance
submits at most one management call; pending requests never retry automatically.
The public API documentation contains compiled examples of the advance loop.

For restore, open and verify the original capture source, call
`connection.plan_restore`, approve the digest and create the restore under its
authenticated completed predecessor. Drive the same advance loop. Snapshot
allocation, upload, full readback, load and stopped-state verification are fixed
parts of the procedure, not arbitrary caller-selected stages.

## Capacity profile

The qualified first-release profile contains **20 MiB of combined module, heap,
stable memory and actual chunk bytes**. Linux PocketIC exercises capture, restore,
interrupted-transfer recovery and offline replay at that size. Qualification is
shape-bounded; arbitrary chunk counts have no blanket guarantee. See the maintained
[capacity evidence](https://github.com/dragginzgame/ic-backup/blob/main/docs/architecture/capture-transfer.md)
for measured qualification status and costs.

The qualification profile uses 1 MiB extents, at most two chunks (including an
empty chunk), a 22 MiB payload allowance, 128 network and signing reservations,
one observation allowance and 1 GiB of evidence allowance per operation. Capture
uses an 8 KiB request / 4 MiB result ceiling; restore uses 2 MiB / 4 MiB. The extra
payload allowance accounts for metadata reserving a full extent per chunk before
its actual length is known. Allowances remain spent after interruption and small
results do not refund them. Remote signer costs require additional planning.

Payload declarations up to 1 GiB are structural limits, not tested end-to-end
capacity. Many small or empty chunks can exhaust call/evidence budgets well below
the byte limit. Replay cost grows with history; do not infer production throughput
from fixture results. Production HTTPS and Canic adapters need separate qualification.

## Recovery and migration

Keep the **entire private store**, original operation IDs and approval digests,
plus the application's maintenance/fence authority. A payload directory or a
listed manifest alone cannot authorize restore. Inventory is a declaration;
`open_capture_source` authenticates original completion and verifies payloads.

Reopen with `open_capture(id, approval)` or `open_restore(id, approval)`.
Use `reconcile_reply` for a surviving original certified reply, or one explicitly
reserved `reconcile_observation` lookup. A post-response error can return bounded
certificate bytes; `reconcile_observation_reply` imports a surviving original
lookup reply. These mechanisms retain original spending and authority. They do
not permit a replacement request. Completed replay needs no live signer/network.

The restart's store and intent formats are incompatible with legacy stores.
There is no automatic migration or import of old operations. Finish or recover
them with their matching preserved reader, original IDs, approvals and evidence.
Do not delete occupied stores, partial files or journals to bypass an uncertain
operation. There is no generic abandonment or reset API.

Backup does not own fleet scheduling, pruning, snapshot deletion, cloud storage,
cross-canister/cross-release restore, application acceptance or automatic start.
Each capture and restore allocation retains a remote snapshot; the integration
must govern snapshot capacity and retention separately.

See the [operation contract](https://github.com/dragginzgame/ic-backup/blob/main/docs/design/operation-contract.md),
[Canic boundary](https://github.com/dragginzgame/ic-backup/blob/main/docs/design/sibling-integration.md)
and [current status](https://github.com/dragginzgame/ic-backup/blob/main/docs/status/current.md).
