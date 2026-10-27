# Current handoff — 2026-10-27

## Released source and ownership

Released Backup is **0.46.0**, `bbe4c18ab81b4567631e9b7e6d0f101a56483498`.
The [released handoff](https://github.com/dragginzgame/ic-backup/blob/bbe4c18ab81b4567631e9b7e6d0f101a56483498/docs/status/current.md)
retains the preceding ten simplifications and their original evidence scope.
Core owns bounded records, guarded persistence and original-plan coordination;
Agent owns reserved signed transport. Applications retain authority, membership,
release binding, fencing and external obligations.
[Implementation contracts](../implementation-contracts.md) remain authoritative.

## Pending 0.47.0 simplifications

All ten reviewed changes are implemented. Capture, upload and transfer-read
coordinators now preserve returned evidence through `AfterReply` with a typed
Journal, Association or Stage cause. Each ordered check and original journal
exclusion remains in place, including the decoded-view continuation used by
compound transfers. Source, pending spending, replies and recovery evidence remain
retained; no hidden reservation, retry, receipt or cleanup is introduced.

The three association policies expose `Identity(ResponseIdentityMismatch)` for
Authority, Attempt, Context and Target denials. `policy` exports the shared cause.
Original reservation admission still precedes identity comparison; bounded decoding
follows it. Consumers must update their matches on these and the workflow errors.
[The pending changelog](../../CHANGELOG.md) selects **0.47.0** for that hard cut.
There are no compatibility aliases or retained-data migration requirements.

The final settlement API follow-up exposes `Identity(SettlementIdentityMismatch)`
for lifecycle/data-write claims and `Identity(InventorySettlementMismatch)` for
capture/metadata-allocation claims. Both causes are exported from `policy`.
Four duplicated per-policy conversion implementations are removed; the existing
comparison functions, closed-baseline-before-identity order, attribution checks
and domain-specific denials are unchanged. This completes the agreed error API
scope for the pending 0.47.0 cut.

Reference synchronization uses Host FS's existing no-follow, nonblocking regular
file opener, retaining native IO errors and InvalidInput for nonregular entries.
Upload construction and hashing reuse the metadata owner's admitted Platform
Principal. Inventory closure and cycle checks use canonical target lookup instead
of a temporary parent map, with missing-parent admission still preceding cycles.
Restore-reference decoding sorts once and checks adjacent paths; multiply corrupt
inputs now report the first duplicate in canonical path order. Version, field,
count and authority denials remain strict. Planned observation reservation reuses
its existing complete-original-progress admission method before the same transition.

Hook fixtures delegate Rust and member-manifest selection to Shared's existing
`--rust-sources`, including tracked deletions and untracked additions. Testkit
command fixtures reuse Shared's lifecycle helpers and declare their complete
helper closure. They explicitly create their default target parent; success cleans
only their fixture, while failure preserves it and its original exit status.

Four read-only release-adapter subprocess paths share checked output capture.
Selected Git reads, manifest projection, version and increment commands still
require successful child completion before consuming stdout. Candidate-rendering
and original receipt/recovery custody remain explicit. The existing metadata
selection now includes the shared roster of seven version-reader cases.

## Focused evidence

The preceding ten-change batch passed **113** selected Core tests before the
settlement API follow-up. Filters cover snapshot workflows; mutation,
upload and transfer-read association policies; upload and inventory models;
restore-reference decoding; layout lifetime, attempt journals and execution
progress. The 54 affected workflow cases pass again after the final caller-pattern
cleanup. Existing assertions retain typed causes, exact bytes/hashes, pending
attempts, cancellation, original source references and bounded provider calls.
New cases cover regular/missing/symlink/directory/FIFO reference synchronization
and deterministic duplicate-path reporting, including conflicting authorities.

The settlement follow-up passes **24** owning policy unit tests and **four**
public consumer tests in `ic_observation` and `ic_snapshot_upload`. All identity
components, baseline drift, exact evidence, attribution and pending-spending
assertions remain active. Capture/allocation tests additionally reject baseline
drift before conflicting identity claims; public lifecycle/capture consumers match
the exported typed stale-challenge causes and retain byte-identical journals.
Focused Clippy for Core's library and those public targets passes with
`-D warnings`. Logs are `/tmp/backup-settlement-{lifecycle,data,capture,allocation,public,clippy}.log`.
Formatting, documentation links and diff checks pass for the extended batch.

The following receipts predate the settlement follow-up and keep that source scope.
Clippy passes with `-D warnings` for Core's library and `pocketic_snapshot` caller,
and separately for Agent's production library. Simulator fixture callers compile
and lint; no simulator effects were executed. An initial 101-line fixture helper
was shortened by removing redundant all-fields rest patterns. Its diagnostic
excerpt is retained at `/tmp/backup-next10-initial-clippy-diagnostic.txt`; the
complete initial Cargo log was replaced by the passing rerun.

All five hook cases pass at `target/hook-tests.zKmed1/`. All 12 selected metadata
cases pass at `target/shared-release-tests.dQeStL/`, and all 13 receipt/recovery
cases pass at `target/shared-release-tests.SuZtRe/`. Testkit routing/refusal passes.
An isolated copy without target/ proves parent creation, success cleanup and
exact failure-23 retention at `/tmp/backup-next10-lifecycle-us724sjp/`.
Initial fixture refusals remain retained with their logs. Evidence and commands
are under `/tmp/backup-next10-*`. Matching ShellCheck `-x`, Bash/Perl syntax,
formatting, local documentation links and diff checks pass.

Cargo.toml and Cargo.lock remain byte-identical to HEAD. Lock SHA-256 is
`c5ba971535f9ae7d55b6d47d4d0c544167325ebcad9ed7dd203c782d8d31d10a`.
The actual released lock already selects Host/Testkit **0.69.1** and Platform
**0.7.0**; editor selection remains **0.69.0**, Shared Tooling **0.26.23** and
Rust floor **1.91**. Package versions remain **0.46.0**. No snapshot payload,
sibling checkout, dependency declaration or lock was edited.

## Limits and delivery

No full suite/CI, simulator execution, new Agent HTTP run, native macOS or MSRV
run is claimed. These focused checks add no live IC effect, authenticated
application settlement, terminal proof or fence/reference-release qualification.
Earlier Agent crypto-bigint qualification remains unresolved. Changes are unstaged
and uncommitted; no own version bump, tag, push, publication, deployment or release
was performed.
