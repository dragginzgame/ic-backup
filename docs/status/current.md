# Current handoff — 2026-10-21

## Ownership and delivered source

Core owns backup records, bounded spending, guarded persistence, reconciliation
and coordination. Agent owns reserved signed transport. Platform owns external
IC types/codecs/SDKs. IC Tooling owns generic host custody, release rendering and
simulator assets/processes. Applications retain fresh authority, consistency,
fencing and external-effect settlement. The [implementation contracts](../implementation-contracts.md)
remain mandatory; storage, wire formats and spending semantics are unchanged.

Default branch **`a72f7fe` / published 0.37.0** supplies Host Artifacts/FS/Process,
Testkit and the separately installed Build release editor **0.63.0**. Build and
Testkit setup/checks explicitly select `cli`; library consumers keep it disabled.
Shared Tooling remains **0.25.9**, revision
`85aded191d85ac4a8c0e5deab207f6be18895862`.
The snapshot is authoritative; see [Shared guidance](../shared-tooling.md).

## Pending terminal qualification

The compatible **0.37.1** draft adds
`read_complete_execution_workflow_locked`. A terminal provider can replay complete
original workflow settlement without releasing and reacquiring its held journal
lock. Wrong journal guards and original digests, malformed/oversized/symlinked
records, changed predecessor evidence and incomplete settlement still refuse.
The ordinary reader retains its existing behavior and shares the stage replay.
No schema, budget, receipt, production provider or public API removal is added.

The connected PocketIC journey now reaches guarded reference release after cold
reconstruction. It rechecks source bytes/manifest, exact load/start bytes and
source/destination-bound stopped post-load acceptance. Missing or substituted
application acceptance refuses even with all-Applied checkpoints. Lost allocation,
data, load and start replies preserve references without reissue. Successful
release changes only its exact reference; another unfinished restore stays retained
and continues to prevent pruning. Already-released replay is effect-free and still
requires fresh terminal qualification. See [fixture scope](../pocketic-qualification.md).

This inspected single-owner program has no timers, outbound work, payments or
application fence. Calls are in-process and its complete external command set is
empty. Native process tests separately cover nonempty command custody. These
fixture facts are not a production application safety default.

## Focused evidence and preserved work

Evidence is retained at `/tmp/backup-29-20261021-qrcsyx1t/`, including failed
attempts and original fixture bundles. The isolated PR worktree keeps the
committed **Platform 0.6.1** lock and its own target directory. On that exact graph,
strict all-target Clippy, **38 workflow native cases**, **six native guarded-release
cases**, **six connected PocketIC cases**, Rust **1.91** all-target/all-feature
compilation and warnings-denied
Rustdoc pass. The maintainer checkout's incoming **Platform 0.6.2** lock update
is preserved and excluded from the PR; its early/final Clippy, 25 persistence
workflow cases and the six connected cases also pass. This is focused evidence,
not complete dependency-adoption qualification.

Earlier Tooling adoption evidence remains at
`/tmp/backup-tooling-adoption-20261021-0omaduks/`: 14 focused targets, 638 native
tests and 90 release-adapter cases. Its graph/source scope is historical, not
qualification of later changes. Adoption records remain in the
[Shared archive](archive-shared-tooling-2026-10-21.md) and
[Platform archive](archive-platform-adoption-2026-10-21.md).

No full local `make ci`, new native macOS, installed-IC, production application
journey, release or publication is claimed. Published Host public-type alignment
has independent registry-only Rust 1.91 evidence at
`/tmp/backup-issue-review-20261021-vre9c3ph/registry-consumer/`; its Intel macOS
exact-source CI gate was still running at this review.

## Remaining application acceptance

Keep [#29 — Complete original-plan execution and stage binding before Canic
runner adoption](https://github.com/dragginzgame/ic-backup/issues/29) open. Complete
acceptance requires the installed authenticated runner and independently qualified
current membership/control/read authority, whole-selection fencing, retained
command/byte custody and external-obligation settlement. Local checkpoints and
this isolated terminal provider do not supply those facts. Work is scoped to
Backup; no Canic edits or live IC effects are authorized by this batch.
