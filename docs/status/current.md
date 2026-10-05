# Current handoff — 2026-10-04

The maintainer requested adoption of Shared Tooling as the top-level Dragginzgame
rules. The starting worktree was clean at `3a94848` (merge after Release 0.1.7),
with Cargo and the dated changelog at `0.1.7`. The tooling batch below is uncommitted.
One undated `Draft` now heads the changelog; no next version has been selected.
Dated history, Cargo/lockfile, licenses and historical release receipt remain
unchanged. No version transaction, commit, tag, push, upload or live IC effect ran.
Canic and every sibling remain read-only.

## Shared engineering and tooling batch

[Shared adoption](../shared-tooling.md) records upstream committed revision
`956236a3848c2cfae6ae05f5c77e9c37b01b3366` and the separately hashed dirty upstream
rules. `DRAGGINZGAME.md` retains those exact reviewed working-tree rules locally;
AGENTS.md identifies its backup-specific overlay within the mandatory shared
baseline. Shared and local rules agree on GitHub-only feedback tracking and
explicit broad-validation authority. CI does not inherit a mutable sibling checkout.

The local snapshot manifest binds exact committed principles, consumption/host
guides, checksum/snapshot verifiers, validation runner and runner regressions.
Make verifies it before fetching dependencies, then preserves the prior sequential
fail-fast gate with target-labelled failures, timing/result and GitHub summaries,
and full/highlighted failure logs under `target/validation-failures/`. The consumer
adapter checks exact bytes/modes, missing files, symlinks, duplicate/escaping manifest
paths and artifact retention. Vendored files stay unchanged.

Release preparation now labels/dates one top-level unnumbered or selected numbered
draft. It rejects ambiguous/empty/misplaced drafts, preserves imported undated
history and keeps the exact receipt/tag requirements. Registry publication still
delegates directly to Cargo. No package version or backup contract changed.

Targeted checks passed: `make tooling-check`, `make release-check` and
`make shell-check`. The real Make dependency gate was exercised with substituted
Cargo and selected targets, not a full native build or network fetch. Read-only
release planning, snapshot identity/mode checks and diff checks also passed.
An initial new test used a fixed date instead of the generated receipt date;
the corrected test passed. Its failed fixture and trace remain at
`target/release-tests.7mc795/`. Successful isolated test temporaries were removed
by their owners; existing build/package/recovery evidence remains retained.

The consistency qualification below is retained evidence from the prior extraction
batch, not a fresh Rust run for this tooling change. The maintainer's current Candid
dependency declaration is preserved. No broad CI/release gate or native compilation
was requested; product and host qualification boundaries remain as described below.

## Current consistency batch

`model::consistency::ConsistencyRequirementRecord` retains strict v1 `version`,
canonical full original `plan_intent` and explicit `per_canister` or
`application_coordinated` requested guarantee. Missing/unknown fields, other versions
and obsolete flag names reject. A domain-separated binary hash binds original plan
intent and guarantee, not current consistency. Full original intent includes context,
inventory, selection, graph, requests and original allowances.

`create_consistency_requirement` requires the original plan already retained under
layout exclusion, then durably creates fixed `consistency-requirement.json` under
journal exclusion without replacement. Reads require exact original expected
requirement digest and original retained plan. Raw input/canonical output have a
1 KiB bound. Lost local creation replies reconcile through exact retained reads;
missing/corrupt/changed evidence never silently downgrades or recreates original
requirements. No fence or allowance is retained or created by this declaration.

Ephemeral `ConsistencyRequest` binds original requirement/operation, caller-owned
challenge, BeforeCapture/AfterCapture boundary and a 0–1,024 descriptive remote-call
ceiling. Per-canister requests require no expected fence; coordinated requests require
an exact `ApplicationFenceBinding` with retained identity AND original membership
revision recovered by the integration's durable obligation owner. Both fields bind
the request hash. Constructor equality proves neither durable fence retention nor
current Active custody; capture wire/effect admission remains separate.

Model-admitted current observations retain exact request, actually observed canonical
context/full inventory, a nonempty canonical sorted unique 1–1,024 set of inventory-backed
targets with actual Running/Stopping/Stopped states and required opaque stopped/drained
evidence, actual revision, explicit evidence lane, opaque observation evidence and actual
call reporting. Coordinated evidence includes actual Active/Inactive fence state,
identity/revision and required whole-unit write/membership/timer/external-work fencing
and drained-work evidence digests. New request/parameter/result/target/fence/view types
have no Serde/default or persisted authority admission. Only the requirement is persisted.

`ports::consistency::ConsistencyProvider` has a fallible typed signature without an
installed/default implementation. It observes existing obligations; no acquisition/
release API is installed. Pure `policy::consistency::validate` requires exact current
request/context/full inventory/exact selected set/call reporting, every target Stopped
and the original guarantee. Coordinated evidence must have the exact retained Active
fence; both actual and fence revisions equal the original retained revision. This
prevents a changed observation and fence revision from silently rebinding an obligation.
No accepted/expiry/Proven flag or parent/component inference supplies a guarantee.

Integrations qualify actual authenticated state, stop/drain, fresh unique challenge
timing, continuously retained whole-selection fence custody, opaque evidence meaning
and prior per-call accounting. Matching before/after values or sequential stops alone
prove no continuity/distributed checkpoint. Unknown custody fails; known inactive
fences deny. Unavailable/Unsupported reject before effects; Indeterminate retains
spent allowance/evidence and obligations, then stops without retry/reset/release.
Timeout, process death, failure and dropping model values do not release obligations.
The descriptive ceiling grants no paid call/reservation/preflight allowance. Matching
views grant no signing, dispatch, restart, release, capture completion or restore/payment
settlement. Terminal replay invokes no provider. See
[the requirement schema](../contracts/consistency-requirement.schema.json),
[typed current port](../contracts/consistency-port.json) and
[maintained boundary](../extraction-boundary.md).

Fresh native cases cover strict records and independent requirement/request binary
hashes, original plan/allowance sensitivity, challenge/boundary/fence/revision binding,
inactive/non-stopped denial, every member of a multi-target selection, full inventory
drift including an unselected parent, exact selected/lane mismatches, canonical target
aliases/duplicates and 1,024-target admission. Persistence cases cover no downgrade/
replacement, original-plan presence, lock contention, raw bounds, malformed/rebound
records, symlinks and replaced layouts. A public local provider fixture reopens original
requirement and spent pending journals, rejects stale/wrong-fence results and preserves
exact original authority, journal bytes, requirement and integration-owned obligation
fixture bytes across all typed failures/drop. These prove local contracts and recovery,
not actual management/application behavior or continuous fence custody.

Targeted checks passed: 153 unit tests and twelve public integration tests (165 total),
warning-denied Clippy/rustdoc, Rust 1.91.0 all-target/all-feature compilation, formatting,
diff checks and standalone Cargo package verification. The changed unselected-parent
regression also passed in the 13-case focused consistency suite. The archive contains
110 exact current Rust source/test files, the exact IC wire JSON fixture, current README
and exact regular MIT license; verified current archive/source remain under
`target/package/`. Previous snapshot-read archive/source remain retained under
`target/consistency-package-evidence.k2rfq3zw/`, along with earlier snapshot-read logs.
Current logs are `target/consistency-focused-tests.log`, `target/consistency-tests.log`,
`target/consistency-clippy.log`, `target/consistency-msrv.log`,
`target/consistency-docs.log` and `target/consistency-package.log`.
Schema examples/negative cases, independent binary digests, exact fresh source references,
prior Rust/provenance preservation and archive contents passed consistency checks.
No broad CI/release gate ran.

[Fresh consistency provenance](../consistency-source.json) records six inspected Canic
files/sections and 180 exact consumer references. Canic HEAD was
`3978e02d28fea9022c7a6e84a7ec6d6e4a0d4af3` with dirty working-tree source. Exact
hashes identify inspected bytes, not application qualification. Quiescence choices,
request/receipt binding and negative cases are adapted; CrashConsistent/RootCoordinated
names, accepted/Proven flags, expiry and parent-derived application units are not copied.
All source consumers remain unchanged. Earlier tracked provenance and source/lock/license
bytes are preserved, including the prior [snapshot-read provenance](../snapshot-read-source.json).

## Retained boundaries and remaining scope

Snapshot-read contracts retain independent exact list payload/challenge binding, actual
context/target/snapshot visibility and pure controller/public/exact-viewer paths. The
1,024 descriptive call ceiling grants no spending; viewers/controllers are bounded to
10 canonical unique principals. Unknown controllers cannot establish controller access;
independent public/viewer evidence needs no controller projection. Permission evidence
cannot settle a lost observation or replenish original authority. Native snapshot-read
qualification is retained; no metadata/data codec or live provider exists.

Earlier artifact/staging/durable publication, bounded JSON, layout/journal exclusion,
restore reference retention, owned command custody, local download/attempt journals,
canonical inventory/selection, explicit graphs, immutable operation plans and pure
retained-journal progress remain implemented. Their exact bounds/contracts are in
[the maintained boundary](../extraction-boundary.md) and AGENTS.md. Pending paid replies
stay pending, missing journals never mean zero consumption, and allowances never refund/
replenish. Applied prerequisites cannot prove cross-journal actual dispatch chronology
or full terminal/reference-release admission. No prune/reference release exists.

The six-method exact IC host-ingress codec and separate membership/control ports remain.
They bind current evidence to original intent/challenge with pure checks but provide no
live authentication, controller custody, continuity/fence or dispatch permission. Load
origin control, same-ID/same-release safety and external-obligation disposition remain
separate requirements. Every earlier [source provenance](../extraction-boundary.md)
remains retained. This batch adds no dependencies, lock changes, Canic imports, sibling
patches, unsafe code or shared target. Rust remains 2024, development 1.99.0 and MSRV
1.91.0. Maintainers own releases; see [development](../development.md) and
[releasing](../releasing.md).

Full B1/B2 and independently usable backup/restore remain unestablished. Application
fence acquisition/release/uncertain-effect recovery and same-release restore safety
still precede runners. Real membership/control/read/consistency providers, transfer/
response codecs, selected backend snapshot/lifecycle qualification, bounded authenticated
calls and lost create/upload/load reconciliation remain necessary. Prior per-call
observation spending, actual cross-journal chronology, complete execution/restore journals/
manifests, terminal reference release, prune, transport and CLI remain proposed. Follow
[the design](../extraction-design.md) for sequencing. Canic adoption and live effects
need separate instructions; the no-commit rule remains.
