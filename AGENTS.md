# AGENTS.md

Apply the installed [Toko Tooling baseline](TOKO-TOOLING.md) and its revision-bound
references. [.toko-tooling.snapshot](.toko-tooling.snapshot) owns managed files;
never edit that receipt or files under `vendor/toko-tooling/` by hand.

Read [README.md](README.md), [the handoff](docs/status/current.md) and
[the restart direction](docs/design/restart.md) at session start.
The handoff owns current implementation and qualification status. Read the
affected contracts below before changing behavior; distinguish implemented
Backup scope from unqualified Canic integration.
The legacy checkout is read-only reference material; do not import its source,
backlog, caches, build outputs, journals or release state wholesale.

## Learn from Backup Legacy before subsystem work

Before substantive subsystem design, implementation or refactoring:

- Inspect the relevant `ic-backup-legacy` implementation, regression tests and
  Git history, including the fixes that explain its current checks. Start from
  the revision pinned in [the legacy assessment](docs/design/legacy-assessment.md);
  record any other exact revision consulted. The broad assessment does not replace
  this focused review.
- Identify the concrete problem each relevant solution solved, its safety and
  recovery guarantees, and which assumptions have changed in the new library.
  Do not dismiss a check as legacy sediment without understanding its purpose.
- Reuse or adapt sound solutions where they fit the selected scope and current
  Platform/Host ownership. Prefer the simplest mechanism that preserves the
  required guarantee; do not copy legacy architecture or dependencies wholesale.
- Bring applicable regression cases into the owning new tests and run them
  against the new implementation. Preserve source revision and test provenance,
  plus licenses when copying code. Adapt cases to changed contracts and explain
  material exclusions; historical passes are not new qualification. IC effects
  still require an actual IC backend, not native filesystem tests or fakes.

Record concise source/test anchors, changed assumptions and reuse decisions in
the relevant existing contract or design document. Apply this review to the
already implemented inventory and payload checks before extending them. Keep
legacy recovery material read-only and under its original authority.

## Product boundaries

- Build a standalone host-side library with Canic as a required integration. Its
  Canic-owned adapter consumes the public contract; Backup production must not
  depend on Canic runtime/control-plane crates. Read the concrete boundaries in
  [the sibling review](docs/design/sibling-integration.md) before selecting providers.
- The implemented local slice covers initialization, inventory, payload checks and capture intent;
  read [its implemented contract](docs/architecture/local-inventory.md) before changes.
  Read the [capture-intent contract](docs/architecture/capture-intent.md) before changing
  plan, ownership or operation persistence, and the
  [transfer contract](docs/architecture/capture-transfer.md) before metadata/schedule work.
  Capture/restore follows the [operation contract](docs/design/operation-contract.md)
  and [local-store design](docs/design/local-store.md). Do not add speculative CLI,
  transport, storage, feature, simulator or qualification packages.
- Use published Platform identities, external management DTOs and codecs when
  those boundaries are implemented. Do not preselect unused runtime dependencies.
- Existing legacy artifacts and unfinished recovery obligations retain their
  original integration, IDs, authority and evidence. The restart grants no reset,
  destructive cleanup or permission to interpret old records under new schemas.
- Keep integrations responsible for membership, routing, application quiescence
  and external-effect settlement. Exact new authority/custody contracts remain
  to be designed before effects are implemented.
- The maintainer authorized the experimental direct-controller release on
  2026-11-16. Root workspace metadata admits only `crates-io`; the product inherits
  it with `publish.workspace = true`. Qualification and any future independent
  workspace or canister package remain non-publishable unless separately authorized.
- `0.100.0` began the unpublished incompatible restart. Preserve historical
  release identities and review consumer migration before publication.

## Dependency selection

Host FS/Artifacts 0.80.2 and HTTP 0.80.0 are selected from verified registry archives.
Backup owns its Reqwest 0.13.5 client; Host bounds complete response bodies. The
maintainer-authorized temporary sibling overrides have been removed after
consumer qualification. Keep dependencies registry-backed and builds in this
repository's target directory; do not add sibling patches or shared targets.

## Local development

[Setup](docs/development/setup.md) owns commands and validation. Production
libraries belong under `crates/`. Add a separate `qualification/` workspace only
when actual test-only packages justify one; product must never depend on it.
Use directory modules with `mod.rs` when modules exist. Keep tests with owners.

The private `qualification/` workspace owns actual management ingress through
Testkit 0.107.4 and its native `ic_testkit::pocket_ic` boundary. Product dependencies
never point back into it. Its independent lock selects PocketIC 16.1 and that
client's exact `thiserror` 2.0.18 constraint; the product lock retains 2.0.21.
The compatible product requirement admits both. Revisit this constraint when
PocketIC relaxes its pin. Fixture keys, upstream DTOs and runtime construction
remain test-only; no production identity alias or direct PocketIC dependency.

`make install-testkit-server` explicitly downloads the locked Testkit library's
selected archive and uses its authenticated offline preparation through the private
qualification example. No Testkit CLI is selected. Assets live under
`.tools/testkit-server/`; old `.tools/testkit/` evidence stays preserved.
`make testkit-server-check` admits the server offline; `make test-qualification`
runs the actual-IC caller under `qualification/target/`. Fixtures independently
call the locked Testkit library's offline check before managed startup and retain
server logs, original journals and byte evidence under that target. Keep failed
fixtures. Native core checks use root `target/` and do not run this separate
qualification target implicitly. Shared formatting covers both workspaces.

The configured native CI hosts are Linux x86-64 and macOS ARM64/x86-64; configuration
alone does not prove hosted support. Ordinary work leaves staging, commits and
remote delivery to the user unless specifically authorized. Update the handoff
after meaningful work. Preserve the legacy checkout and its recovery evidence.

## Post-push review focus

Apply the baseline's [post-push review](TOKO-TOOLING.md#post-push-review).
For Backup, check competing recovery paths, unnecessary stage APIs/authority
inputs, extra journals or persisted representations and legacy mechanisms without
a current consumer. Preserve original evidence, finite spending and recovery
guarantees. Keep detailed qualification in its owning contract, not this overlay.
