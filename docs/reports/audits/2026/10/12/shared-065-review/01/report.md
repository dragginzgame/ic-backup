# Shared Tooling 0.6.5 inspection

Shared Tooling remote main selects committed 0.6.5
`42f2091768b5f71c98a684d3dab563f408ca2dd9`. Backup's actual recorded snapshot
remains 0.6.4 `6fc8eb9a94f83fb0b9d0a26377d4232508cdbe2d`.
This is inspection and canonical preview, not adoption or full consumer qualification.
A clean isolated source checkout excludes the sibling's newer dirty catalog/docs edits.
The preview changes nine selected files and keeps the 102-file roster, with no
missing snapshot companion or published library contract change.

## Consumer impact

The validation runner accepts repeated `--input FILE` declarations and compares
SHA-256 identities before/after each target. Persistent drift or failed observation
stops subsequent dispatch and retains evidence. Same-checkout nested gates inherit
the checkpoint; foreign fixtures establish their own. This is checkpoint detection,
not isolation from arbitrary edits within a running target. Backup currently opts
in no files, so exporting the new runner alone would leave this feature inactive.
The smallest consumer change is to declare the root Cargo.toml and Cargo.lock at
the existing `make ci` stage loop; any additional maintained input declaration
belongs to Backup's actual gate contract.

The runner now always snapshots `verify-file-checksum.sh` with its execution helper.
Backup's release fixture copies the runner without this companion. Adoption must
extend that fixture export; a bounded exact-source probe confirms omission refuses
before dispatch. The root snapshot already contains the companion.

The staged hook supplies an exact exported-worktree context for prepared Rust-tool
receipt checks. Pins/compiler still come from the index export; checks remain
non-installing and reject mismatches. Existing stronger consumer prerequisites stay
intact. Backup's default formatting owner does not add this stronger receipt
prerequisite, so the new context is support for callers rather than a new installed
Backup formatter or tool identity. Common versions/build inputs stay unchanged;
no reinstall is indicated.

Shared Make guidance now requires fixtures to substitute effectful scripts or use
single-recipe fixture Makefiles. Backup's examined release/tool-routing fixtures
already use command substitution; retained qualification logs contain no recipe
warning matches. No local recipe-override cleanup is demonstrated.

## Evidence and disposition

A custom bounded fixture copied the exact new runner/checksum/execution helpers.
On Bash 5 and genuine Bash 3.2.57 on Linux, a target mutating its declared Cargo.lock
refuses before the following target dispatches, retaining original/observed hashes.
This is a substitute Make probe, not Cargo preparation, native macOS qualification,
producer suite execution or a full Backup gate. The missing-companion probe refuses
before effects as expected. Source, graph, index and HEAD remain unchanged.
Ignored preview/source/probe evidence stays at `target/shared-065-review/`;
[compact evidence](qualification.json) identifies the reviewed files and logs.

Exact-source [Shared 0.6.5 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38218219902)
passes lint/security, with Linux running and both macOS jobs queued at inspection.
Shared 0.6.4 owner CI passes completely. Candidate Backup adoption has no hosted
result because it has not been applied. Preserve the separately qualified pending
0.20.0 Metrics work and its original evidence. Product APIs, records and recovery
owners are outside this inspection; no version, Git, release, setup or sibling edit
is performed.

Recommended adoption belongs to [Backup #36](https://github.com/dragginzgame/ic-backup/issues/36),
with input detection owned by [Shared #129](https://github.com/dragginzgame/shared-tooling/issues/129)
and fixture guidance by [Shared #128](https://github.com/dragginzgame/shared-tooling/issues/128).
