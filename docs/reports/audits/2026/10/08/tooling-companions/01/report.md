# Installer fixture companion repair

The maintainer's complete `tooling-check` failed at consumer commit `9be366b`.
The retained log identifies status 127 from missing
`scripts/ci/test-tool-evidence.sh`. Both adopted host/IC installer suites call it
unconditionally. Earlier focused qualification did not run those suites; its
recorded passes do not qualify this complete target.

Adopt committed Shared Tooling correction
`db039347d2372b877c1c46dcdd2b5c3aa9412009` from
[upstream #73](https://github.com/dragginzgame/shared-tooling/issues/73).
The canonical exporter now rejects the old incomplete selection before manifest
replacement. Explicitly add `test-tool-evidence.sh`, `select-tool-evidence.sh` and
`.github/actions/retain-failure-evidence/action.yml`. All 80 source bytes/modes
match the exact committed source; six previous paths change and three are added.
No vendored source is patched. [Source proof](source-proof.json) binds the inventory.

The complete `make tooling-check` target passes on Linux, including both installer
suites and the actual shared collector shell body against substituted binaries and
downloads. It covers successful compaction, failed verification, unknown selections,
selection changes and retained candidates. Shell and dependency declaration checks
also pass. Native macOS and hosted upload are not newly qualified.

The shared action is an unconditional fixture dependency. Consumer CI retains its
own explicit archive roots, pinned uploader and seven-day retention. Importing this
action does not select it for production CI. No Rust, library graph, package version,
spending, record, recovery or publication contract changes; no symbol is removed.

[Qualification](qualification.json) keeps original failure, incomplete-selection
refusal, canonical refresh, preparation failures and corrected logs separate.
The [previous review](previous-review.json) retains its original source/scope.
The compatible 0.8.1 draft stays undated; package/receipt remain 0.8.0. Work remains
uncommitted. No root Git writes, sibling edits, release, registry publication or
live IC effects run, and the original failed fixture remains retained.
