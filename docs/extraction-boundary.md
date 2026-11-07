<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-backup/ic-backup-readme-header.svg" alt="IC Backup — Verified backups and safe recovery for Internet Computer apps" width="100%">
</p>

<!-- helper-navigation:start -->
<p align="center">
  <a href="https://github.com/dragginzgame/canic"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/canic.svg" width="18" height="18" alt=""> <strong>canic</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/icydb"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/icydb.svg" width="18" height="18" alt=""> <strong>icydb</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-timers"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-timers.svg" width="18" height="18" alt=""> <strong>ic-timers</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-memory"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-memory.svg" width="18" height="18" alt=""> <strong>ic-memory</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-query"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-query.svg" width="18" height="18" alt=""> <strong>ic-query</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-backup"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-backup.svg" width="18" height="18" alt=""> <strong>ic-backup</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-blob-storage"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-blob-storage.svg" width="18" height="18" alt=""> <strong>ic-blob-storage</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-testkit"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-testkit.svg" width="18" height="18" alt=""> <strong>ic-testkit</strong></a>
</p>
<!-- helper-navigation:end -->


# Record and API boundary

This page indexes the maintained contracts. [Implementation contracts](implementation-contracts.md)
own invariants, effect ordering, custody and recovery. The linked machine contracts
own exact fields, encodings, hash framing, bounds and independent golden vectors.
[Source modules](../crates/ic-backup/src/lib.rs) implement those contracts;
[the design](extraction-design.md) contains rationale and proposed product work.

## Current responsibilities

| Owner | Maintained responsibility |
| --- | --- |
| Core model | Canonical identities, bounded records, wire codecs and valid transitions |
| Core policy | Pure admission, progress and settlement projections; no IO or scheduling |
| Core ops | Guarded original records, durable spending, artifact custody and publication |
| Core workflow | Original-plan execution, mandatory admission, explicit receipts and checkpoints |
| Agent | Configured signed ingress, retained original envelope/request ID and one submission |
| Integration | Fresh membership/permissions, exact attribution, application safety and acceptance |

The standalone assembly composes host-driven capture, download, upload and same-ID,
same-release restore. Its application contract remains mandatory. Neither codecs,
matching declarations nor a completed local checkpoint supply application authority.
See [the standalone workflow](../crates/ic-backup-agent/examples/standalone/workflow/mod.rs)
and [required application acceptance](canic-adoption.md#required-application-acceptance).
There is no default live application authority or arbitrary-program backend.

## Record and wire contracts

All current repository-owned persisted records are strict v1. Preserve their
existing discriminators and exact encoding; narrowing an API never authorizes
reinterpretation, regeneration of intent, reset spending or disposal of evidence.

### Original records and publication

| Contract | Canonical specification |
| --- | --- |
| Physical inventory and selection | [Inventory](contracts/inventory.schema.json) |
| Explicit dependency graph | [Graph](contracts/effect-graph.schema.json) |
| Full original plan and derived authority | [Operation plan](contracts/operation-plan.schema.json) |
| Per-operation reservation/receipt history | [Attempt journal](contracts/attempt-journal.schema.json) |
| Original stage catalog | [Workflow](contracts/execution-workflow.schema.json) |
| Child identity and learned predecessors | [Stage binding](contracts/execution-stage.schema.json) |
| Immutable all-Applied history checkpoint | [Execution settlement](contracts/execution-settlement.schema.json) |
| One original snapshot artifact per stage | [Journal](contracts/download-journal.schema.json) |
| Immutable local manifest and fingerprints | [Manifest](contracts/download-manifest.json) |
| Original-bound integrity admission | [Integrity](contracts/download-integrity.json) |
| Fixed IC artifact tree and interruption recovery | [IC artifact](contracts/ic-snapshot-artifact.json) |
| Conservative source dependencies | [Restore references](contracts/restore-references.schema.json) |
| Same-release source and application-safety binding | [Restore requirement](contracts/restore-safety-requirement.schema.json) |

The [records and spending rules](implementation-contracts.md#records-plans-and-spending)
own complete original admission. The [filesystem rules](implementation-contracts.md#filesystem-publication-and-command-custody)
own no-follow publication, durable completion and refusal of replaced custody.
[Stage/checkpoint rules](implementation-contracts.md#workflow-preparation-and-checkpoints)
separate record-only inspection, complete-journal continuation and terminal evidence.
Missing originals never mean unused allowance; failed preparation stays occupied.

Public durable reservations use `reserve_planned_mutation` and
`reserve_planned_observation`. They admit the retained original plan and complete
journal set before consuming allowance. Hold no other attempt guard during their
sequential admission. Pure model transitions remain available for in-memory
accounting; they perform no persistence or dispatch. Qualified receipt recording
retains its existing owner and integration responsibilities.

### Management, transfer and upload bytes

| Contract | Canonical specification |
| --- | --- |
| Closed lifecycle/capture host requests | [Request record](contracts/ic-request.schema.json) |
| Bounded capture/list replies | [Snapshot replies](contracts/ic-snapshot-reply.json) |
| Bounded lifecycle/status projection | [Lifecycle replies](contracts/ic-lifecycle-reply.json) |
| Declared inventory comparison | [Inventory delta](contracts/snapshot-inventory-delta.json) |
| Exact metadata request and raw evidence | [Metadata](contracts/ic-snapshot-metadata.json) |
| Metadata-bound extents and data replies | [Data](contracts/ic-snapshot-data.json) |
| Source-bound metadata/data upload | [Upload](contracts/ic-snapshot-upload.json) |

[Wire admission](implementation-contracts.md#management-wire-and-response-admission)
owns exact identity/check ordering, per-call limits, finite decode work and redaction.
[Artifact and transfer rules](implementation-contracts.md#snapshot-artifacts-and-transfer)
own complete coverage, partial-byte custody, fixed trees and verified publication.
Raw IC snapshot IDs remain distinct from exact opaque local artifact tokens;
integrations qualify their association. No codec infers authentication or receipts.

### Single-step effects and reconciliation

| Contract | Canonical specification |
| --- | --- |
| Originally reserved mutation | [Mutation port](contracts/ic-mutation-port.json) |
| Originally reserved status/list observation | [Observation port](contracts/ic-observation-port.json) |
| One reserved metadata/data read | [Transfer read](contracts/ic-snapshot-transfer-read-port.json) |
| One reserved metadata/data upload | [Upload port](contracts/ic-snapshot-upload-port.json) |
| Original metadata-allocation list | [Allocation observation](contracts/ic-snapshot-upload-observation.json) |
| Exact destination extent readback | [Data observation](contracts/ic-snapshot-upload-data-observation.json) |
| Explicit capture attribution | [Capture settlement](contracts/ic-capture-settlement.json) |
| Explicit metadata-allocation attribution | [Allocation settlement](contracts/ic-snapshot-upload-settlement.json) |
| Explicit original-write attribution | [Data settlement](contracts/ic-snapshot-upload-data-settlement.json) |
| Explicit original stop/start/load attribution | [Lifecycle settlement](contracts/ic-lifecycle-settlement.json) |
| Same-ID load/start application admission | [Restore safety](contracts/restore-safety-port.json) |

[Async coordination](implementation-contracts.md#async-effect-coordination) owns
persist-before-effect accounting, exclusion across awaits, mandatory admission,
retained replies before cancellable qualification and explicit receipt/checkpoint
ordering. [Settlement rules](implementation-contracts.md#explicit-recovery-settlement)
require independently qualified attribution; status equality, matching bytes and
inventory cardinality imply no outcome. Lost replies preserve pending consumption.

[Terminal/reference release](implementation-contracts.md#terminal-evidence-and-reference-release)
is separate from all-Applied local replay. The provider authenticates the complete
original journal, current application acceptance and settled external obligations.
Current terminal release admits exclusively in-process execution and refuses
retained command sidecars. It performs no IC effect, fence release or cleanup.

## Retained recovery boundaries

| Retired surface | Required disposition |
| --- | --- |
| Consistency/fence framework and distinct learned-edge continuations | Recover with 0.57.1 and the original qualified integration before upgrading. Guarded operations refuse retained `consistency-requirement.json` or `fence-obligation.json` entries, including malformed bytes/symlinks; advancement refuses distinct-edge bindings. |
| Command-backed restores, multi-artifact journals and Downloaded records | Recover with 0.58.11 and the original qualified integration. Command sidecars and retired journal shapes refuse without reinterpretation. Valid singleton Created/ChecksumVerified/Durable v1 identities retain their meaning. |
| Generic restore copies | Preserve `restore-artifact-{sequence}.tmp`, canonical copies and their metadata. Settle unfinished copy-based restores with their original released tool or explicit safe disposition from their owning integration. |
| Raw durable reservation entrypoints | Use the planned public methods with complete retained originals. Read/open and qualified receipt recording remain available; no record format changes or allowance reset are supplied. Missing original plans require original-tool recovery, not fabricated replacement plans. |

Never delete markers, sidecars, journals, partial/canonical bytes or references to
bypass admission. No compatibility reader, migration engine, cleanup or refund is
provided. [The migration guide](canic-adoption.md#059-api-migration) owns consumer
API changes; [application fencing](implementation-contracts.md#application-admission-and-fencing)
owns the remaining product obligations.

## Qualification and provenance

[Coverage ownership](development.md#simulator-coverage-ownership) and
[simulator qualification](pocketic-qualification.md) distinguish native record/IO
proof from authenticated IC effects and complete consumer journeys. The
[Agent guide](agent-transport.md) owns signed ingress and trust boundaries.
[Current status](status/current.md) records the latest source/dependency evidence;
producer checks do not qualify Canic's production integration or every host.

The earlier [extraction ledger](https://github.com/dragginzgame/ic-backup/blob/d6bf34c490e676240b23426ceef8fc3d59d0f2c5/docs/extraction-boundary.md) preserves source inspection, retired
interfaces and historical qualification at its exact revision. The original
[extraction provenance](extraction-source.json), [source baseline](source-baseline.json)
and topic-specific `*-source.json` files remain unchanged. They document extraction
history rather than another current implementation contract. Contributor notices
remain in the root [license](../LICENSE).
