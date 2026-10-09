# Shared Tooling 0.1.34 and issue follow-up

Released base is 0.9.2, `f7868ce3f672da5ea1489172fd97e83699fe4f99`.
The maintainer explicitly chose fleet retirement; select the undated 0.10.0 draft
because the local report command changes behavior. Package versions, receipt and
the incoming Host 0.8.7/Metrics 0.2.16/Testkit 0.25.3 lock remain untouched.

Adopt committed Shared Tooling 0.1.33,
[`ddd3e1c01ba8aab13a56277e05679e43a8a9d88a`](https://github.com/dragginzgame/shared-tooling/tree/ddd3e1c01ba8aab13a56277e05679e43a8a9d88a),
through the canonical exporter in a clean isolated checkout. Four retained files
change. Deliberately remove the two complete verified fleet copies and manifest
rows, then export the narrowed 95-file selection. Every committed blob and
executable mode matches. The baseline stays unchanged; no vendored file is patched.

Shared Tooling 0.1.34 was committed during the review. Refresh the same clean
isolated source to [`3d33cd250fcae7dbe5cabe44b2abd6b2c91a1822`](https://github.com/dragginzgame/shared-tooling/tree/3d33cd250fcae7dbe5cabe44b2abd6b2c91a1822)
and export all 95 selected files again. Four additional selected files change:
concise handoff/issue guidance and literal manifest-path admission under CDPATH.
No additional selection or effect owner is introduced. Focused offline PocketIC
fixtures cover spaces, leading dashes, final newlines and directory loss before
Cargo; actual locked alignment and final ShellCheck remain distinct evidence.
All final source bytes/modes match. Native candidate qualification stays pending.

## Fleet retirement

[#30](https://github.com/dragginzgame/ic-backup/issues/30) and
[shared #83](https://github.com/dragginzgame/shared-tooling/issues/83) own the
optional selection. Delete `scripts/dev/cloc-tooling.pl` (279 code LOC) and
`scripts/ci/test-cloc-tooling.sh` (126), measured with prepared cloc 2.10. Remove
consumer Make/help/native-CI callers. The shared optional command diagnoses an
absent reporter and directs callers to Shared Tooling. Workspace `make cloc`,
setup, offline checks and required companions remain. No sibling is scanned.

Removed Perl subs are `usage`, `capture`, `read_file`, `write_file`, `safe_path`,
`is_linked`, `in_scope` and `load_snapshot`, all from the retired reporter. Shared
Tooling owns their fleet purpose. The deleted shell suite contains no named
functions/types. No Rust function, method or type is removed.

Focused shared-command wiring, prepared workspace-LOC fixture, actual workspace
LOC, snapshot integrity and ShellCheck pass. The first direct LOC fixture invocation
refused because the prepared cloc directory was absent from PATH; its failure
fixture/log remains. Supplying the already-installed directory passes without
installation. Actual retired entrypoint refusal (exit 2) is expected evidence.
#30 stays open for matching committed native caller qualification.

## HTTP fixture repair

[#31](https://github.com/dragginzgame/ic-backup/issues/31) records macOS WouldBlock
followed by a destructor panic/abort. Accepted sockets now explicitly select
blocking mode before finite reads under the original deadline. Apple's
[accept documentation](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/accept.2.html)
describes inherited socket properties; this motivates explicit mode selection,
but does not prove the uncommitted repair passes macOS.

Bound request headers at 16 KiB and the body at the existing 3 MiB signed-update
ceiling. Admit one exact Content-Length body; refuse malformed/duplicate lengths,
transfer encoding, trailing bytes and oversized input. Retry Interrupted within
the original deadline. Retain incomplete bytes and original IO kind/raw OS error/
message before exposing capture failure. Explicit finish reports worker failures;
Drop records cleanup diagnostics without masking an existing primary panic.

Eight Linux HTTP cases pass, including real fragmented nonblocking sockets,
incomplete-body timeouts and controlled worker failure during primary unwind.
Original one-request, signed body/routing and pending-spending assertions remain.
The same eight HTTP cases also pass on incoming Host 0.8.7. Both libraries pass
all-target/all-feature warnings-denied Clippy and Rust 1.88 on that graph,
including two independent normal consumers; formatting passes. Earlier compilation
and Clippy failures are retained separately from the corrected results. Production
transport, reservations and retry behavior are unchanged. #31 stays open for
native Intel/Apple Silicon and matching committed CI; #25 remains full transport
acceptance, and #29 remains installed orchestration/application/terminal work.

## Incoming Host graph

A concurrent lock update selects all four published Host 0.8.7 packages. Preserve
it; the agent performs no dependency reselection. Four non-yanked registry rows
and archive hashes match the selected lock, and 65 published Rust files match
clean committed VCS bytes. Dirty sibling proposals are excluded. Repeat HTTP,
artifact, Clippy and Rust 1.88/independent-consumer checks on this graph; the
earlier 0.8.6 results remain separate. Backup retains component-by-component
no-follow descriptor traversal and already delegates streaming reader arithmetic;
the new general path-based file hashing does not replace that custody boundary.

## Released archive acceptance

[#28](https://github.com/dragginzgame/ic-backup/issues/28) integration was committed
in 0.8.0. Current inspection confirms the existing explicit roots, pinned uploader
`ea165f8d65b6e75b540449e92b4886f43607fa02` and seven-day retention.
Native archive fixtures pass on all three 0.9.1 jobs in
[run 37816554603](https://github.com/dragginzgame/ic-backup/actions/runs/37816554603),
covering newline/colon filenames, hidden files, executable/0640 modes, symlink
policy, early input refusal and retained partial writer output.

Actual failed-job archive/upload succeeds while each original job stays failed:

| Host | Run/job | Downloaded exact artifact | Inner archive |
| --- | --- | --- | --- |
| Linux | [37808140696 / 113417715180](https://github.com/dragginzgame/ic-backup/actions/runs/37808140696/job/113417715180) | [11564225687](https://github.com/dragginzgame/ic-backup/actions/runs/37808140696/artifacts/11564225687) | 8 members, early failure logs |
| Intel macOS | [37816554603 / 113446554555](https://github.com/dragginzgame/ic-backup/actions/runs/37816554603/job/113446554555) | [11576766843](https://github.com/dragginzgame/ic-backup/actions/runs/37816554603/artifacts/11576766843) | 20,420 members, 2 links |
| Apple Silicon | [37816554603 / 113446555303](https://github.com/dragginzgame/ic-backup/actions/runs/37816554603/job/113446555303) | [11576004326](https://github.com/dragginzgame/ic-backup/actions/runs/37816554603/artifacts/11576004326) | 20,420 members, 2 links |

Downloaded ZIPs contain only `evidence.tar.gz`. Inspecting their inner archives
retains exact relative names, modes, link targets and regular-file hashes. Each
macOS payload has 4,172 private-mode entries. Ordinary payloads have no newline/
colon names: those cases are separately qualified by native fixtures and upstream
hosted uploader acceptance, rather than invented in these downloads. Archive
acceptance does not turn the failed HTTP jobs into successful product CI.

Raw evidence remains in `target/issues-shared-033-review-01/`, including downloads,
full archive manifests, native logs, API responses, removed-source copies and
check logs. [Qualification](qualification.json) binds their hashes and scope.
The initial Shared 0.1.33 and released 0.9.2 CI were in progress at inspection. No hosted
workflow is triggered, and this uncommitted candidate has no native CI evidence.
No dependency reselection, sibling edit, Git/release, publication or live IC effect
is performed.
