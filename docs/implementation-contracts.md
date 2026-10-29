# Backup implementation contracts

Read the relevant sections before changing planning, persistence, transport or
recovery. These are current obligations. [The extraction boundary](extraction-boundary.md)
specifies record and API details; [the design](extraction-design.md) retains
rationale and proposed complete-product behavior. It supplies no additional
implemented recovery modes or current status ledger. Dependency selections belong
to manifests and the lockfile, migration history to the
[changelog](../CHANGELOG.md), and qualification receipts to the
[current handoff](status/current.md) and its archives.

## Scope and authority

- Backup runs on the operator host. Core owns records, bounded spending, guarded
  persistence, reconciliation and coordination. Agent owns reserved signed updates.
  Applications own membership, routing, release identity, quiescence, fencing and
  external-effect settlement. No standalone CLI or complete application runner is
  installed by these libraries.
- Restore targets the same existing canister IDs on the same network and release.
  Creation, relocation, cross-release migration and identity rebinding require
  separate authority. Keep one maintained product protocol/schema generation, v1;
  preserve historical evidence without adding compatibility readers or aliases.
- Persist exact intent and consume original bounded authority before effects.
  Missing records never mean unused allowance. Cancellation and lost replies keep
  spending pending; neither unlock, reconstructed requests nor error text permits
  redispatch, refund or reset. Uncertain means a qualified settled observation
  cannot resolve a mutation; a lost observation remains pending.
- Matching hashes, decoded replies, Applied projections and sequential local byte
  checks do not authenticate evidence or establish fresh permissions, stable
  noncooperating custody, complete transfer, application acceptance or terminal
  authority. Integrations must qualify those facts at their actual boundaries.
- Restoration cannot rewind payments or external obligations. Keep original
  journals, artifacts, partial trees and references until explicit safe disposition.
  Terminal replay is local and effect-free; fresh verification is separate. Local
  prune and live snapshot deletion are separate operations. No prune is implemented.

## Dependency and identity boundaries

Platform owns all Candid, Principal, management DTO and Agent SDK imports. Core
selects `candid` and `management-canister-types`; only Agent adds `agent` and PEM
support. Internal principal admission uses Platform's owned `Principal`; external
Candid DTOs and SDK calls retain upstream 0.1.5 through Platform's contract or
external exports and explicit conversions. Preserve management DTO 0.11.0,
canonical text/raw bytes, wire formats and hashes. Direct upstream declarations
and local pin exceptions are forbidden. Changes require the compatibility and qualification review in
[AGENTS.md](../AGENTS.md). The selected Agent SDK is 0.49.3.

Host Artifacts owns accepted-byte hashing, typed SHA-256 and declared-directory
checksum composition. Host FS owns descriptor IO, `FileIdentity`, sidecar locks
and typed publication. Host Process owns descriptor inheritance and quiescence.
Backup selects product bounds, framing, custody and recovery transitions.
`PersistenceError::Publication` carries Host FS's
`NamedWriteError<std::io::Error>` directly; preserve producer, cleanup and
before/after-publication evidence. Public dependency identities follow the owning
workspace selection; consumers sharing those types must align their dependencies.

Testkit owns authenticated simulator assets and managed server processes; it is
only a development dependency. Shared Tooling owns common reviewed tooling.
Reqwest configuration remains Backup's explicit transport policy. Generic document
IO uses caller-bounded encoding and Host FS; product consumers use guarded,
record-specific operations. Finite generic JSON value writers are test fixtures.

## Records, plans and spending

Preserve these bounds, strict version/field admission and canonical encodings:

| Owner | Maintained limits and identity |
| --- | --- |
| Inventory/selection | 1,024 targets/selectors, 256 UTF-8 role bytes, 1 MiB IO; unique normalized principals, required nullable fields and closed acyclic parent forest |
| Effect graph | 8,192 nodes, 1,024 dependencies per node, 65,536 total edges, 1 MiB IO; unique opaque sequences and closed acyclic dependencies |
| Operation plan | Exact network/caller/release, full inventory, selection, graph and one target/request/immutable budget per node; 65,536 combined attempts and 1 MiB IO |
| Attempt journal | 1,024 combined attempts, 2,048 events, 1 MiB IO; immutable authority/budgets and chronological reservations/receipts |
| Workflow/stage binding | 1 MiB workflow, 512 KiB binding, 1,024 predecessors; exact catalog purpose/target/ceilings and learned child plan |
| Execution settlement | 8,192 rows, 2 MiB IO; full original plan plus exact complete chronological journal fingerprints |
| Consistency, fence obligation, restore requirement | Strict immutable v1, 1 KiB IO; exact original plans and requirement/fence/revision bindings |
| Download manifest | Existing v1 download-journal schema, 1 MiB IO, 1,024 artifacts, 256-byte tokens; exact metadata, derived paths and checksums |

Preserve hash domains, field order, length framing and integer byte order,
including Agent's little-endian attempt IDs. Directory framing orders relative
UTF-8 path components and emits path/NUL/lowercase-digest/newline; malformed or
duplicate rows reject through Host's `DirectoryChecksumError` with Backup's error
projection. Declaration hashing performs no IO or durability check. Checksum JSON
still requires/emits `algorithm: "sha256"` in its original field order even though
memory retains only the normalized hash. Independent tests may use `sha2`.

A physical inventory does not imply dispatch/lifecycle order. Progress requires
exact complete original journal coverage, authority/context/request/limits and
Applied prerequisites for attempted operations. Equal final views cannot hide
changed chronological history. Unassigned aggregate headroom never becomes an
operation's allowance. Original-plan progress and settlement share sequential
journal admission; hold no unrelated attempt guards when entering it.

## Workflow preparation and checkpoints

`ExecutionStageGuard::prepare` is the sole public fresh-stage constructor;
record-only allocation remains private. Catalog rows declare stage purposes and ceilings; only bound child plans own
attempt journals. Keep exact target/context/full inventory, original stage budgets,
fixed create-only directories and predecessor bindings. Before allocation, derive
all authorities and encode/admit plan and binding bytes. Fresh preparation creates
every original journal with one lock at a time and re-admits predecessors afterward.
Preserve partial records on failure/death. Occupied preparation refuses; reopen
never repairs missing journals, rebuilds plans or resets spending. Record-only
inspection stays distinct from complete-journal resume.

Stage admission checks every transitive ancestor's retained binding/plan and exact
Applied settlement histories, iteratively bounded by catalog size. Shared edges
must agree on binding/settlement; learned-input commitments remain edge-specific.
Recheck original stage/ancestors around effects and after checkpoint publication.

`reconcile_execution_settlement` and `ExecutionStageGuard::reconcile_checkpoint`
require complete original Applied histories. Derive and recheck exact fingerprints
before writing. Exact occupied checkpoints replay without replacement; absent
checkpoints may be created. Changed or partial evidence refuses without repair.
Hold no attempt guards during checkpoint admission. Read-only replay never writes.
Checkpoints prove local settlement only, not authentic receipts, cross-journal
chronology, backend completeness, application safety or terminal release.

Fresh workflow execution refuses occupied evidence before driver callbacks.
Retained continuation requires the complete integration-supplied occupied-stage
set and replays every settled checkpoint before any new driver callback. Validate
frozen predecessor commitments for each exact successor edge. New edges default
to the predecessor checkpoint digest; retained overrides must name original edges.
Integrations own learned-input qualification and actual effects. Partial/pending
originals refuse; no automatic redispatch or second spending owner is installed.

## Filesystem publication and command custody

- Keep layout lifetime exclusion, 0700 parents and 0600 regular no-follow sidecars,
  no-wait contention, unsafe-entry/native IO projections, close-on-exec and explicit
  non-spawning unlock on drop. Resolve journal parents once; absent or regular
  leaves are admitted, aliases rejected. Preserve caller-specific missing-name errors.
- Product JSON is bounded and encoded once before effects. Mutable guards encode
  before invalidation, then publish those exact bytes. Reopened canonical-size
  admission uses a counting sink. Host owns atomic create-only/replace, staging
  identity/cleanup and final parent sync; keep synchronized pre-publication and
  durable-completion crash barriers and the shared production/test adapter.
- Private-parent preparation refuses observed symlink ancestors and `..` before
  creation. Intentional root aliases must be resolved at input admission. Host
  rejects NUL paths before publication effects. These checks grant no root
  confinement or immunity to namespace races.
- Command custody preserves v1 filesystem identities, journal/operation binding,
  the descriptor environment name and 250 ms grace/5 ms polling. Existing sidecars
  and inherited locks bind descendant custody. Group cleanup, simulator reset or a
  managed process exit cannot substitute for command quiescence.
- Preserve original partial/canonical trees after publication errors. Recovery
  synchronizes and admits retained evidence; it never recreates spending authority.
  Persistence guards retain no diagnostic synchronization or metrics state;
  instrumentation cannot add filesystem passes or alter ordering, errors or effects.
  Keep the unsafe-code prohibition.
- Bounded Unix JSON reads use Host regular-file admission with original byte limits,
  JSON decoding/model admission, growth/shape rejection and Backup error projection.
  Retain bounded retained-file synchronization where durability recovery requires it.

## Application admission and fencing

Membership requests bind original plans, independent challenges and actual full
inventory/context. Their 1,024 descriptive remote-observation ceiling is not spending
authority. Mandatory async workflow admission callbacks obtain fresh evidence and invoke the
corresponding pure policy; no synchronous observation-provider interface is installed.
Integrations own authentication, freshness, prior per-call accounting and
opaque revision meaning; matching revisions do not prove continuity or a fence.

Control observations require the selected caller itself in the complete canonical
controller set, at most 10, with exact request/context/target and descriptive call
limits. Root proxies, subnet-admin exceptions or read/status visibility do not
supply this lane. Snapshot-read observations separately admit public or exact
viewer access (at most 10 viewers); unobserved controllers, status/log flags and
Root configuration cannot substitute. Actual fresh permission providers remain
integration-owned.

Consistency requires exact full inventory/selection, every target stopped and the
requested lane. Coordinated capture additionally needs the original Active fence
and identity/membership/external-obligation revisions. Restore requirements bind
same-network/release/ID source and restore plans plus retained artifact digest;
subsets require application qualification. Load needs every target stopped. Start
needs its target stopped, accepted restored state for the whole selection and
qualified fenced execution. The no-irreversible-effects lane has no generic default;
the fenced lane requires rewind-independent custody and replay safety. Preserve
1,024 actual target rows and descriptive call bounds, distinct from spending.

Fence obligations bind original acquisition operations and purpose-specific
capture/restore requirements under retained layouts, including outside-snapshot
restore custody. Retain obligations for pending, Applied, NotApplied and Uncertain.
Acquisition uses one previously accounted update with canonical receiver, replicated
mode, 1..128 visible ASCII method bytes and at most 1 MiB opaque arguments. Original
payload admission precedes spending. Hold the journal across fresh whole-unit
application admission and submission; recheck stage/requirement/obligation around
it. Passive replies produce no receipt or Active custody. No default application
semantics or observation provider is installed.

## Management wire and response admission

Candid is required for management calls. Platform owns exact single-result decoding
and decoder setup; Backup owns bounded DTO visitors, raw/work/skip/type/header
ceilings and redacted errors. Preserve exact codec/golden bytes and public upstream
Rust identities. Ordinary management requests retain 256 raw snapshot-ID bytes,
4 KiB arguments and 8 KiB JSON IO. Capture retains code; host ingress omits sender
canister version, which is never a target-version guard. Raw IDs are distinct from
generic backend tokens.

- Capture/list replies retain required nat64 timestamps/sizes, 1 MiB raw input,
  1,024 entries, 256 ID bytes, 16 type entries and finite decoding work. Reject
  duplicate IDs; evidence hashes the exact raw reply.
- Lifecycle/status replies preserve canonical empty stop/start/load tuples, required
  status/settings/controllers, 1 MiB raw input, 2 MiB decoding work, 64 KiB skipped
  work, 64 type entries and 10 controllers. Unknown metadata is skipped, never
  qualified; missing required fields reject. Status/list headers retain 64 KiB bounds.
- Metadata replies preserve 1 MiB raw, 2 MiB work, zero skipped work, 64 type entries,
  16 KiB headers, 4,096 ordered global slots, exact floats/unsigned 128-bit globals,
  1,024 distinct ordered 32-byte chunk hashes and 32 certified-data bytes. Optional
  source/global/timer/hook absence never implies upload permission or defaults.
- Data reads preserve checked nat64 offset/size arithmetic, nonzero ranges of at
  most 1 MiB, 2 MiB raw replies and known bounded chunk hashes. Reply lengths must
  exactly match ranges; known empty chunks require their exact hash. Preserve the
  data decoder's original work/type/zero-skip quotas and 4 KiB header bound.
- Uploads bind original source plan, exact metadata, durable tree checksum and wire
  bytes. Reject unavailable globals; preserve float bits and optional timer/hook
  absence. Replacement stays absent. Each at-most-1-MiB write binds its own original
  plan/reservation and a distinct 1..256-byte destination. Preserve 2 MiB argument,
  4 KiB reply/header and original finite decoder quotas. Empty replies retain the
  exact six-byte comparison.

`IcUpdateResponseInput` shares ordered attempt/byte/target admission and redaction;
wrappers retain method-specific errors and ceilings. Recheck current reservation
before ordered authority/attempt/context/target association, then perform bounded
method decoding. Decoded status, IDs, matching bytes or inventory cardinality never
supply an authenticated outcome or automatic receipt.

## Snapshot artifacts and transfer

Download journals retain immutable snapshot identity, model-owned four-state
transitions and derived resume views under layout exclusion. Coverage retains three
contiguous nat64 region cursors and at most 1,024 chunk bits under original metadata
request/raw evidence. Reject gaps, overlaps and duplicates without state changes;
regions may interleave and chunks may arrive in any order. Empty regions need no
reads; known empty chunks still need exact replies. Reconstructed coverage starts
empty and supplies no partial-transfer resume or journal authority.

The private v1 artifact writer retains exact metadata wire/arguments and fixed files.
Host FS creates no-follow named files and synchronizes the staging directory before
returning; content still needs its own sync and qualification. Regions use Host
`HashingWriter<File>` while Backup checks held identities, exact extents and original
coverage. Preserve 1 MiB per chunk, three incremental region hashes, 1,024 chunk rows,
original metadata/journal bounds and checksum framing. Member expectations own
names, lengths, maximums, optional known hashes and payload roles. Regions acquire
actual hashes; no expected region digest is invented.

Append consumes the writer on error, retaining partial Created bytes. Occupied
staging refuses recreation. Full byte verification durably retains the expected
checksum before publication; recovery uses ChecksumVerified state. Retained IC-tree
verification requires original persisted plan, unchanged guarded journal and complete
Durable selection; admit exact metadata timestamp, wire/request hashes, region
lengths and known chunk hashes. Keep both closed-tree checks, no-follow bounded
reads and closing directory/file/journal identity/extent admission.

Manifest publication verifies original-plan-bound bytes before create-only
publication. Replay admits only exact retained local declarations and never reads
trees or calls providers. Fresh local restore-source verification binds both plans,
requirement, exact manifest and unchanged journal under both layout lifetimes;
check every original source artifact, including outside a restore subset. Generic
integration artifact digests keep their meaning. Preserve 1 KiB requirements,
1 MiB IO, 1,024 artifacts and 256-byte tokens. Source/manifest verification never
changes progress, references or allowance.

IC upload reads verified original artifacts directly. Generic restore-copy staging,
publication and verification are retired. Preserve old copies/metadata; unfinished
copy-based restores require their original released tool or explicit safe disposition.
Historical contracts do not authorize another reader or cleanup.

## Async effect coordination

Every coordinator admits original payloads before spending, reserves through the
complete-plan owner, holds selected journal exclusion across mandatory fresh
admission and one provider await, then re-admits the journal, associates bounded
returned evidence and rechecks stage/ancestors. Fence steps also recheck requirements
and obligations. Missing, held, pending or Applied originals deny fresh dispatch.
Return exact bounded replies in typed post-reply errors. Core supplies no executor
or `Send` requirement. Retain exact replies before cancellable qualification work.
Cancellation preserves pending/Applied spending, earlier receipts and partial bytes.

Capture accepts only original take requests and returns passive pending evidence.
Metadata coordination requires an exact singleton stage and independently qualified
byte retention/attribution before its explicit Applied receipt. Learned evidence
binds exact request/raw metadata; successor preparation remains explicit.

Download planning preflights region/chunk counts with checked arithmetic against the
original at-most-1,024 combined stage attempts. Preserve module/heap/stable/chunk
order, sequential dependencies, one mutation/zero observations per read, exact
singleton metadata predecessor and reply evidence. Read-free metadata creates no
placeholder operation. The data loop rejects consumed stages and changed writer,
metadata or intent. Append admitted bytes before recording each independently
qualified exact Applied receipt, then admit the next read. Finish via the existing
durable publisher; retain the checksum on closing rejection. No partial-read resume
or automatic manifest/checkpoint follows.

Upload planning preflights all extents against original capacity before source IO.
One bounded batch retains canonical payload digests from exact bytes, verifies the
retained tree checksum and brackets the batch with full source admission. Bind the
exact allocation reply and Applied predecessor before publishing the data plan;
metadata allowance supplies no data allowance. Complete upload execution brackets
one session with full source verification. Each write checks the closed tree and
file identity/length/change observations and freshly hashes its bounded payload
against the immutable original request commitment. Metadata observations are drift
checks, never content authority or a fence against noncooperating writers. Final
full verification precedes checkpointing; independent stable byte custody remains
mandatory. Standalone payload preparation retains its full per-call verification.
Reject prior consumption
of the complete original data stage before any new write.

Allocation and data uploads use the same single-upload owner through independent
qualification and the sole Applied receipt transition, retaining the original guard
and admitted decoded reply. Qualification must authenticate exclusive original
ingress attribution and durably retain request/reply bytes. A returned ID or empty
reply never supplies that qualification. Stop on every error. Release the journal
before checkpointing canonical request/raw-reply evidence; final data evidence
covers the ordered original plan. Checkpoint failure retains Applied histories and
acknowledgements and never permits repeat allocation/upload.

Singleton same-ID load/start rechecks retained source plan and exact safety
requirement under both layouts before spending and across effects. Fresh admission
returns actual evidence to restore-safety policy before one update. Independent
authentication, attribution and durable byte retention precede the explicit exact
Applied receipt. Checkpoint only after guard release. A load receipt never supplies
later restored-state acceptance. Coordinators establish neither complete backend
transfer nor product terminal/fence/reference release.

`IcRestoreSource` additionally binds the exact snapshot identity independently of
the load plan being admitted. Existing snapshots use their qualified metadata
request. Uploaded snapshots reuse the exact source-derived data plan and complete
original stage checkpoint, including allocation ancestry. Reopen and re-admit that
evidence across admission/settlement; no incomplete or pending stage supplies a
source. An all-empty upload binds the exact Applied allocation receipt without a
placeholder data stage. Same-workflow loads require that exact upload predecessor;
separate restore workflows retain explicit source provenance. These local checks
authenticate no backend bytes or application fact and supply no new authority.

The connected fixture freezes separate capture and restore catalogues before
capture, binding restore's purpose to the original capture allocation and its
learned requests to exact retained source evidence. The source layout is outside
the restore catalogue's ancestor chain, so source custody and ancestor locks can
coexist. Its local completed-workflow replay supplies no product terminal or
reference-release authority. See [qualification scope](pocketic-qualification.md#connected-original-capture-and-restore-workflows).

## Explicit recovery settlement

Recovery observations bind full original plans, original mutation bytes and both
already reserved attempt IDs to exact status/list/readback arguments. Preserve
original 1,024-attempt, 4 KiB argument, 1 MiB ordinary reply and method-specific data
bounds. Reserve observations separately before calls; remote preflight and
qualification also require independent prior accounting. No automatic observation,
receipt, repeat call or allowance reset is installed.

Settlement identity denials share the public `policy::SettlementIdentityMismatch`
and `policy::InventorySettlementMismatch` causes, wrapped by each settlement
policy's `Identity` error variant. Reservation and wire admission precede claim
comparison; capture/allocation closed-baseline admission precedes identity checks.
Status/readback, attribution and destination denials remain domain-specific.

- Capture and metadata-allocation settlement require exact original/current list
  targets, full baseline IDs/timestamps/sizes unchanged, original authority/both
  attempts, fresh challenge, exact list/raw digests and opaque observation evidence.
  Applied explicitly attributes a bounded new ID; allocation additionally rejects
  the source ID. Zero/one/many candidates imply no outcome. Negative evidence must
  exclude transient capture/allocation followed by deletion. Authenticate chronology,
  exclusive attribution, original baseline custody and current read permissions.
- Data readback binds original source/upload plan, exact destination/extent/hash and
  known uploaded metadata/dimensions. Matching bytes alone never prove a write.
  Applied additionally requires exclusive original-write attribution; NotApplied
  requires qualified exclusion even when preexisting bytes match. Bind both attempts,
  challenge, exact metadata/readback and observation evidence. Settled uncertainty
  differs from failed or lost reads; byte/read cardinality grants no inference.
- Stop/start/load settlement binds original authority, attempts, challenge and exact
  status/evidence. A state-equivalent current lifecycle is insufficient. Load needs
  independent complete restored-state verification; negative evidence excludes
  transient application. Qualified settled ambiguity keeps the mutation unresolved.
- Fence reconciliation binds original obligation/authority/mutation/challenge to one
  separately reserved observation, at most one descriptive remote call and 1,024
  actual target rows. Acquisition needs exclusive original-request attribution and
  exact Active revisions; current presence/absence alone cannot settle it.
  Explicit late-reply settlement reopens original evidence, admits complete progress
  and holds the journal through qualification. Require exact attempt/request/outcome/
  evidence before recording one receipt, with no reservation or provider dispatch.
  Recheck stage/requirement/obligation before and after recording. Uncertain closes
  the observation while retaining the mutation. Write or closing-admission failure
  may leave a receipt; replay originals to resolve it without repeating a read.

## Terminal evidence and reference release

`BackupLayoutGuard::release_restore` requires actual held layout/journal guards,
fresh bounded terminal bytes, exact original intent and the integration's complete
retained command-custody set. Preserve 16 MiB terminal input and 8,192-command bounds.
Integrations authenticate their journal, settle effects/application obligations and
qualify completeness. Applied counters alone are insufficient. Validate the complete
command set and reject duplicate or cross-journal identities before probing custody.
Reject missing/replaced sidecars and active descendants.
Hold existing inode-bound quiescence guards through durable reference publication,
recheck terminal bytes/layout and synchronize exact absent-reference replay.
Providers validating a workflow journal can call
`read_complete_execution_workflow_locked` with that same held `JournalLock`;
the complete original stage replay retains its ordinary bounds and digest checks.
This API performs no IC effect, application fence release or artifact deletion.

## Agent transport and qualification

`AgentUpdateProvider` implements mutation and source-bound upload ports through
`ReservedUpdate::Mutation`/`Upload`; the closed reserved request family also covers
transfer reads, recovery/readback and application acquisition. Preserve each request
validator, reply ceiling and evidence domain. All SDK imports use Platform.

Opaque application ingress uses `ApplicationIngressPayload` and distinct original
mutation/observation request types. Bind non-management receiver and effective
target, fixed update mode, 1..128 visible ASCII method bytes, at most 1 MiB arguments
and a caller reply ceiling of 1 byte through 1 MiB in the original payload digest.
Recovery uses its separately reserved digest and observation allowance; no fence semantics
or application codec is inferred. Both synchronous durable retainers are mandatory
in `AgentApplicationIngressProvider`. Retain exact signed bytes/ID before dispatch
and exact bounded reply/ID before returning to cancellable qualification. Original
reservation-kind/attempt witnesses must refuse prior or uncertain dispatch. Passive
responses do not settle either host or Root-paid effects. Existing custom request
hashes and pending layouts cannot be rebound into this new payload declaration.

Configure exact context/caller, HTTPS or literal loopback HTTP origin, external
trusted root and finite 1..300s timeout/ingress expiry. Keep HTTP/1, no proxy,
redirects, HTTP/TCP retries, implicit waits/polling or root fetch. Custom Reqwest
middleware bypasses Agent's default 429/503 retry wrapper. Preserve 4 MiB HTTP,
3 MiB signed-envelope and smaller method/application reply ceilings. Signing
provider resource/custody limits remain explicit integration responsibilities.

Validate original journal and signer, sign without network IO, then require durable
retention of the exact signed envelope/request ID with original plan/reservation
before submitting once. `PreparedUpdate` is a passive retention view; no unreserved
call, envelope re-import or reissue API exists. Certificate replies are passive;
accepted/processing returns Indeterminate without polling. Failed/oversized/lost
replies and retention failures preserve original spending. `TransportError` owns
construction failures; ports own pre-dispatch refusal and indeterminate outcomes.
The Agent needs a network/timer executor; Core remains runtime-free.

The [transport guide](agent-transport.md) owns configuration details and the
[PocketIC guide](pocketic-qualification.md) owns simulator qualification scope.
Fixtures install and retain exact handwritten-WAT-derived bytes without product
optimization. They keep original plans, ingress/reply bytes, journals, hashes,
references and fixture-specific application safety. Core and Agent call locked
Testkit's offline asset admission before managed startup and retain the admitted
absolute server's bounded exact digest and private output. Setup is explicit;
Make retains the separate CLI/tool gate. No reset/reuse/funding/retry policy is
adopted from Testkit. Fixture setup, application calls and pre-load chunk-store
changes are independently accounted outside backup authority. Original isolated
attribution qualifies no default application lane. Native macOS, installed-IC and
complete consumer acceptance remain separate from local fixtures and owner CI.

Package validation checks both exact unpacked Cargo archives in an independent
normal consumer with original selections and immutable bytes. Its fixture-only
Core override prevents same-version registry fallback; root manifests/lock and
sibling repositories stay unchanged. The [release guide](releasing.md) owns release
and publication authority. Tools, passing tests and package checks authorize no
commit, release, deployment or destructive cleanup.
