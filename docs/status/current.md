# Current handoff — 2026-10-18

Released baseline: `ic-backup` and `ic-backup-agent` **0.31.2**, commit
`db9c5d99c00657b1fda6a766a256599bc195f235` (`v0.31.2`). The tag resolves to the
same source as the [successful native CI matrix](https://github.com/dragginzgame/ic-backup/actions/runs/39130374520).
Release metadata retains authority. Earlier receipts remain in the
[October 18 archive](archive-2026-10-18.md),
[October 17 archive](archive-2026-10-17.md) and
[ownership review](../reports/audits/2026/10/18/architectural-sediment/03/report.md).

## Current boundary

Core owns backup records, bounds, guarded persistence and coordination. Agent owns
one reserved update's signing, retention and transport. Platform owns Candid,
Principal, management DTO and SDK dependencies. Core selects `candid` and
`management-canister-types`; only Agent adds `agent`. The
[Platform contract](../ic-platform.md) retains the original 0.1.8 adoption evidence.

Host owns hashing, descriptor IO and process custody. Testkit owns simulator assets
and managed processes. Backup retains fixture evidence, bounded DTO visitors,
Reqwest client policy and original journals. Applications own membership, routing,
release identity, consistency and external-effect settlement. A standalone CLI and
complete application runner remain outside the implemented scope.

The released graph selects Testkit 0.54.0, Host 0.27.4 and Platform 0.1.10. Shared
Tooling 0.24.0 remains at `cab12cd3dfa4fdb34846b25397f0d72881bb9ae0` with 114
exact payloads. Development Rust is 1.99.1; consumer MSRV remains 1.91.0.

## Delivered qualification

The exact 0.31.2 CI source passes the complete configured gate on Linux x86-64,
Intel macOS and Apple Silicon. Every job passes explicit tool preparation and
offline reuse, including application flag/wrapper isolation, the admitted common
formatter directory and the locked Testkit CLI's physical identity. Native logs
include 34 Core and 12 Agent PocketIC cases, actual Rust 1.91 independent Core/Agent
consumers and compilation of both current package archives together. The earlier
[0.31.1 failure](https://github.com/dragginzgame/ic-backup/actions/runs/39123427210)
remains historical evidence; its stale tool comparison is corrected in 0.31.2.

These receipts satisfy the containing native-matrix requirements for
[#55 — Testkit library server admission](https://github.com/dragginzgame/ic-backup/issues/55),
[#58 — Rust 1.99.1 qualification](https://github.com/dragginzgame/ic-backup/issues/58)
and [#60 — Admitted Rust tool comparison](https://github.com/dragginzgame/ic-backup/issues/60).
The advisory [#59 — README compiler floor](https://github.com/dragginzgame/ic-backup/issues/59)
correction is delivered too. CI does not qualify installed-IC or complete
application acceptance. Application transport authority and full runner/terminal
proof remain [#25 — Reserved Agent transport qualification](https://github.com/dragginzgame/ic-backup/issues/25)
and [#29 — Original-plan runner adoption](https://github.com/dragginzgame/ic-backup/issues/29).

## Remaining acceptance and evidence

The remaining application acceptance requires a concrete integration contract for
current membership/control/read authority, whole-selection fencing and external
obligations. The restore-safety and terminal-evidence ports intentionally install
no default provider. Their test implementations qualify fixture behavior only.
The next implementation must supply these application-owned inputs before wiring
the complete runner; local all-Applied checkpoints cannot substitute for them.

The linked Canic adoption issues remain open. Its reviewed default-branch source
at `53f1f22d2e23a16c5566b8590f3b0a2a643b6d84` already implements the
[terminal-evidence adapter](https://github.com/dragginzgame/canic/blob/53f1f22d2e23a16c5566b8590f3b0a2a643b6d84/crates/canic-backup/src/restore/persistence/retention/mod.rs#L94)
and calls guarded reference release. Its
[live-create entry point](https://github.com/dragginzgame/canic/blob/53f1f22d2e23a16c5566b8590f3b0a2a643b6d84/crates/canic-cli/src/backup/create/mod.rs#L38)
still refuses execution. This is source inspection, not fresh consumer execution.
The remaining work is application authority/fencing integration and complete
retained-plan capture-to-restore qualification with interruption, reconciliation
and effect-free terminal replay. It does not require a second terminal API or
Canic control-plane dependencies in Backup.

Exact 0.31.2 hosted job metadata and all three raw native logs are retained at
`/tmp/backup-issue-reconcile-0wqan5nq/`. Earlier focused Linux receipts remain at
`/tmp/backup-issue-pass-mlttf_tl/`, `/tmp/backup-native-followup-wafn4qmr/`,
`target/msrv-consumers.dGrOin/`, `target/ci-fixtures/native-reuse.y_641iwl/` and
`target/package-check.CN2Dxh/`. Those receipts retain their original source and
dependency selections. The earlier 0.1.8 adoption has separate provenance in the
Platform contract.

This documentation reconciliation changes no library behavior, dependency, public
API, persisted schema or tool selection. No new local full-suite run, live IC
operation, release, publication, deployment or artifact cleanup is part of it.
