# Current handoff — 2026-11-04

## Completed delivery

**0.57.1** is released on `origin/main` with annotated tag `v0.57.1`, both pointing
at release commit `40d9e3234df06df589ef591786e3a77ce2aa43b7` before this handoff
update. Both `ic-backup` and `ic-backup-agent` 0.57.1 are published on crates.io.
The complete Linux release gate passed against source `8edede0`; the committed
[receipt](../release.json) binds the original source, inputs, date and version.

Independent registry/archive checks verified non-yanked versions, exact SHA-256,
release VCS identity and all 328 Core / 53 Agent non-generated source files:

- Core: `56df47c36747441b7e2cd446a4b49966cc2374b44e6d67bfd07b726872a7f921`.
- Agent: `05566ffe9fd74d0d7c01ccd5035a53560b17c041225b73c617d79a6168569430`.

Release/publication logs, downloaded archives and source verification are retained
under `target/deploy-0571.ytcigfjv/`. Saved Git release/validation evidence is under
`.git/release-state/0.57.1.*`; publication selection and command evidence is under
`.git/release-state/publication/{selection.mMJuGH,publish.selection.mMJuGH}`.
The annotated release tag remains on its original release commit; this handoff
records delivery only. No live backup/restore is selected.

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
independent consumers. The completed release gate freshly qualified the combined source.
Preserve prior input snapshots, failed logs, compiler evidence and tool generations.

Previously delivered **0.57.0** is commit `97671f3`; both libraries and its
annotated tag were published. The [Canic migration](../canic-adoption.md#workflow-api-migration)
remains relevant to that API consolidation. Canic owns integration-specific tests;
this batch makes no Canic source contribution. Native macOS, live IC and installed
Canic acceptance are separate from local Linux qualification. Preserve unfinished
journals, referenced/partial artifacts and recovery evidence.
