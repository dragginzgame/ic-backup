# Real single-canister snapshot qualification

The `pocketic_snapshot` integration target exercises real management ingress on
an isolated PocketIC 16 application subnet through the public extracted library.
It qualifies this small fixture's same-release, same-ID recovery path. The driver
lives in `tests/`; it is not an installed transport, workflow runner or operator
command. Canic remains a separate, unchanged consumer.

The fixture stores an independently observable value in heap memory, an exported
mutable Wasm global and stable memory, sets certified data, and retains a nonempty
Wasm chunk store. It has no outbound-call imports, timers, heartbeat, low-memory
hook or external obligations. Its stopped/drained and no-irreversible-effects
admission applies only to that inspected fixture. Arbitrary applications need
their own consistency and restore-safety qualification.

The driver checks actual status and the complete controller projection before
lifecycle effects. It stops the target, retains the complete original snapshot
inventory, captures once, and reads actual metadata and every module/memory/chunk
extent through production bounded request/reply codecs. The existing artifact
writer enforces coverage and durably publishes checksummed original bytes. The
existing journal and immutable local manifest are reopened under the original
plan. The source-bound upload preparers freshly verify the retained tree before
encoding allocation and each data write.

A distinct allocated snapshot receives every original extent. Fresh destination
metadata and complete readback must match region sizes, every byte, ordered
global encodings, certified data, chunk identities and timer/hook metadata. The
application changes all three observable values and certified data from 42 to 99,
and clears its current chunk store before loading that uploaded snapshot into the
same existing ID. Complete stopped-state verification then allocates a distinct
fresh snapshot with its own retained plan/reservation and checks every region/chunk
byte and global/certified/timer/hook metadata against the original source. This
separately accounted verification retains the additional remote snapshot; it never
replaces a source or repeats load. Only afterward does explicit start proceed, and
all three application values must return 42. Active timer/hook execution, SIMD or floating globals, multiple
canisters, external-effect fencing and other subnet configurations are unqualified.

## Original spending and deliberately lost replies

Each primary management ingress has a retained immutable single-operation plan
with a unique operation sequence and original one-mutation/one-observation budget.
The fixture durably reserves before invoking the client. Read methods also have
explicitly accounted ingress; a recovery read consumes the original pending
mutation's separate observation budget. These per-call test plans do not implement
a full executable product workflow or authorize replenishment of an existing plan.
The driver admits at most 64 management ingresses and reads regions in at most
32 KiB extents. Fixture creation, installation, cycle funding, chunk seeding/clearing/listing and
application writes/queries are explicit simulator setup/assertions outside these
backup/restore journals. Reported management counts cover accounted backup,
restore, verification and recovery ingress; these fixture operations are separate.

Registered cases run the same complete journey normally and with capture,
metadata-allocation, first data-write, stop, load or final start reply deliberately
discarded after real completed ingress. Discarded mutation replies are retained
separately as test oracles; reconciliation never reads them. Reopen must retain the pending mutation and
exhausted mutation allowance; attempting another mutation returns the existing
typed pending error. An exact observation is retained and reserved before dispatch.
The existing passive settlement checks admit the integration's exclusive-instance
original-request attribution, then the existing journal owner explicitly records
its receipt. Complete isolated ingress history and absence of competing writers
qualify attribution in this fixture; inventory cardinality or byte equality alone
do not. Load attribution additionally binds the complete independently checked
stopped restored state before its existing receipt is recorded. Exact intended
stop/start/load ingress counts exclude compensating repeats. Recovery never repeats
the original effect or replenishes spending.

A negative journey also discards the originally reserved load-status reply. No
successful response or settlement claim is constructed, and no receipt, verification
capture or restart follows. The test oracle alone decodes that discarded status to
assert the actual isolated canister remains stopped; those bytes never enter
settlement. Both original reservations and exhausted allowances survive exact local
reopen. Another mutation or observation returns the existing typed pending denial
without changing journal bytes or making a call. All other original journals,
source artifacts/manifest and reference bytes are replayed unchanged as well.

This is post-effect reply-discard qualification, not a network-fault injector or
process-crash test. PocketIC's client submits ingress and awaits that original
message; its simulator HTTP busy handling and result polling are distinct from
another management ingress. The explicit client request limit is 60 seconds.
The ordinary local journal/manifest replay invokes no management calls and checks
unchanged originals, consumed allowances and retained restore-reference bytes.
It is not full product terminal replay or reference-release admission.

## Reproduce and retain evidence

Prepare the reviewed local tools and locked Cargo cache explicitly before running:

```bash
make install-tools
make deps
cargo test --offline --locked -p ic-backup --test pocketic_snapshot -- --nocapture
```

The test requires the explicitly selected `.tools/ic/bin/pocket-ic` binary,
checks its installed checksum and exact server version, and never downloads or
silently substitutes a server. CI already prepares that reviewed tool set before
running the native package tests. Missing tools reject rather than skip cases.
The server stays local; no public IC network is contacted.

Fresh fixture directories remain retained beneath the selected temporary parent
on success and failure. Each records the actual server checksum, Wasm bytes,
original plans, reserved journals, arguments/replies, management trace, durable
source artifacts/manifest/references, stopped post-load verification or retained
pending-lifecycle evidence, and a successful qualification result with instance,
target, raw source/destination IDs, elapsed time and accounted ingress count.
The negative case's successful test result qualifies safe stopping with pending
evidence, not a completed restore.
Simulator instance disposal does not remove those host-side records.

[The machine review](pocketic-qualification.json) records source/dependency identity,
registered case names, the reviewed backend and exact scope. Linux results do not
supply native macOS qualification. Follow-up delivery and backend work belongs to
[issue #17](https://github.com/dragginzgame/ic-backup/issues/17); production ICP
transport, executable workflows/CLI, Canic adaptation, application-specific
lifecycle recovery, fencing and terminal/reference-release qualification remain
separate. The inspected
[management snapshot contract](https://docs.internetcomputer.org/references/ic-interface-spec/management-canister/)
supplies platform shapes;
its metadata or status alone never establishes original load attribution.
