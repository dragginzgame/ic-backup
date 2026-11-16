# Current handoff — 2026-11-16

Work: `/home/adam/projects/ic-backup`, branch `main`, pushed
`af240881239112b8a8a9250b4c37e3f00dc68b19` (0.102.0, unpublished).
Both locks select Backup 0.102.0; suppliers and managed receipt are unchanged.
The minor increment replaces APIs/store formats. Older operations retain their
preserved readers and authority; no migration is provided. The pending 0.102.1
patch repairs fixture socket modes without changing product APIs or stored data.
GitHub delivery of this fixture correction is authorized; hosted CI is the next
check. Package versions remain 0.102.0 and no registry upload is requested.

## Publication audit — next work

The [full comparison/readiness audit](../reports/audits/2026/11/16/publication-readiness/01/report.md)
finds the small direct-controller workflow implemented but publication not ready.
The pushed revision failed both macOS jobs (ARM: 10 tests, Intel: 21). Pending
HTTP/TLS/proxy fixtures explicitly select blocking accepted sockets with the same
timeouts; fresh hosted ARM/Intel results remain required. The maintainer revised
the target to 20 MiB; qualify its complete journey and measured costs. This fits
below the identified reservation ceiling for bounded shapes, not arbitrary chunks.
Archive build/offline replay pass at the audited revision. Publication wiring,
packaged README/migration guidance and coordinated Canic manager admission/replay
remain unfinished.

## Legacy review corrections

All four findings from the [comparison](../design/legacy-assessment.md#review-corrections--2026-11-16)
are addressed. Typed post-response errors retain bounded sensitive certificates
and original request IDs, with redacted Debug. `reconcile_observation_reply`
imports only an original submitted unresolved lookup under its separate retained
time policy. Failed custody stays closed; no repair, second journal or retry exists.
New profiles allow 2 MiB request / 4 MiB result / 2 MiB certificate ceilings around
1 MiB payload chunks. Existing approvals and smaller limits remain unchanged.
A separate normal-dependency consumer qualifies fresh-process continuation/replay;
current source/restore documentation and shared API names are corrected.

Audited size: 8,098 product / 196 consumer / 12,583 test Rust LOC, zero unclassified.
Against frozen legacy, app/lib is 46.99% smaller; total 53.48%, repository production
48.24%, crate-only production 40.36%. Narrower scope establishes neither parity
nor speed; the [assessment](../design/legacy-assessment.md) retains the comparison.

## Implemented Backup scope

The direct-controller single-canister library implements capture, bounded history/
latest, authenticated original source/payload, same-ID/network/release restore,
original-call recovery, cold continuation and completed-target succession.
Capture/restore leave the application stopped. One `BackupConnection`, signer,
action guard and private sealed recorder serve both fixed procedures. Shared
errors are `OperationStoreError`, `OperationAdvanceError` and `BackupHttpError`;
transport is `BackupHttpTransport`.
The [operation contract](../design/operation-contract.md) owns approvals, cold reopen and ordering.
Complete metadata/byte checks precede load. Pending calls never repeat; spending
survives interruption. Explicit observation retains separate original timing.
Dependent effects require live permission. Completed replay uses original evidence
without clock, guard, signing, HTTP or writes. Unknown tails/orphans/changed files
refuse intact; uncertain writes invalidate the handle. No reset/refund/start occurs.
[Capture](../architecture/capture-intent.md), [transfer](../architecture/capture-transfer.md)
and [restore](../design/local-store.md#immutable-restore-approval-and-original-custody) own behavior.

## Acceptance and preservation

Fresh `target/evidence/macos-fixture-*`: 232 native tests, strict Clippy for both
workspaces, formatting and documentation links pass on Linux. The actual-IC
capture/restore/succession/offline-replay case passes through the repaired proxy
(158 seconds). Hosted macOS still needs the repaired revision.
Prior `review-fixes-*`: ten actual-IC/five public examples and strict Rustdoc pass.
`publication-audit-*` covers the original archive; failed fixtures remain retained.
Every package remains non-publishable.

New IC cases qualify a full 1 MiB whole chunk through capture/restore, offline load
settlement from an error-returned certificate after failed local writing, and a
separately built consumer whose normal graph excludes Testkit/PocketIC. Separate
processes advance/continue capture and restore, then inspect completion after
backend shutdown with only store/ID/approval. Existing multi-extent, empty-chunk,
changed-stable-state, history, succession and lost-reply journeys still pass.
Original wires, roots, journals, child outputs/graph and server logs remain under
`qualification/target/fixtures/`, including failed runs. Native cases preserve
original spending, late-deadline evidence, uncertain writes and changed custody.

The original reader remains at `b7c2c6ae754a5ae2452e65b9be537b517614ebd2`,
`preserve/backup-0.101.13-before-replacement`, with hashed source/build evidence
under `.git/recovery/backup-0.101.13-before-replacement/`. The newer capture plus
status-only restore reader remains under
`.git/recovery/host-store-1-before-restore-20261116/`: 185 source files, both locks,
six artifacts; archive SHA-256
`6658a8963fe3d56cb5fe13ec210c9170f8d6b7b50f0de33583e627287a371b06`.
[Setup](../development/setup.md#host-store1-reader-before-complete-restore) owns details.
Old operations retain their matching reader/IDs/authority/evidence. This correction
changes no stored encoding, migrates no records and touches no legacy/sibling data.

## External integration and limits

Canic issues remain [#5 signing](https://github.com/dragginzgame/canic/issues/5),
[#6 manager execution/original-result lookup](https://github.com/dragginzgame/canic/issues/6),
[#7 fenced recovery](https://github.com/dragginzgame/canic/issues/7), linked from
[#1](https://github.com/dragginzgame/canic/issues/1). The [sibling review](../design/sibling-integration.md)
retains exact ownership. Full integration needs the actual manager's authenticated
receipt protocol/surviving ledger, bounded signer, continuous application/rewind
fence and real consumer qualification. No substitute route is invented locally.

Replay remains quadratic; 20 MiB journeys, Canic adapters, production HTTPS,
hosted macOS, physical power loss and registry/release consumers remain unqualified.
Publication and deployment require separate authorization.
