# IC Backup

A fresh workspace for a standalone host-side Internet Computer backup and recovery
library. Canic is a required integration alongside standalone consumers. Its
adapter consumes the public library contract; application coordination and
control-plane authority remain Canic-owned. The planned v1 stores complete
snapshot bundles on the operator host and provides local backup history. Canic can coordinate
independent operations for several canisters; Blob Storage integration is v2.

The `ic-backup` library implements private local-store initialization and bounded
backup inventory: `list_backups` and `latest_completed_backup`. Inventory reads
completion declarations. A separate `verify_payload` method stream-checks all
payload files against a caller-trusted manifest hash and explicit byte limit.
`open_payload` retains those original files and verifies bounded source reads
against ephemeral block hashes; it does not yet verify capture evidence or admit restore.
Capture intent creation/reopening now retains an immutable approved plan and durable
target ownership. It validates bounded chained ledger frames under held original
file custody; see the [intent contract](docs/architecture/capture-intent.md).
`CapturePlan::validate_status` checks decoded Platform status for stopped state,
the approved module hash and the actual management caller as a controller.
`CapturePlan::validate_status_reply` applies those checks to a bounded raw reply,
with separate limits on skipped fields.
Snapshot list/take decoders bound replies and reject duplicate or reused IDs;
`CapturePlan::take_snapshot_arguments` checks explicit capacity and preserves
existing snapshots and installed code without performing a call.
`CapturePlan::plan_transfer` now produces bounded, deterministic data-read requests
from Platform metadata. `CapturePlan::decode_metadata` bounds raw metadata and
checks it against the original plan. `CaptureTransfer::decode_data` bounds raw replies and checks
exact region lengths and chunk hashes; see the [transfer contract](docs/architecture/capture-transfer.md).
`CaptureTransfer::decode_uploaded_metadata` checks bounded restore readback metadata
against the captured state, including exact float bits and the complete chunk set.
`CaptureTransfer::begin_readback` adds ordered byte comparison of every destination
extent and chunk against caller-held source slices. Completion requires full local
comparison coverage; authenticated evidence, source custody and load authority
remain separate requirements.
`CaptureOperation::advance_status` now durably reserves signing and submission,
retains exact request/result evidence and replays the initial direct-host status
call without repeating pending effects. `inspect_status` recovers the retained
stage, original admission, reserved counts and request identity without provider
calls or writes. `DirectCaptureProvider` composes external signing, live admission
and authenticated HTTP under those reservations. Prepared call content is checked against the
original plan and request ID through Platform. `status_certificate_evidence` builds bounded reply/rejection evidence, and
`authenticate_status_evidence` verifies retained certificates against original trust, request ID
and admitted time windows, using Platform. `authenticate_status_request` verifies
original direct Ed25519/secp256k1 signatures and sender bindings without changing bytes.
`StatusHttpTransport` authenticates those requests and sends retained
status ingress once with bounded deadlines/body reads and no retries or polling.
Host HTTP supplies the shared body reader and Reqwest re-export; Backup owns the
request and recovery policy.
Integrations supply a bounded sign-only adapter, continuous application-authority
guard and explicit certificate timing policy. Backup retains the original time
window and verifies settled replay without live authority or a current clock.
`advance_snapshot` follows authenticated status with a durable list and one
non-replacing snapshot take. It preserves original requests, per-call budgets
and unresolved outcomes; `inspect_snapshot` reports cumulative reservations.
A private PocketIC fixture qualifies status, snapshot creation and offline replay; Canic's process and maintenance adapters remain separate integration work. See the [first-call contract](docs/architecture/capture-intent.md#durable-initial-status).
Download, original-request reconciliation, completion writing, full bundle verification, restore and CLI
workflows remain unimplemented. See the
[implemented local-store contract](docs/architecture/local-inventory.md) for scope
and the public Rustdoc example. Publication remains disabled; `0.101.2` is an
unpublished development version following the legacy `0.67.x` line.

The [legacy repository](https://github.com/dragginzgame/ic-backup-legacy) retains
existing implementations, published history and recovery obligations. Its source,
backlog, release automation and retained data do not define the new implementation.

After [tool preparation](docs/development/setup.md):

```bash
make fetch-dependencies
make deps-list
make check
```

See the [library design proposal](docs/design/library-design.md),
[legacy assessment](docs/design/legacy-assessment.md),
[sibling integration review](docs/design/sibling-integration.md),
[proposed operation contract](docs/design/operation-contract.md),
[local storage and inventory](docs/design/local-store.md),
[restart direction](docs/design/restart.md), [local instructions](AGENTS.md)
and [pending notes](CHANGELOG.md). Every package remains non-publishable until
the implementation is mature and publication is explicitly authorized.
