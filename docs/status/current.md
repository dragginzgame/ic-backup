# Current handoff — 2026-11-15

Work lives at `/home/adam/projects/ic-backup`, branch `main`; legacy is read-only.
GitHub `main` was verified at `f86f274` (0.101.9), adding source-bound restore
planning. Product manifests and both locks select 0.101.10; qualification stays
0.1.0. All packages remain unpublished. No tag, registry publication or deployment
occurred. Post-push review covered plan encoding/replay, shared transfer ordering
and affected native/IC callers; no actionable drift was found in that scope.
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
They expose inspection only. Pure restore status validation now shares the capture
wire reader and policy, selecting the restore sender and original source module.
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
Only complete initial journals are admitted; no later restore event exists yet.
Bind allocation-dependent exact requests and consume bounded authority before
later effects. Restore targets the same existing ID and leaves it stopped.
Next add bounded initial restore status admission and spending before signing.
Keep historical request/certificate authentication separate from the new pure
status predicates; do not manufacture a capture plan to reuse provider APIs.
The 0.101.10 GitHub checkpoint contains additive local intent and status-policy APIs.
Its delivery and post-push review precede continuation; no registry release is selected.

## Dependencies and evidence

Platform 0.33.7/Types 0.20.4, Host FS/Artifacts/HTTP 0.79.0, Tooling 0.44.3,
Tokio 1.54.1 and Rust/Cargo 1.100.0 remain selected. Private qualification retains
Testkit 0.106.0/PocketIC 16.1 and `thiserror` 2.0.18; product uses 2.0.21.
The preserved local rand 0.10.4 graph was fetched and used for this batch's checks.
No sibling patches or shared targets. [Setup](../development/setup.md) owns commands.

`target/evidence/restore-status-policy/` records two new restore policy groups
within 40 completion/source regressions, 15 shared status cases and lint/docs checks.
No signing/network reservation or new restore journal event was added.

`target/evidence/restore-intent/` retains six native groups (five in the aggregate
run plus the corrected older-source rerun), shared capture regressions and an
actual-IC offline creation/reopen fixture, plus lint/docs checks. Earlier failed
fixture/compiler logs remain. The distinct completed-owner native fixture needed
its own transfer commitment and canonical event bytes; no production check was relaxed.

`target/evidence/restore-transfer/` retains four source-schedule groups, 48 shared
capture/readback regressions and the actual-IC offline source journey.
`target/evidence/restore-plan/` retains four authenticated plan groups, one independent
encoding vector, product/private lint and the actual-IC offline plan consumer.
Version-only build/format/docs checks are retained in the isolated delivery checkout
`/tmp/ic-backup-release-0.101.9/target/evidence/0.101.9/`.
Earlier completion, succession, completion-verification and capture-source evidence,
including failed bundles/logs, stays under root `target/evidence/` and qualification
`target/`. Detailed historical qualification remains in Git at `f86f274` and the
owning contracts; no evidence or recovery files were removed.

Small fixtures do not qualify physical power loss, external signer processes,
Canic fences, trusted HTTPS, hosted macOS, maximum-size throughput or restore.
[Canic #1](https://github.com/dragginzgame/canic/issues/1) owns the integration adapter;
Canic owns concurrency, membership, routing, release identity and maintenance.
Blob storage remains v2. Restoration cannot rewind payments or other obligations.
