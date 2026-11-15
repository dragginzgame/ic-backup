# AGENTS.md

Apply the installed [Toko Tooling baseline](TOKO-TOOLING.md) and its revision-bound
references. [.toko-tooling.snapshot](.toko-tooling.snapshot) owns managed files;
never edit that receipt or files under `vendor/toko-tooling/` by hand.

Read [README.md](README.md), [the handoff](docs/status/current.md) and
[the restart direction](docs/design/restart.md) at session start.
Local inventory, payload integrity, capture intent, transfer planning, bounded reply
decoding, direct-host status/list/take continuation and local settlement from
surviving original reply evidence are implemented. Separate observation-certificate
authentication and strict signed lookup admission are local helpers. One-shot
read-state transport and one durable lookup per operation are qualified. Terminal
retained observations can settle the original call locally; replay preserves
the separate observation time policy and dependent effects require live admission. PocketIC
qualifies these calls, interrupted reply settlement and offline replay with fixture signing and application authority.
Durable metadata acquisition authenticates the retained take, reserves finite
signing/network allowance and retains exact certified evidence for offline replay. Canic process/maintenance
adapters, download and restore remain unqualified. Continue the selected capture/restore scope through the contracts below; distinguish proposals from implemented behavior.
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
- Keep every package non-publishable until the user authorizes publication of
  the mature implementation. Root workspace metadata sets `publish = false`;
  members inherit it with `publish.workspace = true`. Any future independent
  workspace or canister package must also disable publication.
- `0.100.0` began the unpublished incompatible restart. Preserve historical
  release identities and review consumer migration before publication.

## Dependency selection

Host FS/Artifacts 0.79.0 are selected from verified registry archives. The
maintainer-authorized temporary sibling overrides have been removed after
consumer qualification. Keep dependencies registry-backed and builds in this
repository's target directory; do not add sibling patches or shared targets.

## Local development

[Setup](docs/development/setup.md) owns commands and validation. Production
libraries belong under `crates/`. Add a separate `qualification/` workspace only
when actual test-only packages justify one; product must never depend on it.
Use directory modules with `mod.rs` when modules exist. Keep tests with owners.

The private `qualification/` workspace owns actual management ingress through
Testkit 0.106 and its native `ic_testkit::pocket_ic` boundary. Product dependencies
never point back into it. Its independent lock selects PocketIC 16.1 and that
client's exact `thiserror` 2.0.18 constraint; the product lock retains 2.0.21.
The compatible product requirement admits both. Revisit this constraint when
PocketIC relaxes its pin. Fixture keys, upstream DTOs and runtime construction
remain test-only; no production identity alias or direct PocketIC dependency.

`make install-testkit-server` selects the unique locked Testkit CLI through the
shared Cargo installer, then uses Testkit's authenticated setup.
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

After every successful push, apply the installed baseline's
[post-push review](TOKO-TOOLING.md#post-push-review) to the delivered commits and
directly affected callers before starting the next increment. In Backup, explicitly
check architectural drift, creeping complexity and legacy sediment: duplicated
policy or recovery paths, unnecessary stage APIs/authority inputs, extra journals
or persisted representations, and mechanisms imported without a current consumer.
Prefer reuse, deletion and simplification while preserving original evidence,
finite spending and recovery guarantees. Report concrete findings and their
smallest correction, or the reviewed scope with no actionable findings. This
adds no separate report framework and does not authorize another push.
