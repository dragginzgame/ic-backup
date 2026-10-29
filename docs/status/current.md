# Current handoff — 2026-10-29

## Local reusable operator composition

Delivered **0.49.0** is `4f89df65c60d10aed970cd120e7baba5ff841aa4`, tagged
`v0.49.0`. The working tree implements [#75 — Reusable capture/restore operator
composition](https://github.com/dragginzgame/ic-backup/issues/75) over its existing
workflow, plan, journal, transfer and guarded release owners. No second journal,
operator schema, scheduler, refund or default authority provider is added.

`advance_execution_workflow` reuses complete retained-stage admission and the
existing dependency/execution loop with a positive limit on new stages. It returns
Paused between settled stages or Complete after full local checkpoint replay.
Pending observations/mutations and partial/changed originals refuse before new
callbacks. A different limit never authorizes redispatch. Each stage retains its
original finite effect allowance; the limit is not a timeout or per-call budget.

`read_execution_workflow_progress` projects every original stage through existing
binding, ancestor and complete child-journal admission. Prepared rows preserve both
pending lanes; absent rows supply no never-dispatched custody. Missing/unsafe/partial
originals remain typed errors. The read supplies no learned evidence, checkpoint,
artifact completeness, application acceptance or terminal authority.

The independent normal consumer now uses this bounded operator composition through
capture, metadata/data transfer, immutable manifest publication, source-bound upload,
same-ID load, a normal pause before independently accepted start, and guarded source
release. Inspection produces one passive JSON result. Missing learned evidence
refuses explicitly, without a placeholder hash or oracle-based inference.

[The operator map](../canic-adoption.md#reusable-operator-composition) covers each
maintained operator operation; [the Canic removal map](../canic-adoption.md#canic-engine-removal-map)
identifies the exact generic modules/callers to replace. Read-only inspection used
Canic `b9ed7f298bdaf8c9cceac142edf7e56b15d33208`; source fingerprints and scope are
`/tmp/backup75-canic-source.json`. No Canic source, active restore path or old layout
was changed. Actual Fleet/Root policy, Root-paid effects, application hooks and
terminal acceptance remain Canic ports. No forwarding crate or copied host engine
is the intended migration outcome.

Pending **0.49.1** notes cover these compatible additions and the earlier 0.49
adoption-guidance correction. Package versions remain **0.49.0**. All local edits
are unstaged/uncommitted; new APIs are not in the published 0.49.0 packages.
[#74](https://github.com/dragginzgame/ic-backup/issues/74) is delivered in 0.49.0;
#75 remains open pending delivery. No issue mutation, Git/release effect, publication
or live IC operation was performed. GitHub remains the issue tracker.

## Focused operator qualification

**11 workflow unit tests pass**, including bounded pauses/cold replay, changed
original identity, missing journals, partial preparation and pending-observation
inspection/refusal. Strict Core/Agent Clippy passes for all targets/features.
Warning-denied Core Rustdoc and the actual Rust **1.91.0** Agent library/Core
normal-dependency build pass on this batch's graph.

All **7 independent normal-consumer registered cases pass across focused runs**:
the complete capture/restore/start/terminal journey, process death after authenticated
load, the new three-fixture ingress-retention/reply-retention/partial-output case,
and four opaque application-ingress/recovery cases. Actual snapshot counts, complete
loaded state, exact original spending, pending identities, partial Created staging,
no incomplete manifest/start and unchanged files on refusal/replay are checked.
The original six-pass/one-failure run remains `/tmp/backup75-independent.log`.
Its sole failure expected a checkpoint mutex after missing learned evidence; early
refusal now creates none. The stronger unchanged-files assertion passes in
`/tmp/backup75-death-final.log`. Native fixture assertion/lint failures stay retained.

Logs are `/tmp/backup75-*`. Initial independent build/graph evidence is
`/tmp/ic-backup-agent-independent-build-2456269-0`, with original fixtures under
`/tmp/ic-backup-agent-pocketic-2456269-*`. The corrected death-case build is
`/tmp/ic-backup-agent-independent-build-2635797-0`, fixture
`/tmp/ic-backup-agent-pocketic-2635797-1`. Keep raw process/server output, exact
program/graph identities, manifests, source bytes, envelopes, replies and journals.
Final formatting, links, dependency boundaries/inheritance, repository inventory and
snapshot integrity checks are logged in `/tmp/backup75-source-checks.log`.

These are candidate-path normal-dependency and isolated single-canister Linux
proofs. No full local gate, unpacked archive/registry-only acceptance of the new API,
new native macOS matrix, installed Canic migration or live application qualification
ran. Registry mode needs the matching containing release/harness, not published
0.49.0 with these unpublished APIs. Original Root-paid, multi-unit/fencing and
external-obligation acceptance remains under Canic #513/#550/#394.

## Graph, tooling and retained evidence

The incoming lock already selected Host/Testkit **0.69.12** and Platform **0.7.6**.
This batch changes no dependencies. `/tmp/backup75-graph.sha256` binds unchanged
manifest `7d622b5590c53b3a74637f1451a756d711c4037508fdeecf37b7154798d76b69`
and lock `1f74ee318d2bb970985f911a46b5d66dcc0584c21bf67b37e5a15b311647a353`.
Declared Host/Testkit/editor minimums **0.69.9**, Platform **0.7.3**, management DTO
**0.11.0**, external Principal **0.1.5**, Agent **0.49.3** and Candid **0.10.39** stay
unchanged. Explicit setup prepared the exact 0.69.12 CLI; offline admission and
fixture library checks retain prepared PocketIC **16.1.0** and all older bundles.
No supplier crypto/TLS acceptance follows; [Platform #50](https://github.com/dragginzgame/ic-platform/issues/50)
retains that distinct ownership.

Shared **0.27.6**, revision `5a72359b5be19d5221334cf2c9b3204621d3f3f6`, stays
exporter-owned with no vendored changes; [its review](../shared-tooling-review.json)
retains original adoption evidence. Preserve `/tmp/backup74-*`,
`/tmp/backup-049-guidance-*`, `/tmp/backup-simplify-*`, earlier independent/PocketIC
fixtures, Canic/release/consumer journals, `/tmp/backup-reference-release-*`,
`/tmp/backup-tooling696-*`, `/tmp/backup-platform07*`, `/tmp/backup-72*`,
`/tmp/backup-shared27*` and the 20261027/20261028 failure logs in
`target/validation-failures/`. Successful tests and closed issues authorize no cleanup.
