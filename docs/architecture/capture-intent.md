# Implemented capture intent and continuation

The current public capture is one fixed direct-host operation. Construct a
`BackupConnection` from `BackupHttpTransport`, `BackupSigner` and
`BackupApplicationGuard`. Its `plan(request, options)` binds the actual
connection before approval. `create_capture(plan, digest)` persists intent;
`open_capture(id, digest)` and `inspect()` authenticate original progress.
`advance(connection)` performs the next fixed work, at most one new management
submission. It returns `Advanced`, `NeedsAttention(step)` or `Complete(receipt)`.
Typed `OperationStoreError`, `OperationAdvanceError` and `BackupHttpError` errors
retain authentication, policy and custody causes.

This replaces the phase APIs and their persisted encodings. The store marker is
`ic-backup/host-store/1\n`. Older/unknown stores refuse without writes; no dual
reader, reset, migration or orphan adoption exists. Retained older operations
remain under their preserved implementation and original authority.

## Exact declaration

`CaptureRequest` binds a 32-character lowercase hex operation ID, exact network/
canister, trust-root digest, initiator, release, module hash, direct sender and
operation-wide `OperationLimits`. Payload/evidence limits are 1..=1 GiB; extent is 1..=1 MiB
and no larger than payload; network/signing are 1..=8192 and observations cannot
exceed network allowance. These structural limits do not guarantee affordability.

`OperationOptions` binds the original application witness, snapshot capacity
1..=1024, signing/submission durations, certificate/delegation/skew policy,
signing-service network cost and complete request/result ceilings. The profile
also commits actual transport, signer/guard protocols and original root DER.
Signing duration is positive and no longer than submission duration; submission
is at most 300 seconds. Request/result ceilings are at most 2/4 MiB respectively. Certificate admission
is capped at 2 MiB (delegation at 1 MiB), leaving protocol headroom for the 1 MiB
data vector. Complete retained evidence includes both certificate and projected
reply. Existing intents keep their original smaller request/result ceilings;
reopening never enlarges their approval or reservation.

The strict canonical JSON domain is `ic-backup/capture-intent/1`; approval is
`SHA256("ic-backup/capture-intent/1\0" || original_intent_bytes)`.
Unknown/duplicate fields, noncanonical bytes or identities and invalid roots/
arithmetic refuse. An independent literal wire/digest vector tests this contract.
Reopen cannot replace trust, timing, profile or allowance.

## Files and ordering

```text
operations/<id>/
  lock
  intent.json
  journal.bin
  <derived-step>-<derived-evidence-role>.bin
```

The private recorder owns one common lifecycle: CallReserved, CallPrepared,
CallSubmitted, CallResult, and corresponding Observation facts. MaterializationStarted
and PublishReady are capture-local transitions; restore has one LoadedStopped footer. The frame JSON is
`{"sequence":N,"fact":{"kind":...}}`; bounded little-endian length/payload/SHA-256
chaining starts from `ic-backup/operation-journal/1\0` plus original intent.
Bounds remain 16 KiB/frame, 65,536 frames, 128 MiB/journal and the original smaller
evidence ceiling. Evidence filenames derive from fixed steps and roles; references
bind exact lengths/hashes. Signed envelopes are original bytes. Request IDs,
absolute expiry and historical windows derive from those bytes and retained
invocation time; no second persisted policy/schedule/digest is added.

Reservation accounts for the complete signing/network/request/result/frame ceiling
before invoking an external signer. Network allowance includes signing-service
cost. Whole known remaining costs are admitted before take and first data read.
No result refunds allowance. Capture and restore share the limits schema and
validator; every intent still binds its own original values and journal spending. Prepared replay can submit only its exact original
bytes under unexpired authority; reserved signing with missing output and already
reserved submission return attention without another invocation.

Exact original files, identities, permissions, single links, closed directory
membership and target ownership are rechecked around awaits and writes. Appends
disable the handle before writing and sync before granting the live continuation.
Uncertain write/sync/custody failure leaves the handle unusable. Complete uncertain
frames stay spent; partial tails, replaced files and unreferenced evidence refuse
intact. No truncation, cleanup or automatic retry is performed.

The writer takes its caller's already-checked prefix and rechecks exact custody
before writing. Closing admission must match the expected new length/head and
returns that checked journal; callers do not immediately repeat the same scan.
After awaits, original bytes are still checked independently. A live invocation
extends its authenticated procedure with the newly retained response; the HTTP
boundary authenticates its owned exact request/certificate once, and local reply
import authenticates its supplied evidence before retention. Certified rejection
and reply policy have one procedure owner. No cross-invocation or persistent
authentication cache is added.

## Status preconditions

The private bounded Platform-backed reader requires stopped state, the original
module hash and actual direct sender in a duplicate-free controller set of at most
ten. Initial and final status use distinct signed requests. An initiator or other
controller cannot substitute for the management sender.

Status raw input is at most 1 MiB; header 64 KiB; type entries 64; codec work and
allocation 2 MiB each; skipped-work allowance 64 KiB. Required fields/types and
single-argument/trailing-byte checks are independent of signature/certificate
authentication. Resource statistics are skipped, never synthesized into a full DTO.

## Direct-key status request authentication

Platform owns standard Ed25519/secp256k1 verification, IC domains, Principal
identities and certificate primitives. Backup checks original sender, exact derived
method/arguments, request ID, expiry and root/target/time bindings. Delegated,
canister and unsupported identities refuse. Historical replay uses original
certificate and delegation windows, independently of current time and permission.

Host HTTP supplies the client/body reader. Backup disables redirects, environment
proxies and retries, sends once, bounds status/body/deadline and retains exact
terminal certificates before applying capture policy. Disconnect, acceptance,
timeout, cancellation, oversized body and invalid proof stay uncertain.
No polling loop or fallback transport exists. After bounded extraction, a
certificate survives authentication, closing-deadline, source/custody and retention
failures in a typed error. `received_certificate()` exposes its exact bytes and
original update request ID; Debug omits both. These bytes are sensitive and may be
unauthenticated. They grant neither success nor permission to repair changed files.

## Local settlement from surviving replies

`reconcile_reply(certificate)` verifies one original terminal certificate locally
and retains its result under the original pending call. It grants no new signing,
network or observation allowance. Duplicate, wrong-request/trust/time and
nonterminal evidence refuse. Certified rejection and policy-invalid replies remain
settled and block dependent work; they cannot be replaced by a later success.

## Durable single lookup

`reconcile_observation(connection)` explicitly reserves one read-state
lookup for an original pending call, within the operation observation allowance
and separately retained observation time policy.
The signed request contains the exact original sender/request ID and single
request-status path. Its request, submission and certificate remain in the same
journal with separate spending. There is no automatic or repeated lookup of that call.

Terminal retained observations resolve the original update during semantic replay,
without a second result file or settlement event. Nonterminal or interrupted lookup
stays spent and pending. A surviving independent original reply may still resolve
the update. Dependent calls require fresh live action admission even after local
settlement.

`reconcile_observation_reply(certificate)` imports a surviving lookup certificate
locally only when that original lookup was already submitted and has no retained
result. It authenticates the original lookup request and its separate retained
time policy, never the update window or current clock. Nonterminal observations
stay pending; terminal observations settle by the same replay rule. Both local
import methods require intact original custody; an orphan/partial tail still
refuses. There is no new journal, automatic adoption, retry or allowance.

## Final capture status and completion

The [transfer contract](capture-transfer.md) owns exact payload coverage and
materialization. A separate final status follows complete payload verification.
PublishReady freezes the original host timestamp and closed journal hash.
Create-only publication reconciles only exact complete original bytes.
Published replay and `verify_completion(manifest_hash)` authenticate full original
evidence and payload without sync, guard, clock, signing, HTTP or writes.

A completed receipt may survive target succession. Structural custody checks
require its exact published manifest to bind original intent, closed journal and
actual closed payload tree; staged or missing receipts cannot excuse changed
ownership. Public reopen/source/advance boundaries still authenticate the complete
procedure. Held restore custody additionally pins its original trusted manifest and
payload descriptors. This is no persistent authentication cache.

`create_capture_after(plan, approval, predecessor)` locks target then predecessor,
authenticates/synchronizes its exact published receipt, then atomically changes
ownership. `open_capture_source(operation, receipt, limit)` authenticates the exact
same-store source and holds its capture lock plus bounded payload descriptors.
Neither source admission nor restore create/open needs a signer/connection.
The predecessor selector is a closed `OperationPlan::Capture/Restore` view; restore
completion and its source dependencies are authenticated before succession.
Unfinished operations still require their original owner.

## Focused legacy review — 2026-11-15

Reviewed frozen `4ae087fad8c23de712aa60bf569bde4b2b932651`: Agent
`submission/mod.rs`, `tests/http_boundary/{mod.rs,recovery/mod.rs}`,
`ops/persistence/attempt_journal/{mod.rs,tests/mod.rs}`, snapshot coverage and
artifact completion tests. History `d245b0f03a505ba5215d65c10bdebf1890c7dc37`
binds retained results to the original request; `a860d14421b9ff1c61c1dce079cdc6a19316bfd0`
separates observation from dependent-effect admission;
`9f7b886a824eec67dda8f81792a560436419bb2a` preserves original pass ownership.
Their no-retry, original-evidence and independent-authority guarantees remain.

The new common recorder replaces phase catalogs, repeated lifecycle events and
accepted/schedule/assembly digest records. Tests now live with
`operation/{tests,procedure_tests}.rs`, custody/runtime/facade tests and the
retained request/certificate/HTTP groups under `advance/tests/`.
Repeated per-phase mechanics are covered once; phase policy stays in bounded
status/snapshot/transfer/procedure tests. Removed fake-provider route cases supplied
no IC evidence; unsupported Canic configuration is removed rather than qualified.
No legacy source was copied.

The macOS fixture correction re-reviewed frozen `4ae087fa` Agent
`tests/http_boundary/mod.rs` and `tests/http_capture/mod.rs`, plus histories
`0d4671ed9bf7a5e217e06a67e3fba490871daff4` (retention/failure matrix),
`7f6df93bfbf4d21844942dd3f4451df47c1f2460` (cancel only after captured ingress)
and `1af7e219c3073395992c316ecaa5e83e74b1aecc` (bounded Host capture).
Those cases protect original spending, zero-or-one requests and cancellation
without retries. The replacement's fixture reads directly from `TcpStream`;
accepted HTTP/TLS/proxy streams now explicitly select blocking IO before their
existing timeouts, independently of listener-mode inheritance. Existing HTTP,
lookup cancellation and runtime cases retain those guarantees; no production
transport policy or legacy fixture framework is imported. Native macOS execution
remains required separately from local Linux validation.

The limits consolidation additionally reviewed frozen `4ae087fa`
`model/operation_plan/{budget/mod.rs,tests/mod.rs}`: changed original budgets,
overflow and assigned-allowance refusal. The same reviewed `a860d144` ownership
history retains independent original operation authority. The new library shares
only schema/range validation, not spending; it keeps its selected nonzero network/
signing and payload/evidence bounds rather than legacy's graph-wide allocation or
zero-attempt declaration rules. Existing capture/restore plan/golden/refusal tests
exercise both callers without changing original bytes.

Native tests cover partial/complete uncertain frames, orphan/changed files,
finite spending, cancellation, exact prepared replay, certified rejection,
materialization/publication interruption, completed succession and original source
custody. An acknowledged child is killed after durable signing reservation; cold
reopen preserves intent, target claim and spending with zero further signing/HTTP.
This process-death case is not physical-power-loss qualification.

The public-API [IC fixture](../../qualification/ic-backup-qualification/tests/capture.rs)
covers multi-extent module/heap/stable payloads, empty/nonempty chunks, exact
publication, succession, an older-source complete restore, interrupted take
observation, original payload-reply settlement after shutdown and offline replay.
It also qualifies a full 1 MiB whole chunk and a load result whose local write
fails: the returned error supplies the certificate for offline settlement, without
a proxy recovery copy. The separate normal-dependency
[consumer executable](../../qualification/consumer/src/main.rs) is built alone
without Testkit/PocketIC in its graph and continues capture/restore in fresh
processes, then replays completed operations after server shutdown. Fixture
signing/authority does not qualify Canic process or maintenance adapters.

The post-reply repair re-reviewed frozen legacy
`workflow/qualified_receipt::retain_reply` and the restore
`failure_boundaries_retain_pending_or_applied_and_never_repeat_provider` regression,
plus history `f332118eeed000e4615e48155c162a4e113076a8` and
`36789d26768b7b2eac2f5d4e8c29cc11bf80bdde`. Its bounded acknowledgement retention
guarantee is adapted to certificates and the original recorder; legacy stages
and receipt storage are not copied. Native late-deadline, old-allowance and failed
write/import cases exercise the new owner.
