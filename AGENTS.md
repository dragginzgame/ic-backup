# AGENTS.md

This file is normative for automated agents and contributors working in
`ic-backup`.

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
- Package metadata starts at the unreleased `0.1.0`; registry publication is
  disabled. Repository setup is not release or publication authority.

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
  begins. Do not allocate one patch version per slice or bump without authority.
- Read [the release guide](docs/releasing.md) before release/version work.
  Preparing commands does not authorize running them. Agents may inspect
  `release-plan` and test isolated helpers; maintainers own `release-commit`
  and commit-producing `release-*` commands.
- Use `make hooks-install` once per clone for the tracked pre-commit formatter.
  Release validation checks formatting without editing source.
- Report complete-batch readiness and material limitations. Passing one test
  does not prove a finished extraction or independently usable product.
