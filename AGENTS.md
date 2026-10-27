# AGENTS.md

This file is normative for automated agents and contributors working in
`ic-backup`.

## Shared baseline and local overlay

Apply the engineering sections of [the Dragginzgame baseline](DRAGGINZGAME.md).
This is a reviewed local snapshot of Shared Tooling 0.26.23's committed rules, not
implicit inheritance from a moving sibling checkout.
[The snapshot manifest](.shared-tooling.snapshot) binds the common baseline and
tooling to one committed revision; [adoption and provenance](docs/shared-tooling.md)
describe the review. Refresh deliberately; never edit vendored tooling in place.
Exclude every path declared in `.shared-tooling.snapshot` from local branding
and documentation rewrites; keep consumer guidance in unlisted local documents.

This file is the product-specific overlay: backup contracts, effect targets,
numeric bounds, consumer adapters, supported hosts and qualification gates stay local.
Release entry points follow the common command contract.
The reviewed `make/consumer.mk` owns common setup, formatting, hook, snapshot and
release recipes. Backup retains its ordered Testkit extension, delivery admission
and mandatory exact resume receipt check as local inputs and prerequisites.
Common engineering rules are mandatory; record maintainer-approved exceptions
with their scope and reason. GitHub issues also own reusable feedback, with the
baseline's evidence fields instead of a duplicate feedback file.
Shared Tooling's own validation commands apply upstream, not to this repository.

## Start of session

Read `docs/status/current.md` first, then the relevant sections of
`docs/implementation-contracts.md`. Consult `docs/extraction-design.md` for
extraction rationale and proposed product work; it is not an implementation ledger.
Distinguish proposed behavior from implemented and qualified behavior. Update the
handoff after a meaningful batch; do not turn it into an issue tracker or release
authority.

## Scope and authority

Apply the shared [contribution rules](rules/contributions.md). Ordinary work
leaves staging, commits and release effects to the user; a requested PR includes
its scoped contribution workflow. Live backup/restore, publication and destructive
cleanup retain their explicit target/effect authority.

Do not add local Cargo patches pointing at siblings, shared target directories
or dependencies on Canic runtime/control-plane crates.

## Product safeguards

- Before product changes, read the relevant [implementation contracts](docs/implementation-contracts.md)
  and [record/API boundary](docs/extraction-boundary.md). Their exact bounds, authority,
  custody and recovery requirements remain mandatory. The [extraction design](docs/extraction-design.md)
  retains rationale and proposals; proposed flows do not widen implemented contracts.
- Backup runs on the operator host. Integrations own membership, routing,
  release identity, application quiescence and external-effect settlement.
- Same-release restore targets the same existing canister IDs. Persist intent
  and consume bounded attempt authority before effects; lost replies require
  reconciliation, never blind retries or reset spending.
- Preserve unfinished journals, referenced artifacts and evidence. Terminal
  replay is effect-free; restoration cannot rewind payments or obligations.

## Simulator qualification

Simulator fixtures use published `ic-platform::pocket_ic` clients and DTOs for
exact management ingress and async gateway qualification. Published `ic-testkit`
owns the harness extensions; Platform's `pocket-ic` feature is development-only.
Testkit's explicit managed server owns bounded readiness/cleanup and retains raw
output in each caller-owned fixture directory. No baseline reuse/reset, funding
policy, hidden snapshot retry or receipt inference is adopted. Preserve original
plans, journals, byte evidence and fixture-specific application safety.

The snapshot-owned `ci/ic-tools.tsv` selects optional IC executables. Manual
`make install-ic-tools IC_TOOLS="..."` and the matching `make ic-tools-check`
remain available with an explicit executable selection; aggregate
setup, CI and release preflight require no IC executable bundle or IC PATH.
After common host/Cargo steps, Backup registers locked fetch, Testkit and release
editor setup, with offline Testkit/editor admission after common checks. Testkit owns
PocketIC release selection, authenticated assets, explicit setup and offline
admission. The unique locked Testkit package also selects the exact CLI through
Shared Tooling's Cargo installer; no second server catalog, version equality
policy or shared-bundle fallback exists. `make install-testkit-server` prepares it;
`make testkit-server-check` prints the admitted absolute server path without setup.
Both simulator suites call the locked Testkit library’s offline
`pic::server_assets::check` before managed startup; the Make check remains the
standalone CLI/tool gate. Keep product
fixture custody, digests/logs and original journals local, and retain old/failed
bundles. Native consumer acceptance remains required; owner CI is separate.

## Qualified dependency constraints

Platform owns the registry declarations for external IC types; no local exact-pin
exceptions remain. Its `management-canister-types` feature selects exactly 0.11.0
and exports the qualified management wire DTOs, status variants and public Rust
identities. Changes to that selection require exact codec/golden/bounds review,
public compatibility assessment, native checks and separately applicable
authenticated IC qualification. Core and Agent consume those identities only
through Platform; direct management DTO declarations are forbidden.

`ic-platform::Principal` owns internal principal admission. Platform supplies
its owned representation; required Candid DTOs and external SDK calls use
Platform's `candid::contract::Principal` or `principal::ExternalPrincipal` exports
over upstream 0.1.5, with explicit boundary conversions. No direct principal pin
or exception remains. Changes require normalization, identity/hash regression
checks and public compatibility assessment, followed by relevant native qualification.

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

- [Development](docs/development.md) describes local focused selections, aggregate
  commands and target-directory ownership under the shared validation cadence.
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
- Follow [the shared changelog rules](rules/changelogs.md); changelog
  presentation never gates registry publication.
- Read [the release guide](docs/releasing.md) before release/version work.
  Preparing commands does not authorize running them. Agents may inspect
  `release-plan` and test isolated helpers. Running `release-patch`,
  `release-minor`, `release-major` or `release-resume` requires an explicit request
  for its selected repository and destination. All delegate to the shared
  runner; consumer adapters own metadata and validation, never Git effects.
- Use `make install-hooks` once per clone for the exact shared pre-commit formatter.
  Use `make install-rust-tools` for the snapshot-selected formatter tools;
  `fmt` sorts Cargo manifests before formatting Rust. The hook refreshes only
  selected files, rejects partial
  staging and preserves unrelated edits. CI/release use matching non-mutating
  `fmt-check`; prepared release metadata must also pass it before staging.
- Report complete-batch readiness and material limitations. Passing one test
  does not prove a finished extraction or independently usable product.
