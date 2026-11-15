# Current handoff — 2026-11-15

Work lives at `/home/adam/projects/ic-backup`, branch `main`; legacy is read-only.
GitHub `main` was verified at `6f6cb771da5ce606d95f8a0ecdbff59d8dff36b6`
(0.101.11), delivering restore signing preparation, flat tests and Tooling 0.44.4.
No tag, registry publication or deployment occurred; every package stays unpublished.
0.101.12 is prepared for GitHub delivery; qualification remains independently 0.1.0.
Its exact rand 0.10.3 graph passes 343 unit tests, 24 examples, five IC journeys
and strict product/private lint and Rustdoc in `target/evidence/delivery-0.101.12/`.
Post-push review covered shared framing/signature admission, restore reservations,
source/owner custody, flat test fixtures, qualification paths and documentation.
No further actionable drift was found in that delivered scope. The pre-push review
corrected signer HTTP wording and stale test selectors without changing behavior.
The separate local `rand` 0.10.4 edits are preserved under
`target/evidence/delivery-0.101.12/`; delivery retains 0.10.3. Restore those exact
local selections after delivery. Do not silently stage that refresh.

## Implemented behavior and boundaries

The [inventory contract](../architecture/local-inventory.md),
[capture contract](../architecture/capture-intent.md) and
[transfer contract](../architecture/capture-transfer.md) own implemented capture.
Immutable intents, bounded journals and original authority/evidence remain mandatory.
Uncertain writes invalidate handles; complete uncertain frames remain spent.
Partial tails, orphan/changed evidence and custody failures refuse without repair.
Prepared continuation never resigns; pending calls never repeat. Surviving original
replies and one durable lookup can settle status/list/take. Dependent effects need
live permission; historical replay reads no current clock, guard or signer.
Only direct Ed25519/secp256k1 are qualified. Root/network/route, sender, protocol
and application bindings remain mandatory. Integrations own signing, maintenance,
snapshot custody and external obligations; Backup owns no keys or fleet scheduling.

Metadata/data acquisition retains exact requests, certified results and reservations.
Settled invalid payload blocks progress; accepted payload replays offline. Full
1 MiB extents exceed the 2 MiB result cap with proof/framing and refuse.
Metadata/data and final-status reconciliation remain unfinished. Assembly retains
partial/altered files without repair. Final status has distinct request/spending.
[Completion](../design/local-store.md#completion-and-interrupted-publication)
authenticates evidence, synchronizes assembly and closes the journal at readiness;
exact create-only manifest publication can reconcile locally. No later event is valid.
Succession uses target-then-predecessor locks and Host atomic owner replacement,
preserving old receipts, occupied claims and interrupted files. Held source reads
bind completion, same-store capture lock and block-checked payload descriptors.
Completed receipts outlive ownership changes; unfinished paths need original owners.

`RestoreTransfer` calculates exact same-target/release extents and `2*N + 7` base
management calls using shared capture ordering. Whole chunks cannot split; empty
chunks count. `RestorePlan` binds source/ordered requests, direct sender, initiator,
trust and finite limits. Strict decoding re-admits source and derived fields while
retaining original bytes. Local create/reopen now retain original intent, target
ownership and source custody.
They support signing-only status preparation and read-only inspection. Status
policy uses the shared bounded capture reader with the restore sender and original
source module. Request authentication reuses exact direct-key/call admission.
Local `RestorePlan::authenticate_status_evidence` now checks that request plus
original root, effective canister, historical certificate/delegation windows and
exact certified outcome/response. It reuses capture's bounded verifier, without a
fabricated capture plan or new provider framework. No clock, IO, spending or
journal changes occur; callers still establish original custody. An authentic
rejection is not successful status; apply status policy separately.

## Current continuation

The [restore creation contract](../design/local-store.md#restore-creation-and-interrupted-intent)
owns original source versus current completed-owner admission under target-first
custody. Both retain authentication and original evidence. No pruning or reverse
reference files exist. Creation uses existing Host mechanics and one journal;
reopening needs the original approved plan and does not write or synchronize.
Initial, signing-reserved and prepared journals revalidate original custody,
canonical transitions and exact signed evidence. Partial/unknown suffixes and
orphan files refuse intact. Uncertain persistence invalidates handles.
One synchronized signing reservation precedes signer entry. Failure, cancellation
or invalid output stays spent; pending/prepared replay never resigns. The IC
fixture signs locally and cold-reopens; its opaque observation placeholder is not
a certificate policy or dispatch authority. Physical power loss/external signer
processes remain unqualified.

The [certificate contract](../design/local-store.md#restore-status-certificate-authentication)
records focused legacy review and changed assumptions. Two native groups cover
both direct keys, wrong roots/results/outcomes/IDs, altered proof, strict observation,
exact frame limits, historical windows and delegated target. They preserve source
journal and supplied bytes without creating a restore operation or issuing HTTP.
Named shared certificate fixtures keep restore tests flat under `restore/tests/`.
All 343 unit tests, 24 documentation tests and five IC journeys pass, with strict
product/private Clippy, warning-denying Rustdoc, formatting, links and snapshot
admission. Evidence: `target/evidence/restore-certificates/`. The IC cases retain
capture qualification; they do not qualify restore dispatch.

Next: original route/transport and live application/timing admission, then reserved
one-shot status submission and retained certified evidence in the same journal.
Unknown outcomes stay pending; original requests/spending cannot reset. Actual-IC
restore status qualification accompanies dispatch. Upload/load, CLI and Canic
adapters remain unfinished; restore targets the same existing ID and leaves it stopped.

## Dependencies and evidence

Platform 0.33.7/Types 0.20.4, Host FS/Artifacts/HTTP 0.79.0, Tooling 0.44.4,
Tokio 1.54.1 and Rust/Cargo 1.100.0 remain selected. Private qualification retains
Testkit 0.106.0/PocketIC 16.1 and `thiserror` 2.0.18; product uses 2.0.21.
The preserved local rand 0.10.4 graph was fetched and used for this batch's checks.
No sibling patches or shared targets. [Setup](../development/setup.md) owns commands.

The delivered 0.101.11 graph (rand 0.10.3) passed 341 unit tests, 24 examples,
five IC journeys and strict product/private lint in `target/evidence/delivery-0.101.11/`.
Earlier preparation, request, policy, intent, transfer and plan evidence remains
under root `target/evidence/restore-*/`; earlier completion/succession/source and
failed fixtures remain in root and qualification targets. Git `f86f274` and the
owning contracts retain historical detail. Nothing was removed.

Toko Tooling 0.44.4 was checksum-verified and adopted through exporter preview/update;
both workspace pins/locks and receipt agree. Only the managed baseline changed.
Private qualification lives directly in `qualification/ic-backup-qualification/`;
evidence stays in `qualification/target/fixtures/`. Five IC journeys, strict lint,
tool/snapshot admission, formatting, links, dependency listing and LOC reporting
passed in `target/evidence/tooling-0.44.4/`. Flat test cleanup passed the unchanged
341-test registry plus 24 documentation tests in `target/evidence/test-layout/`.
Both batches shipped in 0.101.11.

Small fixtures do not qualify physical power loss, external signer processes,
Canic fences, trusted HTTPS, hosted macOS, maximum-size throughput or restore.
[Canic #1](https://github.com/dragginzgame/canic/issues/1) owns the integration adapter;
Canic owns concurrency, membership, routing, release identity and maintenance.
Blob storage remains v2. Restoration cannot rewind payments or other obligations.
