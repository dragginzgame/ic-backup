# Current handoff — 2026-11-04

## Selected delivery

The user requested deployment of the completed dependency/tooling/test-layout
batch. Select compatible **0.57.1** for both libraries, with the established direct
`origin/main` Git release and crates.io publication. Root package versions are
still 0.57.0 before the release runner. The complete gate must pass before
version preparation; the generated [receipt](../release.json), saved release plan
and registry evidence own completion. No live backup/restore is selected.

## Implemented batch

Production and Testkit share published **Platform 0.21.0**. Verified archive
checksums/VCS and all 115 retained non-PocketIC source files match the prior
facade's behavior. Types 0.18, management DTO 0.11.0, Agent SDK 0.49.3,
target-specific Runtime 0.19.0 and Host 0.74.2 remain selected. No product public
type, schema, wire/hash, authority or custody migration accompanies this update.

Testkit **0.84.1** owns the maintained simulator client and managed synchronous/
asynchronous builders; Platform's simulator feature and separate client package
leave Backup's development graph. Keep the managed instance intact through setup,
then transfer its client/server into explicit client-before-server fixture fields.
Original topology, request/startup bounds, raw logs and fixture custody remain.
Authenticated PocketIC stays **16.1.0**; no reset, retry or funding policy is adopted.

Shared Tooling **0.29.3** selects committed revision `1fa2d90`, excluding dirty
supplier changes. This is a source selection, not a matching-tag publication claim.
Its 148 exported payloads pass exact byte/mode review; executable catalogs are
unchanged. See the [adoption review](../shared-tooling-review.json). Keep the
coordinated workspace and complete release gate; no proof reuse or reduced CI.

Two inline Agent unit modules now use `tests.rs`; Core shared fixtures, three
regression modules and read-count implementation live beneath `tests/`. Keep the
85 existing test directories and small reader bindings at real IO boundaries.
All 510 registered unit cases and 626 inventoried test functions remain; 11
qualified paths and exact child-process selectors changed with their relocation.
The 1,881 LOC reclassification is test layout, not removal of runtime behavior.

## Evidence and limits

Platform evidence: `target/platform-021-adoption.l5sacyet/`. Focused Rust 1.99.1
checks pass 78 IC model codec/bounds/hash cases, five inventory cases, complete
Agent transfer/restore, untrusted-root rejection, lost-stop-reply recovery and
independent normal Core/Agent consumers (`target/msrv-consumers.5aM1UU/`). Retain
the initial sandbox socket failure beside the passing network-enabled run.

Shared/Testkit evidence: `target/shared-tooling-adoption.fdd2862x/`,
`target/testkit-084-adoption.vh8s9xnx/` and
`target/testkit-adoption-0571.bijimlj5/`. Prior locked inputs passed strict Clippy,
510 unit cases, 62 simulator cases, consumer/tooling/hooks/metadata checks and
independent consumers. The new release must qualify its exact combined source.
Preserve prior input snapshots, failed logs, compiler evidence and tool generations.

Previously delivered **0.57.0** is commit `97671f3`; both libraries and its
annotated tag were published. The [Canic migration](../canic-adoption.md#workflow-api-migration)
remains relevant to that API consolidation. Canic owns integration-specific tests;
this batch makes no Canic source contribution. Native macOS, live IC and installed
Canic acceptance are separate from local Linux qualification. Preserve unfinished
journals, referenced/partial artifacts and recovery evidence.
