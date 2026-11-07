# Real single-canister snapshot qualification

## Current server ownership cut

Testkit owns authenticated server assets, version admission and managed processes.
`make install-testkit-server` explicitly prepares the locked selection;
`make testkit-server-check` is the standalone CLI/tool gate and prints its admitted
absolute server path. The unique locked Testkit package selects that exact CLI.

Both simulator suites independently call the locked library's
`ic_testkit::pic::server_assets::check(&bundle)` before startup. They do not spawn
Make or the CLI, download assets during admission or substitute another server.
Shared's optional IC executable bundle supplies no PocketIC fallback. Backup owns
fixture topology, private output paths, server digests, original evidence and
product spending/recovery assertions. Current source/graph receipts belong to
[the handoff](status/current.md).

## Current simulator dependency boundary

The suites use the published async `ic_testkit::pocket_ic` client directly as a
development dependency. Production management DTOs remain Platform's qualified
0.11.0 identities. There is no production simulator dependency, Platform simulator
feature, forwarding client or client-owned launcher. Dependency versions belong
to manifests and the lockfile; adoption history remains in
[the changelog](../CHANGELOG.md).

Each fixture supplies the admitted server path, topology, retained state/output
paths and explicit request/startup bounds to Testkit's managed `build_instance`.
Both builders supply `Duration::from_secs(60)` at construction for the required
finite request budget. One deadline covers dispatch, body reads and accepted-operation
polling; the separate startup timeout remains 120 seconds. Testkit rejects invalid
budgets before network activity. Its deadline never grants replay or fresh spending.
The managed owner stays intact through setup, then transfers its client/server
pair into client-before-server field order. Operations run on the caller's async
runtime. Successful fixtures await instance deletion before releasing the server
and retain teardown failures; panic cleanup still stops/reaps the owned server.
No baseline reuse/reset, hidden snapshot retry or funding policy is adopted.

## Original snapshot and recovery journeys

The [Core integration target](../crates/ic-backup/tests/pocketic_snapshot.rs)
qualifies focused capture, transfer, restore and reconciliation through the public
library on an isolated application subnet. Its inspected fixture stores observable
values in heap, an exported mutable Wasm global and stable memory, with certified
data and a nonempty chunk store. It has no outbound-call imports, timers,
heartbeat, low-memory hook or external obligations. Its application safety applies
only to that fixture. Arbitrary applications need their own admission and
restore-safety qualification.

Actual status and complete controller projections precede lifecycle effects.
Production codecs and artifact operations retain exact metadata, extent bytes,
coverage, checksums and source references under the original plans. Focused load
recovery uses the original native snapshot and complete independent stopped-state
verification. The connected Agent workflow below owns the complete uploaded
round trip; Core retains no second complete backup/upload/restore runner.

## Original snapshot transfer reads

Metadata/data reads use the public original-plan/reservation boundary. The
test-only backend submits exact management updates; production response
association checks the pending journal and bounded original reply before an
explicit success receipt. Its private submitted-original set rejects repeated
invocation and supplies no persisted production proof of non-dispatch.

Lost or malformed metadata/data replies stop before successor work. Metadata loss
creates no download staging. First-data loss retains Created partial staging and
its source reference without a manifest or canonical artifact. Later-data loss
retains the completed prefix. Reopen preserves pending attempts and exhausted
allowances; oracle replies never enter association or settlement. These cases
qualify safe stopping. The
[transfer-read contract](contracts/ic-snapshot-transfer-read-port.json) owns the
boundary.

## Original spending and deliberately lost replies

Exact immutable plans and bounded reservations precede each accounted management
ingress. Recovery observations consume separate original authority; neither error
text nor repeated declarations replenish it. Simulator creation, installation,
funding, chunk setup and application writes/queries are explicit parent-owned
setup/assertions outside workflow allowances.

Focused cases discard capture, allocation, upload, stop, load or start replies
after real ingress. They retain the pending mutation and original journals across
reopen and refuse redispatch. Qualified reconciliation uses a separately reserved
observation, exclusive-instance original-request attribution and an explicit
receipt. Load also requires complete independent stopped-state verification.
Inventory cardinality, byte equality and discarded oracle replies alone grant no
settlement authority.

A lost load-status reply preserves both exhausted reservations and unchanged
journals, with no receipt, verification capture or restart. Ordinary local replay
makes no management call. An independent readback oracle requires expected bytes
for every extent before another call; incomplete chunk coverage rejects without
further ingress. Post-effect reply discard qualifies these recovery boundaries;
process death is exercised separately by the connected consumer. Client HTTP busy
handling/result polling is distinct from repeating a management ingress.

## Connected original capture and restore workflows

The [standalone assembly](../crates/ic-backup-agent/examples/standalone/README.md)
is the maintained complete journey. Its
[fixture application adapter](../crates/ic-backup-agent/examples/standalone/tests/pocketic_transport/independent/consumer/authority/mod.rs)
runs in a separate temporary workspace/target with normal library dependencies.
Testkit and independent application checks stay in the parent.

Capture and restore catalogues/layout ancestry freeze before capture. Restore
commits the capture catalogue; learned child plans bind original metadata, source
plan/manifest and allocated destination. Source custody coexists with restore
stage locks. Every dispatch consumes original bounded authority before signed
Agent ingress. Parent setup/verification has separately retained accounting.

After capture the parent changes live state from 42 to 99. Before accepted start,
stopped-state readback compares complete module, heap, stable, global/certified
metadata and chunk bytes with the original. The Agent fixture has an empty chunk
store; Core's focused restore cases also clear a nonempty live store and verify
restoration of the original chunks. These inspected programs do not qualify
another application's obligations, fencing or safety lane.

[Registered journeys](../crates/ic-backup-agent/examples/standalone/tests/pocketic_transport/independent/mod.rs)
cover cold capture/restore/accepted start, effect-free terminal replay, process
death after authenticated load and cold continuation after each restore stage.
Continuations reconstruct original bytes and acceptance while preserving settled
prefixes. [Lost-reply cases](../crates/ic-backup-agent/examples/standalone/tests/pocketic_transport/independent/restore_failures/mod.rs)
cover allocation, the second upload and start; retained reentry refuses successor
work without changing original files or recovering authority from oracle replies.

The [terminal provider](../crates/ic-backup-agent/examples/standalone/workflow/restore/mod.rs)
holds the original workflow lock, verifies source provenance and requires fresh
exact post-load acceptance. [Parent checks](../crates/ic-backup-agent/examples/standalone/tests/pocketic_transport/independent/terminal/mod.rs)
withhold/substitute acceptance before start, reject changed acceptance even on
released replay and preserve unrelated unfinished references. Successful replay
changes no file bytes. No artifact or remote snapshot is deleted.

## Independent authenticated consumer processes

The [normal consumer](agent-transport.md#independent-normal-consumer) retains frozen
catalogues, durable artifacts/manifest, signed ingress, raw replies, references and
separately named failed-process logs. Operator cases add typed inspection, bounded
pause before start and retention/partial-output failures. Opaque application cases
cover mutation/observation, process death, bounds and replay. Registered cases own
membership; prose does not maintain aggregate counts.

Current receipts distinguish candidate-path execution from registry-only
qualification. Installed Canic, native macOS, installed-IC and production
application acceptance remain separate. See [application responsibilities](canic-adoption.md).
Native fakes qualify local persistence/process behavior; these simulator suites
exercise actual management ingress and authenticated Agent transport.

## Reproduce and retain evidence

Prepare tools and the locked Cargo cache explicitly, then select either suite:

```bash
make install-tools
make deps
make install-testkit-server
make testkit-server-check
cargo test --offline --locked -p ic-backup --test pocketic_snapshot -- --nocapture
cargo test --offline --locked -p ic-backup-agent --example standalone pocketic_transport -- --nocapture
```

Missing/changed assets reject rather than skip cases. Library admission stays
offline; the server is local and no public IC network is contacted.

Fixture directories remain beneath the selected temporary parent on success and
failure. Preserve original server identity/output, copied lock and digest, Wasm,
plans, reservations, signed ingress, arguments/replies, traces, artifact bytes,
manifests, source references, stopped-state verification and result records.
Pending cases retain partial staging and original spent authority. Instance
disposal does not remove host evidence. Retain old and failed bundles; successful
safe stopping is separate from completed backup/restore.

## Historical fence-specific qualification

Fence acquisition/reconciliation fixtures retired with the 0.58 API cut. Their
[0.57.1 guide](https://github.com/dragginzgame/ic-backup/blob/v0.57.1/docs/pocketic-qualification.md)
retains the original qualification/recovery behavior. Layouts containing
`consistency-requirement.json` or `fence-obligation.json` refuse before spending.
Preserve them and finish/reconcile with **0.57.1 and the original qualified
integration**; never delete markers, rebuild plans or re-sign to bypass admission.
Current opaque ingress covers signed/reply retention, pending attempts,
cancellation and separately reserved recovery. Canic owns actual checkpoint/fence
semantics; maintained fenced restore retains rejection and source-reference checks.

## Pinned CLI capability probe

The ICP probe is retired. Its
[exact historical review](https://github.com/dragginzgame/ic-backup/blob/b1c1c33f1e9d80c2c22a605ed4b39383b281ede7/docs/pocketic-qualification.md#pinned-cli-capability-probe)
retains the CLI routing refusal, original pending accounting, evidence locations
and upstream/consumer references. It grants no current transport qualification.

## Earlier qualification

The [complete prior guide](https://github.com/dragginzgame/ic-backup/blob/b1c1c33f1e9d80c2c22a605ed4b39383b281ede7/docs/pocketic-qualification.md)
retains Testkit migrations, earlier Host/Metrics graphs, original result counts
and per-run evidence paths. The [historical machine review](pocketic-qualification.json)
keeps its named sources, selections and scope. Neither is current qualification;
[the handoff](status/current.md) owns current receipts. Original journals, recovery
artifacts and [historical evidence](reports/README.md) remain preserved.
