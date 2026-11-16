# IC Backup

A standalone host-side Internet Computer backup library. Canic is a required
integration and owns its adapter, membership, application maintenance and external
obligations. Backup production depends on Platform and Host, without Canic runtime
or control-plane crates. Every package remains unpublished.

Capture runs through one fixed public loop. Configure `BackupConnection` with a
bounded transport, sign-only adapter and action-aware application guard, then use
`connection.plan(request, options)` and approve its exact digest.
`BackupStore::create_capture` retains that immutable intent and target ownership.
Call `CaptureOperation::advance` until it returns `Complete` or `NeedsAttention`.
Backup derives status/list/take/metadata/data/final-status calls, materializes the
payload and publishes its exact manifest. Each advance submits at most one
management call. There are no public phase commands.

Signing and network allowance stay spent after interruption. Pending work never
resubmits automatically. Explicit `reconcile_reply` imports an original certified
reply locally; `reconcile_observation` reserves one bounded original-request
read-state lookup. After a received certificate cannot be recorded, typed errors
return its bounded bytes and original request ID. `reconcile_observation_reply`
imports a surviving lookup proof under its original separate time policy.
Cold terminal replay authenticates original evidence without
a clock, signer, live guard, HTTP or writes. Dependent effects require live admission.

The private store supports bounded `list_backups`/`latest_completed_backup`,
trusted-manifest `verify_payload`/`open_payload`, authenticated
`open_capture_source`, and completed-target `create_capture_after`.
Older completed sources remain usable after authenticated succession.
Both `open_capture(id, approval)` and `open_restore(id, approval)` recover original
intent and progress without reconstructing a plan or supplying live credentials.
Use `connection.plan_restore(source, request, options)` to approve a same-canister
restore. Its fixed loop allocates/uploads, compares metadata and every byte, loads
and verifies stopped state, then derives `LoadedStopped` from the original journal.
Capture and restore share one recorder and recovery boundary. Canic process,
manager execution and maintenance/fence adapters are tracked in Canic
[#5](https://github.com/dragginzgame/canic/issues/5),
[#6](https://github.com/dragginzgame/canic/issues/6) and
[#7](https://github.com/dragginzgame/canic/issues/7).
The selected direct-controller library workflow is implemented; full Canic
integration remains dependent on those concrete consumers.

The public capture replacement uses the distinct `ic-backup/host-store/1` marker.
It refuses older stores intact. Their original implementation, authority and
recovery evidence remain preserved; no migration or reset occurs.
Actual PocketIC tests qualify multi-extent capture, empty/nonempty chunks,
publication, succession, full older-source restore and capture after restore,
interrupted allocation/upload/load, interrupted take observation,
original payload-reply settlement and offline replay with fixture signing and
application authority. They additionally exercise full 1 MiB whole chunks,
offline recovery from a failed load-result write, and a separately built normal
consumer continuing capture/restore across processes. They do not qualify Canic
or physical power loss.

After [tool preparation](docs/development/setup.md):

```bash
make fetch-dependencies
make deps-list
make check
```

See the [capture contract](docs/architecture/capture-intent.md),
[transfer contract](docs/architecture/capture-transfer.md),
[inventory contract](docs/architecture/local-inventory.md),
[restart direction](docs/design/restart.md), [handoff](docs/status/current.md),
[sibling review](docs/design/sibling-integration.md) and [pending notes](CHANGELOG.md).
The [legacy repository](https://github.com/dragginzgame/ic-backup-legacy) retains
its own release identities and recovery obligations; its artifacts are not imported.
