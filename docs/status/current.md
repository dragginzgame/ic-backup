# Current handoff — 2026-11-15

Work lives at `/home/adam/projects/ic-backup`, branch `main`; legacy is read-only.
GitHub `main` was verified at `eb99058` (0.101.10), delivering local restore intent
persistence and status policy. Local product manifests and both locks now select
0.101.11; qualification stays 0.1.0. All packages remain unpublished. No tag, registry publication or deployment
occurred. Post-push review covered restore creation/reopen, source/owner custody, shared
framing/status policy and affected native/IC callers. It found duplicate completion
authentication/sync when source and predecessor are the same capture. Local 0.101.11
removes that repeat, retaining exact source admission and closing custody checks.
Distinct older sources still synchronize separately; no other actionable drift
was found in the delivered scope.
The separate local `rand` 0.10.4 updates in both locks were preserved and excluded
from delivery. The delivered graph retains 0.10.3; do not silently stage that refresh.

## Implemented behavior and boundaries

The [inventory contract](../architecture/local-inventory.md) owns private stores,
bounded list/latest queries, manifest-bound integrity and held payload reads.
The [capture contract](../architecture/capture-intent.md) owns immutable intent,
target claims, bounded chained journals and exact status/list/take evidence.
Uncertain writes invalidate handles. Complete uncertain frames remain spent;
partial tails, orphan evidence and changed custody refuse without repair.
Prepared continuation never resigns; pending calls never repeat. Surviving replies
and one durable lookup can settle original calls; lookup evidence retains its
separate original time policy. Dependent effects require live permission.
Historical replay never consults a current clock, guard or signer.

The direct provider composes bounded external signing and one-shot Host HTTP.
Original root/network/route, sender, protocol and application bindings remain
mandatory. Only direct Ed25519/secp256k1 are qualified. Integrations own signer
custody, application maintenance, snapshot custody and external obligations.
Backup owns no keys, Canic runtime, fleet membership or scheduling.

The [transfer contract](../architecture/capture-transfer.md) owns bounded metadata/
data decoding, the lazy schedule and sequential acquisition. Exact signed requests,
certified results and reservations remain in the original journal. Settled invalid
payload blocks progress; accepted payload replays offline. Full 1 MiB extents cannot
fit the 2 MiB result cap with proof/framing and refuse. Metadata/data reconciliation
remains unfinished. Assembly writes exact files directly to the unpublished bundle;
partial/altered files remain retained and refuse automatic repair.

Final status reserves a distinct original request and spending after assembly.
Policy failures stay settled; final lookup/surviving-reply reconciliation is absent.
[Completion](../design/local-store.md#completion-and-interrupted-publication)
authenticates original evidence and synchronizes assembly before `PublishReady`
freezes the timestamp and closes the ledger. Exact create-only manifest publication
can reconcile locally; partial/changed/aliased/duplicate files refuse intact.
No event follows readiness. Inventory consumes the same published manifest.

Authenticated completed-capture succession uses target-then-predecessor locks,
Host atomic owner replacement and shared intent creation. Initial capacity is
admitted before writes. Old receipts and lock identities remain. Interrupted
replacement/creation preserves occupied claims and partial files without repair.
Read-only `verify_completion` authenticates exact original receipts and bytes.
`open_capture_source` binds this to the same store, original capture lock and held
payload descriptors. Bounded block-checked reads refuse changed bytes without
reauthenticating certificates per extent. Completed receipts outlive ownership
changes; unfinished/readiness/staging paths still require their original owner.

`RestoreTransfer` calculates exact same-target/release extents and `2*N + 7` base
management calls using shared capture ordering. Whole chunks cannot split; empty
chunks count. `RestorePlan` binds source/ordered requests, direct sender, initiator,
trust and finite limits. Strict decoding re-admits source and derived fields while
retaining original bytes. Local create/reopen now retain original intent, target
ownership and source custody.
They now support signing-only status preparation and read-only inspection. Pure restore status validation now shares the capture
wire reader and policy, selecting the restore sender and original source module.
`RestorePlan::authenticate_status_request` now reuses bounded call-content,
per-call ceiling and direct-key signature checks for that plan's sender/target.
It preserves original bytes; signature validity grants no spending or dispatch.
Upload/load effects, CLI and Canic adapters are unfinished.

## Continuation selected after delivery

The [restore creation contract](../design/local-store.md#restore-creation-and-interrupted-intent)
now describes implemented original-source versus current-owner admission and retained
interruption refusal. An older source may differ from the current
completed capture owner; both require authentication under target-first custody.
The immutable restore plan itself retains the source dependency. Separate reverse
reference files were removed from the proposal: no pruning or reference release
exists, and every source/operation stays retained. Completed bundle admission is
unchanged. Creation uses existing Host mechanics and one initial journal; reopening
requires the caller's original approved plan and does not write or synchronize.
Initial, signing-reserved and prepared journals are admitted with exact original
request evidence; partial/unknown suffixes and orphan evidence refuse intact.
Bind allocation-dependent exact requests and consume bounded authority before
later effects. Restore targets the same existing ID and leaves it stopped.
One signing reservation is now synchronized before signer entry. Cancellation,
failure and invalid output stay spent; prepared and pending replay never resign.
Next integrate bounded status submission and certificate/route/timing admission,
retaining original spending and exact requests. Upload/load remain unfinished.
The 0.101.11 batch is prepared for the requested GitHub-only checkpoint.
The exact delivery graph (rand 0.10.3) passes all 341 unit tests, 24 examples,
five IC journeys and strict product/private lint under
`target/evidence/delivery-0.101.11/`. The separate local rand update stays excluded.
Next development version: 0.101.12, after delivery and its post-push review.

Four new native preparation groups, ten shared journal regressions, strict
product/private Clippy, warning-denying Rustdoc, formatting and documentation links
pass under `target/evidence/restore-preparation/`. The IC fixture signs locally and
reopens the exact prepared request without another signature; it submits no restore
request. Physical power loss and external signer processes remain unqualified.

Test layout is now flat: capture workflow cases stay in `capture/advance/tests/`,
restore cases in `restore/tests/`, with named shared fixtures. All 341 unit tests
and 24 documentation tests pass; the registered unit-test inventory is unchanged.
Strict product/private lint passes. `target/evidence/test-layout/` retains results.
No runtime behavior or release metadata changed in this layout cleanup.

## Dependencies and evidence

Platform 0.33.7/Types 0.20.4, Host FS/Artifacts/HTTP 0.79.0, Tooling 0.44.4,
Tokio 1.54.1 and Rust/Cargo 1.100.0 remain selected. Private qualification retains
Testkit 0.106.0/PocketIC 16.1 and `thiserror` 2.0.18; product uses 2.0.21.
The preserved local rand 0.10.4 graph was fetched and used for this batch's checks.
No sibling patches or shared targets. [Setup](../development/setup.md) owns commands.

`target/evidence/0.101.11/` retains same-owner and distinct older-source regressions,
the offline actual-IC intent fixture and lint/documentation checks for the local fix.
The exact 0.101.10 checkpoint was checked with rand 0.10.3 in the isolated delivery
checkout under `target/evidence/0.101.10/`: 27 native cases and the offline IC
fixture passed, with strict product/private lint and formatting/documentation checks.

Earlier request, status-policy, intent, transfer and plan qualification remains
under `target/evidence/restore-status-request/`, `restore-status-policy/`,
`restore-intent/`, `restore-transfer/` and `restore-plan/`. These older batches
qualify their named local boundaries, not signing or remote restore effects.
Historical completion/succession/source evidence and failed bundles remain under
root and qualification `target/`; Git `f86f274` and the owning contracts retain
historical detail. Isolated delivery evidence remains under
`/tmp/ic-backup-release-0.101.9/target/evidence/`. Nothing was removed.

Toko Tooling 0.44.4 was checksum-verified and adopted through exporter preview and
update; both workspace pins/locks and the receipt agree. Only the installed
baseline document changed. The private package now lives directly at
`qualification/ic-backup-qualification/`, matching its package name; the redundant
`crates/` level is gone. Original evidence still uses `qualification/target/fixtures/`.
All five IC journeys, strict product/private Clippy, tool/snapshot admission,
formatting, links, dependency listing and LOC reporting pass. Evidence is under
`target/evidence/tooling-0.44.4/`. Package versions, publication flags and unrelated
dependency selections were unchanged by adoption. GitHub delivery is now requested.

Small fixtures do not qualify physical power loss, external signer processes,
Canic fences, trusted HTTPS, hosted macOS, maximum-size throughput or restore.
[Canic #1](https://github.com/dragginzgame/canic/issues/1) owns the integration adapter;
Canic owns concurrency, membership, routing, release identity and maintenance.
Blob storage remains v2. Restoration cannot rewind payments or other obligations.
