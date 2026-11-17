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

Before substantive subsystem work, inspect the relevant `ic-backup-legacy`
implementation, regressions and explanatory history read-only, starting from the
revision in [the legacy assessment](docs/design/legacy-assessment.md). Identify
the original failure, safety/recovery guarantee and changed assumptions. Reuse
sound solutions within current Platform/Host ownership; do not import old
architecture, dependencies or recovery authority wholesale. Carry applicable
regressions into the owning tests and run them against the replacement, preserving
revision/licence provenance and explaining exclusions. IC effects require an
actual IC backend; native filesystem tests or fakes are insufficient.
Record concise decisions in the existing owning contract/design, including before
extending the implemented inventory and payload checks.

## Product boundaries

- Build a standalone host-side library with Canic as a required integration. Its
  Canic-owned adapter consumes the public contract; Backup production must not
  depend on Canic runtime/control-plane crates. Read the concrete boundaries in
  [the sibling review](docs/design/sibling-integration.md) before selecting providers.
- Current implemented capability and qualification status belong in the handoff.
  Read [the inventory contract](docs/architecture/local-inventory.md) before changes.
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
  and external-effect settlement. Follow the implemented direct-controller
  authority/custody contracts; the manager-route proposal remains separate and
  must not be treated as an enabled provider.
- The maintainer authorized the experimental direct-controller release on
  2026-11-16. Root workspace metadata admits only `crates-io`; the product inherits
  it with `publish.workspace = true`. Qualification and any future independent
  workspace or canister package remain non-publishable unless separately authorized.
- Preserve historical release identities and review consumer migration before
  publication; the changelog owns restart history and delivered versions.

## Dependency selection

Host FS/Artifacts and HTTP are selected from verified registry archives; the
handoff records the current locked versions and qualification evidence.
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
the locked Testkit library and its native `ic_testkit::pocket_ic` boundary. Product dependencies
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
