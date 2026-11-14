# Single-canister operation contract

Proposed effectful contract, 2026-11-14. Capture/restore remains unimplemented;
[local inventory](../architecture/local-inventory.md) and
[capture intent](../architecture/capture-intent.md) and
[pure transfer planning](../architecture/capture-transfer.md) are implemented subsets.
Intent creation does not yet reserve attempts or authorize any provider calls.
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
