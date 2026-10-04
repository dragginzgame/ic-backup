# Current handoff — 2026-10-04

The maintainer completed `0.1.5` at `b01cc9b` (Release 0.1.5), then requested
an undated `0.1.6` changelog draft and continued extraction. The draft covers the
closed IC request codec, membership port and direct-control contract batches,
immediately below empty Unreleased. Existing dirty codec/membership work remains preserved. All changes remain
uncommitted for maintainer review. Canic and siblings remain read-only. No live IC
effects, release/version transactions, commits, pushes or package uploads ran.

`model::membership::MembershipObservationRequest` borrows the original immutable
operation plan and derives one exact operation binding. Private fields retain
full plan intent/context/target/payload identity, original full inventory/selection,
a caller-owned challenge, explicit BeforeEffect/AfterEffect boundary and a 0–1,024
descriptive remote-observation ceiling. Unknown operations and excessive ceilings
reject before provider use. The canonical binary request digest binds full original
intent, sequence, challenge, boundary and ceiling. It changes with original context,
inventory, graph, requests or allowances through the original plan digest.

`ports::membership::MembershipProvider` is a fallible typed integration contract,
without an installed/default provider. It returns a passive `MembershipObservation`
with exact current request digest, actually observed canonical network/caller/release,
complete current inventory, optional opaque revision, required opaque evidence
identifier and reported actual remote-call count. Unavailable/Unsupported reject
before remote effects. Indeterminate retains consumed allowance/evidence and stops;
no failure implies retry, mutation or replenishment.

Pure `policy::membership::validate` checks exact request, actual context, complete
inventory equality and reported calls within the descriptive ceiling. Changed
unselected parents/metadata reject. Its private-field read-only view borrows
current inventory, original selection, revision and evidence. Policy performs no IO,
provider invocation, record serialization, plan/journal mutation or scheduling.
Request/result/view have no Serde or persisted fresh-authority admission; existing
persisted schemas remain unchanged. See [the typed contract](../contracts/membership-port.json)
and [the maintained boundary](../extraction-boundary.md).

The caller qualifies provider authenticity, actual observations, challenge freshness
and uniqueness, timing, locked authenticated context and coherent custody. A
matching digest is binding, not proof of freshness. Before/after inventory or
revision equality cannot exclude intermediate membership changes or certify a
consistency fence, control/read permissions, same-release restore safety or effect
completion. The ceiling bounds one invocation's reported calls; it is not a
spending allowance, reservation, remaining reconciliation allowance or dispatch
permit. Integrations still need separately approved prior per-call accounting and
retained uncertainty; no preflight budget owner or paid-call workflow is implemented.
Zero calls permits a qualified local path, never a paid probe. Terminal replay
must not invoke this provider; new live verification stays a distinct operation.

Fresh native cases cover an independently constructed exact request hash, original
binding/challenge/boundary/call-ceiling sensitivity, canonical aliases, observed
context mismatch, full inventory drift, zero/max ceilings and 1,024-target admission.
A public external fixture implements the fallible port, reopens original plan and
spent pending journal, rejects an old observation under a new challenge without a
provider call, and preserves spent allowance through all typed provider failures.
This qualifies API binding and local recovery, not actual current membership or
an IC management backend.

`model::control_authority::ControlObservationRequest` now derives exact original
operation identity and validates actual closed IC mutation target/digest before
provider use. Its canonical digest binds full original intent, sequence, exact wire
digest, challenge and a 0–1,024 descriptive call ceiling. Observation methods and
changed payload/target reject; caller-owned challenges are not proof of freshness.
Requests/results/known controller sets/views have no Serde or persisted authority
lane. `ControllerSet` normalizes/sorts at most 10 principals and rejects equivalent
duplicates. An explicitly known empty set denies caller control; missing/unknown
actual controller evidence must fail at provider admission.

`ports::control_authority::ControlAuthorityProvider` is a fallible direct-host
controller observation contract with no installed/default provider. Model admission
canonicalizes actual observed target; results retain actual canonical context,
complete known controllers, opaque qualified evidence and reported calls. Pure
`policy::control_authority::validate` matches request/context/target/call bound and
requires the selected caller itself in controllers. Another controlling Root/parent,
public/status/snapshot read access and subnet-admin exceptions do not substitute.
The private-field view grants no signing, dispatch, fresh spending, lifecycle/fence
or terminal proof. Load still needs source/origin controller permission, ownership
and same-ID/same-release restore safety qualification. Descriptive ceilings grant
no reservations or replenishment; actual authenticated freshness, custody and prior
per-call accounting remain integration-owned. See
[the typed control contract](../contracts/control-authority-port.json).

Fresh control tests cover every registered mutation, independently constructed
request binary golden, controller normalization/duplicates/empty/max bound,
request/class/payload/target/context/challenge/ceiling mismatches, revoked caller and
Root-only denial, and zero/max call reporting. A public provider fixture reopens
original plan and spent pending journal, rejects stale results without provider
invocation, denies revoked caller and preserves exact original authority/bytes and
spent allowances through all typed provider failures. These qualify local contracts
and recovery only; no actual controller custody or management backend is modeled.

Targeted checks passed: 132 unit tests and ten public-API integration tests
(142 total), warning-denied Clippy/rustdoc, formatting, Rust 1.91.0 all-target/
all-feature compilation and standalone Cargo package verification. All earlier
native model/artifact/persistence/process, membership and IC-codec regressions also
passed. The final archive contains 93 exact current Rust source/test files, the
exact IC wire golden JSON fixture, current README and exact regular root MIT
contributor license. No broad CI/release gate ran. Fresh logs are
`target/control-authority-focused-tests.log`, `target/control-authority-tests.log`,
`target/control-authority-clippy.log`, `target/control-authority-msrv.log`,
`target/control-authority-docs.log` and `target/control-authority-package.log`.
Verified current archive/source remain under `target/package/`; the previous
membership archive/source is retained under
`target/control-authority-package-evidence.tatfq160/`. Earlier membership/codec
logs, `target/membership-package-evidence.ye2ctbws/` and
`target/ic-request-package-evidence.0mrm4gav/` remain retained. The earlier codec
batch arrived without `target/`; older handoff paths were absent then, not removed
by these batches. Existing tracked recovery provenance remains.

[Fresh control provenance](../control-authority-source.json) records five inspected
Canic files/sections and 192 exact consumer references with ownership dispositions.
Source control declarations/receipt admission/controller projections become exact
mutation-bound current caller-controller checks. Root/Operator/Proven flags, mutable
plan upgrades, expiry and optional/default/alias status parsing are not copied.
Existing consumers remain unchanged. The primary management interface is linked in
[the maintained boundary](../extraction-boundary.md); native tests establish no
live provider qualification.

[Fresh membership provenance](../membership-source.json) records five inspected
Canic files/sections and 133 exact consumer references with ownership dispositions.
Canic HEAD was `3978e02d28fea9022c7a6e84a7ec6d6e4a0d4af3` with dirty working-tree
source. Exact hashes identify inspected bytes, not authority qualification. Topology
request/receipt responsibilities were adapted into ephemeral generic request/result
binding. Serialized Proven/apply-receipt flags, timestamp expiry, Fleet/Root/scope
labels, full preflight bundles and runner loops were not copied. Existing consumers
remain unchanged. Earlier [planning](../source-baseline.json),
[artifact](../extraction-source.json), [layout](../layout-source.json),
[command custody](../command-custody-source.json),
[download](../download-journal-source.json), [attempt](../attempt-journal-source.json),
[inventory](../inventory-source.json), [graph](../effect-graph-source.json),
[plan binding](../operation-plan-source.json), [progress](../execution-progress-source.json)
and [IC request](../ic-request-source.json) provenance remain retained.

The prior `model::ic_request` boundary remains intact: six closed host-ingress
status/inventory, stop/start, capture/load methods; canonical effective target;
required nullable snapshot ID with 1–256 raw load bytes; pinned official Candid
encoding with 4 KiB argument and 8 KiB record bounds. Receiver/mode/method/routing/
exact bytes bind request digests; mutation and independent observation payload
checks grant no authority. Capture retains code and replaces no snapshot; host
sender canister version stays absent and never guards target version. Independent
wire goldens, official DTO decoding and exact retained-journal recovery remain
native codec qualification only. No SDK/Candid source is copied; Apache-2.0 registry
dependencies remain pinned. Earlier local artifact, custody, retention, download,
finite attempt, inventory/selection, graph, plan and progress owners remain intact.
Missing journals never mean zero consumption; pending mutations block blind retry;
lost observation replies remain pending. Allowances never refund/replenish. Retained
Applied prerequisites cannot prove actual cross-journal dispatch chronology; all
Applied is distinct from full terminal proof or reference-release admission.

Cargo remains `0.1.5`, Rust 2024, development Rust 1.99.0 and MSRV 1.91.0.
Membership/control batches add no dependencies or lockfile changes beyond the prior
codec batch's 30 added registry packages; all 39 originally locked versions remain
preserved. No Canic imports, sibling patches, unsafe Rust or shared target were
introduced. Dated `0.1.5`, older changelog history and historical `docs/release.json`
remain unchanged. Selecting the `0.1.6` draft runs no release/version transaction.
Maintainers own release preparation; see [development](../development.md) and
[releasing](../releasing.md).

Full B1/B2 completion and independently usable canister backup/restore remain
unestablished. Snapshot read authority, application consistency/fence/settlement and
same-release restore-safety contracts still precede runners. Membership and direct
control ports require deliberately qualified real integrations; transfer/response codecs,
selected backend snapshot/lifecycle qualification, bounded authenticated calls and
lost create/upload/load reconciliation remain contract/backend work. Complete
backup/restore semantics, prior observation spending, retained cross-journal
chronology, full execution/restore journals, completion manifests, terminal reference
release, prune, transport and CLI remain proposed. Read [the design](../extraction-design.md)
for sequencing. Canic adoption and live effects need their own instructions;
the no-commit rule remains.
