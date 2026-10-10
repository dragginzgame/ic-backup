# AGENTS.md

This file is normative for automated agents and contributors working in
`ic-backup`.

## Shared baseline and local overlay

Apply the engineering sections of [the Dragginzgame baseline](DRAGGINZGAME.md).
This is a reviewed local snapshot of Shared Tooling's committed rules, not
implicit inheritance from a moving sibling checkout.
[The snapshot manifest](.shared-tooling.snapshot) binds the common baseline and
tooling to one committed revision; [adoption and provenance](docs/shared-tooling.md)
describe the review. Refresh deliberately; never edit vendored tooling in place.
Exclude every path declared in `.shared-tooling.snapshot` from local branding
and documentation rewrites; keep consumer guidance in unlisted local documents.

This file is the product-specific overlay: backup contracts, effect targets,
numeric bounds, consumer adapters, supported hosts and qualification gates stay local.
Release entry points follow the common command contract.
Common engineering rules are mandatory; record maintainer-approved exceptions
with their scope and reason. GitHub issues also own reusable feedback, with the
baseline's evidence fields instead of a duplicate feedback file.
Shared Tooling's own validation commands apply upstream, not to this repository.

## Start of session

Read `docs/status/current.md` first, then the relevant sections of
`docs/extraction-design.md`. Distinguish proposed behavior from implemented and
qualified behavior. Update the handoff after a meaningful batch; do not turn it
into an issue tracker or release authority.

## Scope and authority

- Edit only this repository. Canic and every other sibling repository are
  read-only unless the maintainer separately authorizes changes there.
- The initial request authorizes creating this named repository and writing its
  design/instructions. It does not authorize extracting source, changing Canic,
  performing live backup/restore, publishing packages or creating a remote.
- Ordinary implementation and continuation leave work uncommitted for review.
  Explicit commit, contribution-PR and release requests follow the reviewed
  [contribution rules](rules/contributions.md); PR delivery does not authorize
  merging, direct integration-branch pushes or a release. Ordinary continuation
  requests no Git writes or release effects.
- Continue accepted implementation batches autonomously when requested. Generic
  continuation does not authorize live effects, release/version transactions,
  pushes, package publication or destructive cleanup.
- Preserve dirty worktree state and retained recovery evidence. Do not discard
  unfinished journals, snapshots or artifacts as ordinary cleanup.
- Do not add local Cargo patches pointing at sibling checkouts, shared target
  directories or dependencies on Canic runtime/control-plane crates.

## Current implementation scope

- JSON publication delegates allocation, atomic create-only/replace, staging
  identity/cleanup and final parent sync to the published typed Host descriptor
  engine. Retain Backup's 0700 parents, 0600 files, serialize-once-before-effects
  byte buffer, original record bounds and one adapter for production/crash tests.
  Preserve synchronized pre-publication and durable-completion barriers. The
  public `PersistenceError::Publication` retains original producer, cleanup and
  visibility evidence; no local-write error grants paid retry or resets spending.
  The Host 0.5 error identity was delivered in 0.7.0. The externally selected
  Host 0.7 graph changed that exposed Rust identity again in released 0.8.0.
  Released 0.8.1 qualifies Host 0.7.1. Released 0.9.0 qualifies the Host 0.8
  exposed Rust identity. Released 0.11.0 selects direct Host 0.9. Released
  0.12.0 selects direct Host 0.10 artifact/FS packages. Released 0.13.0 selects
  direct Host 0.11, changing that public Rust identity again; consumers sharing
  publication errors must align Host 0.11. Pending 0.14.0 selects direct Host
  Artifact/FS 0.12, changing that public Rust identity without a library behavior
  or record change. Consumers sharing publication errors must align Host 0.12.
  Testkit 0.31 retains its independently owned test-only Host 0.11 graph.
  No compatibility adapter or record change follows. The direct process
  dependency retired with its sole ICP probe caller; Testkit's dev-only managed
  server owner does not grant backup command custody or spending authority.
  Local bounded capture retains its direct-child semantics. Group cleanup does not
  replace inherited descriptor custody or prove descendant quiescence; bounded
  test capture retains its separate direct-child semantics.

- Pure directory checksum composition accepts exact canonical relative UTF-8 file
  identities and validated checksums through the same owner as local artifact
  traversal/publication. Preserve original path-component ordering and path/NUL/
  lowercase-digest/newline framing. Reject malformed and duplicate rows through
  `DirectoryChecksumError` without expanding `ArtifactError` or reopening paths.
  Declaration hashing proves no byte custody, sync, completeness or publication.
- The maintainer authorized Rust repository setup following `ic-delegated-auth`
  and `ic-blob-storage`. The root is workspace-only under the
  [shared workspace layout](rules/rust-workspaces.md); the initial library lives
  in `crates/ic-backup/`. Add transport and CLI packages when their behavior is
  implemented, rather than creating placeholder packages.
- The repository foundation does not qualify backup/restore or complete B1.
  Follow the extraction design's inventory and contract work before importing
  the engine. Canic adoption remains separately authorized downstream work.
- The maintainer authorized copying backup machinery from Canic. The first
  accepted extraction covers local artifact checksums, secure staging, durable
  publication, bounded JSON IO and journal locks. Read
  [the implemented boundary](docs/extraction-boundary.md) and
  [fresh source provenance](docs/extraction-source.json) before extending it.
  Runner/authority contract work remains part of B1 before importing runners.
- Continued extraction includes layout lifetime exclusion and durable restore
  reference retention. The v1 records and bounds are documented in
  [the implemented boundary](docs/extraction-boundary.md). No reference release
  or prune operation is implemented; terminal proof and command custody must own
  release admission before that API is added.
- Command custody now uses owned descriptor inheritance, exact v1 filesystem
  identity records and fresh non-spawning quiescence guards. Preserve the
  unsafe-code prohibition. These local contracts do not qualify an authenticated IC backend
  or grant terminal/reference-release or paid-call authority.
- Local download journals now retain immutable snapshot identity, model-owned
  four-state transitions and derived resume views. Persistence guards borrow
  layout exclusion and verify/publish exact local bytes. Download completion is
  an integration attestation; backend metadata/extent qualification and
  remote-effect reconciliation remain pending.
- Local attempt journals bind immutable operation identity and original mutation/
  observation limits, with chronological reservations and exact receipts.
  Preserve the 1,024 total-attempt, 2,048 event and 1 MiB input/output bounds.
  A persisted reservation grants no fresh authority or backend dispatch permit.
  Uncertain means a qualified settled observation cannot resolve its mutation;
  a lost observation reply stays pending. No allowance is refunded or replenished.
- Physical inventories now retain a bounded, canonical, closed parent forest.
  Preserve unique normalized principals, required nullable fields, acyclic edges,
  the 1,024 target/selector limits, 256 UTF-8 role-byte bound and 1 MiB IO bounds.
  Pure selection expands exact principals into read-only views in canonical order;
  that order grants no dispatch/lifecycle order. A declared inventory and its hash
  prove no fresh membership, controller custody or application consistency.
- Explicit effect graphs now retain unique opaque operation sequences and closed
  acyclic dependencies. Preserve the 8,192 node, 1,024 dependency-per-node,
  65,536 total-edge and 1 MiB persistence bounds. Graph order and causal declared
  progress are pure projections, not completion receipts or dispatch permits.
  Do not infer universal parent/lifecycle order from physical inventories.
- Operation plans now bind declared network/caller/release, original inventory,
  exact selected targets, dependency graph, request digests and immutable attempt
  allowances. Preserve one exact binding per graph node, selected-target coverage,
  the 65,536 combined plan-attempt ceiling and existing node/selection/IO bounds.
  Derive journal intent from the full canonical plan digest; repeated derivation
  never creates/resets a journal. Declarations grant no fresh preflight or executable
  request, consistency/lifecycle contract, receipt or dispatch permit.
- Execution progress now admits a complete exact set of original attempt journals
  under the plan digest, context, operation and original limits. Missing evidence
  never means zero consumption. Attempted operations require retained Applied
  prerequisites; projections cannot prove cross-journal dispatch chronology,
  authenticate receipts, grant fresh authority or establish terminal/reference release.
  Keep unused aggregate headroom out of assigned allowance totals.
- Local execution progress now reopens the exact persisted plan and complete original
  journals through one sequential-lock reader shared with settlement replay.
  Plan-bound mutation/observation reservations retain the selected journal lock and
  delegate spending to the existing attempt owner. Missing evidence never supplies
  zero consumption; mutation prerequisites need retained Applied outcomes. Preserve
  unchanged original limits, pending replies, source references and obligations.
  No new schema, provider, fresh authority, cross-journal chronology, terminal proof
  or dispatch/release permit follows. Hold no other attempt guards at admission.
- The IC request boundary now owns closed host-ingress management argument encoding
  and exact receiver/routing/method/update-mode/byte digests. Preserve 256 raw snapshot
  bytes, 4 KiB derived arguments and 8 KiB JSON IO bounds. Capture creates a new
  snapshot with code retained; sender canister version stays absent for host ingress
  and is never a target-version guard. Raw IDs are distinct from generic backend
  tokens. Codec/binding checks grant no freshness, signing, dispatch or effect proof.
- The membership port now has ephemeral challenge/boundary-bound original-plan
  requests and passive provider results. Pure admission checks actual context/full
  inventory and a 1,024 remote-observation descriptive ceiling. That ceiling is no
  spending allowance; provider freshness, prior per-call accounting and opaque
  revision/evidence meaning remain integration-owned. Matching revisions or hashes
  do not prove continuity, a fence, permissions or dispatch authority. No default
  provider, persisted fresh-authority flag or membership backend is implemented.
- Direct control observations now bind exact original IC mutation payloads to an
  ephemeral challenge and actual context/target/complete canonical controller set
  (at most 10). Pure admission requires the selected caller itself; no Root proxy,
  subnet-admin or status/read-visibility fallback exists. Keep the 1,024 descriptive
  call ceiling distinct from spending authority. Provider freshness/custody and
  load origin permissions, lifecycle/fence/restore safety remain separately qualified.
  No serialized Proven flag, live provider or dispatch permit is implemented.
- Snapshot-read observations now bind original mutation intent and an independent
  exact list payload, challenge and actual context/target/snapshot visibility.
  Preserve 10 unique canonical viewers and the 1,024 descriptive call ceiling.
  Known public or exact viewer access needs no controller projection; unobserved
  controllers cannot establish the controller path. Read evidence grants no control,
  spending or lost-reply settlement. Status/log/Root-configured flags never substitute.
  Actual authenticated providers remain unimplemented; separate metadata/data
  codecs grant no fresh read permission.
- Consistency requirements now retain a strict v1 original-plan-bound guarantee
  under 1 KiB IO and immutable layout publication. Current consistency requests bind
  original operation, challenge, capture boundary and retained exact fence/revision.
  Preserve 1,024 unique canonical actual target rows and the separate descriptive
  call ceiling. Pure checks require full current inventory/exact selection, every
  target stopped, exact requested lane and coordinated Active fence/original revision.
  Opaque stopped/drained and whole-unit fence evidence requires actual integration
  qualification. No acquired/released fence, restart, spending or restore-safety
  permit exists; failures/drop retain original obligations and consumed allowances.
- The maintainer requested fixing registry publication after a rejected upload.
  Package metadata permits crates.io publication. Configuration changes and dry
  runs do not themselves upload packages. Repository release preparation,
  tagging and pushing retain their clean-source, tag and receipt requirements;
  registry publication delegates admission to Cargo for the current package.
- Snapshot capture/inventory reply decoding now retains exact raw IDs and required
  nat64 timestamp/size fields under a declared request. Preserve the 1 MiB raw,
  1,024-entry, 256-ID-byte, 16-type-table and finite decoder-work bounds. Canonical
  inventory views reject duplicate IDs while evidence hashes bind exact raw bytes.
  Wire association proves no network/caller/target authentication or fresh permission;
  decoded metadata never settles a pending attempt or attests complete transfer.
- Pure snapshot inventory comparison requires the exact capture/list targets and
  retains unchanged baseline IDs/metadata. Candidates borrow bounded canonical
  replies and retain original evidence. Zero, one or multiple candidates establish
  neither capture outcome nor attribution; never derive an Applied receipt, retry,
  restart or release from cardinality. Integrations own original baseline custody,
  observation chronology, authenticated association and exclusive attribution.
- Lifecycle reply decoding now admits canonical empty stop/start/load tuples and
  required status/settings/controller projections. Preserve 1 MiB raw input,
  2 MiB decoding work, 64 KiB skipped work, 64 type-table entries and the existing
  10-controller bound. Unknown status metadata is skipped, not qualified; missing
  required fields reject. Reuse ControllerSet admission and upstream status variants.
  Reply association authenticates no caller/target/context or timing. Acknowledgements
  and Stopped/controller projections never settle attempts, prove drain/load safety,
  authorize restart/release or replenish spending.
- Explicit fresh download integrity verification now requires the retained original
  plan, unchanged guarded journal, exact selected targets and durable checksums.
  Preserve existing bounded owners and no-follow directory traversal. Ordinary
  resume reads retained progress only. Structural views and sequential byte checks
  grant no transfer completeness, atomic snapshot, terminal proof, new spending or
  reference-release authority; integrations own stable byte custody.
- Original fence obligations now bind the full plan, explicit application acquisition
  operation, original capture/restore requirement and exact fence revisions. Preserve
  strict v1 1 KiB immutable IO under retained plans/requirements and both restore/source
  layout guards. Attempt journals remain the only spending/reconciliation owner;
  absence cannot mean zero consumption. Retain obligations for pending, Applied,
  NotApplied and Uncertain outcomes. No active fence, dispatch, automatic disposition,
  terminal proof or fence/reference release is inferred from local declarations/views.
- Restore safety requirements now bind exact original restore/source plans and
  artifacts under strict immutable v1 publication and 1 KiB IO. Preserve same-network/
  same-release existing selected IDs; a restore subset needs application qualification.
  Fresh exact load/start requests bind original bytes, challenge and descriptive calls.
  Current full inventory/exact selection must match. Load needs every target stopped;
  start needs its target stopped, every selected restored state accepted and fenced
  execution qualified. The explicit no-irreversible-effects lane has no generic default;
  the fenced lane requires original identity/membership/external-obligation revisions
  and rewind-independent custody/replay safety. Preserve 1,024 target/call bounds.
  Pure views settle no lost load, spend nothing and release no fence or source reference.

- Fence-acquisition reconciliation binds the original obligation, authority, mutation
  and challenge to an already reserved exact observation and rechecks the current
  journal. Preserve the existing 1,024 combined-attempt bound, 1,024 canonical actual
  targets and separate single-remote-observation descriptive ceiling. Acquisition
  requires exclusive original-request attribution and exact Active fence/revisions;
  negative evidence must exclude transient acquisition, not merely show absence.
  Provider failures stay pending; only a qualified settled observation can claim
  uncertainty. Passive views grant no automatic receipts, redispatch, refund,
  acquisition/release, installed provider or terminal proof. Integrations own actual
  authentication, freshness, custody, permissions and prior per-call accounting.

- The application fence acquisition port binds exact original canonical receiver,
  replicated update mode, 1..128 visible ASCII method bytes and at most 1 MiB opaque
  arguments to the existing reserved mutation. Preserve nonrecursive payload hashing:
  build payload before plan/requirement/obligation. Integrations retain original bytes
  and qualify whole-unit purpose/fence/revision semantics, current permissions and
  proof of no prior dispatch. A reconstructed pending request grants no repeat call.
  Only one previously accounted update is allowed, without hidden retries/observations.
  Acknowledgements are passive association only and never produce an acquisition
  outcome, Active custody or receipt. Failures retain pending spending and original
  obligations/references; no provider implementation or release authority is installed.

- Execution settlement now retains exact full original plan and complete unique
  Applied journal fingerprints, without copying counters or terminal flags. Preserve
  v1, 8,192 rows, 2 MiB checkpoint IO and original per-journal/plan-attempt bounds.
  Full chronological hashes bind original authority, reservations, outcomes and
  evidence; identical final views cannot substitute. Reuse canonical execution progress
  for original identity/coverage/causality admission. Immutable publication/read require
  retained original plan and all journals under layout exclusion, with sequential
  journal locks and bulk original authority derivation. Replay is local and retains
  all spending, obligations and references. This is not full backup/restore completion,
  authenticated receipts, cross-journal chronology, byte/extent/application qualification,
  command quiescence or fence/reference release. No release API is implemented.
  Canonical checkpoint derivation reads every exact retained original journal and
  delegates publication to the existing owner, rechecking full histories before
  writing. Hold no journal guards; occupied checkpoints remain retained. The
  additive derivation error leaves the existing publication/replay error unchanged.

- Immutable local download manifests now reuse the exact v1 download journal schema
  after original-plan-bound fresh byte verification. Preserve 1 MiB IO, 1,024 artifacts,
  exact tokens/metadata/derived paths/checksums and canonical binary identity. Replay
  admits exact original retained plan/journal evidence locally and never reads artifact
  trees or calls providers. Publication/replay changes no progress, allowance or
  references. Complete backend transfer, authentic snapshots, consistency, full product
  manifests/terminal proof and fence/reference release remain independently qualified.

- Explicit fresh local restore-source verification now joins the original requirement,
  both exact retained plans, immutable local manifest and unchanged guarded download
  journal. Reuse canonical same-network/release/ID subset and complete Durable source
  owners. The opt-in source binding is the existing manifest digest; generic integration
  artifact digests keep their meaning. Preserve existing 1 KiB requirement, 1 MiB IO,
  1,024-target and 256-token-byte bounds, and both layout/journal lifetimes. Check every
  original source tree, including artifacts outside a restore subset. No new record,
  progress/accounting owner, provider, upload/load/start or terminal/release permit is
  introduced. Integrations own authentic capture/complete transfer, stable noncooperating
  bytes, application subset safety and effect-boundary permissions/custody.

- Private local restore artifact staging now binds an original opaque operation
  sequence to its exact selected source artifact and retained checksum. Reuse the
  original-source admission and safe descriptor-copy owners under both layout
  lifetimes. Create fixed direct children with 0700/0600 permissions; preserve
  partial copies and reject occupied destinations without replacement or cleanup.
  Explicit retained-copy verification checks exact retained originals and copy bytes
  without re-reading source trees or creating an artifact. This staging has no fsync/
  durable publication, spending transition, upload/load/start permit or release
  admission. Integrations own stable noncooperating destination/byte custody.

- Metadata-stage coordination now joins one exact singleton original metadata read
  to mandatory integration-qualified byte retention/attribution, the sole receipt
  owner and canonical all-Applied checkpoint. Learned evidence binds exact request/
  raw metadata; successor plan/writer preparation stays explicit. Lost replies stay
  pending; failed checkpoint publication retains Applied history and returned bytes.
  Reopen/checkpoint recovery never reissues. No new schema, default Agent provider,
  spending owner, terminal or fence/reference release follows.

- The IC mutation port now binds exact existing capture/load/start/stop payloads to
  the full original plan, operation authority and already pending mutation. Preserve
  original allowances, the separate recovery-observation owner and existing codecs:
  1 MiB raw replies, 256 raw snapshot-ID bytes and 4 KiB arguments. Pure association
  checks actual claimed context/target, authority/attempt and bounded method-specific
  wire shape. It authenticates no reply and produces no outcome, receipt, retry,
  spending replenishment, terminal or release permit. One provider invocation permits
  only the originally accounted exact update; no hidden reissues or observations.
  Durable originals, proof of no prior dispatch, fresh permissions/prerequisites,
  capture consistency, same-release load/start safety and command/byte custody remain
  integration-qualified. No provider, transport or runner is installed.

- Exact IC recovery observations now bind original mutation bytes and pending
  mutation/observation IDs to the full original plan and already reserved status/list
  digest. Preserve the separate journal spending owner, 1,024 total attempts, 4 KiB
  argument and 1 MiB raw-reply bounds plus existing status/inventory decoder quotas.
  Pure association rechecks current reservations, exact payload and actual claimed
  context/target. It authenticates no provider and writes no receipt. Status/controller
  projections and zero/one/multiple snapshots never settle a mutation automatically.
  Lost replies remain pending, not settled Uncertain. Fresh method-specific read
  permissions, chronology, attribution and proof of no prior observation dispatch
  remain integration-owned. No installed provider, reissue or release permit exists.

- Explicit durable local restore-copy publication now reuses original-source admission
  and canonical descriptor synchronization/checksum/atomic no-replace publication.
  Fixed temporary/canonical direct-child paths share the existing operation lock.
  Retain both layouts/source journal and all original plans/requirement/manifest;
  re-admit original records before and after publication. Recovery checks/synchronizes
  the canonical copy without source-tree reads or recopying. Conflicting/missing paths,
  changed/unsafe bytes and failures retain evidence without repair or cleanup.
  Published-copy verification is a distinct fresh byte check, not durability inferred
  from a path or ordinary resume/terminal replay. No journal, schema, digest, spending,
  obligation, reference or fence changes. Stable noncooperating custody, backend
  completeness, application safety and upload/load/start/terminal/release admission
  remain integration-owned.

- The snapshot metadata codec now retains exact ephemeral raw-ID requests and pinned
  SDK metadata under 1 MiB raw, 2 MiB decoder work, zero skipped work, 64 type-table
  and 16 KiB header bounds. Preserve 4,096 ordered global slots, exact float bits,
  unsigned 128-bit globals, 1,024 distinct ordered 32-byte chunk hashes and 32
  certified-data bytes. Optional source/global/timer/hook absence supplies no upload
  default. Reuse existing payload hashing without changing lifecycle/recovery v1
  records. Metadata/request association authenticates no target, grants no fresh
  permission/spending, settles no attempt and proves no data extent/transfer completion.
  No provider, transport, upload/load/start or terminal/release admission is installed.

- Snapshot data reads now borrow exact retained metadata and encode one checked
  Wasm/heap/stable range or known chunk-store hash. Preserve nonzero 1 MiB ranges,
  checked nat64 offset/size admission, 1 MiB actual data, 2 MiB raw wire, 8 MiB
  decoder work, zero skipped work, 16 type-table entries and 4 KiB headers/arguments.
  Replies need exact range lengths or actual matching SHA-256 chunk bytes; known
  empty chunks are admitted. Evidence binds original metadata, exact request and
  raw reply; no aggregate coverage, new journal/progress, freshness, spending,
  automatic settlement, repeat call, upload/load/start or terminal/release follows.

- Incremental snapshot data coverage now owns three independent contiguous nat64
  region cursors and at most 1,024 chunk-presence bits under exact original metadata
  request/raw evidence. Preserve gap/overlap/duplicate rejection with unchanged state
  on errors; regions may interleave and chunks may arrive in any order. Empty regions
  need no reads; empty known chunks need their exact hash-checked reply. Complete
  views retain no bytes, durability, authentic transfer or effects. Reconstructed
  coverage starts empty and supplies no journal, allowance, resume or release permit.

- Ordinary snapshot transfer reads now bind the full original plan, exact existing
  metadata/data payload and pending mutation-lane replicated update. Preserve
  current reservations, 1,024 attempts, 256 raw-ID bytes, 4 KiB arguments, metadata
  1 MiB raw and data 1 MiB chunk/2 MiB raw plus original decoder quotas. Passive
  response checks match actual claimed authority/attempt/context/target and reuse
  existing decoders. Original metadata custody, authentication, fresh read permission
  and never-dispatched command custody remain integration-owned. Lost replies stay
  pending; no hidden reissue, receipt, allowance reset, complete transfer, installed
  provider or terminal/fence/reference release follows. Real PocketIC read-discard
  cases qualify the isolated fixture's safe stop only.

The IC local artifact writer now joins exact original metadata/data coverage to the
existing download journal and durable publisher. Preserve 1 MiB per data chunk,
three incremental region hashes, 1,024 chunk rows and existing metadata/journal bounds.
Its distinct v1 tree retains exact metadata wire/request bytes and fixed private files.
Appends consume the writer on any error; partial bytes remain Created and occupied
staging rejects recreation. Full fresh byte checks retain the expected checksum
atomically before publication; recovery uses the existing ChecksumVerified owner.
No new spending/transfer journal or partial-read resume is added. Generic token/raw-ID
association, authentic complete transfer, fresh permissions/accounting and stable
noncooperating custody remain integration-owned; no terminal/release authority appears.
Explicit retained IC-tree verification requires the full original persisted plan,
unchanged guarded journal and complete Durable selection. Preserve the fixed closed
direct-child format, exact original metadata/request hashes, nat64 region lengths and
bounded known chunk hashes. Only the requested tree is checked; passive sequential
byte evidence gives no fresh custody, complete-set verification or upload permit.
Ordinary resume remains artifact-free; no rewriting, accounting or release follows.

- Snapshot upload declarations now bind the original source plan, exact metadata,
  retained tree checksum and canonical wire bytes. Preserve available global bits
  and optional timer/hook absence; unavailable globals reject and replacement stays
  absent. Each 1 MiB region/known-chunk write binds a distinct 1..256-byte destination
  and its own original plan/reservation. Keep 2 MiB argument, 4 KiB reply and finite
  decoder quotas separate from lifecycle bounds. Fresh guarded source preparation
  verifies exact local bytes without progress/reference changes or future custody.
  Passive current-attempt association authenticates no allocation/write, writes no
  receipt and grants no retry/refund. Lost replies remain pending; destination
  attribution, original per-call dispatch, fresh controllers, complete transfer and
  restore/fence/load/start/terminal safety remain integration-owned. No upload provider,
  transport or runner is installed.

- Upload coordination now binds exact source/upload context and payload before
  durable original reservation, mandatory fresh admission and one provider call.
  Retain stage/ancestor checks and bounded returned acknowledgements on rejection.
  Success stays pending; independently qualified explicit receipts alone permit
  successors. Actual source/controller/destination/byte/never-dispatched custody
  remains integration-owned. Preserve original allowances/references and no batching,
  retry, automatic allocation, default Agent bridge or terminal/release authority.

- Singleton upload allocation now delegates spending/fresh admission to the
  existing one-upload coordinator. Mandatory independent qualification durably
  retains original request/reply bytes and authenticates exclusive allocation
  attribution before an explicit exact Applied receipt. Hold the selected journal
  lock through qualification/receipt, then release it before the canonical learned
  checkpoint. Preserve bounded acknowledgements on post-reply failure, original
  consumption, Applied/occupied evidence and explicit data-stage binding. Reopen
  uses original replay; no allocation reissue, automatic receipt, byte store,
  replacement, default Agent bridge or terminal/fence/reference release follows.

- Local IC artifact diagnostics reuse registry `ic-metrics` arithmetic under a
  guard-local synchronized owner. Preserve explicit nanosecond/byte units, separate
  returned success/rejection samples, valid zero and repeated preparation semantics.
  Preparation timings include nested verification and are not exclusive totals.
  Metrics start empty on create/open, retain no IDs/history and write no records.
  Successful prepared sizes have one shared four-bound histogram; derive the
  existing summary from it. Preserve zero/32 KiB/256 KiB/1 MiB inclusive bounds,
  disjoint independently saturating counts and failure exclusion. Six duration
  summaries remain separate; distributions grant no payload admission.
  Never feed saturated summaries or diagnostic samples into exact spending,
  completion, receipt, retry, fresh-authority or terminal/release admission. Keep
  the IC instruction-reader feature absent from the host implementation.
- Bounded Unix JSON record reads reuse registry `ic-host-fs` regular-file
  admission. Preserve existing record limits, JSON decoding and typed persistence
  errors, including nonblocking FIFO rejection and final-component no-follow.
  Caller-selected parents, confinement, stable byte custody, durable publication,
  locks/journals and command descriptor inheritance keep their local owners.
  This dependency provides no IC effect, paid retry or fresh authority. Qualify the
  actual locked graph and reader on supported hosts; no sibling path patch exists.
  Raw identities, hashing/copying and record output limits use `ic-host-artifacts`.
  Public `checksum_reader` retries Interrupted locally; callers own blocking and
  timeouts. Preserve other original IO errors, invalid-count refusal, canonical
  checksum records and local directory digest framing/private publication owners.

- Metadata-upload inventory observations bind the full original source/upload plan,
  exact list bytes and both already pending attempt IDs. Reuse canonical observation
  responses, errors, reservation checks and inventory decoding; data uploads and status
  observations reject at this boundary. Preserve existing attempt/argument/reply/ID/
  inventory bounds and sole journal spending ownership. Zero/one/many snapshots never
  settle allocation or qualify exclusive attribution. Retain original baseline custody,
  fresh read permissions, authentication/chronology and proof of no prior observation
  dispatch as integration responsibilities. Failures/lost replies stay pending; no
  provider, hidden reissue, refund, terminal or fence/source-reference release exists.

- Data-upload readback binds the full original source/upload plan, exact destination
  raw ID/extent/hash, known uploaded metadata/original dimensions and both pending
  attempts to an already reserved read digest. Reuse canonical upload authority,
  reservation/claim checks and bounded data decoding. Preserve 1,024 attempts,
  256 ID bytes, 4 KiB read arguments, 1 MiB metadata/chunk, 2 MiB raw data reply
  and original decoder quotas; existing status/list responses remain at 1 MiB.
  Equal/different bytes establish no original write attribution or outcome, and
  missing metadata/data cannot supply a default. Authentication, fresh permission,
  timing, actual allocation/readback capability and never-dispatched observation
  custody stay integration-owned. Lost replies retain pending spending and all
  references/obligations; no installed provider, retry/refund, complete upload or
  terminal/fence/reference-release permit exists.

- Data-upload settlement admission now reuses original-plan-bound successful exact
  readback and current reservations. Passive claims additionally bind original
  authority/both attempt IDs, a fresh qualification challenge, the existing metadata/
  request/raw data digest and exact opaque observation evidence. Applied requires
  matching bytes plus integration-qualified exclusive original-write attribution;
  NotApplied requires exclusion of transient application/overwrite, even for matching
  preexisting bytes. Unresolved requires a settled authenticated read, never lost or
  malformed replies. Views perform no IO, extra calls or automatic journal transition.
  The existing journal remains the sole receipt/spending owner; exhaustion never
  refunds. Original allocation, authenticity, chronology and stable custody remain
  integration-qualified. No provider, schema/digest, backend completion or release
  admission is introduced.

- Metadata-allocation settlement now binds original full upload authority, both
  pending attempts, a current challenge and exact original/current list/raw digests
  and opaque observation evidence. Reuse closed-baseline inventory comparison and
  canonical destination admission: all baseline descriptors remain unchanged;
  Applied names an independently attributed new non-source raw ID. Zero/one/many
  candidates infer no outcome. Negative proof excludes transient allocation/deletion;
  Unresolved requires an actually settled authenticated successful list, not a lost
  reply. Preserve existing 1,024-entry/attempt, 256-ID-byte and 1 MiB raw bounds.
  Integrations retain original baseline before mutation and qualify chronology,
  authentication, freshness, custody and exclusive attribution. Passive views change
  no journal, spending, obligations or references and grant no release permit.

- Lifecycle settlement now matches independently qualified original stop/start/load
  claims to the existing reserved status observation, complete original authority,
  exact attempts, fresh qualification challenge and status/evidence digests. Preserve
  the existing 1,024 attempts, 4 KiB arguments and 1 MiB status reply/decoder owners.
  Current status/controllers/code hashes alone establish no outcome; load attribution
  must independently qualify exact original restored state. Negative evidence excludes
  transient application. Unresolved requires a settled authenticated successful read;
  lost/absent/malformed replies remain pending. Pure matching writes no receipt or
  spending transition and grants no retry, restart, terminal or fence/reference release.
  Integrations authenticate, qualify custody/chronology/read permission and explicitly
  use the existing journal transition; no provider, schema or new accounting owner exists.

- Capture settlement now matches independent original capture claims to the full
  original plan, pending capture/list reservations, exact retained baseline/current
  inventory digests, caller challenge and unchanged observation evidence. Reuse the
  closed baseline owner: every original ID/timestamp/size stays unchanged. Applied
  explicitly names a new bounded raw ID; zero/one/many candidates never infer an
  outcome. Negative evidence excludes transient capture/deletion; actual settled
  uncertainty differs from lost replies. Preserve existing 1,024 attempt/inventory,
  256 ID-byte, 4 KiB argument, 1 MiB reply and finite decoder bounds. Pure borrowed
  views write no receipt or allowance and release no reference/fence. Actual original
  baseline chronology, authenticated attribution, fresh permissions/custody,
  consistency/transfer and terminal/release qualification remain integration-owned.

- A test-only PocketIC driver now qualifies actual single-canister capture, durable
  streamed artifact transfer and same-ID upload/load/start with deliberately discarded
  capture/allocation/data and stop/load/start replies. Complete stopped post-load
  state checks use a distinct freshly captured snapshot under separately retained
  original authority. A lost load-status reply retains both pending reservations
  and stops before verification or restart without writing a receipt. Preserve
  explicit original per-ingress plans, unique operation sequences, prior bounded reservations, independent original attribution,
  complete byte/metadata checks and local replay with retained references. The selected
  server is explicitly prepared and checksum/version checked; missing tools reject.
  Fixture setup, pre-load chunk-store changes/assertions and application calls are
  separate from backup accounting. Its inspected no-external-effects/stopped-drain
  admission is fixture-specific, never a generic
  default. No production transport, runner/CLI, Canic adapter, active fence or full
  terminal/reference-release qualification is installed. See
  [the exact qualification scope](docs/pocketic-qualification.md).

- The maintainer selected direct Rust `ic-agent` transport and retired the ICP
  backend/probe as a hard cut. `crates/ic-backup-agent` owns actual bounded async
  single-update submission, depending only on the generic core and registry Agent.
  Use exact existing reserved request/payload owners and recheck the current journal.
  Configure explicit exact context/caller, fixed HTTPS or literal loopback HTTP
  origin, externally trusted root and finite 1..300s timeout/expiry. Disable proxies,
  redirects, HTTP/TCP retries and implicit waits/polling/root fetching. Preserve
  4 MiB HTTP, 3 MiB signed-envelope and smaller original method reply bounds.
  Retain original plan/reservation plus exact signed envelope/request ID before
  consuming preparation. No envelope re-import/reissue or arbitrary call hook exists.
  Certificate-verified replies are passive, not automatic receipts; pending/error/
  cancellation/lost replies retain spending and all original obligations/references.
  Real local HTTP and PocketIC fixtures qualify transport, not fresh permissions,
  never-dispatched custody, application safety, full runners or terminal release.
  Old ICP logs/journals remain historical evidence; no ICP backend fallback remains.

- The provider-driven snapshot capture step accepts only the exact original
  take payload, opens its existing stage journal and durably reserves through
  complete-plan admission. Hold that journal through mandatory fresh actual
  control/consistency/quiescence/fence and never-dispatched custody admission,
  one existing mutation-provider invocation and bounded passive association.
  Recheck retained stage/ancestors around dispatch; retain bounded returned
  acknowledgements on later rejection. Success and all post-reservation failures
  stay pending; missing, held, pending or Applied originals never grant recapture.
  Integrations explicitly authenticate and record qualified outcomes. Preserve
  original schemas, limits, source references and fence obligations. The isolated
  capture-to-download fixture does not install a default Agent provider, complete
  runner, fresh permission, terminal proof or fence/reference release.

- The provider-driven snapshot transfer read step opens the original stage journal
  and reserves through canonical complete-plan admission before mandatory explicit
  fresh integration admission and exactly one provider call. Hold the selected
  journal through dispatch/association and recheck stage/ancestors around it.
  Pre-spending payload binding reuses the existing model; no new journal, allowance,
  schema or default permission/provider is introduced. Every post-reservation
  failure and successful structurally associated reply leave spending pending.
  Retain returned bounded responses in association/post-reply errors. Never
  redispatch reconstructed pending requests or automatically record outcomes.
  Integrations still own authentication, actual access, original metadata/raw-ID
  and command/byte custody, application requirements and explicit qualified receipts.
  This single step does not qualify full runners or terminal/fence/reference release.

- The provider-driven data download loop consumes an existing private writer under
  an exact retained metadata-derived data stage. Reject read-free, changed metadata/
  writer/intent and any originally consumed data stage before new spending. Reuse
  the single-read coordinator; require a separate integration-qualified exact Applied
  receipt under selected journal exclusion. Append admitted bytes before recording
  that receipt through the sole attempt owner, then admit dependent reads. On failure
  consume the writer, retain partial bytes/spending/recorded receipts and return
  bounded replies in post-read errors. Finish through the existing checksum/durable
  publisher and recheck stage/ancestors; retain returned checksum on closing rejection.
  No partial-read resume, schema, allowance, default provider, manifest/checkpoint,
  complete product runner or terminal/fence/reference release is added. Metadata and
  token/raw-ID authentication, fresh read/application admission and stable noncooperating
  custody remain integration-owned.

## Simulator qualification

Simulator fixtures depend on published `ic-testkit` and use its complete
`pocket_ic` re-export for exact management ingress and async gateway qualification.
Testkit's explicit managed server owns bounded readiness/cleanup and retains raw
output in each caller-owned fixture directory. No baseline reuse/reset, funding
policy, hidden snapshot retry or receipt inference is adopted. Preserve original
plans, journals, byte evidence and fixture-specific application safety.

`ci/ic-tools.tsv` selects the five shared IC executables only. Testkit owns
PocketIC release selection, authenticated assets, explicit setup and offline
admission. The unique locked Testkit package also selects the exact CLI through
Shared Tooling's Cargo installer; no second server catalog, version equality
policy or shared-bundle fallback exists. `make install-testkit-server` prepares it;
`make testkit-server-check` prints the admitted absolute server path without setup.
Both simulator suites invoke that check before managed startup. Keep product
fixture custody, digests/logs and original journals local, and retain old/failed
bundles. Native consumer acceptance remains required; owner CI is separate.

## Tracking

GitHub issues are the sole tracker for bugs, review findings and follow-up work.
Use the owning repository's issues once its remote is established. No issue
prefixes, local issue lists or duplicated feedback files. Design stages are
implementation sequencing, not a parallel tracker. If a remote is unavailable,
report a finding to the maintainer without inventing an issue URL or claiming it
was recorded. Linking provenance or describing implementation status is allowed.

## Product contracts

- Metadata-derived download planning counts all region/chunk requests with checked
  arithmetic before allocating, within the original stage's at-most-1,024 combined
  attempts. Reuse exact metadata-bound payloads, ordinary plans and stage bindings.
  Preserve module/heap/stable/chunk order, explicit sequential dependencies and
  one mutation/zero observations per request; unassigned headroom grants no retry.
  Binding requires the exact single-request original metadata stage and reply
  evidence. Entirely read-free metadata creates no placeholder operation. The real
  Testkit driver qualifies only its isolated application and explicit originals;
  no installed runner, authenticated input, fresh application admission or terminal/
  reference release follows. Lost/malformed replies preserve pending spending,
  original journals, partial Created bytes and all references without follow-up.

- Distinct workflow allocations and learned-stage bindings reuse canonical plan,
  layout, journal and complete chronological settlement owners. Preserve original
  target/context/full inventory, exact stage ceilings, fixed create-only stage
  directories, strict v1 1 MiB workflow/512 KiB binding IO and 1,024 predecessors.
  Catalog request hashes commit integration-owned purpose/input contracts; catalog
  rows are never journal authority. Only exact child plans own original journals;
  no unassigned headroom, rebuilt plan, missing journal or interrupted preparation
  resets spending. Reopen admits exact originals and predecessor histories without
  allocation/repair. Learned evidence bytes, authentic derivation, fresh permissions,
  effect safety and never-dispatched custody remain integration-qualified. No runner,
  automatic receipt, transfer completion or terminal/release authority follows.

- Explicit fresh stage preparation additionally publishes every original child
  attempt journal through the existing create-only owner before returning.
  Derive all authorities before stage allocation; retain one journal lock at a
  time and re-admit original stage/predecessor records after publication. Preserve
  all partial records on failure/death. Occupied stages reject another preparation;
  reopen never creates missing journals, repairs gaps or resets allowance. The
  record-only create primitive stays separate. No reservation, new record/schema,
  provider, fresh dispatch custody, installed runner or release authority follows.

- Stage admission additionally checks every transitive ancestor's exact retained
  binding/plan and complete chronological Applied settlement/original journals.
  Traverse iteratively with at most one row per bounded catalog node and sequential
  ancestor layout/journal admission. Shared edges must agree on binding/settlement
  identity; their learned-input evidence remains edge-specific. Missing/changed
  history rejects preparation, reopen and retained-layout access without repair or
  new spending. No atomic noncooperating custody or terminal authority follows.

- Stage-owned checkpoint publication joins the canonical all-Applied journal owner
  to exact retained workflow/binding/child-plan and complete ancestor admission
  before and after publication. Return the existing predecessor identity with the
  integration's opaque learned digest; authenticate no learned input or receipt.
  Hold no attempt guards. Post-publication failures retain checkpoint/originals;
  occupied checkpoints reject without replacement, repair or spending changes.
  No provider, successor dispatch, product terminal or fence/reference release follows.

- Complete stage resume joins exact record-only admission to the existing complete
  original child-journal progress reader and rechecks original stage/ancestor records
  before returning. Preserve original typed stage/progress errors and both layout
  lifetimes. Missing/changed/held journals reject without creation, artifact/provider
  reads or allowance reset. Record-only open remains available for retained evidence
  inspection. Sequential progress grants no atomic custody, fresh authority, receipt,
  installed orchestration or terminal/fence/reference-release permit.

- The product runs on the operator host. Canisters receive no local filesystem,
  signing keys or restore runner permissions.
- The generic library owns snapshot/artifact/journal/recovery mechanisms.
  Integrations own authoritative membership, control routing, release identity,
  application quiescence and external-effect settlement.
- Initial restore scope is same-release recovery into the same exact existing
  canister IDs. No creation, relocation, cross-release adoption, migration,
  rollback compatibility or identity rebinding is authorized by this design.
- Before 1.0, keep one maintained product protocol/schema generation, v1.
  Hard-cut obsolete contracts; do not add aliases, compatibility readers,
  wrappers or v2 product lanes. Historical evidence may retain its real version.
- A persisted declaration is not fresh authority. Observe exact network,
  caller, target, controller/read permissions and relevant current state at
  effect boundaries. Unknown authority or an ambiguous paid effect stops safely.
- Persist exact intent and consume bounded attempt authority before effects.
  Lost responses require reconciliation, not blind retry. An observation may
  resolve an exhausted attempt without authorizing another paid call.
- Terminal replay validates retained local completion evidence and performs no
  remote observations or effects. Fresh verification is a distinct operation.
- Snapshot restoration cannot rewind external payments or obligations. Require
  application-owned fencing/settlement evidence before restoring or restarting.
- Local prune and live snapshot deletion are separate operations. Source
  artifacts referenced by unfinished restores remain retained.

- The snapshot upload port reuses exact source-bound metadata/data attempts,
  bounded passive IC acknowledgements and the canonical IC update failure owner.
  Each invocation permits only one original replicated update, with independent
  original data spending and no batching, hidden retries or observations. Preserve
  upload codec bounds, current reservation rechecks and all source references.
  Integrations qualify authentic complete source, new destination attribution,
  fresh controllers/prerequisites, stable byte/command custody and proof of no prior
  dispatch. Lost replies stay pending; no provider, transport, settlement, complete
  upload or terminal/reference-release authority is installed.

- Complete snapshot data-upload preparation counts exact metadata extents against
  the original stage ceiling before source IO, then retains canonical payload
  digests while buffering one bounded payload at a time. Bind the exact singleton
  allocation reply and Applied predecessor; preserve source network/release and
  independently qualified current caller permissions. The fresh-stage coordinator
  refuses any consumed original journal, freshly prepares each payload and delegates
  one update to the existing spending owner. Mandatory independent qualification
  durably retains original request/reply bytes before each exact Applied receipt.
  Stop on every failure; pending spending, returned replies, source bytes/references
  and occupied checkpoints remain. Final ordered reply evidence uses the existing
  all-Applied checkpoint, not backend completeness, partial-upload resume, load/start,
  terminal or fence/reference release authority. No new record/provider is installed.

## Qualified dependency constraints

The maintainer-authorized tooling adoption retains these existing exact registry
requirements without changing locked selections. Machine-readable exceptions in
`ci/dependency-pinning-exceptions.json` match only these root declarations:

- `command-fds =0.3.3` owns the qualified safe descriptor-inheritance API used for
  command custody. Changing it requires focused process/descriptor qualification
  on Linux and both macOS architectures; lock reproducibility alone is not the reason.
- `ic-management-canister-types =0.11.0` owns the qualified management wire DTOs,
  upstream status variants and public Rust type identity. Changing it requires exact
  codec/golden/bounds review and public compatibility assessment, then native checks
  and separately applicable authenticated IC qualification.
- `ic_principal =0.1.5` owns the qualified normalized principal boundary and exposed
  Rust identity. Changing it requires normalization/identity/hash regression checks
  and public compatibility assessment, followed by relevant native qualification.

These are reviewed compatibility constraints, not universal exact-pin policy.
Shared tools use the vendored version/checksum owners under `ci/`; local setup is
explicit through `make install-tools`, and `make tools-check` never installs.
Product transport/runtime qualification stays separate from executable version checks.

## Architecture and style

- Rust edition 2024. Versions and dependencies inherit from the owning workspace;
  `make dependency-pins-check` enforces the shared Cargo inheritance gate.
- Use directory modules with `mod.rs`; no `#[path]` module layout overrides.
- DTOs are passive boundary data; persisted schema types end in `Record`; views
  are read-only projections. Requests do not implement non-neutral `Default`.
- CLI delegates to workflow; workflow calls pure policy and ops; ops accesses
  model/storage and performs approved single-step effects. Policy never calls
  ops, performs IO, serializes records, schedules work or mutates state.
- Workflow must use model/ops transition methods rather than constructing or
  mutating persisted records directly.
- Normalize equivalent principals/hashes at their owning boundary. Use named
  authority structs and predicates instead of long mixed boolean expressions.
- Prefer `#[expect(...)]` for justified lint suppression.
- Human-authored configuration is TOML. JSON is for machine records and output.
- CLI commands and help are ASCII lexicographically ordered. Help pages have
  at most three representative examples. JSON stdout contains one structured
  result; progress and redacted diagnostics go to stderr.
- Explain maintained behavior directly. Do not freeze prose with release gates.

## Validation and delivery

- Run focused checks during development and the documented full `make ci` suite
  before delivering completed code as ready, under the shared standing validation
  authority. Reuse unchanged passing evidence; rerun affected checks after edits.
  Documentation-only and inspection-only work follow the baseline's narrower
  validation scope. Release, publication and deployment still require explicit
  authorization. [Development](docs/development.md) describes local commands
  and target-directory ownership.
- Before compilation, check for an active command using this repository's
  target directory. Do not alter source or locks beneath active validation.
- Unit tests live beside code; integration tests live in `tests/`. Canister
  snapshot/capture/load and lifecycle behavior require PocketIC or a deliberately
  selected real local IC backend. Native fakes prove persistence/process
  behavior, not IC effects. Do not fake management behavior in production code.
- Test typed errors, state, exact identities, bytes, hashes and bounded effects.
  Derive test registry membership from registered cases, not aggregate counts.
- Preserve licenses and source provenance. A copied regression is not fresh
  qualification; rerun it against the extracted production implementation.
- Keep a root `CHANGELOG.md` with one open entry when meaningful implementation
  begins. Put the current draft or latest release first, with no Unreleased queue.
  Automatically maintain one numbered, undated `## [X.Y.Z]` under
  [the shared changelog rules](rules/changelogs.md), using the latest finalized
  release and the complete pending batch's compatibility impact. Keep a compatible
  maintainer selection; never allocate one patch per slice or change package
  versions without authority. Changelog presentation never gates registry publication.
- Read [the release guide](docs/releasing.md) before release/version work.
  Preparing commands does not authorize running them. Agents may inspect
  `release-plan` and test isolated helpers. Running `release-patch`,
  `release-minor`, `release-major` or `release-resume` requires an explicit request
  for its selected repository and destination. All delegate to the shared
  runner; consumer adapters own metadata and validation, never Git effects.
- Use `make install-hooks` once per clone for the exact shared pre-commit formatter.
  Install cargo-sort 2.1.4 explicitly with `--locked`; `fmt` sorts Cargo manifests
  before formatting Rust. The hook refreshes only selected files, rejects partial
  staging and preserves unrelated edits. CI/release use matching non-mutating
  `fmt-check`; prepared release metadata must also pass it before staging.
- Report complete-batch readiness and material limitations. Passing one test
  does not prove a finished extraction or independently usable product.
