# Current handoff — 2026-11-16

Work: `/home/adam/projects/ic-backup`, branch `main`. Version 0.102.0 is an
unpublished GitHub checkpoint. The user authorized the version commit and push
to `origin/main`; no tags, registry publication or deployment are selected.
The previous remote checkpoint is `dc4bba68bbaeb0966ecefb8335c4ede85d95e125`
(0.101.12), followed locally by the preserved 0.101.13 reader. The minor increment
covers incompatible APIs/store formats: consumers adopt the fixed operation API;
older operations retain their preserved reader and authority, without migration.
Both locks select local Backup 0.102.0; the qualification lock also adds the
test-only consumer. Supplier versions and the managed receipt remain unchanged.

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

Fresh size: 8,098 product Rust LOC; 196 test-only consumer LOC; 12,583 test LOC;
20,877 total, zero unclassified. Against frozen legacy, raw repository app/lib is
46.99% smaller and total is 53.48% smaller. Product-only repository comparison is
48.24% smaller; crate-only production is 40.36% smaller. Scope is narrower and
Canic work remains external; these figures establish neither parity nor speed.

## Implemented Backup scope

The direct-controller single-canister library implements capture, bounded history/
latest, authenticated original source/payload, same-ID/network/release restore,
original-call recovery, cold continuation and completed-target succession.
Capture/restore leave the application stopped. One `BackupConnection`, signer,
action guard and private sealed recorder serve both fixed procedures. Shared
errors are `OperationStoreError`, `OperationAdvanceError` and `BackupHttpError`;
transport is `BackupHttpTransport`. No compatibility aliases were added.

The [operation contract](../design/operation-contract.md) owns immutable approval,
ID-only reopening and capture/restore ordering. Complete metadata/byte checks
precede loading. Pending calls never repeat; original signing/network spending
survives interruption. Explicit observation retains separate original timing.
Dependent effects require live permission. Completed replay uses original evidence
without clock, guard, signing, HTTP or writes. Unknown tails/orphans/changed files
refuse intact; uncertain writes invalidate the handle. No reset/refund/start occurs.
[Capture](../architecture/capture-intent.md), [transfer](../architecture/capture-transfer.md)
and [restore](../design/local-store.md#immutable-restore-approval-and-original-custody)
own exact behavior; [restart](../design/restart.md) owns composition.

## Acceptance and preservation

Current `target/evidence/review-fixes-*.log`: 232 native tests, ten actual-IC journeys,
five public examples, strict Clippy for both workspaces, warning-denying Rustdoc,
shared formatting/tooling admission, links and whitespace checks pass. Final
comment/error-message/import cleanup changed no runtime behavior; Clippy/Rustdoc
cover those edits. Earlier `target/evidence/backup-completion/` remains retained.
Version preparation reuses those unchanged runtime checks; both locked metadata
graphs, strict Clippy and formatting/links/tooling pass at 0.102.0, with evidence
under `target/evidence/version-0.102.0-*`. Every package remains non-publishable.

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

The retained paired IO measurements in `target/evidence/reduction-followthrough/`
do not measure complete restore or establish throughput. Historical replay still
accumulates quadratic work. Full 1 GiB journeys, Canic maintenance/process adapters,
production HTTPS, hosted macOS, physical power loss and registry/release consumers
remain unqualified. Publication and deployment require separate authorization.
