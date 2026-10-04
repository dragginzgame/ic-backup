# Current handoff — 2026-10-04

The maintainer completed `0.1.4` at `f59669d` (Release 0.1.4), staged the prior
operation-plan batch, selected an undated `0.1.5` changelog draft and requested
continued extraction. The draft now covers both immutable operation-plan binding
and pure progress from exact retained attempt journals. Maintainer staging remains
preserved; source, tests, documentation and provenance remain uncommitted for
review. Canic and siblings remain read-only. No live IC
effects, release/version transactions, commits, pushes or package uploads ran.

`policy::execution_progress::progress` borrows the original `OperationPlanRecord`
and a complete set of validated `AttemptJournalRecord` references. At most 8,192
journals are admitted, with exactly one per graph operation. Unknown/duplicate
operations, missing journals and changed full intent/context/target/request/original
allowances reject. Missing evidence never implies zero consumption. The original
plan digest is computed once. Applied identities come from model-replayed retained
receipts rather than a caller-supplied completion list or editable progress counter.

Every operation with consumed mutation allowance requires retained Applied evidence
for every declared prerequisite. Pending, settled NotApplied and Applied dependent
attempts with unmet prerequisites reject. This validates retained causality; existing
per-operation journals lack a shared dispatch sequence, so current Applied evidence
cannot prove that a dependent call actually began after its prerequisite. Future
trusted workflow admission must enforce and retain cross-journal chronology.

Views bind exact original plan/graph digests and retain journal authority/accounting
and unresolved attempt identities in deterministic graph planning order. Conditions
distinguish awaiting dependencies, available/exhausted mutation allowance, unresolved
mutation, unresolved observation, exhausted reconciliation and retained Applied.
Lost observation replies stay observation-unresolved even with exhausted allowance.
A qualified settled Uncertain observation leaves the mutation unresolved. No
exhaustion refunds or grants retry authority. Zero allowance is not completion;
Applied operations can retain unspent allowances that cannot be used.

Checked totals sum used/remaining originally assigned operation allowances only;
unassigned aggregate headroom stays excluded. Output views have no Deserialize,
record-admission or persistence API. Policy performs no IO, record serialization,
journal creation, authority probes, state mutation or scheduling. Callers own
coherent retained evidence under appropriate custody and integration-qualified
actual receipts. All Applied is not a terminal proof, current live-state claim,
manifest/artifact verification, fence disposition or reference-release permit.
See [the maintained boundary](../extraction-boundary.md).

The prior operation-plan owner remains intact: v1 canonical context, full original
inventory, explicit physical selection, original dependency graph, one exact
sequence/target/request/original-budget binding per node and immutable aggregate
ceilings. Selection admits 1–1,024 exact inventory targets; every operation targets
that set and every selected target is covered. The graph/table admits at most 8,192
operations. Original per-operation combined attempts remain bounded to 1,024;
checked aggregate mutation/observation ceilings admit at most 65,536, including zero.
Assigned totals fit each original ceiling independently; no replacement API exists.

The canonical binary full plan digest binds context, full inventory/graph hashes,
selected principals, every operation target/request/limit and original root ceilings.
Even unselected inventory and unused headroom affect intent. JSON formatting,
derived views/authority and journal progress are excluded. Original per-operation
`AttemptAuthorityRecord` derivation consumes nothing and creates/resets no journal.
Immutable `operation-plan.json` creation and exact-digest reads use layout/journal
exclusion with independent 1 MiB input/canonical-output bounds. Separate retained
inventory/graph files are not adopted, rewritten or assumed equal. The
[plan schema](../contracts/operation-plan.schema.json) and existing persisted v1
contracts remain unchanged by progress policy.

Earlier qualified owners remain intact: artifact checksums/staging/publication,
bounded JSON and locks, stable layout exclusion and conservative restore references,
inherited command custody, local download transitions, immutable finite attempt
accounting, canonical physical forests/pure selection and explicit dependency
ordering/pure causal declared-progress views. Backend transfer completion and
qualified receipts remain caller-owned. Lost paid responses block blind retries;
allowances never refund/replenish. Local replay makes no remote calls. Durable
artifact/Applied evidence and graph readiness do not permit terminal reference release.

Fresh targeted Linux checks passed: 109 unit tests and seven public-API integration
journeys (116 total), warning-denied Clippy/rustdoc, formatting, Rust 1.91.0
all-target/all-feature compilation and standalone Cargo package verification.
The public progress journey was rerun after fixture refactoring. Fresh progress
cases cover every original identity/limit, complete coverage, maximum journal count,
exact 65,536 assigned allowance, explicit reverse dependencies, causal attempted
prerequisites and all local conditions. Public recovery reopens persisted original
plan/journals, compares unchanged bytes/consumption and retains a lost exhausted
observation reply; missing/discarded prerequisite evidence rejects. All existing
local model/artifact/persistence/process regressions also passed. Source hashes,
consumer references, existing schema/examples, changelog history and links validate.
The archive contains 74 exact current Rust source/test files and the exact regular
root MIT contributor license. This is native local evidence qualification only;
no request codec, fresh authority, cross-journal chronology, complete authenticated
plan, terminal proof, PocketIC or live IC backend was qualified. No broad gate ran.

Fresh logs are `target/execution-progress-tests.log`,
`target/execution-progress-public-tests.log`, `target/execution-progress-clippy.log`,
`target/execution-progress-msrv.log`, `target/execution-progress-docs.log` and
`target/execution-progress-package.log`. Verified current source/archive remain
under `target/package/`; prior operation-plan package/source evidence is retained
under `target/execution-progress-package-evidence.ifsmxpdw/`. Earlier package
evidence remains under `target/operation-plan-package-evidence.m9fvsg3b/`,
`target/effect-graph-package-evidence.asr33i_v/`,
`target/inventory-package-evidence.a7jne1oj/`,
`target/attempt-journal-package-evidence.kqp9ti8e/` and
`target/download-journal-package-evidence.45iah_5j/`. Earlier logs, recovery fixtures
and release evidence remain retained.

[Progress provenance](../execution-progress-source.json) records seven inspected
Canic files/sections and 76 exact consumer references with ownership dispositions.
Canic HEAD was `3978e02d28fea9022c7a6e84a7ec6d6e4a0d4af3` with dirty working-tree
source; exact hashes identify inspected files, not qualification. Existing consumers
remain unchanged. Earlier [planning](../source-baseline.json),
[artifact](../extraction-source.json), [layout](../layout-source.json),
[command custody](../command-custody-source.json),
[download](../download-journal-source.json), [attempt](../attempt-journal-source.json),
[inventory](../inventory-source.json), [graph](../effect-graph-source.json) and
[plan binding](../operation-plan-source.json) provenance remain unchanged.

Cargo remains `0.1.4`, Rust 2024, development Rust 1.99.0 and MSRV 1.91.0.
No dependency/lockfile changes, Canic runtime imports, sibling patches, unsafe Rust
or shared target were introduced. The sole populated open changelog is undated
`0.1.5` beneath empty Unreleased. Dated `0.1.4`, older history and historical
`docs/release.json` remain unchanged. Maintainers own release preparation;
see [development](../development.md) and [releasing](../releasing.md).

Full B1/B2 completion and independently usable canister backup/restore remain
unestablished. Qualified exact request codecs and typed authoritative membership,
revision, permission and application-fence/settlement ports still precede runners.
Complete authenticated backup/restore plans need capacity/lifecycle semantics and
retained cross-journal dispatch chronology beyond local declarations/projections.
Backend metadata/transfer extents, bounded executor calls and lost create/upload/load
reconciliation remain integration/port work. Full execution/restore journals,
completion manifests, terminal reference release, prune, transport and CLI remain
proposed. Canic's backup executor preflight still rejects. Read
[the design](../extraction-design.md) for the maintained sequence. Canic adoption/live
IC effects need their own instructions; the no-commit rule remains.
