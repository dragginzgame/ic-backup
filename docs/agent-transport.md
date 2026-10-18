# Direct Agent transport

The maintained product transport is `crates/ic-backup-agent`, using registry
Platform 0.1.8’s native `agent` feature over the unchanged `ic-agent` 0.49.2
identity. Reqwest is an explicit consumer dependency for the bounded HTTP client.
All SDK imports, including `CallResponse` and `AnonymousIdentity`, use Platform;
there is no direct SDK declaration. The maintainer selected the transport hard cut on 2026-10-08; the complete
transport was first released in 0.8.0. The core 0.28 hard cut removes the local
`PublicationError` mirror: `PersistenceError::Publication` directly carries
`ic_host_fs::durable::NamedWriteError<std::io::Error>`. Consumers matching that
variant use Host's enum and align direct Host FS with the
[workspace selection](../Cargo.toml). The ICP transport/probe is retired;
historical evidence and original journals remain retained. Testkit owns explicit simulator selection/admission
through the [current setup contract](development.md); shared IC executables
provide no product backend.


`ic-backup` owns generic models, policy and local persistence; it has no Agent
runtime dependency. `ic-backup-agent` depends on those original public request
owners. Canic and other applications own fresh membership, permissions, consistency,
release identity, stable source bytes and restore/fence safety. Full runners and
installed Canic adapters remain separate work in
[#29](https://github.com/dragginzgame/ic-backup/issues/29).

Mutation and transfer-read adapters construct the shared
`ic_backup::model::ic_update_response::IcUpdateResponseInput`. The previous input
names are removed in the 0.29 hard cut. Response wrappers keep their distinct byte
ceilings and errors; transport reply limits and evidence domains remain unchanged.

## Configuration and admission

Configure `AgentTransport` with exact original `PlanContextRecord`, fixed HTTPS
origin or literal loopback HTTP origin, integration-owned identity, externally
trusted root key and explicit 1..300s HTTP timeout/ingress expiry. URL credentials,
paths, queries, fragments and literal whitespace/control characters reject,
including written dot-segment or backslash paths that a URL parser would
otherwise normalize to the root. The actual signer must match the original
caller at construction and signing. Exact network/release context must match;
these declarations do not derive authenticated network or application release
identity from a URL. Integrations own correct out-of-band trust provisioning.

The client uses HTTP/1 and disables proxies, redirects, Reqwest retries and
Agent TCP retries. Explicit middleware bypasses Agent's default 429/503 retry
wrapper. No automatic root-key fetch, query substitution, request-status wait,
polling or capability-discovery call exists. Each invoked HTTP request has a
finite deadline and 4 MiB body ceiling. Signed envelopes are at most 3 MiB;
method replies reuse the existing core limits (metadata/status/list/lifecycle
1 MiB, data 2 MiB, upload 4 KiB). Opaque application fence replies have a separate
1 MiB transport ceiling. A per-request timeout is not a complete workflow deadline
or a hard cycle cap. Trusted signing providers own their own resource/custody limits.

`ReservedUpdate` accepts exact original mutation, recovery observation, transfer
read, upload, allocation observation, data readback or fence-acquisition requests.
The configured provider rechecks the original current journal and signs canonical
receiver, effective target, method and argument bytes. Hold the original guarded
journal across its await. The mandatory retention callback receives a passive
`PreparedUpdate` view; retain exact `envelope()` and `request_id()` privately and
durably with original intent/reservation before returning success. Only then does
the provider submit once. Signing and dispatch are internal; there is no public
prepare/submit or envelope reissue API. An async network/timer executor is required.
Reconstructing a pending request proves neither fresh effect authority nor that
its original was never sent.

## Async mutation coordination

`TransportError` describes transport construction only: configuration, context
and identity failures. The former runtime variants are removed without aliases
for 0.30. Provider errors own pre-dispatch refusal and indeterminate outcomes;
private preparation no longer constructs an intermediate transport error.
Signed-ingress retention still precedes the sole network submission.

`AgentUpdateProvider` implements the core async `IcMutationProvider` for exact
capture/stop/load/start mutations. Configure it with `AgentTransport` and a mandatory
fallible callback that durably retains the exact prepared envelope and request ID
with original plan/attempt and exclusive never-dispatched custody. It rechecks the
currently guarded journal, signs, runs retention, then consumes one submission.
Retention failure performs no HTTP call. Accepted responses and submission errors
return `Indeterminate`; neither creates a receipt or an automatic observation.

Await `capture_snapshot` and `restore_snapshot`; their admission callbacks are async,
as is restore's independent qualification callback. The original selected journal
stays locked across awaits. Cancellation releases the guard while preserving its
current pending/Applied state; reopening cannot invoke those callbacks/providers
again. Restore qualification must durably retain the returned reply before awaiting
cancellable work. The core imposes no executor or `Send` bound and has no runtime
dependency. The configured Agent needs a network/timer executor.

`AgentSnapshotTransferReadProvider` implements the async metadata/data-read port
with the same mandatory durable signed-envelope/request-ID retainer. It rechecks
the currently guarded journal before one exact update. Await `read_snapshot`,
`read_snapshot_metadata` and `download_snapshot`; admission and independent
metadata/data qualification callbacks are async. Qualifiers retain each exact reply
before awaiting cancellable work. Dropping a download preserves partial files,
earlier Applied receipts and its current pending reservation. There is no partial
resume or implicit repeat call. Real Agent simulator reads exercise this adapter
and the single-read coordinator; the Testkit fixture separately exercises the
metadata/download loop. Neither supplies a default application qualification lane.

`AgentUpdateProvider` implements source-bound metadata/data uploads with
the same mandatory signed-ingress retention and current guarded record admission.
Await `upload_snapshot`, `allocate_snapshot` and `upload_snapshot_data`; fresh
admission and independent qualification are async. Integrations qualify authentic
complete source, exclusive destination attribution, current controllers and stable
byte/command custody. Retain exact replies before cancellable qualification work.
Cancellation preserves source evidence, earlier receipts and pending spending;
neither allocation nor a partially settled data stage can reenter dispatch. Actual
Agent simulator metadata/data uploads use the adapter and single-upload coordinator;
the Testkit fixture separately exercises allocation and complete data-stage loops.

Configure `AgentObservationProvider` for all three separately accounted recovery
ports. Await `observe`, `observe_upload` or `observe_upload_data` with the currently
guarded original journal. Its mandatory retainer receives the exact reserved request
and signed preparation; durably retain both original attempt identities, envelope and
request ID under exclusive never-dispatched custody before one status/list/readback
update. Retention must reject prior dispatch. Cancellation and lost replies retain
both pending attempts; certificate-verified projections do not settle them.
Configure `AgentFenceAcquisitionProvider` with mandatory ingress and reply retainers.
Await `acquire_fence` with the current guarded original journal. Retain signed bytes
and request ID before one original opaque application update, and retain its exact
bounded raw reply before returning passive acknowledgement. Failed reply retention
is Indeterminate because dispatch may already have applied. Application codecs,
permissions, whole-unit purpose/revisions, outside-snapshot restore custody and
exclusive attribution remain integration-owned. Await `observe_fence_acquisition`
with the original guarded record for independently reserved application reconciliation;
no generic application read method or reconciliation provider is supplied. Full
application runners remain pending; no default safety/retention callback exists.
Use the core `workflow::fence_acquisition::acquire_fence` step to open/reserve the
prepared stage and retain journal exclusion across fresh application admission
and this adapter. Purpose-specific retained requirements/obligations are rechecked
around dispatch. Its passive success stays pending; post-reply errors retain the
acknowledgement. The isolated actual counter fixture exercises this orchestration
without establishing an Active application fence.

## Results and interruption

Provider success carries bounded raw reply bytes after Agent verifies the
certificate against the configured trust root and effective target. The original
request ID is retained before dispatch and bound into passive reply evidence.
Canonical method decoding and association remain separate; a reply grants no
Applied/NotApplied receipt or application proof. Accepted/processing responses
return Indeterminate without polling or another call, leaving spending pending.
The single submission owner directly handles Agent's response and performs reply
retention before returning. It does not introduce another submission result enum.

Every error, timeout, cancellation or lost reply leaves original spending pending.
Even authenticated rejection is conservatively returned as indeterminate rather
than manufacturing a journal outcome. Reconcile through independently qualified,
already reserved observation requests; do not repeat original updates or fetch
status implicitly. There is no prepared-update Clone or signed-envelope re-import/
reissue API. Reconstructing the original request still grants no repeat-call custody.
All source references and fence obligations remain with their existing owners.

## Qualification

Real local HTTP operations verify signed-body equality, effective-target POST path,
no network during preparation, one request for accepted/429/503/redirect/malformed/
oversized/disconnected/timed-out responses, no implicit polling and byte-identical
pending journals across reopen. Context and reservation mismatches reject before
network access. These controlled servers prove HTTP behavior, not management effects.

The selected published Testkit 0.33 manages explicit PocketIC 16.1 startup/cleanup and retains
server stdout/stderr in each fixture root. The async client uses its complete
PocketIC re-export; exact call accounting remains in Backup. Actual isolated
PocketIC HTTP gateways qualify the production transport's separate
management receiver/effective target and certificate verification. A no-external-
effects fixture captures and reads exact metadata, transfers every Wasm/1 MiB heap/
1 MiB stable byte, uploads metadata/data, loads the same existing ID, and restores
heap/stable/global/certified state through an actual Ed25519 controller. A discarded actual stop reply halts with unchanged
pending original accounting and denies a repeat reservation. A distinct actual
recovery trace reserves one original status observation after that discarded stop
reply, awaits the Agent recovery port, decodes Stopped and retains both attempts
pending without another stop or automatic receipt. A wrong root rejects an
actual reply while an independent oracle proves the stop applied. Neither path
writes an automatic receipt. Actual allocation-list and full 1 MiB Wasm readback
also use the configured recovery provider: both original attempts remain pending,
exact destination bytes match and retained ingress refuses repeat dispatch. The
readback wire exceeds 1 MiB and exercises the distinct 2 MiB data-response bound.
This isolated fixture does not establish generic allocation attribution or settlement.
Fixtures retain exact original plans/journals, signed
requests/IDs, raw replies and simulator state. Current Testkit evidence is under
`target/testkit-adoption-090`; the original released transport evidence stays under
`target/agent-hard-cut-080`.
[The current review](reports/audits/2026/10/08/testkit-adoption/01/report.md)
records the final graph, 158 focused cases and tooling correction.

[The source-bound review](reports/audits/2026/10/08/agent-transport/01/report.md)
records dependencies, commands, results and failed/corrected attempts. Linux
qualification does not supply current native macOS proof; configured CI selects
both libraries on Linux, Intel macOS and Apple Silicon macOS. The complete runner,
authenticated application providers, production identities/endpoints, exclusive
never-dispatched custody, complete installed product and terminal/fence/reference
release remain independently qualified. [Transport acceptance #25](https://github.com/dragginzgame/ic-backup/issues/25)
owns that boundary; historical ICP routing acceptance no longer blocks this transport.

Agent 0.31 uses one `AgentUpdateProvider` for mutation and source-bound upload
ports. Replace both former provider names and change retention callbacks to
accept `&ReservedUpdate<'_>`. Match `Mutation` or `Upload` when accessing the typed
request; the callback receives the exact borrowed request selected for dispatch.
Both ports preserve their own admission, reply limits and evidence domains.
