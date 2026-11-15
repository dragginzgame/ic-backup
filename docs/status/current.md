# Current handoff — 2026-11-15

Work lives at `/home/adam/projects/ic-backup`, branch `main`; legacy remains read-only.

The 0.101.9 GitHub-only checkpoint adds restore transfer scheduling and immutable plans.
Product manifests and both locks select 0.101.9; private qualification remains 0.1.0.
All packages remain `publish = false`; no tag or registry publication is selected.
GitHub `main` was verified at `e838fec` (0.101.8). The scoped post-push review
covered completion authentication, assembly verification, held source custody and
native/IC callers. No actionable drift was found in that delivered scope.
Delivery of 0.101.9 and its required review precede the next increment.

## Implemented capture and recovery

The [inventory contract](../architecture/local-inventory.md) owns private stores,
bounded list/latest queries, manifest-bound payload integrity and held reads.
The [capture contract](../architecture/capture-intent.md) owns immutable intent,
target claims, bounded chained journals and exact status/snapshot evidence.
Uncertain writes invalidate live handles. Complete uncertain frames remain spent;
partial tails, orphan evidence and changed original custody refuse without repair.

Status/list/take retain separate original admissions and cumulative reservations.
Prepared continuation never resigns; pending calls never repeat. Surviving replies
can settle original calls. One durable lookup can settle a terminal retained
observation under its separate original time policy. Dependent effects require
live permission; historical replay does not consult a current clock or guard.

The direct provider composes external bounded signing and one-shot Host HTTP.
Original root/network/route, sender, protocol and application bindings remain
mandatory. Integrations own signer/process custody, maintenance, snapshot custody
and external obligations. Only direct Ed25519/secp256k1 are qualified; Backup owns
no keys, Canic runtime, fleet membership or scheduling.

The [transfer contract](../architecture/capture-transfer.md) owns bounded metadata/
data decoding and the exact lazy schedule. Durable metadata and sequential data
reads retain signed requests and certified results in the original journal.
`bind_transfer` reuses retained metadata without a signer. Accepted payloads replay
without HTTP or writes; invalid certified payload stays settled and blocks progress.
All calls share original signer/network/evidence limits. Complete-proof allowance
is admitted before effects; full 1 MiB extents cannot fit the 2 MiB result cap with
proof/framing and refuse. Metadata/data reconciliation remains unfinished.

`verify_data` authenticates complete accepted coverage and the combined module
hash. `assemble_data` writes exact files directly into the unpublished bundle,
with one assembly intent and compact verified tree commitment. Complete files
can reconcile; partial/altered files remain retained and refuse automatic repair.

`advance_final_status` shares the initial-status workflow and policy, but reserves
its own admission and files after verified assembly. Reusing the initial request
ID refuses before submission. Running/module/controller policy failures remain
settled; final lookup/surviving-reply reconciliation is not implemented.
`inspect_final_status` projects phase, original admission and consumed allowance.

[Completion](../design/local-store.md#completion-and-interrupted-publication)
requires successful retained final status and original lookup settlement when a
lookup exists. `complete_capture` authenticates original evidence and synchronizes
assembly, then records `PublishReady` once. It reserves the readiness frame and
1 MiB manifest ceiling, freezes the host timestamp and forbids further events.
The existing manifest schema binds exact plan/closed-ledger/metadata bytes and
payload members. Host publishes `manifest.pending` create-only as `manifest.json`.
Local reconciliation retains the exact receipt/timestamp, without remote work or
new events. Partial, changed, aliased or duplicate manifest files refuse intact.
List/latest and payload verification consume the same completed manifest.

Completed-capture admission retains exact published receipts after ownership changes.
`create_capture_after` takes target-then-predecessor locks, authenticates and syncs
the already-published completion, atomically replaces only the owner pointer using
Host FS, then shares ordinary new-intent creation. Initial journal capacity is
admitted before writes. The old receipt, evidence and lock identities remain.
New intent has zero reserved calls and requires its own bounded live authority.
Readiness, staging, missing or changed receipts cannot release the original claim.
Interrupted replacement/creation retains the owner and partial files; incomplete
successors refuse without rollback, repair, blind retry or authority reset.

Local `verify_completion` checks a selected manifest hash, original certified
evidence and exact assembled source bytes without writes, syncing or publication.
It shares completion authentication and assembly comparison; readiness or staging
alone refuses. Its summary is passive evidence, not a held source or restore
approval. `open_capture_source` now binds that verification to an exact receipt,
caller byte limit, original operation borrow/lock and existing held payload
descriptors. The same store is mandatory. Bounded reads reuse block checks and
refuse changed bytes; they do not repeat certificate authentication per extent.
Source custody is ephemeral. Local `plan_restore_transfer` now computes exact
source extents with the shared capture ordinal mapper, same-target/release checks
and `2*N + 7` base management calls. Whole chunks use actual lengths and cannot
be split; empty chunks still count. The result retains the exact source receipt
but creates no destination or authority. `RestorePlan` now binds the admitted
source and exact ordered request commitment to a direct sender, initiator, trust
root, same target/release and finite restore limits. It encodes a bounded closed
record and approval digest; decoding re-admits source/derived fields and retains
original bytes. This is in-memory intent only. Durable restore creation/references,
restore effects, CLI and Canic adapters remain unfinished. Local completion reconciliation can synchronize
files; read-only inspection/inventory and settled remote-call replay are separate.
Small fixtures are qualified; repeated proof/file checks have not established
practical maximum-size throughput. No cache or generic recovery engine was added.

## Dependencies and evidence

Platform 0.33.7/Types 0.20.4, Host FS/Artifacts/HTTP 0.79.0, Tooling 0.44.3,
Tokio 1.54.1 and Rust/Cargo 1.100.0 remain unchanged. Private qualification retains
Testkit 0.106.0/PocketIC 16.1 and `thiserror` 2.0.18; product retains 2.0.21.
No dependency refresh, sibling patches or shared targets were introduced.
[Setup](../development/setup.md) owns commands and supplier provenance.

`target/evidence/completion/` records 169 advance, 10 ledger and 54 store tests,
24 documentation examples, strict product/private Clippy, warning-denying Rustdoc
and four IC fixtures. Four final-status and six completion groups cover distinct
identity, policy/budget refusal, exact prepared continuation, uncertain frames,
manifest crash images, frozen timestamps, original hashes and existing consumers.
The small IC fixture now completes capture, queries inventory, verifies payload
and reconciles after gateway shutdown and revoked live permission. Its original
evidence allowance is 6 MiB to admit all call and manifest ceilings; product limits
are unchanged. Earlier evidence and failed/successful fixture bundles are retained.
These checks do not qualify physical power loss, external signer processes,
Canic fences, trusted HTTPS, hosted macOS, maximum-size throughput or restore.

`target/evidence/succession/` records six succession and 16 initial-intent tests,
strict product/private Clippy, Rustdoc and the real-IC completion fixture extended
through offline succession and original-receipt replay. The fixture performs no
second-capture IC effects. Native images cover partial successor creation; physical
power loss remains unqualified. The existing initial-budget regression caught a
refactor ordering mistake; admission was restored before writes and a successor
regression added. Earlier failed evidence is retained.

`target/evidence/completion-verification/` retains native completion/succession
regressions and the actual-IC fixture verifying before and after succession, with
no new remote work. Strict product/private lint and documentation checks apply.
Original schemas, dependencies and the published checkpoint version are unchanged.

`target/evidence/capture-source/` retains three authenticated-source tests and
eight existing held-reader regressions, plus offline actual-IC source reads before
and after succession. Strict product/private lint and documentation checks apply.
No upload/load effects or durable restore-reference writer are qualified yet.

`target/evidence/restore-transfer/` records four source-schedule tests, 48
capture/readback regressions and the real-IC offline source journey. Product and
private lint plus documentation checks qualify the local calculation only.

`target/evidence/restore-plan/` records four source-bound plan regression groups,
one independent encoding/approval vector, strict product/private lint and the
real-IC offline plan construction/decode consumer. No restore files are written.

## Next work

Persist the approved restore plan, initial bounded journal, target claim and exact
source reference together under target-before-source lock order. Define retained
interruption admission before implementing this writer; do not grant effects from
partial creation. Bind allocation-dependent exact requests before submission. Then implement bounded
same-release restore to the same existing ID, left stopped.
Preserve original evidence and finite authority; do not add cleanup or reset paths.
Canic owns membership, routing, concurrency, release identity and maintenance;
[Canic #1](https://github.com/dragginzgame/canic/issues/1) tracks its adapter. Blob
storage remains v2. No next delivery or version bump is authorized by continuation.
