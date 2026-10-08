# Original-plan execution admission for downstream integration

The compatible 0.7.1 draft adds `read_execution_progress`,
`AttemptJournalGuard::reserve_planned_mutation` and `reserve_planned_observation`.
They admit the exact persisted original plan and every original journal before
projecting progress or consuming allowance. Missing journals never create zero
consumption. Applied prerequisites gate mutations; existing accounting remains
the sole reservation and receipt owner. Local replay retains pending spending,
source references and fence obligations.

The selected journal remains locked; other journals are admitted sequentially.
Execution settlement reuses the same reader rather than reconstructing a second
original-journal admission loop. No functions, methods or types are deleted.
No new schema, digest, persisted progress flag, permission or provider is added.
Fresh authenticated authority, never-dispatched custody, actual backend capability,
application safety and terminal/reference-release admission remain independent.

[Qualification](qualification.json) binds current source/lock, registered cases,
commands and log hashes. All 134 persistence tests, eight pure progress cases,
six public recovery/source/settlement cases, Clippy, Rust 1.91, strict rustdoc and the actual local
ICP routing-refusal test pass. The public settlement journey reserves through the
new boundary and retains its original fence/source evidence. Five new native
cases cover dependency refusal/admission, missing/wrong/held originals, premature
history and lost-observation reopen. An initial fixture expected the wrong pending
error; its failed log remains retained separately. Current consumer macOS proof
requires its own committed CI; owner CI cannot substitute for changed consumer code.

An external lock update advanced Host from 0.5.1 to 0.5.2 during work; it was preserved.
Fresh locked offline cache/metadata and source-bound tests qualify that selection.
[Published source proof](host-source-proof.json) matches every official archive,
54 Rust files and three original manifests to `c7014995bf0890c1df9cd9b9a6ec14ea70f98c6f`.
Artifacts/fs and manifests are unchanged. Process adds opt-in background handoff;
Backup keeps bounded direct-child capture and independent descriptor custody.
Exact-source [Host CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37762087718)
passes Linux, Intel macOS, Apple Silicon macOS and MSRV. The prior 0.5.1 review is
[preserved unchanged](previous-host-051-review.json). Metrics stays 0.2.11; selected
Host/Metrics features stay empty. No dependency-update operation ran here.

Canic's artifact adoption is already underway, but full executable backup/restore
still requires [generic orchestration and stage binding](https://github.com/dragginzgame/ic-backup/issues/29),
qualified transport and fresh Fleet/application
preflight. No Canic source is edited. ICP CLI remains the selected planned backend;
its actual pinned 1.6.0 probe still qualifies routing refusal and exact pending
exhausted spending, without a NotApplied receipt or repeat update.

For long-term transport, direct Rust `ic-agent` is recommended because its exact
receiver/effective-target updates and non-waiting result fit per-call ownership.
[API source proof](agent-api-source-proof.json) verifies published 0.49.2 against
its official source. Defaults include HTTP retry logic; custom middleware, disabled
HTTP retries/redirects, bounded replies and independent certificate/polling/loss
qualification are prerequisites. An isolated exploratory build compiled, but its
simulator failed sandbox socket admission; the interrupted experiment establishes
no live capability. No Agent dependency or alternative backend is installed here.

All work remains uncommitted; package/receipt stay 0.7.0. No real Git writes,
release, publication or public IC effects run. Local simulator setup/calls are
explicit qualification fixture effects, separate from production authority.
