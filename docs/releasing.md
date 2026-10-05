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

Patch, minor and major use the [common release contract](releases.md) and the exact
vendored Shared Tooling runner. Consumer adapters own Cargo metadata, original-input
validation and the v1 release receipt. The maintainer owns commits, tags and pushes;
agents must never execute these one-shot commands or resume them. Publication is a
separate explicit action. Build, validation and recovery artifacts remain retained.

| Command | Effect | Owner |
| --- | --- | --- |
| `make release-plan VERSION=minor` | Preview the proposed version without effects | Contributor |
| `make release-check` | Run isolated release adapter/runner regressions | Contributor |
| `make release-patch` | Increment patch, validate, prepare, commit/tag and atomically push | Maintainer |
| `make release-minor` | Increment minor/reset patch through the same workflow | Maintainer |
| `make release-major` | Increment major/reset lower components through the same workflow | Maintainer |
| `make release-resume VERSION=X.Y.Z` | Continue that exact retained release plan | Maintainer |
| `make release-tag-check` | Inspect current local annotated tag and receipt | Contributor |
| `make publish-dry-run` | Run Cargo registry admission without uploading | Maintainer |
| `make publish` | Upload the current package to crates.io | Maintainer |

## Pending notes and compatibility

Read [the changelog rules](../rules/changelogs.md) before maintaining notes. Keep
one current pending entry above finalized history and preserve historical bytes.
The selected undated `0.3.0` includes the complete pending batch since `0.2.3`;
its release workflow hard cut requires a minor version before 1.0. A numbered
pending heading must agree with the selected release command. Changelog presentation
does not gate registry publication or prove package publication.

The workspace owns package versions; selecting notes changes neither manifest nor
lockfile. `release-plan` accepts patch/minor/major or an exact preview, but one-shot
release execution accepts the three common increments. The old `bump-x`, `patch`,
`minor`, `major`, `release-x`, `release-stage`, `release-commit` and `release-push`
commands are removed. Use the selected common one-shot command; after interruption,
use `release-resume` once preparation has started. A preflight or validation-only
failure retries the normal target against current reviewed source with fresh
preflight and the complete gate. Do not infer completion from old
standalone prepared metadata: inspect its original receipt/source/tag and retained
backups before choosing a disposition. No compatibility release workflow is retained.

## Maintainer execution

Commit and review the completed source before starting a release. Set the intended
`RELEASE_REMOTE` and `RELEASE_BRANCH` explicitly when they differ from `origin` and
`main`. Exactly one release selection is admitted. All three increments run the same
complete `release-verify` gate; see [development](development.md). Required tools are
GNU Make, Bash 3.2 or newer, Git, core Perl modules, SHA-256, ShellCheck, ripgrep, flock
and cargo-sort 2.1.4 plus the declared Rust toolchains. Install the formatter during
explicit developer setup; validation never installs it. Supported host evidence
remains separately qualified.

The runner selects the source SHA, previous/candidate versions, UTC date, branch and
single push URL before mutation. Preflight admits pending notes and rejects unrelated
staged/unstaged/untracked work and changed original manifest/lock/receipt. It verifies
the snapshot and fetches the selected locked dependency cache. The full gate runs
offline; a consumer validation sidecar binds the original source, selections and
manifest/lock/member/notes bytes before preparation. Missing evidence is never approval.

Preparation requires those exact original inputs. It finalizes the matching pending
notes and edits only root `Cargo.toml`, this package's `Cargo.lock` version and generated
`docs/release.json`. No dependency update occurs. Cargo checks the selected offline
metadata and non-mutating `fmt-check`, and the unchanged v1 receipt binds source,
date, version, gate and exact
manifest/lock/member/changelog hashes. Exact original backups remain under
`target/release-backup.*`; ordinary preparation failure restores the original metadata.

The shared runner alone stages the four explicit release files, records the exact
index tree, creates `Release X.Y.Z` and annotated `vX.Y.Z`, and pushes exactly:

```bash
git push --no-follow-tags --atomic "$remote" \
  "HEAD:refs/heads/$branch" "refs/tags/v$candidate:refs/tags/v$candidate"
```

This disables implicit tag publication and requires both selected refs to succeed
together. It performs no registry upload, deployment, additional version bump or
post-release cleanup. Read-only plan inspection and isolated tests do not authorize
running this workflow.

## Exact interruption recovery

Preflight and validation-only failures create no new release plan or prepared
metadata. Retry the normal selected target after fixing/reviewing source; it selects
the current source and repeats preflight and the full gate. Earlier failure logs
remain retained. Safe earlier plans at preflight/validate are archived in unique
attempt directories after checking base, destination and absence of preparation/tag
effects. An earlier validation sidecar is also copied to a unique retained directory
before a new successful gate records its exact original inputs; no proof is reused.

Immediately after successful validation, before preparation can mutate metadata,
the runner retains `.git/release-state/X.Y.Z.plan`, its NUL-separated release paths
and the consumer `.validation.json` evidence. The directory lock records its owner;
inspect that owner before disposing of a stale lock. Never steal an active release
lock or discard plans, source backups, validation logs or build artifacts as cleanup.

Inspect the saved source/date/version/destination, current metadata/index/HEAD/tag
and exact remote refs before resuming. `make release-resume VERSION=X.Y.Z` uses the
saved phase without incrementing again. Completed commit/tag/push identities are
reconciled rather than blindly repeated. Lost push replies can resolve to the exact
already-published branch and tag. Changed destinations, unrelated work, receipt drift,
missing original validation and conflicting identities reject. A normal release
target rejects every unfinished prepared plan, even if a partial/prepared manifest
would otherwise select a different next version. Resume never increments again.

The local resume wrapper checks retained prepared evidence before dispatch and the
exact local annotated tag after completion. Completed replay validates local receipt/
tag evidence; the runner's complete phase performs no remote push or observations.
An interrupted multi-file metadata replacement may leave a partial candidate; this
fails prepared admission. Review the exact saved inputs/backups and restore or finish
the intended candidate before resuming. Do not launch a fresh increment from partial
metadata, overwrite a tag, force-push or fall back to separate branch/tag pushes.

## Registry publication

`make publish` delegates the current library package's admission to Cargo with
`--locked --registry crates-io`; the package inherits publication policy from the
workspace. `make publish-dry-run` performs that admission without uploading. Neither
command uses changelog presentation, release receipt or local tag as a registry gate.
Both share release exclusion and perform no repository version or Git transaction.
A successful repository push proves neither crates.io publication nor downstream
adoption. Follow-up publication is an explicit maintainer action.

## Focused qualification

`make release-check` exercises actual consumer Make entry points with Git/Cargo
substitutes, plus the exact vendored runner's command-stub regressions. It covers
all three increments, identical phase order, exact staging/push, failure before
mutation, dependency bootstrap, original-input validation, retained metadata/history,
rollback, lost commit/tag/push replies and completed replay. Fixture command traces
and failure evidence are retained under `target/shared-release-tests.*`.

The consumer suite clears inherited release identities and GNU Make recursion/
override state in its child process; each fixture owns its exact context. The
unchanged shared runner suite uses its own `make` substitute, including in macOS
system-Bash CI. Qualification under parent release variables and command-line
overrides checks this boundary. Explicit source mismatches still reject before
recording validation evidence; successful dependency targets do not bypass that
production guard.

`make shell-check`, `make shared-tooling-check` and `make tooling-check` cover syntax,
lints, pinned bytes/modes and rejection/evidence behavior. These are focused tooling
checks, not actual releases, native macOS qualification or real registry publication.
The full `make ci`/`make release-verify` gate remains separately authorized/configured.
