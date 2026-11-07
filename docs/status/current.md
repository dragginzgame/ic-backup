# Current handoff — 2026-11-07

## Pending 0.59.2

Adopt published Platform 0.25.0, source
`0a32e8a2f7977a71062d4edec542bea96d63c508`. Its removal of ordinary facade
entrypoint macros and the `cdk` feature affects no Backup caller. Selected host
codec, principal and Agent implementations are unchanged. Both independent
production graphs select only Platform 0.25.0; development-only Testkit 0.92.0
retains Platform 0.24.0. No unrelated lock versions changed. Host 0.75.0,
Types 0.18.2, management DTO 0.11.0, Agent SDK 0.49.3, PocketIC 16.1.0 and Rust
1.99.1 remain unchanged. No source migration or persisted byte/hash change.

Compiler/snapshot/tool/dependency/Candid admission, independent goldens, strict
all-target/all-feature Clippy, formatting and documentation checks pass. Both
independent Rust 1.99.1 public consumers pass without simulator dependencies;
Core's normal graph excludes Agent and Process. All 87 focused native tests,
20 Core PocketIC and 22 selected Agent tests pass, including authenticated
capture/restore/application acceptance, terminal replay and interrupted recovery.
The online registry-preparation case remains ignored. Evidence is under
`target/evidence/adopt-platform025/`; independent public consumers are under
`target/msrv-consumers.VHaXXp/`, and the Agent log records the 1337746 candidate
graph/runtime fixture family. The selected lock stayed stable through validation.
Changes are uncommitted; package versions remain 0.59.1 with pending 0.59.2 notes.
No full release gate, publication, native macOS or current Canic acceptance is claimed.

## Delivered 0.59.1

Both crates are published as 0.59.1. Release
`5eef59bf7bae6a9b8db602e96bef7a6ba73d60ca` and annotated `backup-v0.59.1`
were verified on origin/main; qualified source is
`eaa8b1eb3bda9659435cd5503a30629789409979`. The normal checkout is reconciled.

Adopt published Testkit 0.92.0, source
`ceb99e1c682a92026a80d743bfccada350319c08`. Its source matches 0.91.0; the
dependency change aligns client codec errors with Platform 0.24.0 and removes
Platform 0.23.6 from the development graph. The intervening 0.91 adoption also
aligned startup/cleanup errors with Host 0.75.0 and removed three older Host
0.74.4 packages. No unrelated lock versions changed. Production and test graphs
now share Platform 0.24.0; Testkit remains development-only. Rust 1.99.1,
management DTO 0.11.0, PocketIC 16.1.0,
60-second request budgets and 120-second startup bounds remain unchanged.

The matching 0.92.0 CLI is prepared and offline server admission passes. Strict
all-target/all-feature Clippy, compiler/snapshot/dependency/Candid boundaries,
formatting and documentation checks pass. All 20 Core PocketIC and 22 selected
Agent tests pass, including independent authenticated capture/restore/acceptance,
terminal replay, interrupted reentry and lost-reply/process-death recovery. The
explicit online registry-preparation case remains ignored. Evidence is under
`target/evidence/adopt-testkit092/`; independent graph/build and runtime roots
are recorded in the Agent log (3099548 fixture family). Prior 0.91 evidence remains
under `target/evidence/adopt-testkit091/`. The selected lock remained stable
through validation. Follow-up review found no further source migration: local
Testkit setup/check routing and original-selection refusal fixtures pass, along
with six exact management-request and ten reply codec/bounds cases on the unified
Platform graph. Their logs remain with the 0.92 adoption evidence.

Batch `batch-lETTlHeO` passed the complete Linux gate: compiler/snapshot/tools,
strict Clippy, boundaries/goldens/formatting, tooling/release/hook regressions,
527 tests, documentation, independent Rust 1.99.1 consumers and archive checks.
The gate's sole ignored case is explicit online registry preparation; that setup
and registry-only authenticated capture/restore/application acceptance/terminal
replay passed separately after publication. Registry checksums and VCS metadata
match the release; all 324 packaged Rust files (279 Core, 45 Agent) and original
manifests match the delivered source. Evidence is under
`target/evidence/deploy-0591/`; published graph/runtime roots are
`target/tmp/ic-backup-agent-independent-build-2981378-0/` and
`target/tmp/ic-backup-agent-pocketic-2981378-1/`. No live IC/provider work occurred.
Native macOS and current Canic acceptance remain separate.

## Delivered 0.59.0

Preserve standalone host-driven same-release/same-ID backup/restore, mandatory
application admission, original spending and retained recovery evidence. Both
crates are published as 0.59.0. Release `b2585a3f0e332ce1423f83cc77bedbd66812fd9e`
and annotated `backup-v0.59.0` match origin/main; qualified source is
`7abb1ff9fc8bef76cc363835f4f82a58571fc817`. The normal checkout is reconciled.
Native macOS and current Canic acceptance remain separate.

Each transfer stage now owns one artifact. Constructors take one request;
collections, persisted Downloaded recovery, verify_artifact and resume projections
are retired. Valid singleton v1 identities and ChecksumVerified publication recovery
keep their meaning. External-command custody and Core's production Host Process
edge are removed. Terminal release admits exclusively in-process restores and
refuses command sidecars before qualification and before publication.

One real capture/download driver owns complete/failure qualification, retaining
lost/malformed replies, pending/successor refusal, original bytes, references,
manifest replay and retained-prefix continuation. Download/upload planning shares
SnapshotDataPlanningError. Directory publication is internal to the journal;
pure integrity returns the sole retained checksum and public fresh source checks
use exact IC metadata. Overlapping generic public recovery journeys are retired;
declared ordering, canonical drift and process-death proofs stay with their owner.

Public durable reservations use reserve_planned_mutation/reserve_planned_observation;
raw helpers are crate-private. Model transitions, read/open and qualified receipts
remain available. Read-only Canic inspection at e6a061999dfae207bc20917ffefa2082e7ffe2e7
found planned reservations and a production record_observation caller; that receipt
API is retained. No Canic files changed or consumer acceptance was inferred.

The boundary page is now a 167-line index, replacing its 2,303-line extraction
ledger. Schemas, codec bounds, golden vectors and provenance files remain intact;
the unique data-upload checkpoint hash recipe now lives in implementation contracts.
Current guidance links canonical owners; historical changelog references are pinned.
See [the boundary](../extraction-boundary.md), [contracts](../implementation-contracts.md)
and [migration](../canic-adoption.md#059-api-migration).

Current Rust inventory is 14,501 production, 37,193 test and 1,236 example code lines;
all 324 files are classified. The two earlier larger cuts removed 1,478 lines.
This reservation review adds 51 test code lines for complete-plan calls and typed
causes, without changing production accounting logic or record formats/budgets.
Requested dependency adoption now selects published Host Artifacts/FS/Process
0.75.0 and Platform 0.24.0. Testkit stays 0.90.0, retaining Host 0.74.4/Platform
0.23.6 only in development graphs. Both independent production graphs select one
Host 0.75 family and Platform 0.24. Types 0.18.2, management DTO 0.11.0, Agent SDK
0.49.3 and Rust 1.99.1 remain unchanged. No unrelated lock versions changed; the
new selected lock is stable through validation. Shared 0.31.5 now selects 160
verified payloads (three companion-annotation updates; executables unchanged).
The frozen minor delivery passed the complete Linux gate and published both crates.

## Release and focused evidence

Batch `batch-Cm_12MIv` passed compiler/snapshot/tool admission, strict Clippy,
boundary/golden/format checks, tooling/release/hook regressions, 527 tests,
documentation, both independent Rust 1.99.1 consumers and archive verification.
The explicit online registry-preparation case is the gate's sole ignored test;
post-publication preparation and registry-only authenticated capture/restore,
application acceptance and effect-free terminal replay then passed separately.
Crates.io checksums and VCS metadata match the release; all 324 packaged Rust files
(279 Core, 45 Agent) and original manifests match the delivered source. Evidence
is under `target/evidence/deploy-0590/`; registry build/runtime roots are
`target/tmp/ic-backup-agent-independent-build-4048577-0/` and
`target/tmp/ic-backup-agent-pocketic-4048577-1/`. No live IC/provider work occurred.

The first requested 0.59.0 delivery, `batch-5pqpnHYc`, stopped at release-check
before real version preparation, push or publication. Its local patch fixture
copied the caller's minor pending notes; the fixture now creates its own matching
notes before committing its disposable source. Preserve the failed batch and
`target/evidence/deploy-0590/delivery.log`. The corrected fresh candidate passed
that fixture and the complete gate before delivery.

The requested Host/Platform adoption passes root locked metadata, dependency
boundaries/inheritance, independent Core/Agent public-type builds and Clippy,
affected native identity/codec/publication/crash cases, shared formatting, docs and
Candid/CBOR checks. Core's normal graph excludes Agent/Process; both independent
production graphs exclude simulator packages. The authenticated standalone consumer
and Core lost-load/status reconciliation pass on the new graph. Evidence is under
`target/evidence/upstream-adoption-059/`; public-type probe graphs are under
`target/msrv-consumers.9wCg5B/`. Current candidate build/runtime roots are
`target/tmp/ic-backup-agent-independent-build-4031968-0/` and
`target/tmp/ic-backup-agent-pocketic-4031968-1/`. Published VCS provenance is Host
`81de54ec4b176a21418f996f229353c20a48f69a` and Platform
`fc6d419474a94a606e3c49729747e61978b4c9c7`. Backup already uses bounded Candid
helpers and does not select the removed offline-key feature. Public Host error
consumers must align to 0.75; see the 0.59 migration.

The current reservation batch passes journal/persistence and complete-plan admission
cases, directly affected public native targets, seven Core PocketIC reconciliation
cases, four Agent HTTP recovery cases and two Agent PocketIC probes (status and
full-MiB allocation/readback). Strict affected library/test/example Clippy, shared
fmt-check, Candid/CBOR admission, Markdown links and diff checks pass. The initial
two-held-journal fixture now uses sequential custody; its passing rerun and original
failure remain under `target/evidence/contracts-and-reservations-059/`.

Earlier hard-cut evidence remains under `target/evidence/hard-cut-059/` and
`target/evidence/surface-cut-059/`. Their independent candidate consumer passes
capture, same-ID restore, acceptance and effect-free terminal replay. Graph/build
proof is `target/tmp/ic-backup-agent-independent-build-1567544-0/`; authenticated
runtime proof is `target/tmp/ic-backup-agent-pocketic-1567544-1/`. Earlier 774369
fixture roots and corrected failed attempts remain named in those logs.
Initial fixture/settlement cleanups retain evidence under
`target/evidence/consolidation-05812/`, `target/evidence/observation-fixtures-05812/`
and `target/evidence/typed-fixtures-05812/`.

## Delivered 0.58.11

Both crates are published. Release `e6471464406e97d77b1c4b610dd2a5c22991f27c`,
annotated `backup-v0.58.11` and qualified source
`16527d4b5cd97f5f699b80f125ca5ecd1290b1c1` were verified on origin/main.
Registry checksums/VCS readback matched all 330 packaged Rust files (285 Core,
45 Agent). The full Linux gate passed strict Clippy, 544 tests, documentation,
tooling/release/hook checks, independent consumers and archive checks; the online
cache-preparation case remains ignored in that gate. Explicit post-publication
preparation and registry-only authenticated acceptance/terminal replay then passed.
Issue [#79](https://github.com/dragginzgame/ic-backup/issues/79) is closed completed.

Delivered selections were Platform 0.23.5, Types 0.18.2, Testkit 0.90.0, Host/Build
0.74.4, Agent SDK 0.49.3, PocketIC 16.1.0 and Rust 1.99.1. Shared 0.31.3 is
`d2a2f395bc06b22cb3f18afc7d2978a150726360` (160 canonical payloads); Testkit VCS
provenance binds `c03161b9f1395c343756fda6b8721e957798c3df`. Both managed fixtures
use 60-second requests and separate 120-second startup bounds. Full delivery proof
is under `target/evidence/deploy-05811/`, including its registry-only 2256280
consumer roots and explicit 2252982 preparation. No live IC/provider work occurred.

## Preserved delivery and recovery state

Batch `batch-lETTlHeO` is complete/reconciled, not retired. Preserve
`target/release-frozen/batch-lETTlHeO/`, its source pin and durable receipts under
`.git/release-state/frozen/batch-lETTlHeO/`. Earlier batches remain intact.

Batch `batch-Cm_12MIv` is complete/reconciled, not retired. Preserve its candidate
at `target/release-frozen/batch-Cm_12MIv/`, source pin and durable receipts under
`.git/release-state/frozen/batch-Cm_12MIv/`. Failed `batch-5pqpnHYc` remains intact.

Batch `batch-_jjlBLUV` is complete/reconciled, not retired. Preserve
`target/release-frozen/batch-_jjlBLUV/`, its source pin and durable Git/plan/
validation/publication records under `.git/release-state/frozen/batch-_jjlBLUV/`.
Failed batches `batch-DWdrLMkj` and `batch-7uUMU2LC` remain intact. They caught
semicolon lints and fixture leakage of publication settings before publication;
corrected source passed the complete gate. The completed, unretired 0.58.10 batch
and older receipts remain in the
[prior handoff](https://github.com/dragginzgame/ic-backup/blob/16527d4b5cd97f5f699b80f125ca5ecd1290b1c1/docs/status/current.md).

Recover consistency/fence declaration files and distinct learned-edge originals
with 0.57.1 and the original qualified integration. Recover command-backed restores,
multi-artifact journals and Downloaded records with 0.58.11 and their original
integration. Preserve old restore copies/metadata and settle them with their original
tool or explicit safe disposition. Missing original plans require original-tool
recovery, not replacement intent. Never remove evidence, re-sign, reset spending,
blindly retry or reinterpret payloads to bypass admission. All terminal replay is
effect-free; fresh application acceptance and external obligations remain mandatory.
[Coverage ownership](../development.md#simulator-coverage-ownership) stays local;
producer passes do not qualify Canic or every supported host.
