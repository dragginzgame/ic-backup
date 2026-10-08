# Host 0.5.1 and Shared Tooling 0.1.24 adoption

The 0.7.1 draft adopts compatible upstream selections after delivered 0.7.0.
Package and release receipt remain 0.7.0. The incoming lockfile is preserved exactly.

All 54 published Host Rust files and three original manifests match official
non-yanked archives and committed `81f9809861159def2fd0987fcb7961cda4afd969`.
They are byte-identical to 0.5.0, so there is no new runtime primitive or Rust type
identity change. The selected artifacts/fs/process and Metrics feature sets are
empty. Retain the serialization buffer, private publication, descriptor custody,
original spending and recovery owners. Dirty sibling work is not a registry API.
[Source proof](host-source-proof.json) records every archive/source hash.

The canonical committed exporter refreshes all 75 existing Shared Tooling paths
from clean isolated `e9bfdc54c0daefc3dbbdfe091e5665dca5468eb3`.
Sixteen paths change; every committed blob and executable mode matches.
[Snapshot proof](snapshot-proof.json) records the exact selection. Common rules
are unchanged. Companion admission, LOC coverage and matching local release
tracking refresh are adopted. No new archive helper or PR delivery is selected.

The release-tracking refresh preserves static symbolic refs and concurrent changed
OIDs. Same-OID ref-type races remain [shared #62](https://github.com/dragginzgame/shared-tooling/issues/62).
Archive admission defects remain [shared #59](https://github.com/dragginzgame/shared-tooling/issues/59);
its uncommitted correction is not exported. The existing CI collection roots remain.
Upstream all-native portable regressions pass, but the later Apple Silicon hosted
artifact download fails under [#63](https://github.com/dragginzgame/shared-tooling/issues/63).
Do not report a green overall owner workflow or current consumer macOS acceptance.

All 142 operations tests and eight public artifact/upload/source/recovery/settlement
cases pass with the selected graph. Clippy, MSRV, consumer release-adapter/runner,
logger/retention, LOC, committed exporter/companions, snapshot, ShellCheck,
dependency declarations, formatting and local document links pass.
[Qualification](qualification.json) binds actual source inputs, commands and log
hashes under `target/upstream-071-review`. No new PocketIC or full gate ran;
unchanged Host Rust supports this focused scope. Historical simulator results
retain their actual dependency selection. Failed initial whole-run log retrieval
is superseded by direct completed-job inspection, not counted as a passed command.

[Backup #27](https://github.com/dragginzgame/ic-backup/issues/27) closes on exact
committed 0.7.0 [all-native CI](https://github.com/dragginzgame/ic-backup/actions/runs/37753291659).
Shared [#60](https://github.com/dragginzgame/shared-tooling/issues/60) and
[#61](https://github.com/dragginzgame/shared-tooling/issues/61) close after their
remaining Intel portable acceptance passes. Production transport
[Backup #25](https://github.com/dragginzgame/ic-backup/issues/25) remains open;
ICP is still 1.6.0 and no new upstream response removes routing, internal-call
accounting or command-custody requirements. Host #5 remains an independently
qualified consumer integration, not a reason to replace Backup's descriptor owner.

No product functions, methods or types are removed. No sibling source, real
repository Git writes, release or publication effects run. Prior reviews are
preserved exactly as [Shared history](previous-shared-review.json) and
[Host history](previous-host-adoption.json); their old log paths retain historical
meaning and are not relabelled as current retained files.
