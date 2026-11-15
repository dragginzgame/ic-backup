# Single-canister operation contract

Proposed effectful contract, 2026-11-14. Capture/restore remains unimplemented;
[local inventory](../architecture/local-inventory.md) and
[capture intent](../architecture/capture-intent.md) and
[pure transfer planning](../architecture/capture-transfer.md) are implemented subsets.
Intent creation alone reserves no attempt. The implemented
[initial-status continuation](../architecture/capture-intent.md#durable-initial-status)
adds bounded direct-host preparation, submission and settlement through a caller
provider; no concrete provider or authenticated IC journey is qualified yet.
This refines the [library design](library-design.md) and its
[Canic integration](sibling-integration.md). It owns the proposed operation,
transport and transfer semantics below. The library design retains storage
rationale and custody requirements. The [local-store contract](local-store.md)
selects framing, completion and inventory behavior. Exact record encodings,
codec/evidence limits and concrete provider qualification remain prerequisites
to effectful implementation.

## Consumer boundary

The same host engine serves an independent Rust program and a Canic-owned host
adapter. Consumers provide exact selection and application authority; Backup
owns the ordered snapshot workflow, durable attempt accounting and local bundle.
Public requests do not expose arbitrary management methods or writable journals.
The following are semantic signatures, not compiling Rust declarations:

| Operation | Required inputs | Result and allowed effects |
| --- | --- | --- |
| `plan_capture` | Network/trust binding, initiator, execution binding, target, release, profile limits. | Immutable plan and approval commitment; pure, no providers. |
| `plan_restore` | Held verified source, target/release/execution binding, profile limits. | Independent plan committing to source manifest and complete upload/readback schedule; pure over admitted inputs. |
| `create_capture` / `create_restore` | Private store, unused operation ID, plan, exact approval commitment. | Durable intent and target claim; restore also retains its source reference. Local IO only. |
| `open` / `inspect` | Existing operation location and bounded local replay. | Held operation or read-only diagnosis; no provider calls or implicit repair. |
| `advance` | Held operation, transport, application admission provider, finite step allowance. | Progress, attention or stored terminal receipt. Only the next eligible original step may dispatch. |
| `reconcile` | Held operation, exact pending attempt, transport, finite observation allowance. | Original result settlement or retained uncertainty; never resubmits the original request. |
| `list_backups` / `latest_completed_backup` | Local store, optional exact target filter and finite scan limits. | Completion summaries without network or payload hashing; no fresh integrity or complete-fleet claim. |
| `verify_bundle` | Private bundle and trusted original manifest commitment. | Held freshly verified source or typed refusal; no network and no trust-on-first-use import. |

`advance` never implicitly polls. A submission that does not supply an authenticated
outcome returns a pending attempt for explicit `reconcile`. One invocation may
perform multiple settled steps within its caller allowance, but never dispatches
concurrently. The allowance controls scheduling, not original spending authority.
An async runtime belongs to the caller; neither the engine nor a dropped future
spawns detached continuation. Inspecting terminal work is local and effect-free.
Different targets can have independent operations in flight; Canic owns their concurrency limit. An occupied
target claim excludes another operation for the same network/canister.

Caller-visible results distinguish invalid input, occupied custody, unsupported
profile, exhausted budget, provider refusal, authenticated remote rejection,
unresolved outcome and damaged local evidence. Each includes the original
operation/step identity when one exists. An IO error after reservation is never
reported as permission to try again. Reopening retains consumed allowances.

## Identity and application admission

A plan binds these identities independently:

- Network instance and provisioned trust root; endpoint URLs are routing inputs.
- Initiator, exact target principal and release identity including module hash.
- Execution route: direct host sender, or a named Canic executor with its protocol
  and authority generation. The executor is the actual management sender.
- Source manifest commitment for restore, original source target/network/release,
  and the admitted product/format profile.
- Operation identity and finite original allowances. Step identities derive from
  this plan and a deterministic ordinal; they do not change across reopen.

A restore target must equal its source target/network/release. The approving
initiator or management sender may differ from the capture operator if the new
restore plan explicitly selects it and current authority admits it. A pending
operation cannot change sender, executor, generation, trust root or plan.

The integration supplies bounded admission evidence binding the exact plan,
maintenance witness and fence/obligation revision. Backup verifies those bindings;
the provider is trusted for application semantics and must have independent
qualification. Provider calls have explicit deadlines and response limits. They
cannot dispatch Backup management effects or grant additional attempt authority.
Provider-owned remote observations require their own retained finite budget;
they are never hidden in a supposedly pure admission predicate.

Maintenance custody must survive process death and cover all controllers,
application producers and the entire interval through completion. Reopen requires
proof of continuity, not merely a new matching status observation. Loss of custody
blocks further effects and success receipts. A reconciliation observation may
still recover the original outcome without authorizing dependent work.
Restore requires current rewind permission immediately before allocation and
again before load. A stopped canister is not proof of settled external obligations.

## Transport without hidden retries

Use one typed transport port with three capabilities. Core constructs requests
from Platform management DTOs. The provider supplies route-specific signing and
authentication; it does not choose the workflow or replacement policy.

| Capability | Contract |
| --- | --- |
| Prepare | Encode/sign the exact admitted call and return bounded original bytes, argument digest, request identity, sender, route, expiry and an observation binding. No management submission. |
| Submit once | Send those retained bytes once within the reserved deadline and byte caps. Return an authenticated result with retained proof, or uncertainty. No resigning, retry, polling, redirect submission or fallback route. |
| Observe once | Use a separately reserved observation for the original pending attempt. Return bounded authenticated original-result evidence, pending/unknown, or provider failure. It cannot execute the original operation. |

External signing is allowed through a Canic-owned sign-only process. Preparation
itself consumes a bounded signing invocation before calling that process. Lost
signer output retains the slot as uncertain and does not silently launch another
signer. Private keys never enter records; signed envelopes and delegations are
sensitive authority-bearing evidence held in the private store.

The durable order is: reserve preparation slot; prepare; retain and sync exact
request evidence; append and sync submission reservation; invoke submission once;
retain and sync returned evidence; append and sync settlement. Effects depend on
the last completed durability boundary, not whether an async function returned.
A preparation failure cannot become a submission reservation without its complete
inspected bytes. A crash after submission reservation blocks redispatch even if
no packet was actually sent. An observation follows the same evidence/reservation
ordering and consumes its own count, deadline and byte allowance.

Cold reopen must be able to authenticate retained settlement evidence with the
original trust and route binding. Current certificate freshness is required when
admitting a new observation; it must not make an already admitted historical
receipt silently disappear. Provider proof format, trust transitions and expiry
rules need golden tests before transport admission. A provider's bare `success`
Boolean is insufficient.

The direct route associates each authenticated management result with its exact
ingress request. In the Canic route, an authenticated executor receipt associates
host intent with the executor's retained management result. These are different
proofs. An ordinary uncertified query returning a plausible receipt is insufficient.
Either certify the exact lookup result under an admitted protocol or use an
accounted authenticated update response. Lookup never causes management dispatch.

## First durable call: initial status

The initial status slice is implemented through `advance_status`:
original admission, preparation, one submission and retained authenticated result.
It must include cold replay and interruption cases. A public append or reservation
API alone is not that unit. Keep one operation ledger, one sequential workflow and
the existing Platform status policy; do not expose its internal events or add
per-step journals. The following states now have a private writer/replayer. The implemented contract
owns exact encodings and limits; concrete provider and IC qualification remain open.

Before spending, admit the held original plan, target owner, locks, ledger and
referenced evidence. The application provider must establish continuous maintenance
custody, and the transport must admit the original sender/route/trust binding,
exact Platform status arguments, deadlines, byte ceilings and route costs. An
unqualified provider or unaffordable call refuses before preparation. Neither a
successful status decoder nor matching controller bytes satisfies this admission.

| Durable state | Bound content and spending | Next permitted action |
| --- | --- | --- |
| `Created` | Original immutable plan and target claim; no provider call reserved. | Admit the first call and append its preparation reservation. |
| `PreparationReserved` | Original step/attempt, method/argument commitment, application/provider bindings, signing slot and finite preparation limits. | Only the live continuation that durably created this reservation may invoke preparation once. Cold replay cannot recreate that permission. |
| `Prepared` | Complete inspected signed request retained create-only, with exact length/hash, sender, route, request identity, expiry and original observation binding. | Recheck custody and expiry, then durably reserve submission of these exact bytes. No re-signing. |
| `SubmissionReserved` | Exact prepared request, finite network cost/deadline and result-evidence allowance; consumed counts never refund. | Only its live continuation may submit once. Cold replay permits reconciliation of the original request, never submission. |
| `Settled` | Durable raw result/proof references and authenticated original-request outcome. | Apply the existing status policy. Advance only on an admitted stopped/module/controller result and continuing application custody. |

Every event has the next ledger sequence and exact original step/attempt binding.
The ledger hash already binds the plan; no second mutable spending record is
introduced. Preparation and submission are distinct because a lost external
signer result can be a paid unresolved invocation even before management ingress.
Authentication requests and observation traffic must fit their declared original
network allowance too; one transport method call is not proof of one network
request. Canic admission includes both hops and its durable executor join.

The evidence ceiling applies to the operation as a whole: ledger bytes, retained
evidence and still-reserved evidence capacity must fit together. Before invoking a
provider, reserve room for its bounded output/proof and the remaining transition
frames needed to retain that result. Replacing reserved capacity with actual bytes
must not double-count it or erase already retained bytes. Unknown or partial files
block continuation; they are not zero-byte evidence or reclaimed headroom. Actual
provider limits and exact frame encodings must be selected and tested with the
writer, before any effectful API becomes usable.

For each append, first validate and encode the complete bounded candidate. Recheck
the original file identity, link count, permissions, prefix length/head and held
operation exclusion, then mark the live session unusable before the first write.
Seek the admitted original descriptor to its exact admitted end, write the frame,
and sync it before installing the new in-memory state or returning control to a
provider call. Any write/sync/closing-custody failure leaves that handle unusable.
Do not truncate a partial frame, retry the append in place or infer success from
cached counters. The Host opener supplies a descriptor, not this product protocol.

Cold `open`/`inspect` remain read-only. Effectful continuation must separately
re-admit the entire original prefix and referenced evidence, establish their
required durability and current custody, then follow the state above. A complete
frame visible after an uncertain sync still counts as retained spending; it is
never ignored because the original invocation did not return. A partial frame
blocks effects and remains preserved. Reconstructed state cannot manufacture the
live-only permission to invoke a signer or submitter.

| Interruption | Required continuation |
| --- | --- |
| Preparation reservation persisted, signer output missing | Retain uncertain preparation and consumed signing allowance; no automatic second signer invocation. |
| Request/evidence file visible without its ledger reference | Preserve it and stop for original recovery; no implicit orphan adoption or new allowance. |
| `Prepared` replayed with no submission reservation | Submission may be reserved only for the original retained bytes, within their original expiry and fresh custody. Expiry returns attention, not automatic re-signing. |
| Submission reserved, packet not sent or response lost | Both replay as pending. Only an independently reserved original-request observation can resolve them. |
| Authenticated result retained, settlement append incomplete | Preserve all bytes; a partial ledger blocks continuation. Visibility alone is not permission to settle or dispatch. |
| Settlement durable, return lost | Replay the original evidence and result locally. A failed status predicate or remote rejection stays a consumed refusal; it never authorizes the dependent snapshot call. |

The implementation must re-author legacy reservation failure and acknowledged
owner-death tests against this append protocol, including failure before any byte,
partial header/payload/digest, sync failure, full frame with lost return and death
before/after provider invocation. Assert exact original bytes, spent counters,
zero forbidden invocations, unchanged target ownership and independent-target
progress. Local injected providers prove call ordering only. Authenticated status
requires a selected production provider and actual IC qualification; capture
acceptance still requires the complete snapshot-to-bundle journey below.

This review revisited frozen legacy's
[reservation owner](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/ops/persistence/attempt_journal/mod.rs),
[model](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/attempt_journal/mod.rs),
[tests](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/ops/persistence/attempt_journal/tests/mod.rs)
and [cached-write invalidation](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/ops/persistence/json/mod.rs).
[History a860d144](https://github.com/dragginzgame/ic-backup-legacy/commit/a860d14421b9ff1c61c1dce079cdc6a19316bfd0)
separates complete-original admission from optional reporting. Retain that
admission and disable-before-write ordering. Replace whole-record replacement and
peer-journal scans with the one held append ledger; legacy process-death passes
do not qualify this different persistence mechanism.

## Ordered work and outcomes


All status/list/metadata reads are original accounted requests too. The fixed
management sequence is:

| Capture | Restore |
| --- | --- |
| Observe stopped state, module and effective controller authority. | Observe stopped state, module and effective controller authority. |
| List snapshots for capacity admission. | List snapshots for capacity admission. |
| Take one new snapshot without replacement or uninstall. | Allocate one new snapshot by metadata upload without replacement. |
| Read metadata; admit the schedule and its conservative resource bounds. | Upload the exact source extents and complete stored chunks. |
| Read all scheduled extents/chunks. | Read destination metadata and every destination extent/chunk. |
| Observe final stopped state under continuous custody. | Recheck stopped state and restore admission; load the exact verified destination once. |
| Verify local bytes, publish manifest, retain `BackupVerified`. | Observe final stopped state; retain `LoadedStopped`. |

List evidence is a preflight refusal aid, not a reservation of remote capacity.
A later capacity rejection stays a consumed failed request. No replacement,
funding, uninstall, stop/start or cleanup fallback is available.

Take uses `replace_snapshot = None`, `uninstall_code = Some(false)`. Upload
allocation uses `replace_snapshot = None`. Direct ingress omits
`sender_canister_version`; a Canic sender must bind its own admitted version when
that argument is used. It never substitutes the target's version. Backup does not
infer a missing take/allocation result from the difference between snapshot lists.

A settled load plus missing final status is **load confirmed, completion pending**.
It must never dispatch load again. A retained final status with an uncommitted
terminal record permits local completion under the original evidence; terminal
replay does not require a fresh status request. Authenticated remote rejection
ends that step without refunding its reservation. Uncertainty remains an attention
state, not a completed failure that frees the target claim.

## Metadata comparison and representability

The pure uploaded-metadata comparison is now implemented by
[`CaptureTransfer::decode_uploaded_metadata`](../architecture/capture-transfer.md#uploaded-metadata-readback).
It reuses bounded metadata decoding and compares source state without effects.
[`CaptureTransfer::begin_readback`](../architecture/capture-transfer.md#ordered-payload-comparison)
now checks complete ordered payload equality against caller-held source slices.
Remote readback, held verified-bundle integration, durable progress, authenticated
association and load admission remain proposed.

Use the selected published Platform snapshot DTOs and codec at the management
boundary. The inspected [native DTO surface](https://github.com/dragginzgame/ic-platform/blob/c90ce9fce0948da0c05e6f5ac333c520fa46d801/crates/ic-platform/src/management_canister_types/native/mod.rs)
contains optional read globals but mandatory upload globals. Persist raw evidence
and an explicit lossless projection; do not serialize Rust float values directly
into JSON and assume that preserves NaNs, signed zero or all bit patterns.

The [management protocol](https://docs.internetcomputer.org/references/ic-interface-spec/management-canister/)
creates fresh metadata when a snapshot is uploaded. Omitted timer/hook fields do
not update those states on load. Backup therefore proposes the following narrow
round-trip profile; these are product restrictions, not new protocol requirements.

| Data | Proposed admission/comparison rule |
| --- | --- |
| Module/heap/stable lengths | Exact source/upload/readback equality, checked arithmetic, then full byte equality. |
| Globals | Preserve order, variant and integer value; floating values compare raw IEEE bits. Every source entry must be present. `v128` must fit 128 bits. |
| Certified data | Exact length and bytes. |
| Timer and low-memory hook | Both must be explicitly present for this first round-trip profile; preserve exact variant/value. |
| Chunk store | Validate unique hash identities; compare complete hash sets and every chunk's bytes. Preserve raw list ordering as evidence, but do not require identical wire order in readback. |
| Snapshot ID, creation timestamp, source kind and recorded canister version | Retain separately as source/destination provenance. Validate their route-specific meaning; do not require equality across take and upload. |

Take metadata must identify a taken snapshot; destination metadata must identify
an uploaded snapshot. Absent or unknown origin is outside this initial profile.
A decodable absent global/timer/hook remains valid external data, but returns
`UnsupportedSnapshotProfile` before transfer or restore allocation. It is not
silently filled, filtered, shifted or called corrupt. The original take and its
evidence remain retained if this refusal occurs after capture.

Present timer/hook state does not make application restart safe. The Canic fence
must cover restored callbacks and timers before any later execution. Raw unknown
fields are not permission to omit required future state; supplier codec bounds
and the selected supported metadata schema govern admission.

## Deterministic transfer and budgeting

Select an extent ceiling `E` in the original plan, no greater than the proposed
1 MiB product ceiling and the qualified provider envelope limits. It is immutable
for that operation. Region order is module, heap, stable. For a region of length
`L`, emit ascending `(offset, min(E, L - offset))` extents until exactly covered.
A zero-length region emits no network requests. All arithmetic is checked.

Then emit one full-chunk request for each chunk hash in lexicographic byte order.
There is no offset/range parameter for a stored-chunk read in Platform's DTO.
Reject duplicates and invalid hash lengths. An empty stored chunk is still a
named chunk: read, verify, upload and read back it once; it is not an empty region.
The whole-chunk cap is also `E` in this profile. A larger chunk refuses; never
fabricate a partial-chunk protocol or silently change `E`.

Metadata supplies region sizes and chunk hashes, not chunk lengths. Before data
reads reserve `C * E` bytes for `C` stored chunks, plus exact region lengths. The
conservative sum must fit the original payload ceiling. Record actual chunk
lengths after bounded reads and verify their SHA-256 identities. The derived
schedule and its commitment are durably recorded before data transfer, bound to the original plan and metadata evidence. This refines
the plan's conditional schedule without replacing the plan or its approval.
Unused capacity does not authorize extra calls. This can reject a snapshot whose
actual small chunks would fit; relaxing that rule requires a separately designed admission
profile. Restore has exact lengths from the verified source.

Let `N = ceil(module/E) + ceil(heap/E) + ceil(stable/E) + C`. The sequence above
requires `N + 5` management calls for capture and `2*N + 7` for restore, before
request-status observations or additional provider authentication requests.
Zero-length regions contribute zero. Count whole chunks even when their byte
length is zero. The restore formula includes all upload and readback requests;
a successful upload response alone never authorizes load.

A route profile supplies the finite per-step outer submissions, management calls,
status/receipt lookups and authentication requests. Sum these over the schedule
with separate purpose limits inside the proposed 8,192 total request ceiling.
For direct execution, a management submission and its underlying host request
are one request, not two. For a Canic executor they are two distinct network
hops and both count. Signing invocations have a separate local budget. No routine
polling or certificate fetch is free merely because a supplier hides it.

Reserve byte budgets for encoded arguments, replies/certificates, decoded metadata,
ledger frames and retained artifacts independently. Derive worst-case evidence
space using each selected provider's encoded-size bounds, not `N * E` alone.
The candidate 1 GiB payload ceiling is not a promise that every fragmented shape
or provider fits the request/evidence ceilings. Reject inconsistent plans before
remote effects; capture performs the metadata-dependent admission after its one
take and metadata read. Restore completes admission before allocation.

## Canic executor obligations

The required join is `(Backup operation, plan digest, step, attempt, executor,
authority generation)`. Canic durably binds it to the admitted method, target,
argument digest, limits and management-call reservation. Same join with different
bytes refuses. Duplicate delivery returns the original result or pending state;
it never emits a second management call. The host still submits only once.

The executor must reserve before the inter-canister call, retain success/rejection
in its continuation and survive upgrades without losing pending liabilities.
A callback trap or lost continuation stays uncertain. Restart, upgrade or host
receipt lookup cannot turn it into permission to call again. Qualification must
cover these boundaries as well as a lost host reply. A completed executor receipt
must include enough original evidence for Backup to settle its exact request.

This ledger remains outside the target rollback set. Generation changes do not
refresh old attempt authority. A new provider cannot reinterpret a pending old
protocol receipt. Backup refuses the route until Canic supplies this protocol
and Platform publishes a qualified Wasm snapshot DTO/codec surface. No local
management declarations or controller-topology changes bypass that dependency.

## Implementation acceptance

These decisions are ready to guide implementation; they are not qualification.
The local-store contract now defines the proposed frame limits, completion
publication, source references and terminal target-claim succession. Byte-exact
record fields/commitments and independent vectors must accompany the first
writer/reader. Do not claim compatibility from the semantic names in this draft.

The first maintained consumer should exercise stopped capture through verified
bundle publication, then the same public boundary through complete readback/load.
Use published Platform and Host dependencies at implementation time. Keep Testkit
private and add its workspace only with the real fixtures. Acceptance must cover:

- Loss of signing output and replies, cancellation at each durability boundary,
  process death after reservation, expired status and effect-free terminal replay.
- Missing globals/lifecycle fields, NaN payloads and signed zero, new destination
  provenance, zero-length regions/chunks, duplicates, unknown sizes and budget
  exhaustion before the next effect.
- Full destination-byte mismatch refusal before load, successful load with lost
  final status, and custody loss with outcome reconciliation but no further effects.
- An independent Rust consumer and a Canic adapter contract using external signing;
  actual manager-controlled capture/restore additionally requires the executor,
  its two-hop failure tests and Canic's fenced reopening journey.

## Recovery continuation after the 0.101.3 checkpoint

The post-push review of `d8955cf` inspected strict lookup admission, the shared
signature verifier, exports, direct-provider callers, regression tests and scope
claims. No actionable defect or duplicate policy owner was found in that delivered
scope. The missing durable workflow is an explicit limitation, not an implemented
capability inferred from the two local authentication helpers.

The implemented [single-lookup workflow](../architecture/capture-intent.md#durable-single-lookup)
retains one ledger owner. Existing status/list/take
events use fixed transition ordinals (1–12); inserting observations must preserve
those original bytes and distinguish mutation ordinals from observation attempts.
Do not renumber existing records, append public arbitrary events or add a second
journal. Observation allowance already belongs to the original plan; signing,
network requests and retained proof must also fit its cumulative limits. A pending
lookup cannot be retried merely because its response was lost. The new event encoding and replay admission have native interruption tests and
a real pending-take read-state fixture. Explicit original-call settlement from retained terminal evidence now records
its observation source and preserves historical authentication. An observation
alone still authorizes no dependent effect.

## Next payload acquisition slice (not implemented)

Use the retained transfer binding and exact ordinal as the data-read identity.
Reauthenticate original capture/metadata and compare the schedule before new
work. Admit one sequential read at a time, with fresh integration-owned read
permission and complete request/proof/payload bounds. Retain its admission and
reserve signing before invoking the signer; retain exact signed bytes before
reserving submission. Reuse one-shot HTTP and the scheduled data authenticator.

Retain certified result evidence before recording an accepted payload. A certified
rejection or invalid payload cannot count toward coverage. Failed signing and lost
HTTP replies remain spent and unresolved; no automatic replacement, polling or
range splitting is permitted. A complete uncertain journal frame survives reopen;
partial frames and orphan files refuse without repair. Replaying an accepted read
must authenticate the original evidence and leave journal and payload bytes intact.
Use the original journal and held-file custody owners; no per-extent journal or
second mutable progress index. Exact evidence layout and cumulative bounds must
be implemented and qualified before this flow dispatches. Final coverage, module
hash, final status and durable bundle completion remain subsequent work.
