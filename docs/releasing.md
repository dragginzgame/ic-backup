# Releasing

The command family follows `ic-delegated-auth` and `ic-blob-storage`, using their
MIT-licensed bounded Bash/Perl helpers adapted to one host-side workspace.
The library inherits `publish = ["crates-io"]` from workspace metadata.
Creating release tooling does not authorize a release or claim an implemented
backup/restore product.

## Preview and preparation

`make release-plan VERSION=patch` accepts patch, minor, major or an exact x.y.z
and performs no effects. `make patch`, `make minor`, `make major` and
`make bump-x VERSION=x.y.z` prepare release files after validating clean committed
source. Only the first release may reuse the initial package version, before
any version tag or completed local receipt exists. Subsequent releases must
increase it. Breaking pre-1.0 contracts use a minor version and a hard cut;
routine implementation does not allocate a version per slice.

Keep one populated `## [Unreleased]` changelog section. A maintainer-selected
undated numbered draft may immediately follow an empty Unreleased section.
Preparation promotes/dates that draft while preserving historical notes.
The maintainer completed the tagged `0.1.1` repository release; initial setup
notes remain under undated `0.1.0`. Add subsequent completed changes to
Unreleased. Cargo metadata remains at `0.1.1` until preparation selects its
successor.
Finish source, changelog and handoff edits before the maintainer commits and
starts release work. Agents must never create or amend commits, including
through release scripts.

Preparation runs `make release-verify`, the complete native and tooling gate
described in [development](development.md). Required tools are the pinned Rust
toolchain, rustfmt, Clippy, Rust 1.91.0, Bash, Git, Make, Perl with core JSON::PP
and Digest::SHA, ripgrep, flock and ShellCheck. Locked dependency fetch precedes
offline compilation. Version mutation uses an offline Cargo update.

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
`make publish-dry-run` and `make publish` require the clean tagged release and
its matching receipt, then delegate to Cargo for crates.io checks. The dry run
performs verification without uploading; `make publish` uploads the package.

The `0.1.1` tag was prepared with publication disabled. Enabling it changes the
manifest bound by that release receipt. Commit the configuration fix, then use
`make release-patch` to prepare, tag and push the next release before running
`make publish-dry-run` and `make publish`. Retain the original tag and receipt
until ordinary preparation creates the successor receipt; do not rewrite them
to admit different source under `0.1.1`.

Repository tags, registry publication and live IC backup/restore are separate
effects. Passing the native gate proves only implemented behavior; platform
and application safety need their own qualification evidence.
