# Real single-canister snapshot qualification

## Current server ownership cut

The simulator suites use the exact locked Testkit CLI for explicit
`make install-testkit-server` and offline `make testkit-server-check`. The check
returns the authenticated absolute server path for both simulator suites;
server selection/asset/version admission belongs to Testkit. Caller-owned private
logs, server digests, topology, spending and lost-reply assertions remain local.
Shared 0.2.0's five-tool bundle supplies no PocketIC binary or fallback.
Qualification for this candidate is recorded in the current handoff; older results
below retain their actual sources and graphs.

## Capture-only application fence qualification

The Agent simulator target now uses a separate one-member application fixture
through the public acquisition, observation and explicit settlement coordinators.
Its immutable acquisition stores the exact fence identity, membership revision
and original nonce; it has no release endpoint, membership changes, timers,
outgoing calls or irreversible external work. An actual application rejection
and unchanged counter qualify its write gate. Exclusive fixture operator custody,
actual installed Wasm/root/signer and this bounded program qualify attribution
here; matching fence hashes alone do not supply that proof for an application.

The configured Agent acquisition adapter retains exact signed ingress and its
certified empty reply before the fixture discards the public acknowledgement.
An independent original observation reservation precedes one replicated Testkit
read. Its fixture-owned provider retains exact request identities/challenge and
raw reply bytes privately and durably. Both attempts stay pending until independent
qualification checks the retained request/reply under selected journal exclusion.
Exact stage reopen and Applied settlement consume no new read/acquisition, leave
original spending unchanged and retain the active obligation. A discarded observation
reply stays pending rather than becoming Uncertain; a separately retained late
observation can settle its exact original attempt. Reentry is denied in each lane.
Cancellation during independent qualification releases journal exclusion while
retaining both pending attempts and exact original bytes. Changed retained reply
bytes cause a typed qualification rejection before any receipt. These actual
simulator cases subsequently settle the original observation without another read;
the changed-byte fixture explicitly restores its original retained file and keeps
the changed bytes separately. Production settlement performs no evidence repair.

Focused command:

```bash
cargo test --offline --locked -p ic-backup-agent --test pocketic_transport fence_reconciliation
```

This capture-only fixture qualifies orchestration with actual application effects;
it installs no production observation adapter or generic application permission.
Restore's rewind-independent external obligations, arbitrary application safety,
full backup/restore completion, terminal proof and fence/reference release remain
separate. Original failure logs, raw server output, signed ingress, state/rejections,
requests/replies, journals and obligations remain retained.

## Current simulator dependency boundary

Testkit 0.64 owns managed server startup, authenticated assets and harness
extensions. Simulator clients and DTOs come directly from published
`ic_platform::pocket_ic`; both libraries enable Platform's `pocket-ic` feature only
in development dependencies. Platform remains at the selected 0.6.7 release.
No forwarding module or production simulator dependency is retained. The existing
exact management bytes, bounded calls, signed transport, original journals and
independent readback assertions remain local.

## Earlier Testkit qualification

The following records retain their original dependency selections and scope;
use the current dependency boundary above and the handoff for current evidence.

Both simulator suites depend on published `ic-testkit` 0.25.1 and its complete
`pocket_ic` re-export. Their exact management calls, byte comparisons and original
reservation/lost-reply assertions remain local. Testkit owns explicit bounded
server readiness and cleanup; raw 0600 stdout/stderr survive in caller-owned
fixture roots. No Testkit snapshot retry, baseline reset or funding policy is used.

The single [consumer-owned matrix](shared-tooling.md#consumer-owned-ic-pins)
selects PocketIC 16.1, explicitly installed and admitted before tests. Ordinary
checks do not download servers. All ten core simulator cases and three Agent
gateway cases pass on Linux with the final Host 0.8 / Metrics 0.2.14 graph.
Current logs and fixture state live under `target/testkit-adoption-090`;
[the source-bound review](reports/audits/2026/10/08/testkit-adoption/01/report.md)
keeps preliminary Testkit 0.24 / PocketIC 16.0 results separate. These results
qualify the isolated fixture and transport; full workflow runners, generic
application safety, Canic adoption and native macOS acceptance remain separate.

## Earlier qualification

Current direct Agent gateway qualification is documented in [agent-transport.md](agent-transport.md).
The earlier 11-case results below retain their original source/graph; the maintained
core target now has ten cases after retiring the ICP probe.

A fresh Linux run on the maintainer-selected published Host 0.5.0 graph passes
all 11 registered journeys, including the actual bounded ICP capture/refusal path.
The pending 0.7.0 draft changes the exposed Host publication-error Rust identity;
package/receipt remain 0.6.0. Exact current graph, source, logs and simulator
results are under `target/host-050-review-3`. This run does not qualify Host's
unused process-group API, escaped descendants, production retry/polling or fresh
permissions. Original journals and pending spending retain their existing owners.
No public IC calls or sibling edits occur; native macOS consumer evidence for
this selected graph remains separate.

The earlier Linux run after typed JSON publication adoption passes all 11 registered
journeys on the unchanged host 0.4.6 / Metrics 0.2.8 lock. The 0.6.0 draft's exact
source, simulator results and local archive proof are under `target/publication-060`.
Earlier results below retain their actual source/dependency identities. This
qualifies the isolated fixture and safe pending refusal, not an installed
production transport, arbitrary application or downstream Canic integration.

## Pinned CLI capability probe

Historical qualification: the executable probe is retired by the Agent hard cut.
Its source-bound logs and original journals remain retained; this section records
the earlier result. Current transport evidence is in [the Agent scope](agent-transport.md).

The same integration target also probes the installed ICP CLI 1.6.0 against a
real HTTP gateway on a separate owned application + NNS instance. It verifies
the installed executable checksum/version through published `ic-host-process`
0.4.2, then passes direct argv, a trusted owned root key, anonymous controller
identity and exact binary management arguments. The CLI's portable `ICP_HOME`
override keeps settings/data/cache in one private retained fixture directory;
the environment is cleared. Each stream is bounded to 64 KiB with a 60-second
deadline. The process dependency is Unix test-only.

A dedicated status command succeeds with the exact Running target and controller
set. The generic `stop_canister` update then exits unsuccessfully with an actual
HTTP 400 `canister_not_found`: the pinned generic path has no explicit effective
target distinct from receiver `aaaaa-aa`. Independent post-call status stays
Running. The original plan/arguments and reservation precede dispatch; pending
spending, typed fresh-reservation denial and exact journal bytes survive reopen.
Neither the error nor the status control becomes a NotApplied receipt or refund.
The test asserts structured status, typed exit failure and retained accounting;
it does not freeze diagnostic prose.

Exact upstream 1.6.0 source and real probe evidence are retained under
`target/icp-backend-054-review`; [ICP CLI #811](https://github.com/dfinity/icp-cli/issues/811)
owns explicit online/offline effective routing. [Consumer #25](https://github.com/dragginzgame/ic-backup/issues/25)
owns independent transport acceptance. A future production adapter must refuse
unsupported modes before effects. This test deliberately probes the incapable
mode in an isolated simulator. One subprocess is not one HTTP ingress: internal
retries/polling, descriptor/descendant custody, authenticated production replies
and interrupted calls remain unqualified. No provider or transport package is
created, and no public IC call runs.

## Original snapshot and recovery journeys

A fresh Linux run after the checked directory-framing change passes all 11
registered cases on the unchanged host 0.4.6 / Metrics 0.2.8 lock. Its exact
uncommitted source, logs and retained simulator results are under
`target/continuation-055`; the machine review records this 0.5.5 draft separately
from earlier runs. Package/receipt remain 0.5.4. This supplies neither native
macOS execution nor production transport or Canic qualification.

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

## Original snapshot transfer reads

All ordinary metadata/data reads now use the public original-plan/reservation
request and passive response association boundary. The test-only provider owns
actual isolated instance/caller/target context and retains original ingress history.
It submits one exact update, then the driver validates the current pending journal
and bounded original codec reply before explicitly recording its qualified success.
A private submitted-original set rejects repeated provider invocation; it is not a
persisted production never-dispatched proof.

Two additional real-ingress cases discard a metadata-read or first data-read reply.
Metadata loss stops before creating download staging. Data loss retains Created
partial staging and its source reference without a manifest or canonical artifact.
Both retain the original pending update and exhausted update allowance across
reopen, refuse another mutation reservation, invoke no provider during replay and
stop before upload/load/start. Oracle reply bytes never enter association or a
success receipt. These cases qualify safe stopping, not a completed backup or
permission to repeat a read. See [issue #22](https://github.com/dragginzgame/ic-backup/issues/22)
and [the port contract](contracts/ic-snapshot-transfer-read-port.json).

## Original spending and deliberately lost replies

Each primary management ingress has a retained immutable single-operation plan
with a unique operation sequence and original one-mutation/one-observation budget.
The fixture durably reserves before invoking the client. Read methods also have
explicitly accounted ingress; a recovery read consumes the original pending
mutation's separate observation budget. These per-call test plans do not implement
a full executable product workflow or authorize replenishment of an existing plan.
The driver admits at most 64 management ingresses; the production download
planner selects original bounded data-stage requests. Independent verification
reads use at most 32 KiB extents. Fixture creation, installation, cycle funding,
chunk seeding/clearing/listing and application writes/queries are explicit simulator setup/assertions outside these
backup/restore journals. Reported management counts cover accounted backup,
restore, verification and recovery ingress; these fixture operations are separate.

Focused recovery cases deliberately discard capture, metadata-allocation, first
data-write, stop, load or start replies after real completed ingress. Each case
ends at its own settlement/no-followup boundary. Successful upload-source setup
uses the existing production capture/metadata/download workflow driver. Load
recovery uses the original native snapshot and independently verifies complete
stopped state; connected workflow cases separately qualify uploaded destinations,
full round trips and terminal replay. The focused cases keep no second complete
backup/upload/restore runner. Discarded mutation replies are retained
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
without changing journal bytes or making a call. Other original journals and any
prepared source artifacts/manifest and references are replayed unchanged as well.

This is post-effect reply-discard qualification, not a network-fault injector or
process-crash test. PocketIC's client submits ingress and awaits that original
message; its simulator HTTP busy handling and result polling are distinct from
another management ingress. The explicit client request limit is 60 seconds.
The ordinary local journal/manifest replay invokes no management calls and checks
unchanged originals, consumed allowances and retained restore-reference bytes.
It is not full product terminal replay or reference-release admission.

The independent stopped-state readback oracle requires expected bytes for every
declared extent before issuing another management call. A missing chunk cannot
silently truncate verification through paired iteration. A real-source regression
checks rejection of incomplete expected-byte coverage with no further ingress.

Each Core fixture retains the admitted server digest and the original
`Cargo.lock`. Focused result records obtain the server version from Testkit and
bind the copied lock's SHA-256 for the exact client/dependency selection. There
is no separately maintained client version label or second server selection.

## Connected original capture and restore workflows

The [existing workflow driver](../crates/ic-backup/tests/pic_journey/planned_runner/mod.rs)
now connects capture, metadata/data download and durable manifest publication to
allocation, data upload, same-ID load, full stopped-state verification and start.
Its [round-trip cases](../crates/ic-backup/tests/pic_journey/planned_runner/round_trip/mod.rs)
use the public coordinators and existing actual-ingress upload/restore providers.
No production API, dependency or application safety default is added.

Capture and restore have separate original catalogues and layout ancestry. Freeze
both allocations before capture: capture has 66 mutations/two observations and
restore has 67 mutations/zero observations, including original 64-call data-stage
ceilings. Unassigned headroom supplies no retry. The restore purpose commits the
capture catalogue; learned child plans additionally bind exact original metadata,
source plan/manifest and actual allocated destination. Preserve the captured source
layout guard during restore; ancestor re-admission acquires the restore stages'
separate layout locks. A source held inside the same catalogue's ancestor chain
would contend with those locks.

The fixture deliberately changes live state from 42 to 99 and clears its chunk
store after capture. Those setup operations and fresh verification observations
have independently retained per-call plans; they are outside the workflow-stage
allowances. Loading must restore the original complete module, heap, stable,
global/certified metadata and chunk bytes before start. The inspected one-owner
program has no timers, outbound work or irreversible obligations. Its authority
and acceptance do not establish a production application lane.

Cold continuation discards the driver before allocation, load and start, then
rebuilds only from exact retained request/reply bytes, original plans/bindings,
manifest, allocation and post-load acceptance. Every settled prefix's records and
artifacts remain unchanged. Discarded allocation, second data-write, load and start
replies keep the original mutation pending with no remaining allowance. Fresh and
retained reentry invoke no new provider or successor. A denied checkpoint read may
allocate its empty mutex sidecar; it changes no product record or artifact.

Completed replay discards all learned driver state, reopens both exact catalogues
and invokes no callbacks or management calls. All retained file bytes stay unchanged.
The [fixture terminal provider](../crates/ic-backup/tests/pic_journey/planned_runner/round_trip/terminal/mod.rs)
extends this journey through guarded source-reference release. After cold evidence
reconstruction, it replays the complete restore under the original held journal
lock, verifies the source artifact/manifest and binds stopped post-load acceptance
to the source metadata, actual upload destination and exact load/start bytes.
Withholding or substituting application acceptance refuses even when every stage
is Applied. Lost allocation, data, load and start replies retain the reference and
all original bytes. Success changes only the reference record; immediate release
replay changes no files and makes no remote calls. An independent unfinished
restore's reference remains present, continuing to block pruning. Even already-released replay
must freshly qualify acceptance. No artifact or remote snapshot is deleted.

This in-process fixture creates no external backup commands; its complete command
set is empty. The inspected program has no timers, outbound work, payments or
application fence. Native process tests separately cover active descendants and
retained command identities. These facts do not install a production terminal
provider or qualify another application's obligations or fencing.

Each case retains `fixture-connected-result.json` alongside its original catalogue
and evidence. Canic integration, independent installed-consumer acceptance and
native macOS/installed-IC qualification remain separate.

## Independent authenticated consumer processes

The Agent suite also runs a [normal Core/Agent consumer](agent-transport.md#independent-normal-consumer)
from a separate temporary workspace and target directory. Testkit and the actual
application checks remain parent-owned. The two registered cases cover cold
capture/restoration/accepted start/terminal replay and process death after an
authenticated load with safe refusal of reentry. Both retain the original frozen
catalogues, durable artifacts/manifest, signed ingress, raw replies and source
references; failed process logs are never overwritten. This is candidate-path
library qualification, not a published-product or Canic acceptance receipt.
The [current handoff](status/current.md#canic-adoption-preparation) records the
separate release-bound registry-only follow-up and its exact harness adaptation.
Published-library qualification still leaves production Canic acceptance open;
the [adoption guide](canic-adoption.md) identifies those application responsibilities.

## Reproduce and retain evidence

Prepare the reviewed local tools and locked Cargo cache explicitly before running:

```bash
make install-tools
make deps
make install-testkit-server
make testkit-server-check
cargo test --offline --locked -p ic-backup --test pocketic_snapshot -- --nocapture
```

The test requires the explicitly prepared locked Testkit CLI and its admitted
server. Its offline check runs before startup and never downloads or silently
substitutes a server. CI prepares that selection before native package tests.
Missing/changed tools reject rather than skip cases.
The server stays local; no public IC network is contacted.

Fresh fixture directories remain retained beneath the selected temporary parent
on success and failure. Each records the actual server checksum, Wasm bytes,
original plans, reserved journals, arguments/replies, management trace, durable
source artifacts/manifest/references, stopped post-load verification or retained
pending-lifecycle evidence, and a successful qualification result with instance,
target, raw source/destination IDs, elapsed time and accounted ingress count.
Transfer-read loss retains its original pending journal and any partial Created
artifact bytes. Successful tests with pending evidence qualify safe stopping,
not a completed backup or restore.
Simulator instance disposal does not remove those host-side records.

[The historical machine review](pocketic-qualification.json) retains its named
source/dependency identities, registered cases, backend and original scope.
Current focused receipts belong to the [handoff](status/current.md) and its retained
per-command/fixture evidence. Linux results do not
supply native macOS qualification. The original journey was delivered under
[issue #17](https://github.com/dragginzgame/ic-backup/issues/17); current transfer-read
adoption is tracked in [issue #22](https://github.com/dragginzgame/ic-backup/issues/22).
Application-qualified Agent integration, executable workflows/CLI, Canic adaptation, application-specific
lifecycle recovery, fencing and production terminal/reference-release qualification remain
separate. The inspected
[management snapshot contract](https://docs.internetcomputer.org/references/ic-interface-spec/management-canister/)
supplies platform shapes;
its metadata or status alone never establishes original load attribution.
