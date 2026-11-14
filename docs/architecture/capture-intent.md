# Implemented capture intent

This Unix-only slice persists local intent, not a backup. `CapturePlan::new` is
pure; `BackupStore::create_capture` writes intent; `open_capture` and
`CaptureOperation::inspect` admit original intent without repair. There is no
reservation API for arbitrary calls. Initial direct-host status continuation is
implemented below; reconciliation, snapshots, completion and target release are not. The
[operation design](../design/operation-contract.md) owns the proposed next steps.

## Exact declaration

`CaptureRequest` selects the 32-character lowercase hexadecimal operation ID,
network digest, canonical Platform canister Principal, trust-root digest,
initiator, release, module hash, execution declaration and `CaptureLimitsRecord`.
Direct execution binds the sender. Canic execution binds executor, protocol
digest and authority generation; recording it does not qualify that route.

The strict JSON record uses format `ic-backup/local-capture-plan/1` and rejects
unknown/duplicate fields, unsupported formats and noncanonical identities.
The writer emits compact JSON in the field order frozen by
[the golden plan](../../crates/ic-backup/src/store/tests/fixtures/capture-plan.json).
Reopen retains the exact original bytes, rather than normalizing or re-encoding.
Plan admission is bounded to 1 MiB. The approval commitment is:

```text
SHA256(UTF8("ic-backup/local-capture-plan/1\0") || original_plan_bytes)
```

Each identity and limit is included. Limits declare payload and evidence bytes
in `1..=1 GiB`, extent bytes in `1..=1 MiB` and no larger than payload, network
and signing requests in `1..=8192`, and observations no greater than network
requests. Observations are a subset, not additional authority. These are
structural bounds, not proof that a transport, snapshot or evidence schedule fits.
Provider deadlines, response limits, application admission and metadata-dependent
cost admission are still required before effects. Changing a declaration requires
a different approval; reopening cannot increase an original limit.

## Status preconditions

`CapturePlan::validate_status` checks an already-decoded Platform
`CanisterStatusResult` without allocating, reordering or changing it. The selected
capture profile requires stopped state, the exact approved 32-byte module hash,
and a duplicate-free controller set of at most ten canonical Principals containing
the original direct sender or Canic executor. Ten is this product profile's bound.
An approving initiator, another controller or status viewer cannot substitute for
that management caller. `CaptureStatusError` distinguishes these refusals.

This pure method does not decode or authenticate evidence. The caller must bound
raw decoding and associate the authenticated reply with the original network,
target, request and observation timing; the DTO contains no target identity.
Success does not establish continuous application custody, external settlement,
reservation, dispatch or completion. Resource statistics, migration readiness and
version continuity remain outside this check. No stop or other IC effect occurs.

## Bounded status replies

`CapturePlan::validate_status_reply` reads the required status, controllers and
module hash from one exact Candid argument, then applies the same precondition
policy as `validate_status`. It returns no synthesized full status DTO. Other
fields, including unknown fields, are skipped with bounded traversal; their
presence and semantics are not qualified. Settings/controllers and status must
be present with the required types. An absent or wrongly typed optional module
hash cannot satisfy capture; a present hash must contain exactly 32 bytes.

Limits are 1 MiB raw input, 64 KiB header, 64 type entries, 2 MiB codec work and
2 MiB codec allocation, with a separate cumulative 64 KiB skipped-work allowance.
These are Platform codec counters, not upstream quotas or transport limits.
Controllers are capped at ten before vector allocation. Hash length is checked
before allocating its bytes. Trailing bytes, surplus arguments, unknown status
variants and exhausted bounds refuse through `CaptureStatusReplyError`; policy
failures retain `CaptureStatusError`. Original raw evidence remains caller-owned.

The private argument projection reads labels from Platform's status/settings
schemas. It introduces no replacement management wire declaration or encoder.
Platform owns header parsing, traversal and allocation accounting; the existing
pure policy owns state, module and caller checks. Decoding does not authenticate
network/target/request association, record progress or grant effect authority.

## Files and ordering


Creation requires the caller's exact plan commitment before any write. It uses
Host private held directories, exclusive create-only files, bounded no-follow
regular-file reads and nonblocking native locks. Directories/files use 0700/0600;
hard-linked records refuse. It creates this subset of the proposed store:

```text
targets/<network-digest>/<canonical-canister>/
  lock
  owner.json
operations/<operation-id>/
  lock
  plan.json
  ledger.bin
```

Creation takes the target lock, durably retains the owner record, creates and
locks the operation, then durably retains plan and initial ledger. Owner JSON
contains `format = ic-backup/local-target-owner/1`, `operation_id` and
`plan_sha256`. Its exact bytes must match the original plan, within 1 KiB.
The target lock is released after creation; the operation lock lives with the
returned handle. Different targets have independent locks. Duplicate operation
IDs refuse; a creation race can leave an incomplete second target claim, retained
for original recovery rather than silently removed.

The initial ledger contains one frame, with JSON bytes exactly
`{"sequence":0,"kind":"created"}`. Its encoding is:

```text
seed = SHA256(UTF8("ic-backup/local-ledger/1\0") || original_plan_bytes)
length = little_endian_u32(json_byte_length)
chain = SHA256(seed || length || json_bytes)
frame = length || json_bytes || chain
```

The independent [golden frame](../../crates/ic-backup/src/store/tests/fixtures/capture-created.hex)
uses raw 32-byte digests. The streaming reader admits at most 16 KiB per payload,
65,536 frames and 128 MiB total, further bounded by the original plan's evidence
allowance. Frame lengths and cumulative bytes are checked before allocating a
payload. It retains one frame at a time, validates each chained digest and refuses
any partial tail. Typed `LedgerError` variants distinguish these failures.

Semantic admission requires the canonical initial record followed only by the
ordered initial-status transitions below. A correctly hashed unknown event,
duplicate creation or changed sequence refuses; no suffix is ignored or
interpreted as unused authority. The original golden
bytes are unchanged. This is an unpublished restart format, not a legacy import.

Creation checks that the initial ledger fits the original evidence allowance
before claiming the target. Reopen does not increase that allowance. The first-status writer reserves aggregate evidence and remaining frame capacity
before signing; individual files do not each receive the full allowance.

## Durable initial status

`CaptureOperation::advance_status` implements the initial direct-host status call
through a caller-supplied `StatusProvider`. The library has no concrete provider;
the trait is a trusted integration contract, not proof that its implementation is
qualified. Canic execution refuses before spending until its executor/receipt
protocol is implemented. No snapshot, observation/reconciliation, completion or
owner-release flow is included.

The workflow constructs Platform's `CanisterIdRecord` for `canister_status` and
encodes its single argument within 256 bytes. A strict `StatusAdmissionRecord`
binds provider and continuous-application commitments, absolute prepare/submit
deadlines, separate preparation/submission network ceilings and evidence sizes.
Deadlines must be nonzero and ordered; submission costs at least one request.
Preparation may use zero network requests for local signing. The combined network
ceiling must fit the original plan; exactly one signing invocation is reserved.
Provider methods enforce actual time, traffic and response bounds while receiving.

Before retaining prepared output, before reserving submission, and during settled
replay or status inspection, Backup reads call content through Platform's
`ic_protocol::decode_call_content_strict`. It requires replicated `canister_status`
ingress addressed to the management canister, the original direct sender, exact
Platform-encoded target arguments, matching declared expiry and the content's
recomputed request ID. Unknown/duplicate content fields, malformed/trailing bytes,
query/read-state envelopes and mismatches return `CaptureError::StatusAdmission`.
Original envelopes are never re-encoded. Nonce and known sender-info metadata
participate in the request ID; signature/delegation and sender-info authentication,
effective HTTP routing, deadlines and observation binding remain provider-owned.

The complete request-file cap is checked before content decoding. Platform's CBOR
input cap is a byte bound, not the Candid codec's allocation/work counters. The
check reads no clock and does not authenticate anything, so old settled evidence
still requires historical proof verification by the provider. A mismatched new
signer result leaves preparation spent without submission. Mismatched retained
content refuses without rewriting records, refunds or another provider invocation.

Request evidence is capped at 1 MiB; response/proof evidence at 2 MiB. Before any
provider call that may sign, Backup reserves both entire file ceilings plus the
initial ledger and four maximum frame sizes (`4 * (16384 + 36)`). This conservative
headroom fits the original evidence ceiling; actual bytes replace reserved output
without introducing another allowance. Structural admission alone does not prove
provider cost or application safety. All admission/authentication predicates are
local and effect-free; remote work belongs only to the reserved bounded methods.

After the original created frame, the private writer supports exactly these
compact JSON records. `kind` is first; fields follow the order shown. Unknown,
duplicate or noncanonical fields and any other sequence refuse.

| Kind / sequence | Following fields | Retained meaning |
| --- | --- | --- |
| `preparation_reserved` / 1 | `admission`, `arguments_sha256` | Original admission, exact argument hash, one spent signing slot and preparation network ceiling. |
| `prepared` / 2 | `request` | Exact original signed request file length/hash. |
| `submission_reserved` / 3 | None | Original submission network ceiling spent; only its live successful writer may submit once. |
| `settled` / 4 | `result` | Exact authenticated response/proof file length/hash. |

The plan-bound chain and closed sequence identify initial step zero, attempt one;
there is no generic event or spending API. Digests are 32-byte JSON arrays.
Evidence references contain `bytes`, then `sha256`. Admission fields follow the
[public declaration](../../crates/ic-backup/src/store/capture/advance/mod.rs).
All records remain under the original frame and cumulative ledger bounds.

`status-request.bin` is request ID (32 bytes), expiry (u64 little endian), envelope
length (u32 little endian), exact envelope, then observation-binding bytes.
`status-result.bin` is outcome (one byte: 0 reply, 1 rejection), response length
(u32 little endian), exact response, then authentication-proof bytes. Both variable
components must be nonempty. Complete files are retained create-only and synced
before their referencing event. Neither file is a standalone transferable receipt.
Request bytes, proof and signed delegations are sensitive and have no derived Debug.

Each commit re-admits the original operation, prepares its bounded candidate,
opens the held original ledger for update and checks identity/link/permission
custody. It invalidates the session before synchronization or writes, synchronizes
the retained prefix and referenced request for cold continuation, then retains
new evidence, seeks the original exact end, writes and syncs the frame. Closing
admission must pass before the session can continue. Write, sync or closing failure
leaves the handle unusable. Reopen stays read-only and never truncates or repairs.

A replayed preparation/submission reservation returns pending without invoking
any provider. A prepared request may proceed only through fresh local admission
of its original bytes, authority and expiry; it cannot re-sign. Historical settled
replay authenticates retained proof locally, checks bounded status policy and
never invokes current admission, signing or submission. Rejections and failed
status policy remain settled and spent. A visible complete frame after uncertain
sync stays authoritative; any partial frame or unreferenced evidence file blocks
continuation. No orphan adoption, retry, target release or cancellation is added.

[Native workflow tests](../../crates/ic-backup/src/store/capture/advance/tests.rs)
cover exact evidence, bounds, refusal, cancellation, uncertain writes, orphan and
changed custody, held-root rename and cold replay. Acknowledged owner-death cases
cover reserved/invoked preparation, prepared bytes, reserved/invoked submission
and settlement, while another target retains independent progress. These tests
adapt the focused legacy reservation lessons below; their injected provider is
only a call-order fixture. Actual IC authentication, continuous application
custody, provider deadlines/traffic, physical power loss and hosted macOS remain
unqualified. This is not an independently usable backup product.

## Read-only status inspection

`CaptureOperation::inspect_status` returns a `CaptureStatusView` after complete
original admission and a closing custody check. It exposes the retained stage,
exact original `StatusAdmissionRecord`, conservative reserved signing/network
counts and the original request ID when prepared evidence exists. The request
framing and original call binding are admitted before projecting its ID. Signed envelopes and proof bytes
remain private; the returned admission is a detached value and cannot mutate
retained authority.

Created intent reports zero spending. Preparation reserves one signing invocation
and the original preparation network ceiling. Submission additionally consumes
its original network ceiling; settlement and rejection never refund either.
These are reservations, not observed traffic or remaining whole-backup capacity.
`Settled` reports retained evidence, not authenticated status success or completion.
Inspection calls no provider and never writes, synchronizes, repairs or retries.
A restarted consumer can pass the recovered exact admission to `advance_status`,
which still independently checks original custody and eligible transitions.

## Recovery and custody

Dropping a handle or killing its process releases the native lock, not the owner.
A complete operation can reopen only with its original approval, plan, ledger,
owner and lock files. The operation also holds the original ledger descriptor.
Inspection checks held child-directory/lock identities, store format, exact record
bytes and member sets. Ledger admission checks file identity before reading and
rechecks the opened file's length and named file's identity/length afterward.
Replacing the ledger with identical bytes therefore refuses a held inspection. Store root rename follows its
held descriptor, consistently with inventory. These checks assume exclusive
cooperating writers and private-store custody; they are not a hostile-writer fence
or protection against rollback of all local files.

Creation failures retain every partial file and may leave the target occupied.
An existing target lock with a missing owner is incomplete, never a fresh target.
Missing established locks are not recreated beside retained records. Reopen does
not create, synchronize, repair or discard anything. No terminal succession is
implemented: even a successfully created intent keeps its target occupied after
the handle is dropped. There is no cancellation or cleanup escape hatch yet.

## Append integration dependency

The original Host FS 0.77.2 selection's [capability implementation](https://github.com/dragginzgame/ic-tooling/blob/9d938d2814b637ea40d4ddbae110e87ab402c881/crates/ic-host-fs/src/capability/mod.rs)
opens existing regular files read-only and creates new files read/write. Safe
ledger continuation needs an existing-file writable opener relative to the held
directory, without create/truncate, with no-follow regular-file admission. Backup
must then verify original identity, link count and permissions before writing.
Path-based lock openers are not this capability interface; using a remembered
display path would change continuation behavior after a store-root rename.

The explicitly authorized supplier contribution now implements
`FileEntry::open_regular_file_for_update(forbidden_permissions)` in
`ic-tooling` [c30c0f92](https://github.com/dragginzgame/ic-tooling/commit/c30c0f92c680c918970c9eb06559d1958de0d615). It shares the existing descriptor engine and retains permissions,
no-follow/nonblocking/type admission and held-parent rename behavior. It neither
creates nor truncates and returns a read/write descriptor at offset zero; caller
identity, hard-link policy, locking and seek/write/sync remain explicit. Linux
qualification passed 18 capability and 40 shared reader tests, strict Clippy with
and without capability, and Rustdoc. No macOS or Backup append/recovery claim
follows. The source was pushed to GitHub under the push-only scope recorded in
[b9700c9d](https://github.com/dragginzgame/ic-tooling/commit/b9700c9d7c8a2375943181dbbc4dad22f23f5ad0).
The user coordinates Tooling delivery. Backup first used the explicitly authorized
local fix, then verified the published Host FS/Artifacts 0.77.3 archive checksums
and qualified the registry selection. The temporary overrides are removed; the
status writer now uses the published opener. [Tooling #358](https://github.com/dragginzgame/ic-tooling/issues/358)
retains producer provenance. [Setup](../development/setup.md) describes the current
registry-backed graph; previous local source evidence remains retained.

The status writer uses this admitted Host boundary without replacing the ledger
or recreating missing records. It adapts the reviewed legacy disable-before-write
ordering, preserves partial frames and qualifies acknowledged owner death locally.
The [operation design](../design/operation-contract.md#first-durable-call-initial-status)
retains broader provider/IC acceptance requirements; the
[pure transfer schedule](capture-transfer.md) remains independent.

## Focused legacy review — 2026-11-14

Reviewed frozen legacy `4ae087fad8c23de712aa60bf569bde4b2b932651`:

- [Attempt persistence](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/ops/persistence/attempt_journal/mod.rs)
  binds reopened journals to exact original authority, reserves durably before
  returning a slot, and invalidates a session after uncertain persistence.
- [Regressions](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/ops/persistence/attempt_journal/tests/mod.rs)
  cover retained identity/limits, failed reservation, unsafe/replaced evidence and
  acknowledged owner death. [The model](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/attempt_journal/mod.rs)
  prevents missing evidence from replenishing attempt authority.
- [Plan simplification history](https://github.com/dragginzgame/ic-backup-legacy/commit/a860d14421b9ff1c61c1dce079cdc6a19316bfd0)
  separates complete-original admission from optional progress projection. Removing
  duplicated projections does not justify omitting original-journal checks.

Adapted the exact-original, no-reset, nonblocking exclusion, retained partials and
acknowledged-death cases in [new native tests](../../crates/ic-backup/src/store/tests/capture.rs).
The missing-owner case reproduced an incorrect new claim before the admission fix.
Golden plan/chain bytes were independently generated with Node's SHA-256, then
compared against the production writer. Tests also cover independent targets,
concurrent claims, ambiguous records, unsafe files and changed custody.

No legacy source was copied. One plan/ledger replaces legacy per-step journals;
there is no generic workflow graph. The next focused review reused the same
attempt persistence/model/tests and plan-ownership history: complete original
admission is still required before any future spending transition. Legacy's
bounded retained-event vector becomes a streaming, length-delimited frame reader.

[Frame and custody regressions](../../crates/ic-backup/src/store/capture/ledger/tests.rs)
cover every truncated initial prefix, chain/plan mismatch, correctly hashed
unsupported events, bounds before body reads, exact frame/count boundaries,
interrupted reads, original IO errors and closing file drift. A separate held
inspection case reproduced same-byte ledger replacement acceptance before the fix.
The frame-count fixture tests framing only; its arbitrary payloads do not qualify
capture transitions. Sparse oversized files are retained as refusal evidence.

The status precondition review also inspected frozen legacy
[controller authority](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/control_authority/mod.rs),
[its regressions](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/control_authority/tests/mod.rs),
[lifecycle observations](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_lifecycle_reply/mod.rs),
[their tests](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_lifecycle_reply/tests/mod.rs)
and [settlement](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_observation/settlement/mod.rs).
These guard exact original caller membership, bounded controller sets and the
separation between a reported state and authenticated effect attribution.
[Codec history 1af7e219](https://github.com/dragginzgame/ic-backup-legacy/commit/1af7e219c3073395992c316ecaa5e83e74b1aecc)
retained those cases while moving lifecycle wire tests to Platform-owned codecs.

The new boundary already receives canonical Platform Principals and distinguishes
initiator, sender and executor in the original plan. It needs no string-based
controller wrapper or sorting. [Re-authored tests](../../crates/ic-backup/src/store/capture/status/tests.rs)
exercise stopped/running/stopping states, canonical duplicates, empty/excessive
sets and the exact ten-controller boundary, sender/executor confusion, visibility,
module mismatch and input preservation. The selected profile adds the approved
module-hash check. Observation wrappers and settlement transitions remain outside
this policy; authenticated observation association and IC effects require their
own work.

The raw-status continuation reviewed the same lifecycle implementation, tests and
codec history, plus its [bounded wire reader](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_lifecycle_reply/wire/mod.rs)
and private field mappings. It retains separately metered skipped work and bounds
before allocating controller elements. The new argument projection uses supplier
schema labels and canonical Principals directly, adds the approved module hash,
and calls the existing policy instead of building a second authority object.

[Eight new wire tests](../../crates/ic-backup/src/store/capture/status/wire/tests.rs)
cover complete supplier results, minimal projections, exact controller bounds,
hostile lengths before payload reads, module option promotion/absence, required
fields, state/caller refusals, cumulative nested skipping and malformed framing.
[Three retained golden replies](../../crates/ic-backup/src/store/capture/status/wire/golden.json)
come from [legacy lifecycle fixtures](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_lifecycle_reply/tests/golden.json);
their exact bytes and SHA-256 values are preserved. Their missing module hashes
cannot satisfy the new capture profile. No legacy implementation was copied.
Acknowledgement, receipt-digest and mutation tests are excluded because this API
performs only status validation, not stop/start/load or authenticated settlement.

The status continuation re-authors reservation failures, lost returns, cancelled
futures, retained results and acknowledged owner death against the append writer.
These native tests prove filesystem/call ordering, not paid IC effects, provider
authentication, power loss or Canic execution. Native Canic plan round-tripping
proves declaration retention only; status continuation refuses that route.

The request-binding continuation inspected frozen legacy's
[single submission](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup-agent/src/submission/mod.rs),
[reserved request owner](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup-agent/src/request/mod.rs)
and [HTTP boundary cases](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup-agent/tests/http_boundary/mod.rs).
Legacy constructs and signs the admitted call inside Agent. The new boundary accepts
external prepared bytes, so immutable call inspection now also belongs at that
boundary. It reuses Platform's published protocol decoder and request-ID owner;
no Agent wrapper, CBOR parser, signing code or legacy implementation is copied.
The existing a860d144 original-admission/projection separation remains intact.

New native cases reject wrong caller/receiver/method/target/arguments, mismatched
ID/expiry, query/read-state, unknown/duplicate fields and every truncated envelope
prefix. Matching file hashes cannot authorize a mismatched prepared or settled call.
Golden envelope/request hashes were independently recomputed with Node SHA-256.
Retention and cold replay cases assert no extra provider calls or spending reset.
The legacy HTTP retry/redirect/timeout cases still belong to the future concrete
transport; these tests establish content admission and persistence ordering only.

## Historical status authentication

`authenticate_status_evidence` is a local helper for a direct provider's
`authenticate` method. It supplies no signer or transport. It reuses original
admission and complete request/result framing checks, then validates the exact
prepared call. For this certificate protocol, `trust_root` is SHA-256 of the
complete 133-byte IC DER public key, never a URL or a hash of raw key bytes.
The integration provisions that root/network binding before approving the plan.

`status_certificate_observation` encodes the provider's explicitly admitted time
policy as `ic-backup/status-certificate/1\n`, followed by six little-endian u64s:
certificate reference/maximum age/maximum future skew, then delegation
reference/maximum age/maximum future skew. Exactly 79 bytes are accepted; both
inclusive windows must have non-overflowing endpoints. Retain these bytes in the
original prepared observation before submission. Never derive the policy from
incoming certificates or replace it with current configuration during replay.
Provider protocol/application admission and original file custody remain required;
this helper cannot establish those facts from freely constructed arguments.

`StatusEvidence.proof` contains the exact original certificate CBOR, and
`response` the exact Candid reply or the rejection diagnostic frame defined below. Platform 0.33.4's `certification-verify` feature
owns strict certificate decoding, BLS verification, delegated canister ranges and
historical time authentication. Backup passes the original target as the effective
canister. Product limits are 1 MiB each for outer/delegation certificates, 8192
combined tree nodes, depth 64 and 1024 selected canister ranges. Original complete
request/result limits apply first. Existing frame encoders bound temporary copies;
byte and tree limits are not an allocation-quota claim.

After authentication, Backup requires the exact original request ID under
`/request_status/<id>/status`. For `replied`, the certified `reply` leaf must
exactly match the retained response. For `rejected`, all retained diagnostics
must match their certified leaves, including the distinction between an absent
and present-empty optional error code. Pending, forgotten, missing and unknown
outcomes refuse. These paths follow the
[IC state-tree specification](https://docs.internetcomputer.org/references/ic-interface-spec/).
No clock, transport, implicit retry or alternative digest proof participates.
The workflow separately applies stopped/module/controller policy to successful
replies. Certified rejection settles the original attempt without refunding it;
even a transient code grants no retry or claim that an effect was rolled back.

`status_certificate_evidence` authenticates an original certificate and builds
bounded evidence for either outcome. The complete response/proof allowance is
checked before copying projected bytes. Replay uses the same private projection
through `authenticate_status_evidence`, comparing the outcome flag and every
response byte; no second parser for the diagnostic record is needed. Both helpers
retain original certificate bytes. The initial request, observation and outer
result/ledger formats remain unchanged. The earlier uncommitted reply-only helper
was renamed; no compatibility alias or old-record migration is added.

A rejected response is `ic-backup/status-rejection/1\n`, one reject-code byte,
message length (u32 little endian), exact UTF-8 message bytes, then one optional
error-code tag. Tag 0 means certified absence, with no following bytes. Tag 1 is
followed by error-code length (u32 little endian) and exact UTF-8 bytes. Empty
messages and present-empty error codes are valid and distinct from absence.
This is a Backup evidence projection, not an external IC response DTO. The
original certificate retains its complete wire representation.

The direct-ingress profile accepts only the single-byte canonical encodings of
codes 1–5. Unknown/overlong encodings and code 6 refuse; the
[HTTPS reject-code contract](https://docs.internetcomputer.org/references/ic-interface-spec/https-interface/)
reserves code 6 for best-effort inter-canister calls. No general natural-number
codec or duplicate external enum is introduced. Optional lookup must return a
leaf or guaranteed absence; pruned/unknown or structurally invalid paths refuse.
Diagnostics are never normalized or included in error messages. Uncertified HTTP
rejections, disconnects and timeouts cannot become terminal evidence.

Focused review used frozen legacy
[submission](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup-agent/src/submission/mod.rs),
[reply evidence](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup-agent/src/reply_evidence/mod.rs)
and its [domain/attempt regressions](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup-agent/src/reply_evidence/tests/mod.rs).
The [94b94db8 history](https://github.com/dragginzgame/ic-backup-legacy/commit/94b94db89f77968eb4b480ef6d0576d41f6472e2)
centralized application reply qualification around original stage, authority,
attempt and exact reply. Preserve that binding lesson and one-shot submission;
do not treat legacy's domain-separated reply digest as a retained certificate.
The current initial-status ledger already fixes the step/attempt, so no parallel
attempt envelope or copied legacy architecture is added. Original legacy records
are not accepted under this protocol. The older digest golden bytes are excluded
because the new authentication proof is the actual certificate, not a digest.

[Consumer regressions](../../crates/ic-backup/src/store/capture/advance/tests/certificate_tests.rs)
use newly authored synthetic trees with real Platform BLS signatures. They cover
exact request/root/reply association, tampering, unavailable outcomes, strict
retained policy, separate delegation time and effective canister, frame/certificate/
tree bounds, unchanged cold replay and spent submission after authentication failure.
The [rejection cases](../../crates/ic-backup/src/store/capture/advance/tests/certificate_tests/rejection_tests.rs)
also cover fixed projection bytes, absent versus pruned/empty diagnostics, strict
text/codes, complete result limits, tampering and effect-free rejected replay.
The same frozen submission and HTTP-boundary review applies: legacy mapped Agent
rejection errors to uncertainty because it did not retain their certificate.
Only a fully authenticated original rejection changes that behavior here; all
unproven outcomes keep the original pending reservation. No legacy or supplier
implementation was copied. These tests qualify the local
cryptographic composition and persistence boundary, not IC effects, a live provider,
application safety or hosted macOS. HTTP/redirect/retry/timeout qualification stays
with the future concrete provider.
