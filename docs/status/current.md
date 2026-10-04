# Current handoff — 2026-10-04

The maintainer requested Rust repository setup following `ic-delegated-auth`
and `ic-blob-storage`, as part of moving independent mechanisms out of Canic.
The Rust foundation is complete and uncommitted for review. Canic and sibling
repositories were inspected read-only; their source and dirty worktrees were
not changed.

The workspace-only root owns the initial unreleased `0.1.0` version, Rust 2024
metadata, dependencies and lints. Its only member is `crates/ic-backup`, with
a documented library entry point and no backup/restore API yet. Development
Rust is pinned to 1.99.0; MSRV is 1.91.0. The lockfile has no dependencies.
Registry publication is disabled. Transport, CLI and IC fixtures have not been
created.

Native Make commands, Linux CI, formatter hooks and bounded release helpers
follow the sibling conventions, using this repository's own `target/`.
`core.hooksPath` is set locally to `.githooks`. The adapted helpers retain MIT
attribution and exact inspected source hashes in
[tooling provenance](../tooling-provenance.json). See
[development](../development.md) and [releasing](../releasing.md).

The initial Unreleased changelog now covers the documentation bootstrap,
workspace, native tooling, formatter hooks and release workflow. All three
public Make entry points (`release-patch`, `release-minor`, `release-major`)
were exercised against isolated substitutes, checking their selected versions
and validation/stage/commit/tag/push sequence. Shell checks, release regressions
and initial changelog validation passed. The real package version remains
`0.1.0`; no release effects ran.

Targeted validation passed: formatting, native all-target/all-feature compilation,
warning-denied Clippy and docs, Rust 1.91.0 compilation, and standalone Cargo
package verification. Release-helper and Git-hook regressions were rerun here
and passed, including failure/retry, index preservation and dependency-bootstrap
behavior. These tooling fixtures create no real commits, tags, pushes or uploads.
The empty library test/doctest harness passes but supplies no backup/restore
qualification. No full CI/release gate or PocketIC suite was run.

Retained logs: `target/repository-setup-tooling.log` and
`target/repository-setup-package.log`. The packaged crate compiled independently
under `target/package/`; build artifacts remain retained.

Read [the design](../extraction-design.md) for the proposed architecture,
contracts, extraction inventory, recovery cases, testing and implementation
sequence. [The baseline](../source-baseline.json) records source hashes and
the inspected Canic HEAD; its working-tree files, not that commit alone, were
the planning input. These hashes are provenance, not test evidence.

The next product implementation batch is B1: refresh the source inventory, trace public
consumers and freeze the current v1 contracts. The current Canic backup crate
has no direct Canic dependencies, but contains Canic-specific Root/Fleet schema
and consistency assumptions. Its live CLI backup preflight remains unavailable.
Neither independence nor live backup availability is established by the design.

The public remote is
[dragginzgame/ic-backup](https://github.com/dragginzgame/ic-backup). The maintainer
previously authorized a one-time initial documentation commit and push; that
published commit is `680baf9`. No further commit or push ran during Rust setup.
The standing no-commit rule remains in force. Repository foundation readiness
does not establish B1 contract completion, independent backup/restore usability
or completed extraction. Canic adoption, release/publication and live IC effects
require their own instructions.
