# Current handoff — 2026-11-07

## Pending 0.59.0

Preserve standalone host-driven same-release/same-ID backup/restore, mandatory
application admission, original spending and retained recovery evidence. Packages
remain 0.58.11; the pending changelog selects 0.59.0 for breaking APIs. Changes are
uncommitted. Release, native macOS and current Canic acceptance remain separate.

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
The requested frozen minor delivery will run the complete Linux gate and publish
both crates as 0.59.0; native macOS and current Canic acceptance remain separate.

## Focused evidence

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
