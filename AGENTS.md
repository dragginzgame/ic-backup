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
numeric bounds, commands, supported hosts and qualification gates stay local.
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
- Never create or amend Git commits, including indirectly through scripts or
  release tools. Leave work uncommitted for maintainer review.
- Continue accepted implementation batches autonomously when requested. Generic
  continuation does not authorize live effects, release/version transactions,
  pushes, package publication or destructive cleanup.
- Preserve dirty worktree state and retained recovery evidence. Do not discard
  unfinished journals, snapshots or artifacts as ordinary cleanup.
- Do not add local Cargo patches pointing at sibling checkouts, shared target
  directories or dependencies on Canic runtime/control-plane crates.

## Current implementation scope

- The maintainer authorized Rust repository setup following `ic-delegated-auth`
  and `ic-blob-storage`. The root is workspace-only; the initial library lives
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
  unsafe-code prohibition. These local contracts do not qualify an ICP backend
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
  Metadata/data codecs and actual authenticated providers remain unimplemented.
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

## Tracking

GitHub issues are the sole tracker for bugs, review findings and follow-up work.
Use the owning repository's issues once its remote is established. No issue
prefixes, local issue lists or duplicated feedback files. Design stages are
implementation sequencing, not a parallel tracker. If a remote is unavailable,
report a finding to the maintainer without inventing an issue URL or claiming it
was recorded. Linking provenance or describing implementation status is allowed.

## Product contracts

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

## Architecture and style

- Rust edition 2024. Versions and dependencies inherit from the owning workspace.
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
  Upstream ICP YAML stays inside the ICP integration boundary.
- CLI commands and help are ASCII lexicographically ordered. Help pages have
  at most three representative examples. JSON stdout contains one structured
  result; progress and redacted diagnostics go to stderr.
- Explain maintained behavior directly. Do not freeze prose with release gates.

## Validation and delivery

- Run only checks targeted to changed packages and behavior during coding.
  Broad validation requires the maintainer's explicit command or the repository's
  configured CI/release pipeline. [Development](docs/development.md) describes
  the local commands and target-directory ownership.
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
  Use an undated `## [Draft]` until a maintainer selects a version. Do not allocate
  one patch version per slice or bump without authority. Release preparation owns
  final labeling; changelog presentation never gates registry publication.
- Read [the release guide](docs/releasing.md) before release/version work.
  Preparing commands does not authorize running them. Agents may inspect
  `release-plan` and test isolated helpers; maintainers own `release-commit`
  and commit-producing `release-*` commands.
- Use `make hooks-install` once per clone for the tracked pre-commit formatter.
  Release validation checks formatting without editing source.
- Report complete-batch readiness and material limitations. Passing one test
  does not prove a finished extraction or independently usable product.
