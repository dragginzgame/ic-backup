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

Each admitted journal call owns its four possible evidence references. Readers
select that call's reference directly and find the original held descriptor by
canonical name; they still re-read and hash every selected file. Closed-directory
admission reads every journal reference, requires the held descriptor count to
match and rejects names outside that exact set and the three fixed operation
files. Journal chronology forbids duplicate steps or repeated evidence roles
within a call. The descriptor map
replaces the held-file list; it is neither persisted nor an authentication cache.

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

On 2026-11-17, the full migration probe against published Protocol 0.3.0
(source `efdc02936dde0319ef32825a499fa28db20cb1ff`, archive SHA-256
`86ff8ba66bf3212d37cc5d5120f4cae5f1e968cb5c98c9d92bdfa980d53cc628`)
found that its new required status `version` field narrows this projection.
A minimal stopped/controller/hash reply and all three existing status goldens
decode under Protocol 0.2.8 but refuse with `Type` under 0.3.0. Five independent
snapshot/metadata controls still decode. The existing native test
`full_supplier_dto_and_minimal_projection_apply_the_same_original_policy` passes
against Backup's selected reader. No migration or actual-IC result is claimed.

Focused legacy review at `4ae087fad8c23de712aa60bf569bde4b2b932651` revisited
`model/ic_lifecycle_reply/wire/mod.rs`, its minimal-projection and bounded-skipping
tests, and codec migration `1af7e219c3073395992c316ecaa5e83e74b1aecc`. Legacy
projects stopped state/controllers; Backup additionally owns original module-hash
admission. Neither needs a canister-version precondition. Keep that distinction
from upgrade-proof consumers, which may require version; do not synthesize values
or weaken their admission to fit Backup. A supported bounded projection and the
still-missing response-fixture encoders are requested in
[Platform #11](https://github.com/dragginzgame/ic-platform/issues/11#issuecomment-6932128252).
Probe source, locks, checked archives and logs remain under
`target/evidence/platform-full-migration/`. No supplier source was copied.

Protocol 0.3.2 subsequently supplied the version-independent capture reader and
response fixtures (source `80a5b488666629dd28152f6bcb6d489091d55994`). A production
reader port passed 13/15 owning status tests, including all three frozen status
goldens, but rejected known option promotion of a bare 32-byte module-hash vector
in `hash_option_promotion_and_absence_never_invent_the_approved_hash`. Preserve
that acceptance with the exact original module check; do not normalize retained
wire bytes or silently change the test. The additional oversized-hash
Limit/Malformed mismatch is only a forwarded error-class change. That initial port was
removed and the original reader's 15 tests passed.
[Platform #11](https://github.com/dragginzgame/ic-platform/issues/11#issuecomment-6944243819)
records that supplier projection defect; `target/evidence/platform-next/`
retains the attempted patch and failing output.

Protocol 0.3.3 (source `64897c9c522ab6730620985b6d58607d9c593903`)
fixes known optional-byte promotion. The initial production port passed all 15 status
regressions, including minimal replies, frozen goldens, hostile bounds and absent
hash policy. Platform owns traversal; Backup retains its exact status/controller/
hash policy. The completed port uses Types 0.21 identities throughout, removing
the temporary controller identity conversion. Oversized hashes now forward `Limit`; short or empty
hashes fail module policy. These public error changes are pending 0.105.0.

Production ingress parsing and direct-key verification now use Platform 0.34.11.
Whole-envelope admission rejects malformed authentication fields earlier; original
bytes and request IDs remain retained unchanged. One envelope projection supplies
lookup binding and signature admission, removing the old dual-projection check.
Focused legacy review at `4ae087fad8c23de712aa60bf569bde4b2b932651` covered
`ic-backup-agent/src/submission/mod.rs` and history
`1af7e219c3073395992c316ecaa5e83e74b1aecc` /
`278016b369f4312b4311410bbb7a0ad770ba3de6`: preserve the signed original before
one dispatch and retain its reply without blind retry. Here the external signer
and operation recorder own those obligations; the legacy Agent lifecycle is not
copied. Native regressions retain direct-key profiles, duplicate/unknown fields,
exact lookup binding, deadline/cancellation and original-call recovery. No source
was copied. Certificate and management DTO migration are now complete.

Certificate verification now uses Platform 0.34.11 bounded decode and checked
`PreparedRootKey`, followed by explicit effective-target and historical time
authentication. Compared with the previously selected 0.33.7 historical reader,
retain 2 MiB outer/1 MiB delegation caps, 8192 combined nodes, depth 64, 1024
ranges, separate original windows, exact trust commitment and original request
path/reply projection. Root admission additionally rejects invalid BLS points
before use. Decode and verification refusals are distinct public error variants.
Native cases retain invalid root/signature/time/range/tree, delegated authority,
pruned/absent paths, certified rejection and complete-result caps. No certificate
normalization, implicit current time, alternate verifier or new recovery path.

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

Lookup admission uses the same `limits::REQUEST_BYTES` ceiling as operation
profiles (2 MiB), while enforcing the original profile's own byte cap. The
former independent 1 MiB policy bound rejected valid larger profiles even for
small lookup envelopes, after spending their observation reservation. The fix
does not refund or renew those reservations or rewrite original approvals.
Native signed-envelope cases cover the old threshold, 2 MiB and invalid/oversized
inputs for both direct key schemes; the actual-IC interrupted-allocation case
now selects 2 MiB and checks one observation plus offline original-result replay.

Focused review: frozen legacy `4ae087fad8c23de712aa60bf569bde4b2b932651`, Agent
`submission/mod.rs` and `tests/http_boundary/recovery/mod.rs`, retain original
requests, separately spent recovery and zero-or-one submission. History
`a860d14421b9ff1c61c1dce079cdc6a19316bfd0` keeps observation authority separate
from dependent effects. Current history `af240881239112b8a8a9250b4c37e3f00dc68b19`
introduced the shared 2 MiB upload profile but left lookup's old bound behind.
The new store uses signed read-state and certified original-result lookup instead
of legacy lane-specific recovery queries; reuse the original-request and finite
spending guarantees, not those old provider/stage APIs. No legacy source was copied.

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

Within each synchronous call replay, `resolve_call` authenticates the original
owned request once before any pending/reply/observation resolution. Result
authentication checks complete framing, original trust/time, request ID and exact
certified response against that same request; it does not repeat the request
signature check. No authentication is cached across invocations or awaits. Native
coverage rejects a changed signature with unchanged content/request ID for pending,
original-reply, terminal-observation and nonterminal-observation-plus-reply cases.

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

The replay-authentication follow-up re-reviewed the same frozen Agent
`submission/mod.rs` (`submit_once`, `submit_admitted`) and HTTP recovery cases
`status_recovery_preserves_both_attempts_and_exact_zero_or_one_submission` and
`data_readback_recovery_preserves_both_attempts_and_exact_zero_or_one_submission`.
The `d245b0f03a505ba5215d65c10bdebf1890c7dc37` submission patch binds reply retention
to its original prepared request. Backup preserves that binding with an owned
authenticated request and independent certificate checks in the shared replay
resolver; it does not reuse the legacy Agent/provider lifecycle. Its existing
spending/no-retry tests remain, with the signature regressions above at the owning
recorder boundary. No legacy code was copied or external authority removed.

The subsequent evidence-lookup cut re-reviewed frozen
`ops/persistence/attempt_journal/mod.rs` (`open`, `record`) and its tests
`retained_limits_exact_identity_and_receipts_survive_reopen` and
`unsafe_corrupt_excessive_and_replaced_evidence_rejects`. The
`a860d14421b9ff1c61c1dce079cdc6a19316bfd0` patch centralizes prerequisite admission
using the original current record. Backup likewise selects references from its
already-admitted call, but keeps the new recorder's independent byte, namespace
and closing-identity checks rather than adopting legacy's cached JSON record.
Existing custody and runtime recovery cases cover the changed lookup path; no
legacy source, state or cleanup behavior was imported.

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
The first hosted repair run reduced ARM failures to only the peer-close assertion.
The original cancellation review also covers `tests/http_boundary/recovery/mod.rs`
at the same frozen revision: cancellation follows actual retained ingress and
cannot dispatch again. The new fixture additionally requires peer closure and
accepts either EOF or `ConnectionReset`; a timeout, extra byte or other IO error
fails with its original result. This accommodates cancellation before response
headers are consumed without weakening the no-detached-retry assertion.
Intel then exposed a 150 ms submit window expiring before any request arrived.
The submit and lookup body-wait regressions now finish fixture/runtime setup
before setting a two-second original bound, with a five-second watchdog below
the 30-second transport timeout. They still require exactly one real request and
termination before the transport timeout; no production deadline is changed.

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
[consumer executable](../../qualification/ic-backup-consumer/src/main.rs) is built alone
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

### Host HTTP 0.80 / Testkit 0.107 adoption — 2026-11-16

The focused review re-read frozen legacy Agent `submission/mod.rs::submit_admitted`
and `tests/http_boundary/mod.rs` at `4ae087fad8c23de712aa60bf569bde4b2b932651`, with
migration history `1af7e219c3073395992c316ecaa5e83e74b1aecc`. Original ingress must
precede submission; uncertain replies, redirect/oversize/timeout/disconnect and
failed retention never replenish a submission. Those cases remain in Backup's
owning HTTP and recovery tests. No legacy implementation was copied.

Host HTTP 0.80.0 only consumes bounded response bodies. Backup now selects Reqwest
directly and preserves the same TLS, no-proxy/no-redirect/no-retry, timeout,
content-encoding, exact-byte and post-response evidence rules. Both declared and
streamed `TooLarge` errors map to the existing response-limit refusal; allocation
and transport failures remain uncertain. No public operation or stored format changes.

Testkit 0.107.3 supplies prepared asset admission and explicit bounded process
ownership. Qualification owns startup, instance deletion attempts and final
cancel/join evidence. It retains original server output/process reports even when
unwinding; finite output exhaustion is a fixture failure, never a relaxed budget.
The previous CLI and `pic` wrapper are removed. PocketIC 16.1 and actual test
journeys remain unchanged; supplier tests alone do not qualify Backup.

Fresh consumer qualification passes all 236 native tests, five documentation
examples, both strict Clippy checks and all 11 actual-IC journeys (107.11 seconds),
including 20 MiB capture/restore, interrupted original-call settlement and
separate-process offline replay. The isolated archive consumer passes. Original
fixtures and joined process reports remain under `qualification/target/fixtures/`;
logs and verified archives are under `target/evidence/supplier-refresh/`. These
Linux results do not qualify macOS or the blocked new Platform cohort.

The private manager approval profile binds Root separately from the host signer.
Capture and restore include the generation, protocol and authority-policy
commitments, attached cycles, maximum cycle cost and retained reply allowance in
their original approval digest. The manager status policy requires Root as the
sole child controller. Direct profiles omit the manager field so retained direct
approval bytes keep their existing meaning. The public planner selects the manager profile through `with_root_manager`.
Recovery selects its original pure codec through `with_root_codec`.
