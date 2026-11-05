# Historical audit conclusions and retention

This is the current summary of the superseded October 2026 adoption/audit runs.
The authorized Shared Tooling 0.30.4 cleanup removed 177 per-run reports/data files
and six superseded handoff archives (3,407,885 bytes). Their exact original bytes
remain in [Git history at `f241568`](https://github.com/dragginzgame/ic-backup/tree/f241568f3714c0fd9283fd505ff34007c6bcf3e8/docs).
This summary does not replace their original qualification or prove that the raw
local logs, simulator state or build artifacts named there still exist.

The retained conclusions are:

| Topic | Current owner and conclusion |
| --- | --- |
| Original records, attempts and custody | [Implementation contracts](../implementation-contracts.md) and [source boundary](../extraction-boundary.md) own bounded intent, original journals, no blind retry and effect-free terminal replay. Early continuation reports are historical slices, not another protocol. |
| Direct Agent and simulator journeys | [Agent transport](../agent-transport.md) and [PocketIC qualification](../pocketic-qualification.md) own the maintained fixtures. Early 158-case or ten-case results do not qualify current dependencies, native macOS or installed application acceptance. |
| Host, Platform, Testkit and Metrics adoption | [Platform boundary](../ic-platform.md), current manifests/lock and [handoff](../status/current.md) own current selections. Old package/archive comparisons remain tied to their historical source and graph; Metrics is not a current dependency. |
| Shared setup, hooks and release machinery | [Shared Tooling adoption](../shared-tooling.md) owns current provenance and caller qualification. Generic tooling implementation regressions belong upstream; Backup tests its actual adapters. |
| Architectural sediment | [Ownership](../ownership.md) retains the policy/workflow/ops split and Host/Platform/Testkit delegation. Earlier JSON-wrapper/FIFO cleanup and source-embedding findings are superseded by the implemented source and its current tests. |

Per-run raw transcripts, repeated source-tree hash tables and copied prior reviews
are no longer maintained documentation. Historical claims and unresolved findings
in those reports retain their original limits; they are not current blockers or
acceptance decisions. Current release identity and qualification belong to the
handoff and release receipt, rather than these historical run counts.

Root `*-source.json`, extraction/license provenance, maintained contract vectors,
release receipts and active deployment/recovery records were not removed. No
ignored logs, failed bundles, journals, artifacts or external obligations were
deleted or moved. The protected original-state recovery instructions remain in
the [handoff](../status/current.md#recovery-and-retained-contracts).

New investigations keep one compact current topic summary and selected small
results when useful. Raw experimental output belongs under `target/evidence/`;
routine checks need no new tracked report. Git history retains superseded text.
Follow the [shared retention contract](../../rules/engineering-details.md#changelogs-and-artifact-preservation)
for budgets, exceptions and separate release/recovery custody.
