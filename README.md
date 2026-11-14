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
Capture intent creation/reopening now retains an immutable approved plan and durable
target ownership. It validates bounded chained ledger frames under held original
file custody; see the [intent contract](docs/architecture/capture-intent.md).
`CapturePlan::validate_status` checks decoded Platform status for stopped state,
the approved module hash and the actual management caller as a controller.
`CapturePlan::validate_status_reply` applies those checks to a bounded raw reply,
with separate limits on skipped fields.
`CapturePlan::plan_transfer` now produces bounded, deterministic data-read requests
from Platform metadata. `CapturePlan::decode_metadata` bounds raw metadata and
checks it against the original plan. `CaptureTransfer::decode_data` bounds raw replies and checks
exact region lengths and chunk hashes; see the [transfer contract](docs/architecture/capture-transfer.md).
Attempt reservation, capture, completion writing, full evidence verification, restore, CLI and simulator
workflows are not implemented yet. See the
[implemented local-store contract](docs/architecture/local-inventory.md) for scope
and the public Rustdoc example. Publication remains disabled; `0.100.0` is an
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
