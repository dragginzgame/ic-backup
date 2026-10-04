# Current handoff — 2026-10-04

The maintainer authorized copying backup machinery from Canic. The first local
artifact/persistence extraction is complete and remains uncommitted for review.
Canic and sibling repositories were inspected read-only; their source and
existing worktree state were preserved. No live IC effects, release transactions, commits
or pushes ran during this batch.

`crates/ic-backup` now provides canonical SHA-256 checksum records, descriptor
based no-follow artifact traversal and private staging, verified durable
directory publication/recovery, bounded JSON reads, durable JSON create/replace
and nonblocking journal locks. Pure checksum records live in `model`; host IO
lives in `ops`. Read [the implemented boundary](../extraction-boundary.md) for
the maintained contracts, caller responsibilities and remaining ownership work.
There are no capture/restore runners, transport or CLI packages yet.

[Fresh source provenance](../extraction-source.json) records the inspected Canic
working tree at HEAD `3978e02d28fea9022c7a6e84a7ec6d6e4a0d4af3`, exact file
hashes, copied inputs/destinations and consumer references. Canic was clean at
this inspection; the exact hashes also identify the copied bytes. The original
[planning baseline](../source-baseline.json) remains retained. Both records are
provenance, not test evidence. Existing Canic consumers and source remain in place.

Targeted fresh qualification passed on Linux: 28 unit tests and one public-API
integration test, warning-denied Clippy and documentation, formatting, Rust
1.91.0 all-target/all-feature compilation and standalone Cargo package
verification. The copied regressions and new cases exercise exact checksums,
unsafe entries, private staging, bounded reads, create-only publication and
acknowledged child-process death around JSON/directory publication. The public
journey retains original intent, adopts matching published bytes after a lost
reply and rejects later corruption. This is local persistence/process evidence;
no PocketIC or real local IC backend was used and no IC effects are qualified.
No broad CI/release validation ran.

Retained logs: `target/extraction-local-tests.log`, `target/extraction-msrv.log`,
`target/extraction-docs.log` and `target/extraction-package.log`. The packaged
crate compiled independently under `target/package/`; build artifacts remain
retained. Earlier foundation/tooling logs and provenance remain available.

The workspace-only root owns version `0.1.0`, Rust 2024 metadata, dependencies
and lints. Development Rust is pinned to 1.99.0; MSRV is 1.91.0. Dependencies
inherit from this workspace and use no sibling Cargo patches or Canic crates.
Registry publication is disabled. Make commands, Linux CI and release helpers
follow the sibling conventions using this repository's own `target/`.
`core.hooksPath` is set locally to `.githooks`. See
[development](../development.md), [releasing](../releasing.md) and
[tooling provenance](../tooling-provenance.json).

The changelog retains the original undated `0.1.0` notes and has one active
undated `0.1.1` draft below the empty Unreleased section. This batch's completed
changes are recorded in `0.1.1`. Cargo metadata and the lockfile remain at
`0.1.0`; no tag or release receipt exists. All three Make release entry points
were previously exercised against isolated substitutes, with no real release
effects. Maintainers own commit-producing release commands.

Read [the design](../extraction-design.md) for the proposed architecture and
implementation sequence. The inventory refresh, consumer trace and local
contracts are implemented; full B1/B2 completion is not established. Generic v1
authority/consistency, budget, journal/transition and executor contracts still
need to be specified before importing runners. Layout lifetime/restore-reference
retention and command custody also remain in Canic. Its backup executor topology
preflight still rejects. This crate is not yet an independently usable canister
backup/restore product.

The public remote is
[dragginzgame/ic-backup](https://github.com/dragginzgame/ic-backup). The initial
documentation commit/push used a one-time authorization (`680baf9`); the
maintainer committed the Rust foundation as `edc8e4b` (`0.1.0`). The standing
no-commit rule remains in force. Canic adoption, release/publication and live
IC effects require their own instructions.
