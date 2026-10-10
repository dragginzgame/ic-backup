# Direct Agent transport

The maintained product transport is `crates/ic-backup-agent`, using registry
`ic-agent` 0.49.2. The maintainer selected this hard cut on 2026-10-08; the complete
transport was first released in 0.8.0. Consumers sharing the core
`PersistenceError::Publication` must align with its selected Host 0.12 Rust error
identity. The ICP transport/probe is retired; historical evidence and original
journals remain retained. Testkit owns explicit simulator selection/admission
through the [current setup contract](development.md); shared IC executables
provide no product backend.


`ic-backup` owns generic models, policy and local persistence; it has no Agent
runtime dependency. `ic-backup-agent` depends on those original public request
owners. Canic and other applications own fresh membership, permissions, consistency,
release identity, stable source bytes and restore/fence safety. Full runners and
installed Canic adapters remain separate work in
[#29](https://github.com/dragginzgame/ic-backup/issues/29).

## Configuration and admission

Configure `AgentTransport` with exact original `PlanContextRecord`, fixed HTTPS
origin or literal loopback HTTP origin, integration-owned identity, externally
trusted root key and explicit 1..300s HTTP timeout/ingress expiry. URL credentials,
paths, queries and fragments reject. The actual signer must match the original
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
`prepare` rechecks their original current journal and signs canonical receiver,
effective target, method and argument bytes without IO. Hold the original guarded
journal; retain exact `envelope()` and `request_id()` privately and durably with
original intent/reservation before calling the consuming `submit().await`.
An async executor with network/timer support is required. Preparation grants no
fresh effect authority or proof that a reconstructed pending request was never sent.

## Async mutation coordination

`AgentMutationProvider` implements the core async `IcMutationProvider` for exact
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

Upload, recovery and fence ports remain separate synchronous contracts;
their async provider integration and full application runners remain pending. No
compatibility wrapper or default application safety/retention callback is supplied.

## Results and interruption

`UpdateOutcome::Replied` contains the exact original request ID and bounded raw
reply after Agent verifies its certificate against the configured trust root and
effective target. Use canonical method decoding and association. This is passive
reply evidence, never an automatic Applied/NotApplied receipt or application proof.
`UpdateOutcome::Pending` returns the same ingress ID without issuing another call.

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
pending original accounting and denies a repeat reservation. A wrong root rejects an
actual reply while an independent oracle proves the stop applied. Neither path
writes an automatic receipt. Fixtures retain exact original plans/journals, signed
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
