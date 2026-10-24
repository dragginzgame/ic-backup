# Preparing Canic adoption

Backup supplies original plans, bounded journals, guarded persistence, workflow
coordination and signed Agent transport. Canic supplies the application facts that
make those operations safe. Blob completion precedes the planned live backup
attempt; preparing this integration does not enable live creation.

## Verified starting point

Released Backup source **0.41.0**, `9bb769b`, contains the original-plan coordinator
and the cleanup described in the changelog. Applications exchanging Host errors
must align with Backup's selected Host family. Platform supplies external IC
identities and SDKs; Testkit remains development-only. Core does not depend on Agent;
an alternative transport needs its own qualification.

Read-only inspection on 2026-10-24 uses Canic HEAD
[`cf2d3d1`](https://github.com/dragginzgame/canic/commit/cf2d3d1fba84a00a76cdbc541ae7e68fe36e9bb5)
**plus its pending 0.112 working changes**, not that commit alone:

- Its working catalog selects Backup **0.40.1**. Artifact, layout/command custody
  and graph adoption are present; complete original stage/attempt adoption is pending.
- `canic-control-plane/src/storage/stable/application_checkpoint` retains Root
  checkpoint state outside participant snapshots. Its workflow connects current
  local-unit discovery and authenticated Wire begin/status/advance/release; ops
  connects catalogue exclusion. These are implemented working changes, not a new
  installed production qualification.
- `canic-cli/src/backup/create` still refuses live creation before effects. The
  active restore runner and its original retained journals remain maintained.
- `canic-backup/src/restore/persistence/retention` already implements
  `RestoreTerminalEvidenceProvider`. Change its evidence inputs when the engine
  changes instead of creating another terminal authority owner.

**0.41 adoption dependency:** Canic's pending
`restore/apply/journal::next_effect_sequence` and its error/test still use removed
`policy::effect_order` APIs. Do not upgrade its dependency alone. Keep the maintained
restore engine on 0.40.1 until that caller is adapted or replaced by the original
plan/journal engine. Caller-declared Completed states cannot be manufactured into
qualified upstream Applied receipts. Graph models/topological order remain available;
the existing restore state machine remains Canic-owned until its replacement is
qualified. This guide adds no compatibility alias or second spending authority.

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

## Bounded implementation sequence

1. Qualify the connected Canic production Root checkpoint storage, authenticated
   commands and whole-unit exclusion under
   [Canic #550 — Application-coordinated checkpoints](https://github.com/dragginzgame/canic/issues/550).
   Reuse those owners; do not add another store or sequence. Installed Root dispatch
   across at least two actual participants, interruption and exact replay remain
   required, separately from the driver-held fixture in PR #607.
2. Bind one new frozen application operation to the existing Backup stage driver.
   Account fresh authority observations separately. Reuse original child planning,
   journal preparation and capture/transfer owners instead of introducing another
   ready-operation loop. Qualify one complete current unit with at least two actual
   participants before exposing live creation.
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
owns live admission. [Backup #29 — Complete original-plan execution and stage binding before Canic runner adoption](https://github.com/dragginzgame/ic-backup/issues/29)
remains open for complete application acceptance. None is closed by dependency
selection, a generic registry consumer or this preparation guide.
