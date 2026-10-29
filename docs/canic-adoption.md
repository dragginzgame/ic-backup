# Preparing Canic adoption

Backup supplies original plans, bounded journals, guarded persistence, workflow
coordination and signed Agent transport. Canic supplies the application facts that
make those operations safe. Blob completion precedes the planned live backup
attempt; preparing this integration does not enable live creation.

## Verified starting point

Delivered Backup source **0.48.2**, `ba7f5d4`, supplies the original-plan
coordinator, explicit restore source provenance and bounded upload sessions.
Applications exchanging Host errors must align with Backup's selected Host family.
Platform supplies external IC identities and SDKs; Testkit remains development-only.
Core does not depend on Agent; an alternative transport needs its own qualification.
Its [complete native CI matrix](https://github.com/dragginzgame/ic-backup/actions/runs/40698974873)
passes on Linux, Intel macOS and Apple Silicon. This qualifies the library and
its controlled consumer fixtures; production Canic acceptance remains separate.

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

Keep the adapter in `canic-backup`, with CLI delegation. Do not import Canic into
Core or copy this repository's test driver into production unchanged.

| Concern | Existing Backup interface | Canic responsibility |
| --- | --- | --- |
| Frozen operation and learned stages | `ExecutionWorkflowRecord`, `ExecutionWorkflowStageDriver`, `run_retained_execution_workflow` | Bind exact Fleet/network/release/current unit and authenticate predecessor evidence |
| Original effect accounting | `OperationPlanRecord`, `ExecutionStageGuard`, `AttemptJournalGuard` | Allocate once; use the same authority across interruption; retain exact commands/bytes |
| Direct management capture and transfer | `capture_snapshot`, metadata/download plans and source-bound upload steps | Select this lane only when the actual host caller has the required direct authority |
| Application Root ingress | Original stage/attempt guards and integration-owned request codec/dispatcher | Retain exact Wire requests; account Root-paid management effects through Canic's existing runtime owner |
| Signed transport | Agent mutation, transfer-read, recovery and fence-acquisition providers | Select identity, endpoint and trust root; retain exact ingress/replies; qualify observations |
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

## Pending application transport adapter

[#74](https://github.com/dragginzgame/ic-backup/issues/74) supplies a concrete
Backup-owned transport gap found during Canic adoption. Its working-source
`AgentApplicationIngressProvider` accepts Core's distinct original application
mutation and recovery-observation requests and reuses the existing single-submit
owner. Published 0.48.3 lacks these new APIs. The
[transport guide](agent-transport.md#opaque-application-ingress) and
[independent consumer](../crates/ic-backup-agent/tests/independent/consumer/application/mod.rs)
show exact payload binding, mandatory ingress/reply retention and separately
budgeted recovery without pretending that every Root command acquires a fence.

When adopting the containing release, bind fresh child requests to the admitted
opaque payload digest and update exhaustive `ReservedUpdate` matches for its new
variants. Do not reinterpret PR #702's older retained commitments or replay uncertain
ingress. Canic still owns Wire, current Root/unit authority, its production provider,
Root-paid replay/cost, full artifact transfer and application/terminal acceptance.
The new adapter supplies no live-create permission or automatic Applied receipt.

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
