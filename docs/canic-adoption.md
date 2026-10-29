# Preparing Canic adoption

Backup supplies original plans, bounded journals, guarded persistence, workflow
coordination and signed Agent transport. Canic supplies the application facts that
make those operations safe. Blob completion precedes the planned live backup
attempt; preparing this integration does not enable live creation.

## Verified starting point

Delivered Backup source **0.49.0**, `4f89df6`, supplies the original-plan
coordinator, explicit restore source provenance, bounded upload sessions and
opaque application mutation/recovery transport.
Applications exchanging Host errors must align with Backup's selected Host family.
Platform supplies external IC identities and SDKs; Testkit remains development-only.
Core does not depend on Agent; an alternative transport needs its own qualification.
The [current handoff](status/current.md) records exact source/graph qualification;
earlier native results keep their original identities. Library and controlled
consumer fixture evidence does not establish production Canic acceptance.

Released Canic 0.112.0 is `5c236ab130c0d63892ef281b45bdc3fa353e012f`.
[PR #702](https://github.com/dragginzgame/canic/pull/702), head
`78a57d5b1e83a357999bfd4afbe88a06b0809c64`, contributes the first original host
preparation adapter from an isolated checkout:

- `canic-backup::checkpoint` selects published Backup 0.48.0, freezes the complete
  installed Root unit and exact bounded Wire requests, and implements Backup's
  existing workflow stage driver. It requires fresh admission, original command
  custody and independent reply qualification; no default live provider is supplied.
- Existing Root stable checkpoint storage, authenticated commands and catalogue
  exclusion remain the runtime owners. The installed original workflow case
  qualifies lost ingress, original observation recovery and cold continuation
  across two participants, then separate fixture-authorized same-ID snapshot/load
  and release. The existing Root interruption/release case also passes. These
  fixture conditions grant no production Root-paid snapshot or live trust authority.
- The 27 focused native cases and both installed cases pass, together with strict
  affected Clippy and consumer/tooling checks. The [handoff](status/current.md)
  retains exact evidence and limitations. This is an open contribution, not
  delivered Canic main or complete application acceptance.
- Live CLI creation still refuses before effects. The active restore runner and
  original journals remain maintained. Its existing
  `RestoreTerminalEvidenceProvider` must adopt the new engine's evidence inputs
  when the whole operation changes; do not create another terminal authority owner.

Canic's `restore/apply/journal::next_effect_sequence` now uses the retained graph
model and its own serial completion checks; the removed `policy::effect_order`
API no longer blocks dependency adoption. This is not original stage/attempt
engine adoption. Preserve the active restore engine until its replacement is
qualified; caller-declared Completed states are not qualified Backup Applied
receipts. No compatibility alias or second spending authority is needed.

## Integration map

Keep Canic policy and effect ports at its host boundary, with CLI delegation.
[#75 — Reusable operator composition](https://github.com/dragginzgame/ic-backup/issues/75)
owns the upstream composition needed to retire the duplicate engine. The consumer
migration must remove `canic-backup`, rather than leave a forwarding crate or copy
its engine into `canic-host`. Do not import Canic into Core or copy fixture authority
and acceptance into production.

| Concern | Existing Backup interface | Canic responsibility |
| --- | --- | --- |
| Frozen operation and learned stages | `ExecutionWorkflowRecord`, `ExecutionWorkflowStageDriver`, `run_retained_execution_workflow` | Bind exact Fleet/network/release/current unit and authenticate predecessor evidence |
| Original effect accounting | `OperationPlanRecord`, `ExecutionStageGuard`, `AttemptJournalGuard` | Allocate once; use the same authority across interruption; retain exact commands/bytes |
| Direct management capture and transfer | `capture_snapshot`, metadata/download plans and source-bound upload steps | Select this lane only when the actual host caller has the required direct authority |
| Application Root ingress | Original stage/attempt guards and `AgentApplicationIngressProvider` | Own Wire encoding and current authority; account Root-paid management effects through Canic's existing runtime owner |
| Signed transport | Agent application, management, transfer-read, recovery and fence-acquisition providers | Select identity, endpoint and trust root; retain exact ingress/replies; qualify observations |
| Restore and source lifetime | Restore-safety requests, guarded reference release and terminal evidence | Whole-unit fencing, current external obligations, post-load acceptance and safe release |

Choose the actual management dispatcher before connecting transport. Backup's
[control-admission contract](implementation-contracts.md#application-admission-and-fencing)
requires the selected caller itself in the observed controller set; an operator
acting through a Root proxy does not satisfy that lane. Canic's dry-run
`root_controller(Declared)` value is not observed authority. If Root dispatches
management calls, qualify that implementation and its exact original accounting
through Core's original stage/attempt guards and the application adapter below.
Do not assert that the host Agent is the controller of a Root-controlled participant.

The [canonical consumer](../crates/ic-backup-agent/tests/independent/consumer/mod.rs)
demonstrates these public interfaces across cold processes. Its private fixed key,
single target, exclusive controller custody, fixture acceptance files and
`NoIrreversibleEffects` safety lane are test conditions. A deployed Blob-backed
application must establish its actual obligations and fencing; restoring snapshots
does not rewind provider payments, references or other external effects.

## Original Root ingress through public guards

The [application-ingress consumer test](../crates/ic-backup/tests/application_ingress.rs)
shows the Core integration seam using public APIs, without a management transport
adapter or new accounting layer:

1. The application retains the exact receiver, update method, encoded arguments and
   actual network/caller/release binding before publishing the workflow. Its codec
   owns an unambiguous commitment to these bytes. The test's JSON is fixture data,
   not Canic Wire or a production authority record; ICP program/environment labels
   cannot substitute for actual context.
2. `ExecutionWorkflowStageDriver::bind_stage` admits those originals and returns a
   child `OperationPlanRecord`. The coordinator prepares every original journal.
3. The driver opens the exact `AttemptJournalGuard` under `stage.layout()` and calls
   `reserve_planned_mutation` before any Root ingress. Current controller/application
   admission, signed-byte retention, bounded one-shot submission and durable reply
   retention are the adapter's responsibilities. Fresh observations need separately
   reserved original allowance. Do not route a Root proxy through `capture_snapshot`
   or the Agent direct-management mutation port.
4. Qualify the exact original host response before recording a `MutationReceiptRequest`.
   Its Applied outcome settles that host operation only. Root's existing runtime
   replay/cost authority separately owns each Root-paid management intent and result;
   host timeout, a command exit or a local checkpoint cannot settle or refund it.
5. On interruption, reopen the same binding and complete journal set. Pending
   occupied stages refuse both fresh execution and retained continuation before new
   callbacks. Resolve their original observations/effects first. Once settled,
   `RetainedStageEvidence` and `run_retained_execution_workflow` admit exact cold
   continuation without redispatching settled stages.

Run `cargo test --offline --locked -p ic-backup --test application_ingress` for the
native binding/accounting/replay checks. They detect substitution of context,
receiver, method, mode, arguments and complete unit; lost host replies preserve
pending spending and the original dispatch witness. Completed cold replay invokes
no callbacks. Independently retained Root-paid evidence stays unchanged even after
the host stage settles. These tests use synthetic host receipts and make no IC
calls; they do not qualify Canic Wire, Root-paid effects, participant membership,
fresh permissions, fencing, terminal release or actual process-death behavior.

## Application transport adapter

Backup **0.49.0** supplies the application adapter requested by
[#74](https://github.com/dragginzgame/ic-backup/issues/74).
`AgentApplicationIngressProvider` accepts Core's distinct original application
mutation and recovery-observation requests and reuses the existing single-submit
owner. The [transport guide](agent-transport.md#opaque-application-ingress) and
[independent consumer](../crates/ic-backup-agent/tests/independent/consumer/application/mod.rs)
show exact payload binding, mandatory ingress/reply retention and separately
budgeted recovery without pretending that every Root command acquires a fence.

When adopting 0.49.0, select both `ic-backup` and `ic-backup-agent`. Bind fresh child
requests to the admitted opaque payload digest and update exhaustive `ReservedUpdate` matches for its new
variants. Do not reinterpret PR #702's older retained commitments or replay uncertain
ingress. Canic still owns Wire, current Root/unit authority, its production provider,
Root-paid replay/cost, full artifact transfer and application/terminal acceptance.
The new adapter supplies no live-create permission or automatic Applied receipt.

## Existing APIs for Canic's remaining findings

These interfaces support Canic's fixes; they do not qualify its existing runner or
turn older retained records into new plans or allowance. Keep one original spending
owner per effect and retain the old evidence while changing the application adapter.

| Canic finding | Existing Backup contract | Remaining Canic responsibility |
| --- | --- | --- |
| [#286 — Exact restore-upload completion](https://github.com/dragginzgame/canic/issues/286) | Source-bound upload plans and `IcRestoreSource::uploaded` require complete original upload-stage evidence before load. Allocation inventory alone does not establish data completion. | Authenticate exact source bytes, destination and completed effects; reconcile missing/changed inventory with qualified evidence. Backup's allocation policy rejects baseline drift rather than guessing an outcome. |
| [#290 — Exact restore identity](https://github.com/dragginzgame/canic/issues/290) | `RestoreSafetyRequirementRecord` binds same-network/release/IDs source and restore plans. `RestoreSafetyRequest` and its policy check exact context, inventory, selection and fresh safety evidence. | Establish actual network/environment, installed identity, current Fleet/controller authority and application acceptance. Declared matching records supply none of those facts. |
| [#308 — Stable restore selection](https://github.com/dragginzgame/canic/issues/308) | Original stage and attempt guards bind exact plan/authority digests. | Resolve and retain a stable backup identity before mutation; numeric list position is a CLI selection concern. |
| [#291 — Partial-restore containment](https://github.com/dragginzgame/canic/issues/291) | Pending original stages retain spending and refuse redispatch; separately reserved observations support qualified reconciliation. | Define safe whole-unit containment after stop versus load. No generic rollback, blind restart or budget reset follows from an error. |
| [#424 — Fleet/subtree planning](https://github.com/dragginzgame/canic/issues/424), [#396 — MaintenanceRoot authority](https://github.com/dragginzgame/canic/issues/396) | Inventory and plans preserve admitted identities and ancestry without defining Canic's authority-canister protocol. | Resolve actual owning Roots, exclude unsupported authority scopes and qualify any seal/resume contract. |
| [#288 — Uploaded-snapshot cleanup](https://github.com/dragginzgame/canic/issues/288) | Guarded source-reference release preserves unfinished and referenced evidence; it does not delete remote snapshots. | Define exact owned-snapshot retention/deletion authority and qualify interruption and terminal replay. No cleanup permission is inferred from Applied records. |

Reader/path closeout in [#562](https://github.com/dragginzgame/canic/issues/562) and
[#317](https://github.com/dragginzgame/canic/issues/317) still needs Canic's maintained
schema, descriptor-budget and path-boundary qualification. Backup's bounded record
readers and closed artifact-tree contracts do not replace that consumer acceptance.

## Reusable operator composition

The #75 working-source composition adds `advance_execution_workflow` and
`read_execution_workflow_progress`; those two APIs are not in published 0.49.0.
It composes the existing owners below. There is no additional operator record,
journal, scheduler, retry transition or default authority provider.

| Operator operation | Reusable owner and required inputs |
| --- | --- |
| Plan and prepare | Build admitted `OperationPlanRecord` allocations and `ExecutionWorkflowRecord` catalogs, binding actual context, complete inventory, selection, dependencies and finite stage ceilings. Publish once with `create_execution_workflow`; learned child plans remain under `ExecutionStageGuard::prepare`. |
| Bounded run/continuation | `advance_execution_workflow` admits the exact original workflow and complete qualified occupied-stage set, then executes at most the positive selected stage limit through the existing driver/coordinator. `ExecutionWorkflowAdvance::Paused` leaves later stages absent; `Complete` replays every original checkpoint. |
| Progress and inspection | `read_execution_workflow_progress` returns typed absent/prepared stage rows with original mutation/observation identities and allowances. Missing journals and partial preparation reject. These are local projections, not permission to execute or release. |
| Capture and metadata | The driver calls `capture_snapshot` and `read_snapshot_metadata`, or implements the application-owned Root protocol through original guarded application ingress. Fresh authority, actual response attribution and durable exact byte retention are mandatory. Retain and qualify learned snapshot identity/metadata before binding successors. |
| Transfer and immutable source publication | Derive `IcSnapshotDownloadPlan` from the original qualified metadata, bind its predecessor, and use `download_snapshot` with the original `DownloadJournalGuard` writer. Verify complete exact artifacts, then `publish_download_manifest`; replay uses the retained manifest identity. Preserve partial bytes and pending attempts on failure. |
| Restore preparation and upload | Retain the source reference to the exact restore workflow before effects. Verify the original manifest/artifact; derive source-bound allocation and data plans with their original predecessors. Use `allocate_snapshot` and `upload_snapshot_data`, with independently qualified receipts and retained exact destination. |
| Same-ID load and accepted start | Bind `RestoreSafetyRequirementRecord` and `IcRestoreSource::uploaded`, then use `restore_snapshot` for the original load/start stages. Application acceptance after load must be qualified before preparing or dispatching start. An original safety lane is explicit; fixture-exclusive custody is not its default. |
| Terminal/reference release | Hold the exact original workflow `JournalLock`, replay complete local settlement and provide actual application/effect/command evidence through `RestoreTerminalEvidenceProvider`. `BackupLayoutGuard::release_restore` remains the guarded release owner; completed release replay makes no remote calls. |

The [normal consumer](../crates/ic-backup-agent/tests/independent/consumer/mod.rs)
is the executable assembly, built with normal Core/Agent dependencies in a separate
workspace. Its [transfer](../crates/ic-backup-agent/tests/independent/consumer/transfer/mod.rs)
and [restore](../crates/ic-backup-agent/tests/independent/consumer/restore/mod.rs)
adapters use those existing APIs through capture, complete byte publication,
source-bound upload, load, stopped-state acceptance, start and guarded release.
Restore advancement pauses normally before start; a later invocation supplies
fresh admission and exact retained stage evidence. Inspection returns one JSON
projection without dispatch. Missing learned evidence refuses explicitly; it is
never replaced with a placeholder hash or inferred from an oracle.

The selected bound counts new stages, not HTTP calls or elapsed time. A stage may
contain multiple originally bounded transfer attempts. There is no partial-transfer
resume: interruption keeps its original stage occupied for explicit reconciliation.
Never derive a new catalog or allowance from progress. Every original occupied
stage must replay before any new driver callback. Each new observation is reserved
separately, and a lost observation stays pending. The driver retains its existing
mandatory authority, custody and effect-specific qualification duties; no port may
default those to success.

## Canic engine removal map

Read-only inspection at Canic `b9ed7f298bdaf8c9cceac142edf7e56b15d33208`
confirmed the following generic owners and maintained callers. Source fingerprints
are retained in `/tmp/backup75-canic-source.json`; this is an inspection receipt,
not Canic compilation or application qualification. The paths below are relative
to Canic. They identify replacement scope after consumer migration, not deletions
performed or authorized by this library work.

| Existing Canic implementation/caller | Replacement and retained product responsibility |
| --- | --- |
| `crates/canic-backup/src/execution/{mod.rs,operation.rs,receipt.rs,types.rs,validation.rs}` and `journal/{mod.rs,types.rs,validation.rs,report.rs}` | Retire generic ready-selection, pending/completion transitions and progress records in favor of original stage/attempt journals, the workflow coordinator and typed inspection. Canic retains policy inputs and CLI report projection. |
| `crates/canic-backup/src/restore/apply/{mod.rs,validation.rs,journal/**}` | Retire the parallel apply-journal construction, command/receipt tables, counters and transitions. Bind exact child plans to the existing catalog; effect ports return independently qualified original receipts. Keep Canic Wire encoding and Root-paid runtime accounting outside this generic ledger. |
| `crates/canic-backup/src/restore/runner/{execute.rs,preview.rs,status.rs,io.rs,types.rs,artifact.rs,precondition.rs}` | Replace generic runner loops, retry-failed transitions, local artifact transfer and stdout snapshot-ID inference with bounded advancement, typed inspection, admitted byte transfer and explicit recovery owners. Preserve current Canic authority, containment and installed-target checks as mandatory ports/policy. |
| `crates/canic-backup/src/plan/build/phases.rs`, `restore/plan/{ordering.rs,summary.rs}` | Build the existing workflow allocations/dependency graph rather than maintain a parallel execution graph. Keep actual Fleet discovery, same-ID selection, Root ownership and application-specific ordering in the thin adapter. |
| `crates/canic-backup/src/manifest/{types.rs,summary.rs,validation/binding.rs}` and `restore/persistence/retention/mod.rs` | Replace generic byte-set/checksum/publication and reference-lifetime decisions with immutable download manifests, verified source guards and guarded release. Keep necessary Fleet metadata as passive product data; preserve authentication, original byte custody and complete terminal evidence. |
| `crates/canic-cli/src/restore/mod.rs::{plan_restore,restore_prepare,restore_run_execute_result,restore_run_dry_run,restore_status}` and `restore/io.rs` | Delegate preparation, bounded advancement and inspection to the existing owners. Remove `RestorePlanner`/`RestoreApplyJournal`/`HostRestoreCommandExecutor` coupling after actual effect-port adoption; retain stable operator selection, authorization, CLI parsing and presentation. |

The final consumer cut removes the superseded `canic-backup` crate once all its
maintained callers use shared owners and the remaining product facts have their
Canic host/contracts homes. A forwarding crate or copied runner in `canic-host`
does not complete that cut. Existing Canic layouts cannot be translated into fresh
Core allowance: preserve their bytes and owning released tool, and resolve unfinished
effects under original authority. This library change neither deletes the active
Canic restore path nor claims its actual Root/application/live acceptance.

## Bounded implementation sequence

1. Complete production Root-paid snapshot intent, finite dispatch and unknown-result
   reconciliation at Canic's existing replay/cost owner. PR #702's installed
   checkpoint and original host accounting provide a preparation boundary; its
   direct simulator management calls do not implement that production dispatcher.
   Retain exact snapshot metadata and complete transfer bytes before publication.
2. Qualify the production provider's actual network/caller/release, controller,
   complete current unit and ingress custody. Account every fresh observation
   separately under original authority. Reuse child planning, journal preparation
   and capture/transfer owners; do not add another ready-operation loop or turn
   fixture custody into deployed admission.
3. Connect stopped restore, complete post-load verification, accepted start and
   terminal release to that same retained operation. Preserve the active restore
   path until its replacement is qualified; each operation must have one spending
   authority. Retained old journals cannot become fresh upstream budgets.
4. After Blob is ready, select the actual application, network, same-release IDs
   and effect authority for the live attempt. Retain the complete source bundle,
   original journals, ingress/replies and provider obligations. This guide grants
   no live-effect or cleanup authority.

## Required application acceptance

| Scenario | Observable requirement |
| --- | --- |
| Complete current-unit round trip | Original capture and transfer bytes, same-ID restore, complete stopped-state verification and accepted start |
| Membership/controller/read authority changes or expires | Refusal before the next new effect; earlier spending and evidence remain intact |
| Later participant capture fails after an earlier capture | No publication combining different freeze windows after containment/release |
| Lost allocation, transfer, load or start reply; process death | Original pending attempt and retained bytes survive; no blind retry, replacement budget or inferred receipt |
| Missing/substituted post-load acceptance | No start-stage allocation or source-reference release |
| Blob/external obligations and checkpoint release | Current application-owned settlement and fence proof; local Applied checkpoints alone are insufficient |
| Completed cold replay | No new dispatch, spending, signatures or file changes; unrelated references remain retained |

Reuse Canic's actual managed checkpoint and Blob fixtures for application facts.
Review [Canic PR #607 — Two-participant checkpoint capture and restore](https://github.com/dragginzgame/canic/pull/607)
before adding another participant fixture; its installed proof and production
Root integration remain distinct.
Backup's [independent consumer qualification](agent-transport.md#independent-normal-consumer)
and [simulator evidence](pocketic-qualification.md#independent-authenticated-consumer-processes)
cover the generic interfaces, not installed Canic or real provider behavior.
Current source-specific results belong in the [handoff](status/current.md).

[Canic #513 — Adopt the generic IC Backup engine](https://github.com/dragginzgame/canic/issues/513)
owns convergence and deletion of superseded mechanisms;
[Canic #394 — Live same-release Fleet backup capture](https://github.com/dragginzgame/canic/issues/394)
owns live admission. [Canic #550](https://github.com/dragginzgame/canic/issues/550)
owns application consistency and external-obligation acceptance.
[Backup #29](https://github.com/dragginzgame/ic-backup/issues/29) is retired as a
coordination tracker, with its unresolved requirements explicitly retained in
those Canic issues. That disposition does not complete application adoption or
enable live backup creation.
