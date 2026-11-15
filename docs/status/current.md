# Current handoff — 2026-11-15

Work lives at `/home/adam/projects/ic-backup`, branch `main`; legacy is read-only.
GitHub `main` remains `dc4bba68bbaeb0966ecefb8335c4ede85d95e125` (0.101.12).
Local 0.101.13 adds the direct restore profile and durable status submission.
A local pre-replacement checkpoint is selected as
`preserve/backup-0.101.13-before-replacement`; no push, release tag, publication or
deployment is selected. All packages remain unpublished; qualification stays 0.1.0.
The later broad audit found four structural maintenance problems that the focused
post-push review did not establish; cleanup now precedes feature expansion.
The checkpoint includes the tested local rand 0.10.4 locks for exact preservation;
remote 0.101.12 uses 0.10.3. This is not dependency delivery authorization.

## Implemented capture and source boundaries

The [inventory](../architecture/local-inventory.md),
[capture](../architecture/capture-intent.md) and
[transfer](../architecture/capture-transfer.md) contracts own capture behavior.
Immutable intents, bounded journals and original authority/evidence remain mandatory.
Uncertain writes invalidate handles; complete uncertain frames remain spent.
Partial tails, orphan/changed evidence and custody failures refuse without repair.
Prepared continuation never resigns; pending calls never repeat. Surviving original
replies and one durable lookup can settle status/list/take. Dependent effects need
live permission; historical replay reads no current clock, guard or signer.
Only direct Ed25519/secp256k1 are qualified. Integrations own keys, continuous
application maintenance, fleet membership, scheduling and external obligations.

Metadata/data acquisition retains exact requests, certified results and reservations.
Invalid settled payload blocks progress; accepted payload replays offline. Full
1 MiB extents exceed the 2 MiB result cap with proof/framing and refuse.
Metadata/data and final-status reconciliation remain unfinished. Assembly retains
partial/altered files without repair. Final status has distinct request/spending.
Completion authenticates evidence, synchronizes assembly and closes the journal
at readiness; exact create-only manifest publication can reconcile locally.
Succession uses target-then-predecessor locks and Host atomic owner replacement,
preserving old receipts and interrupted files. Held source reads bind completion,
same-store capture lock and block-checked payload descriptors. Completed receipts
outlive ownership changes; unfinished paths need their original owner.

## Current restore increment

The [local-store contract](../design/local-store.md#restore-status-submission-and-replay)
owns restore status preparation, original profile admission, submission and replay.
Transfer planning binds same-target/release extents and `2*N + 7` base calls.
Plans retain approved source, ordered requests, direct sender, trust and finite
limits. Create/reopen authenticates original source and current completed-owner
custody; an older source may differ from the current owner. No pruning exists.

`DirectRestoreStatus` wraps the existing signer/admission interface and commits
transport/network/root/timeout, integration protocol and timing before reservation.
It checks exact observation, invocation-time reference, expiry and live authority
before retaining signer output. Capture and restore share timing arithmetic,
prepared-frame capacity, transport binding and the one-shot Host HTTP call path.
There is no second guard API, journal, evidence envelope or general provider engine.

`submit_status` reads the original request/admission from the existing journal.
Fresh work re-admits the direct profile and live application authority, synchronizes
one submission reservation, then rechecks live admission before HTTP entry.
Only a bounded authenticated terminal result is retained, create-only and synced
before its journal reference. Stopped/module/controller policy runs afterwards.
Certified rejection and policy-invalid replies remain settled. Lost replies,
timeouts, cancellation, invalid proof and uncertain persistence never refund work.
Cold pending replay returns attention without calling the provider; cold settled
replay reauthenticates original evidence without clock, live guard, signer, HTTP
or writes. Inspection reports reserved counts and evidence presence, not success.
The existing journal owner is now `restore/persistence/status/`; original preparation
wire names remain unchanged. Older readers refuse the extended history intact.

Native HTTP groups check exact request/path, guard/profile refusal before spending,
terminal replay, HTTP uncertainty, cancellation/timeout, damaged result and partial
reservation bytes, plus an orphan result appearing after request capture.
A new actual-IC case submits restore status from a completed capture source,
independently checks stopped module/snapshot count, closes the server and revokes
the guard before exact terminal replay. The original offline signing case remains.
All 348 unit tests, 24 documentation tests and six IC journeys pass, together with
strict product/private Clippy, warning-denying Rustdoc, formatting and link checks.
Latest evidence: `target/evidence/restore-submission/` (including earlier failures).

Next: review the [minimal replacement proposal](../design/restart.md), including its
concrete retirement map, before more product edits. Incremental phase cleanup and
allocation/upload expansion are paused. The proposal replaces phase APIs and their
call-state duplication with fixed capture/restore procedures; no implementation yet.
Original 0.101.13 source/build preservation is documented in
[setup](../development/setup.md#pre-replacement-reference); no general recovery CLI exists.
Canic manager execution remains an explicit unresolved integration dependency.
Half of legacy is a rough size estimate, not a hard budget or acceptance gate.
The first cleanup shares retained evidence membership across integrity checks, orphan
detection and synchronization, removing 78 production LOC without changing record bytes.
Budget replay now derives totals from validated events instead of scanning all prior
admissions. A state-based oracle matches every completed-capture journal prefix;
data budget refusal survives reopen without renewed signing/network work.
Current Rust LOC: 10,259 production / 29,474 total (+8 production in this batch).
All 349 unit and 24 doc tests pass with four workers; six IC journeys, product/private
Clippy and Rustdoc pass. Evidence: target/evidence/replay-budget-cleanup/.
The earlier default-concurrency fixture deadlines and successful four-worker run
remain under target/evidence/evidence-inventory-cleanup/. Whole-evidence rereads
and metadata/data lifecycle duplication remain unresolved; no throughput claim.
Pending restore reply import/lookup reconciliation is absent. Upload/load, CLI and Canic adapters remain unfinished;
restore must leave the original canister stopped. Do not claim end-to-end restore.

## Dependencies and retained evidence

Platform 0.33.7/Types 0.20.4, Host FS/Artifacts/HTTP 0.79.0, Tooling 0.44.4,
Tokio 1.54.1 and Rust/Cargo 1.100.0 remain selected. Private qualification retains
Testkit 0.106.0/PocketIC 16.1 and thiserror 2.0.18; product uses 2.0.21.
Checks use the preserved local rand 0.10.4 graph. No sibling patches/shared targets.
[Setup](../development/setup.md) owns commands. Qualification remains under
`qualification/ic-backup-qualification/`, evidence under `qualification/target/fixtures/`.
The delivered 0.101.12 graph passed 343 units, 24 examples and five IC journeys in
`target/evidence/delivery-0.101.12/`; preparation profile checks remain in
`target/evidence/restore-profile/`. Earlier source/recovery and failed fixtures
remain under the original targets and owning contracts. Nothing was removed.

Physical power loss, external signer processes, Canic fences, trusted HTTPS,
hosted macOS, maximum-size throughput and upload/load remain unqualified.
[Canic #1](https://github.com/dragginzgame/canic/issues/1) owns integration work.
Blob storage remains v2. Restoration cannot rewind payments or other obligations.
