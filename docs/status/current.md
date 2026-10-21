# Current handoff — 2026-10-21

## Ownership and delivered source

The pending contributor-tooling fix keeps dependency fetching in explicit setup,
removing its repeated dispatch from ordinary validation; see
[#64](https://github.com/dragginzgame/ic-backup/issues/64). Product source and
selected dependencies are unchanged by that fix.

Core owns backup records, bounded spending, guarded persistence, reconciliation
and coordination. Agent owns reserved signed transport. Platform owns external
IC types/codecs/SDKs. IC Tooling owns generic host custody, release rendering and
simulator assets/processes. Applications retain fresh authority, consistency,
fencing and external-effect settlement. The [implementation contracts](../implementation-contracts.md)
remain mandatory; storage, wire formats and spending semantics are unchanged.

Default branch **`c37122e` / delivered source 0.37.2** supplies Host Artifacts/FS/Process,
Testkit and the separately installed Build release editor **0.63.0**. Build and
Testkit setup/checks explicitly select `cli`; library consumers keep it disabled.
Shared Tooling remains **0.25.9**, revision
`85aded191d85ac4a8c0e5deab207f6be18895862`.
The snapshot is authoritative; see [Shared guidance](../shared-tooling.md).
Current main selects **Platform 0.6.3**. The 0.37.1 and 0.37.2 release notes
describe the pending PR, but those releases exclude its held-lock reader and
independent-consumer implementation. Their finalized history is preserved; the
0.37.3 draft records actual delivery of those pending changes. Package versions
remain 0.37.2; this PR runs no release. Merge the qualified implementation before
preparing another release. The maintainer checkout is left unchanged.

## Pending terminal qualification

The compatible **0.37.3** draft adds
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

## Independent consumer qualification

The Agent suite now builds and runs the canonical consumer program in a separate
normal-dependency workspace and target directory. It uses candidate Core/Agent
paths, the original locked external identities and the existing no-Testkit/PocketIC/
WAT graph admission. The parent alone owns the simulator, actual fixed membership,
controller/module/status observations and complete stopped post-load verification.
All parent observations reserve separate original accounting before calls.

Both catalogues precede capture. Distinct processes perform capture, stopped
restore, accepted start and effect-free terminal replay over original stages,
bytes, manifest and source references. Withheld acceptance refuses before the
start stage exists. An actual process exit after authenticated load but before
receipt leaves pending exhausted spending; cold continuation refuses without
redispatch. Its full restored-state oracle stays outside the acceptance channel.
Host owns process cleanup; every invocation retains unique private output logs.
No runtime/transport, schema, dependency, feature or production API change occurs.

## Focused evidence and preserved work

Evidence is retained at `/tmp/backup-29-20261021-qrcsyx1t/`, including failed
attempts and original fixture bundles. The original isolated PR head used the
committed **Platform 0.6.1** lock and its own target directory. On that exact graph,
strict all-target Clippy, **38 workflow native cases**, **six native guarded-release
cases**, **six connected PocketIC cases**, Rust **1.91** all-target/all-feature
compilation and warnings-denied
Rustdoc pass. The then-incoming **Platform 0.6.2** lock update was preserved and
excluded from that original PR head; its early/final Clippy, 25 persistence workflow cases and
the six connected cases also passed. This is focused evidence,
not complete dependency-adoption qualification.

The independent-consumer batch retains evidence at
`/tmp/backup-0371-consumer-b0wb4fhj/`. Strict Agent all-target Clippy passes;
`agent-pocketic.log` covers all 12 unchanged Agent cases and the first two consumer
cases. `independent-final.log` reruns both new cases after strengthening original
intent/witness custody, preserving the initial failed development evidence.
The final normal graph/build is `/tmp/ic-backup-agent-independent-build-3317842-0/`;
actual success/death roots are `/tmp/ic-backup-agent-pocketic-3317842-{1,2}/`.
Both owning tests and the independent normal consumer pass Rust 1.91 checks
(`msrv-owner.log`, `msrv-normal-consumer.log`). Formatting, local links, dependency
boundaries/inheritance, unchanged shared snapshot and exact new source inclusion
in the Agent archive also pass. Manifests, features and lockfiles are unchanged.

Conflict resolution merges current main `1e39353` without rewriting the shared
PR history. Candidate source is unchanged from `b0ea13a`; only current main's
package metadata/Platform 0.6.3 graph and local pending notes/handoff change.
Fresh evidence at `/tmp/backup-pr63-conflicts-20261021-9rk6cm0u/` passes strict
all-target/all-feature Clippy, 38 workflow and six guarded-release native tests,
six connected Core and all 14 Agent PocketIC cases, Rust 1.91 all-target checks,
and warnings-denied Rustdoc. Formatting, links, boundaries, inheritance and the
119-file snapshot also pass. The initial Core filter selected zero tests; the
corrected connected-workflow invocation ran all six affected cases. Those logs
remain retained separately. Native macOS qualification awaits exact-head PR CI;
previous-head CI does not qualify this merge. The main checkout is untouched.

A second conflict resolution merges `c37122e` and preserves its finalized 0.37.2
notes and release metadata. Rust source and the complete resolved external graph
are identical to `7522581`; only local package versions advance with main.
The earlier 64 focused passing tests, Clippy, Rust 1.91 and Rustdoc evidence
therefore retain their exact implementation/external-graph scope. The previous
head also passed hosted Linux CI; both macOS jobs were still running at the
start of this resolution. Exact new-head native-host qualification remains CI-owned.
Fresh strict all-target/all-feature Clippy, both independent authenticated consumer
journeys and Rust 1.91 all-target checks pass with main's 0.37.2 package metadata.
Formatting, local links, dependency boundaries/inheritance and the shared snapshot
also pass. Evidence: `/tmp/backup-pr63-conflict2-20261021-esb6jar_/`.

Earlier Tooling adoption evidence remains at
`/tmp/backup-tooling-adoption-20261021-0omaduks/`: 14 focused targets, 638 native
tests and 90 release-adapter cases. Its graph/source scope is historical, not
qualification of later changes. Adoption records remain in the
[Shared archive](archive-shared-tooling-2026-10-21.md) and
[Platform archive](archive-platform-adoption-2026-10-21.md).

No full local `make ci`, new native macOS, installed-IC, production application
journey, release or publication is claimed. Published Host public-type alignment
has independent registry-only Rust 1.91 evidence at
`/tmp/backup-issue-review-20261021-vre9c3ph/registry-consumer/`. The exact 0.37.0
three-host CI matrix passed, completing the alignment issue. This does not qualify
the pending implementation on native macOS or its later Platform 0.6.3 graph.

## Remaining application acceptance

Keep [#29 — Complete original-plan execution and stage binding before Canic
runner adoption](https://github.com/dragginzgame/ic-backup/issues/29) open. Complete
acceptance requires the installed authenticated runner and independently qualified
current membership/control/read authority, whole-selection fencing, retained
command/byte custody and external-obligation settlement. Local checkpoints and
this isolated terminal provider do not supply those facts. Work is scoped to
Backup; no Canic edits or live IC effects are authorized by this batch.
