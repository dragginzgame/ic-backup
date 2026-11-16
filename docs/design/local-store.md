# Local storage and fleet inventory

Selected scope: local-host bundles, independent single-canister operations and
read-only history. No legacy records or unfinished obligations are imported.
The [capture contract](../architecture/capture-intent.md) owns the current fixed
capture loop, formats, common journal and exact completion; the
[inventory contract](../architecture/local-inventory.md) owns scan/payload bounds.
The source-bound same-canister restore below uses the same recorder and direct
connection. Canic manager/fence integration remains unqualified.

## Small public surface

| Operation | Meaning |
| --- | --- |
| `list_backups` / `latest_completed_backup` | Bounded completion declarations, without payload authentication |
| `verify_payload` / `open_payload` | Exact payload under caller-trusted manifest and byte limit |
| `create_capture` / `open_capture` | Approved immutable intent and original custody |
| `CaptureOperation::inspect` | Authenticated retained progress and conservative spending |
| `CaptureOperation::advance` | Fixed next work; at most one new management submission |
| `reconcile_reply` / `reconcile_observation` | Explicit original-call recovery; never resubmission |
| `verify_completion(expected_manifest)` | Exact already-published receipt/evidence/payload, without writes |
| `open_capture_source(operation, receipt, limit)` | Authenticate and hold same-store original source |
| `create_capture_after(plan, approval, predecessor)` | Authenticated completed-target succession |

No database, index, storage-provider trait, background worker or fleet scheduler is
introduced. Inventory is not an immutable fleet snapshot. Latest means capture
timestamp, then completion timestamp, then bundle ID. Corruption or exhausted scan
limits never becomes authoritative absence or fallback to an older good row.

## One layout, one spending authority

```text
store/
  format
  targets/<network>/<canister>/{lock,owner.json}
  operations/<capture-or-restore-id>/{lock,intent.json,journal.bin,<derived-evidence>.bin}
  bundles/<capture-id>/{manifest.json,payload/}
```

The marker is `ic-backup/host-store/1\n`. Earlier stores refuse intact and remain
under the preserved reader; no dual-format migration/reset exists. Initialization
is explicit into an empty private directory; partial format publication remains
occupied evidence. Directories/files are 0700/0600; Host owns no-follow descriptors,
locks and durable publication. Backup owns record meaning and effect authority.

Each purpose has one immutable intent and one journal of common call/observation facts. No separate source catalog, transfer-binding file, accepted-data
record, assembly digest or mutable spending file exists. No pruning is implemented.
Source dependencies remain in original restore plans and retained files.

## Completion and interrupted publication

Capture advance authenticates all original results, materializes exact payload,
rechecks the original module and performs distinct final status. PublishReady
freezes completion time and closed journal hash. The exact manifest commits original
intent, metadata evidence, journal and closed payload member set. The journal never
commits its future manifest hash.

Missing payload members can be created from original certified evidence; exact
complete members can be reused. Partial/altered/unknown files remain and refuse.
Complete exact manifest staging can reconcile create-only; partial/changed staging
refuses. No automatic rollback, truncation or cleanup is authorized. Read-only
published replay performs no sync or publication.

### Completed receipt admission

Published capture evidence remains admissible after authenticated target ownership
changes. Structural custody requires the actual published receipt to bind original
intent, closed journal and payload tree. A staged/missing receipt cannot excuse
a changed owner. Public open/inspect/source/advance still authenticate full original
semantic history. Held restore custody pins the trusted manifest/payload separately.
There is no persistent authentication cache.

### Explicit completed-capture succession

`create_capture_after` requires exact approval and same target. It acquires target
then predecessor lock, verifies/synchronizes its published completion, atomically
replaces ownership through Host, then retains the new immutable intent. New operation
allowance is separate; predecessor evidence/receipt is unchanged. Unfinished
predecessors, occupied IDs, ambiguous owner publication and held locks refuse.
Failed creation preserves any new claim/partial original records without rollback.

### Read-only completion verification

`verify_completion(expected_manifest)` authenticates original intent, requests,
certificates, complete payload and exact published receipt. It takes no transport,
signer or live application authority and performs no writes/sync. Staged publication
is not a completed receipt. Completion supplies passive evidence, not rewind safety.

### Authenticated held capture sources

`open_capture_source` requires the same held store and exact caller-selected receipt
plus a positive payload ceiling. It holds capture exclusion and original verified
payload descriptors. Bounded reads check exact target/snapshot/geometry against
ephemeral source block hashes and closing custody. Wrong store/receipt, changed
source/lock/member or exhausted bounds refuse; no file repair occurs.

Focused frozen-legacy review is recorded in the capture/transfer/inventory contracts:
`4ae087fad8c23de712aa60bf569bde4b2b932651` attempt-journal, artifact verification/
completion and original source readers; fixes `d245b0f03a505ba5215d65c10bdebf1890c7dc37`,
`a860d14421b9ff1c61c1dce079cdc6a19316bfd0` and
`9f7b886a824eec67dda8f81792a560436419bb2a`. Original-evidence/ownership guarantees
are retained without legacy stage catalogs or mutable reference-release journals.
Native custody/death/publication tests and actual IC succession/older-source cases
now exercise the replacement public consumer. No legacy implementation was copied.

### Exact restore transfer calculation

`CaptureSource::plan_restore_transfer` takes an explicit target, release, extent,
actual-payload ceiling and base management-call ceiling. Target/network and release
must match the authenticated receipt. Extent is 1..=1 MiB, payload ceiling 1..=1 GiB,
and base call ceiling 1..=8192. An invalidated source cannot create a new schedule.
This method reads admitted member lengths only: no filesystem, provider, clock,
signature, serialization or journal mutation occurs.

`RestoreTransfer` retains the exact source receipt and lazily emits source read DTOs.
One shared ordinal mapper owns capture and restore ordering: module, heap, stable,
then chunks sorted by hash. Region extents use the newly selected ceiling and exact
length; empty regions emit nothing. Whole chunks each emit one request even when
empty, and cannot be split to fit a smaller ceiling. Restore uses known actual
chunk lengths, without capture's conservative unknown-length allowance.

For `N` extents/chunks the selected base cost is `2*N + 7`, including upload and
complete readback. Count admission precedes iteration and all retained collections
remain bounded by the already-admitted source profile. This is not network/signing
admission: executor hops, proofs, observations and provider-specific envelope limits
remain separate. Source DTOs identify the original snapshot, not a new destination.

This calculation is not the durable `RestorePlan`: it has no approval commitment,
destination allocation, request digests, spending or source reference. The restore
intent binds this source and the versioned schedule algorithm alongside the actual
connection/application witness before effects. An owned calculation does not keep source files alive;
retain/re-admit the matching `CaptureSource` before consuming its reads.

Focused legacy review inspected frozen `4ae087fa`'s `model/data_stage_plan`,
`ic_snapshot_upload` cases for empty chunks and exact wire identity, the guarded
upload-plan/source readers, and `9f7b886a824eec67dda8f81792a560436419bb2a`'s shared
reader consolidation. Reuse original-source association, fixed ordering, exact
coverage and admission before data-stage work. Do not copy the legacy child-plan
workflow graph, allocation association or retries ahead of a durable restore
contract. Native tests exercise exact region boundaries, zero-length regions and
chunks, sorted chunks, selection/budget refusal and invalidated sources. Existing
capture/readback regressions exercise the shared mapper; the actual-IC fixture reads the original module through the public older source. No restore effect
or physical power-loss qualification is implied; no legacy source was copied.

### Immutable restore approval and original custody

`connection.plan_restore(source, request, options)` binds the actual direct profile
and the authenticated held capture source before approval. `RestoreRequest` retains
independent limits and the same network/canister/release/trust, an explicit sender,
initiator and new operation ID. The original capture may be older than the current
completed owner. No current controller can substitute for the approved restore sender.

The canonical `ic-backup/restore-intent/1` JSON contains format, operation ID,
network/canister/trust, initiator/release/module/sender, source operation ID,
capture approval, trusted manifest, independent limits and direct profile, in that
order. It shares capture's profile/schema bounds and strict canonical decoder.
The source metadata and float bits remain in original certified evidence. Derived
counts and source-request digests are not persisted. The versioned source algorithm
reconstructs the schedule from exact authenticated member lengths.
Approval is `SHA256("ic-backup/restore-intent/1\0" || original_intent_bytes)`.
The independent [JSON vector](../../crates/ic-backup/src/store/restore/restore-intent.json)
and [digest](../../crates/ic-backup/src/store/restore/restore-intent.sha256)
are passive encoding fixtures, not restore authority. The old local-restore-plan/1
reader and exact source/build artifacts are preserved under the
[host-store checkpoint](../development/setup.md#host-store1-reader-before-complete-restore).
No old plan, ledger or status evidence is rewritten under this format.

`create_restore(plan, approval, completed_capture_predecessor)` acquires target,
predecessor, distinct source and new operation locks in that order. The current
owner must be a completed capture; reuse its held source handle when they coincide.
Original source/approval/manifest/bytes and predecessor completion are authenticated
and synchronized before atomic owner replacement. Identical predecessor/source
custody reuses its completed synchronization; initial journal capacity is admitted
before changing ownership. The common recorder then retains
`intent.json`, `journal.bin`, lock and only evidence referenced by original facts.
It creates no bundle, payload copy, dependency index or separate status journal.
Unknown/partial files and occupied IDs remain intact; uncertain publication may
leave an occupied partial operation. No rollback, missing-file completion or reset
is attempted. Future retention must account for original restore source obligations.

`open_restore(id, approval)` is read-only and requires neither a reconstructed
plan nor a signer/connection. A bounded original intent read selects the target
and source; it grants no custody or effect authority. After target/source/operation
locks are acquired, the recorder re-decodes and compares those exact intent bytes,
checks the held target directory/lock identities and authenticates original source
and journal. A different independently valid plan cannot replace the selected
approval. Original spending remains unchanged; unknown/missing/unsafe/oversized
files refuse intact. The recovered `operation.plan()` supplies the retained plan.
Held source descriptors and capture evidence are also rechecked around live
signing/submission awaits; payload reads verify original bounded blocks.

### Fixed restore through the shared recorder

`RestoreOperation::advance` derives exactly initial status, snapshot list,
metadata allocation, N uploads, metadata readback, N byte readbacks, pre-load
status, load and post-load status: `2*N + 7` management calls. Each advance makes
at most one management submission. Pending signing/submission never repeats.
Reply import and explicit observation have the same original-request/time/finite
accounting owner as capture; there is no restore signer/provider family.
[Capture's recorder contract](../architecture/capture-intent.md#files-and-ordering)
owns all call/observation facts, uncertainty, exact evidence, HTTP and authentication.

The source must fit the new approved extent, payload, signing and base network
ceilings. Known payload arguments plus conservative envelope headroom and metadata
encoding must fit the original request ceiling. The complete remaining call,
signing/network/frame/evidence ceiling is checked before allocation. Capacity list
is a refusal aid, not a reservation of remote capacity. Allocation uses
`replace_snapshot = None`; its bounded original reply must yield a fresh nonempty
ID of at most 256 bytes, distinct from inventory and the original source snapshot.
A lost allocation is never inferred from list differences or repeated.

Upload metadata preserves all globals and raw floating bits, explicit timer/hook
and certified data; absent required values refuse without defaults. Ordered uploads
reuse source geometry, including each empty whole chunk. Destination metadata must
identify metadata upload and match all restorable fields and complete chunk hashes;
new timestamp/version are provenance. Every destination byte ordinal must equal
its held original source bytes. A unit upload response alone grants no load authority.
The bounded metadata/data decoders and exact comparison have one shared owner.

Initial, pre-load and post-load status use pairwise distinct original signed
request IDs and require the source module, stopped state
and actual restore sender in a bounded duplicate-free controller set. Guard actions
distinguish allocation, upload, read and load. The integration must maintain the
original continuous witness and admit rewind immediately before loading. It owns
application fencing, external-effect settlement, later acceptance and start.
No stop/start, funding, uninstall, replacement or cleanup fallback exists.

An original confirmed load with missing final status remains completion pending;
load never repeats. After authenticated post-load status, a local `LoadedStopped`
footer freezes completion time and closes this same journal. First restore-call
reservation already includes footer capacity. The returned `LoadedStopped` receipt
is derived from source/approval/destination/time and the closing journal head.
There is no extra receipt file. Cold complete replay authenticates original history
and source without current clock, guard, signer, HTTP or writes. Rejection or
readback/status failure stays durably settled and blocks dependent work.

`create_capture_after(..., OperationPlan::Capture/Restore(predecessor))` admits
either exact completed purpose under target-before-predecessor/source locks.
A completed restore can survive that changed owner only through full original
source/evidence authentication. Missing terminal completion cannot excuse changed
ownership. Restore directly after another restore is not a selected consumer;
new capture after a completed restore is supported.

### Focused restore review and qualification

Reviewed frozen legacy `4ae087fad8c23de712aa60bf569bde4b2b932651`,
`workflow/ic_snapshot_restore/{mod.rs,source/mod.rs,tests/mod.rs}` (`failure_boundaries_retain_pending_or_applied_and_never_repeat_provider`,
`independently_valid_load_plan_cannot_rebind_the_admitted_snapshot_source`),
plus original request/result history `d245b0f03a505ba5215d65c10bdebf1890c7dc37`
and observation/effect split `a860d14421b9ff1c61c1dce079cdc6a19316bfd0`.
Cold reopen additionally re-reviewed legacy source/ancestor re-admission and the
independently-valid-source rebind regression at those exact anchors, plus
`9f7b886a824eec67dda8f81792a560436419bb2a` verification-pass ownership. The new
store selects bounded original intent before locking only to establish lock order,
then re-admits exact bytes/source under the original approval. Native regressions
cover valid replacement plans and lock identities; no legacy stages are copied.
Retain source-before-effect admission, exact allocated identity/readback, retained
result before policy, stopped completion and no repeated uncertain load. Replace
legacy stage/checkpoint orchestration with the two concrete fixed procedures and
one original-evidence recorder. No legacy implementation was copied.

Restore-only request/signature/certificate/HTTP/preparation/submission tests no
longer duplicate the shared direct/runtime failure matrix. Their original-request,
finite reservation, cancellation, orphan, rejection and offline-replay guarantees
are exercised at `store/operation/` and `capture/advance/tests/`. Restore keeps
strict plan/source/custody/transfer cases and fixed-policy allocation, unit-arity,
capacity, metadata-origin, byte mismatch, status/sender, reused status request, rejection and order cases.
Historical status-only encodings remain with their preserved reader. Fake native
calls do not qualify management effects. The retired test-only readback cursor
adds no parallel model: its partial-prefix, source/destination routing, mixed-region
and empty/nonempty-chunk corruption cases now exercise the real fixed procedure.
Original source shape/hash faults stay with the held payload reader, and maximum
wire/data bounds stay with the shared decoder. Unlike the old correctable local
accumulator inputs, a policy-invalid retained original reply permanently blocks
that procedure projection; another reply cannot replace settled evidence.

The public-API actual-IC fixture captures independent module/heap/stable
and empty/nonempty chunks, changes real stable memory, allocates/uploads/readbacks/
loads the old source, checks stopped module and restored stable bytes independently,
then starts a fresh capture after restore. Interrupted allocation, mid-upload and
load retain original certificates; cold local settlement after shutdown/revocation
never signs or submits again. A separate allocation case resolves through one
accounted live observation and then replays offline. Existing take-observation and payload-reply recovery,
source/history and offline terminal replay remain covered. Evidence and failed
fixtures are retained under the repository targets. Full 1 MiB whole chunks use
an explicitly approved 2 MiB request / 4 MiB result profile; original smaller
profiles stay unchanged. Failed load-result writes return the original certificate
for offline import. A separate normal-dependency executable, built without
Testkit/PocketIC, continues capture and restore across processes and replays both
completed operations after backend shutdown. Fixture keys/guards do not
qualify Canic process/maintenance, production HTTPS, hosted macOS or physical power loss.
