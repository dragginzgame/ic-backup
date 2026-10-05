<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-backup/ic-backup-readme-header.svg" alt="IC Backup — Verified backups and safe recovery for Internet Computer apps" width="100%">
</p>

<!-- helper-navigation:start -->
<p align="center">
  <a href="https://github.com/dragginzgame/canic"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/canic.svg" width="18" height="18" alt=""> <strong>canic</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/icydb"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/icydb.svg" width="18" height="18" alt=""> <strong>icydb</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-timers"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-timers.svg" width="18" height="18" alt=""> <strong>ic-timers</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-memory"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-memory.svg" width="18" height="18" alt=""> <strong>ic-memory</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-query"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-query.svg" width="18" height="18" alt=""> <strong>ic-query</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-backup"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-backup.svg" width="18" height="18" alt=""> <strong>ic-backup</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-blob-storage"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-blob-storage.svg" width="18" height="18" alt=""> <strong>ic-blob-storage</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-testkit"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-testkit.svg" width="18" height="18" alt=""> <strong>ic-testkit</strong></a>
</p>
<!-- helper-navigation:end -->


# Releasing

The command family follows `ic-delegated-auth` and `ic-blob-storage`, using their
MIT-licensed bounded Bash/Perl helpers adapted to one host-side workspace.
The library inherits `publish = ["crates-io"]` from workspace metadata.
Creating release tooling does not authorize a release or claim an implemented
backup/restore product.

> **Authority:** Preview and local validation commands are available to
> contributors. Commands that create commits, tags, pushes or registry uploads
> remain maintainer-owned.

## Command effects at a glance

| Command | Effect | Owner |
| --- | --- | --- |
| `make release-plan VERSION=…` | Inspects and previews a proposed version; changes nothing | Contributor |
| `make release-check` | Tests the release helpers locally; performs no release | Contributor |
| `make patch`, `make minor`, `make major`, `make bump-x VERSION=…` | Validates and prepares the bounded release files | Maintainer or explicitly authorized preparation |
| `make release-patch`, `make release-minor`, `make release-major`, `make release-x VERSION=…` | Prepares, commits, tags and atomically pushes the repository release | Maintainer only |
| `make package` | Verifies a local package without uploading | Contributor |
| `make publish-dry-run` | Runs Cargo's registry checks without uploading | Maintainer |
| `make publish` | Uploads the current library version to crates.io | Maintainer only |

## Preview and preparation

`make release-plan VERSION=patch` accepts patch, minor, major or an exact x.y.z
and performs no effects. `make patch`, `make minor`, `make major` and
`make bump-x VERSION=x.y.z` prepare release files after validating clean committed
source. Only the first release may reuse the initial package version, before
any version tag or completed local receipt exists. Subsequent releases must
increase it. Breaking pre-1.0 contracts use a minor version and a hard cut;
routine implementation does not allocate a version per slice.

Keep the latest release or one populated current draft at the top of
`CHANGELOG.md`, without an Unreleased section or separate notes queue. Use
`## [Draft]` while the next version is undecided. A maintainer-selected undated
numbered draft may occupy the same top position. The explicit preparation/release
command selects the final version; an earlier numbered future draft is provisional.
Preparation relabels that section and assigns the date, preserving its notes and
historical bytes. For example, with package `0.2.0` and draft `0.2.1`,
`make release-minor` selects `0.3.0` and relabels the same notes during preparation;
`make release-patch` selects `0.2.1`.
It rejects empty, duplicate, misplaced or competing drafts before mutation.
Undated imported historical versions remain history, not competing future drafts.
Preparation cannot promote a historical top section to a new draft or overwrite
an existing target version. Failed preparation restores the exact original label
and notes along with the other release files.
Changelog presentation does not gate registry publication.

The workspace manifest owns the package version; the changelog owns selected
notes and dated history. The initial `0.1.0` history remains undated. Selecting
draft notes runs no release/version transaction.
Finish source, changelog and handoff edits before the maintainer commits and
starts release work. Agents must never create or amend commits, including
through release scripts.

Preparation runs `make release-verify`, the complete native and tooling gate
described in [development](development.md). Required tools are the pinned Rust
toolchain, rustfmt, Clippy, Rust 1.91.0, Bash, Git, Make, Perl with core JSON::PP
and Digest::SHA, ripgrep, flock, ShellCheck and a SHA-256 implementation. Local
host-specific setup, including GNU Make and flock on macOS, is documented in
[the supported host matrix](development.md#supported-host-scope).
Shared Tooling snapshot verification precedes locked dependency fetch and offline
compilation. Version mutation uses an offline Cargo update.

After validation, preparation updates only root `Cargo.toml`, `Cargo.lock`,
`CHANGELOG.md` and generated `docs/release.json`. The v1 receipt binds the exact
validated source commit, date, version and hashes of the manifest, lockfile,
changelog and unchanged member manifest. Package versions inherit the root
workspace value. Failed mutation restores the original release files and any
prior receipt. Preparation never stages, commits, tags, pushes or publishes.

`make release-check` reruns isolated helper regressions here; copied tests are
not qualification until executed against the adapted helpers. It exercises
the real `release-patch`, `release-minor` and `release-major` Make entry points
against substituted Git/Cargo/validation commands. `make hooks-check`
checks formatting/index preservation without creating commits. Failed fixtures
retain logs under `target/`.

## Maintainer execution and recovery

From committed clean `main` with the intended origin configured, the maintainer
can use `make release-patch`, `make release-minor`, `make release-major` or
`make release-x VERSION=x.y.z`. These prepare, stage the exact release files,
create a release commit and annotated tag, and atomically push `main` and that
tag. Agents must not run commit-producing commands.

Individual commands are `release-stage`, `release-commit`, `release-tag-check`
and `release-push`. Staging must start at the validated source. The release
commit must directly follow that source. Pushing requires a clean `main`, the
matching annotated tag at HEAD and unchanged receipt-bound release files.
It uses no force push and sends no unrelated tags.

After failed preparation, fix the cause and retry. After successful preparation,
resume the failed step without bumping again. If tagging or pushing fails after
commit, retain the commit/tag and reconcile before retrying the exact step.
Release operations retain build artifacts. `make clean` is a separate explicit
cleanup action; never discard recovery evidence as ordinary cleanup.

## Registry publication

`make package` verifies a local package during development without uploading.
`make publish-dry-run` and `make publish` delegate directly to
`cargo publish --locked --registry crates-io -p ic-backup`, with `--dry-run`
for the former. Cargo owns package-file cleanliness, locked dependency checks,
package compilation, registry eligibility and authentication. Commit changes to
files included in the package before publishing. The dry run verifies without
uploading; `make publish` uploads the current library version.

Receipts and annotated tags govern the repository release transaction. Registry
publication uses the current checkout independently of that receipt and tag;
development changelog edits and later publishing-configuration commits do not
require an extra version bump just to satisfy the release helpers. Cargo and
crates.io enforce whether the chosen package version can be published.
Publication leaves existing receipts, versions and tags unchanged.

Repository tags, registry publication and live IC backup/restore are separate
effects. Passing the native gate proves only implemented behavior; platform
and application safety need their own qualification evidence.
