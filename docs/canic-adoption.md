# Preparing Canic adoption

Backup supplies original plans, bounded journals, guarded persistence, workflow
coordination and signed Agent transport. Canic supplies the application facts that
make those operations safe. Blob completion precedes the planned live backup
attempt; preparing this integration does not enable live creation.

## Verified starting point

Published Core and Agent **0.38.0** select Host **0.64**. Applications exchanging
Host errors must select that same family; no checksum conversion bridge is needed.
Platform supplies external IC identities and SDKs. Testkit remains a development
dependency. Agent is optional at the application architecture level: Core does not
depend on it, and an alternative transport needs its own qualification.

Read-only inspection of Canic
[`cf2d3d1`](https://github.com/dragginzgame/canic/commit/cf2d3d1fba84a00a76cdbc541ae7e68fe36e9bb5)
confirms:

- Its catalog/lock already select Backup 0.38.0 and Host 0.64. Its handoff records
  passing artifact and CLI restore-runner checks. This is dependency adoption,
  not adoption of the complete original-plan workflow.
- [Live creation](https://github.com/dragginzgame/canic/blob/cf2d3d1fba84a00a76cdbc541ae7e68fe36e9bb5/crates/canic-cli/src/backup/create/mod.rs)
  returns `LiveCreateUnavailable` before local or remote effects. Dry-run planning
  and the existing restore runner remain separate maintained paths.
- [Terminal retention](https://github.com/dragginzgame/canic/blob/cf2d3d1fba84a00a76cdbc541ae7e68fe36e9bb5/crates/canic-backup/src/restore/persistence/retention/mod.rs)
  already implements `RestoreTerminalEvidenceProvider` and guarded reference
  release. Do not add a second provider merely to satisfy a checklist.
- [The installed checkpoint fixture](https://github.com/dragginzgame/canic/blob/cf2d3d1fba84a00a76cdbc541ae7e68fe36e9bb5/crates/canic-testing-internal/src/pic/application_checkpoint/mod.rs)
  explicitly leaves production Root sequencing and paid orchestration outside its
  proof. Its test-driver authority is not a production provider.

## Integration map

Keep the adapter in `canic-backup`, with CLI delegation. Do not import Canic into
Core or copy this repository's test driver into production unchanged.

| Concern | Existing Backup interface | Canic responsibility |
| --- | --- | --- |
| Frozen operation and learned stages | `ExecutionWorkflowRecord`, `ExecutionWorkflowStageDriver`, `run_retained_execution_workflow` | Bind exact Fleet/network/release/current unit and authenticate predecessor evidence |
| Original effect accounting | `OperationPlanRecord`, `ExecutionStageGuard`, `AttemptJournalGuard` | Allocate once; use the same authority across interruption; retain exact commands/bytes |
| Capture and transfer | `capture_snapshot`, metadata/download plans and source-bound upload steps | Admit current controller/read authority and application consistency before each new effect |
| Signed transport | Agent mutation, transfer-read, recovery and fence-acquisition providers | Select identity, endpoint and trust root; retain exact ingress/replies; qualify observations |
| Restore and source lifetime | Restore-safety requests, guarded reference release and terminal evidence | Whole-unit fencing, current external obligations, post-load acceptance and safe release |

Choose the actual management dispatcher before connecting transport. Backup's
[control-admission contract](implementation-contracts.md#application-admission-and-fencing)
requires the selected caller itself in the observed controller set; an operator
acting through a Root proxy does not satisfy that lane. Canic's dry-run
`root_controller(Declared)` value is not observed authority. If Root dispatches
management calls, qualify that implementation and its exact original accounting
through Core's ports rather than asserting that the host Agent is the controller.

The [canonical consumer](../crates/ic-backup-agent/tests/independent/consumer/mod.rs)
demonstrates these public interfaces across cold processes. Its private fixed key,
single target, exclusive controller custody, fixture acceptance files and
`NoIrreversibleEffects` safety lane are test conditions. A deployed Blob-backed
application must establish its actual obligations and fencing; restoring snapshots
does not rewind provider payments, references or other external effects.

## Bounded implementation sequence

1. Complete Canic's production Root checkpoint ownership and authenticated
   whole-unit orchestration under
   [Canic #550 — Application-coordinated checkpoints](https://github.com/dragginzgame/canic/issues/550).
   Persist issuance/release authority outside restored participant snapshots and
   exclude incompatible membership changes. Couple the store to real callers and
   admission; an unused store or second sequence owner is not progress.
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
