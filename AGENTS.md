# AGENTS.md

Apply the installed [Toko Tooling baseline](TOKO-TOOLING.md) and its revision-bound
references. [.toko-tooling.snapshot](.toko-tooling.snapshot) owns managed files;
never edit that receipt or files under `vendor/toko-tooling/` by hand.

Read [README.md](README.md), [the handoff](docs/status/current.md) and
[the restart direction](docs/design/restart.md) at session start. This is a
scaffold phase. Implement product behavior only after its contract is selected.
The legacy checkout is read-only reference material; do not import its source,
backlog, caches, build outputs, journals or release state wholesale.

## Product boundaries

- Build a standalone host-side library with Canic as a required integration. Its
  Canic-owned adapter consumes the public contract; Backup production must not
  depend on Canic runtime/control-plane crates. Read the concrete boundaries in
  [the sibling review](docs/design/sibling-integration.md) before selecting providers.
- Keep `ic-backup` API-free until a first vertical slice is agreed. Do not add
  speculative CLI, transport, storage, feature, simulator or qualification packages.
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
- `0.68.0` is an unpublished incompatible restart draft. Preserve historical
  release identities and review consumer migration before publication.

## Local development

[Setup](docs/development/setup.md) owns commands and validation. Production
libraries belong under `crates/`. Add a separate `qualification/` workspace only
when actual test-only packages justify one; product must never depend on it.
Use directory modules with `mod.rs` when modules exist. Keep tests with owners.

The configured native CI hosts are Linux x86-64 and macOS ARM64/x86-64; configuration
alone does not prove hosted support. Ordinary work leaves staging, commits and
remote delivery to the user unless specifically authorized. Update the handoff
after meaningful work. Preserve the legacy checkout and its recovery evidence.
