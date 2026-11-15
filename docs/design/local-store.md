# Local storage and fleet inventory

Selected v1 scope: local-host bundles, single-canister operations, independent
concurrent targets and read-only backup history. Blob Storage integration is v2.
This is the persistence design for the [operation design](operation-contract.md).
Initialization, opening, bounded manifest inventory and payload byte checks are implemented;
[the inventory contract](../architecture/local-inventory.md) owns their exact bounds.
The [capture-intent contract](../architecture/capture-intent.md) implements immutable
plans, durable target claims and initial status advancement/inspection. Snapshot advancement and [artifact assembly](../architecture/capture-transfer.md#local-artifact-assembly)
are implemented. Final status and local completion publication/reconciliation are implemented.
Explicit completed-capture succession is implemented; standalone full-bundle admission remains proposed. No legacy records or unfinished operations are imported.

## Small public surface

Keep one concrete local store in the existing library. No database, persistent
catalog index, storage-provider trait, background worker or scheduler is required.
Canic calls capture/restore and supplies the execution/application providers.

| Operation | Meaning |
| --- | --- |
| `list_backups(filter, limits)` | Read completion metadata for matching local bundles. No payload hashing or network calls. |
| `latest_completed_backup(network, canister, limits)` | Select the most recently captured completed bundle in this store. |
| `verify_payload(bundle_id, expected_manifest, maximum_payload_bytes)` | Check exact local payload bytes against a trusted manifest; no evidence replay or restore authority. |
| `open_payload(bundle_id, expected_manifest, maximum_payload_bytes)` | Verify and hold original payload files for bounded source reads; complete capture-evidence admission remains separate. |
| `inspect(operation)` | Read retained progress, failure or uncertainty separately from completed backups. |
| `verify_completion(expected_manifest, transport, provider)` | Authenticate an already-published completion and exact source bytes without writes; returns passive evidence. |

List/latest, payload checking and held payload reads exist on `BackupStore`. `CaptureOperation::inspect`
reads original intent; `inspect_status` additionally reads initial status progress,
original admission and reserved attempt counts. Authenticated held capture sources are implemented below; restore intent and durable references remain proposed.
Prefer `latest_completed_backup` over the earlier sketch `latest_verified_backup`:
a cheap inventory cannot establish that files remain intact today. A manifest
declares completion; the reader does not authenticate capture verification or
inspect its ledger. Restore will require fresh complete verification and trusted
source provenance. A local read never establishes application rewind safety.

## Bundle summary and last-backup meaning

A completion manifest contains the following inventory fields alongside its
content and original-operation commitments:

- Bundle/capture-operation identity; canonical network instance and canister ID.
- Source snapshot identity and capture timestamp from authenticated metadata.
- Local verification-completion timestamp, explicitly an operator-host clock.
- Release binding and module hash; exact payload byte total.
- Exact payload member lengths/hashes, original metadata evidence and final
  capture-ledger commitment. Paths are product-generated relative names.

For one network/canister, rank by capture timestamp, then completion timestamp,
then opaque bundle ID for deterministic ties. Report both timestamps; they answer
when the state was captured and when its local backup finished. Host clock
rollback must not alter original values or replenish authority. Timestamp ordering
is descriptive and supplies no management-call attribution.

Return no backup only after a successful bounded scan finds no matching completion.
Never turn corrupt records, unsupported formats, unreadable entries or exceeded
scan limits into “never backed up”, or silently fall back to an older backup.
Other canisters' malformed metadata also prevents a claimed complete store scan.
Keep incomplete/failed operations in inspection results; they do not compete for
latest successful backup. No stable-memory content is read for listing.

Inventory is a scan of immutable published manifests, not a transactional snapshot
of the whole fleet. Completion published during a scan may appear on the next
refresh. A bundle handle must be admitted again before use. Canic owns display
names and joins rows with its live fleet selection; Backup never discovers fleet
membership. Results cover this store only, not other hosts or remote snapshots.

Callers supply positive entry, cumulative-byte and returned-row limits. The scanner
bounds work while traversing, before allocating a complete file or collecting the
entire store. A limit refusal returns no authoritative latest/absence answer.
The current API requires explicit limits and has no defaults. Latest-only scans
retain one candidate; list scans also enforce the requested row count. No paging
protocol or persistent index is introduced for an unmeasured scale requirement.

## One layout, one spending authority

```text
store/
  format
  targets/<network-key>/<canister>/
    lock
    owner.json
  operations/<operation-id>/
    lock
    plan.json
    ledger.bin
    evidence/
  bundles/<capture-operation-id>/
    manifest.json
    payload/
    references/<restore-operation-id>.json
```

`format` identifies the restart store, not legacy v1. Operation IDs are validated
opaque names; network keys are domain-separated commitments to the exact network
binding. Read retained bindings as well as the path key. No remote value becomes
an arbitrary path. Unknown entries or unsafe file kinds at owned boundaries
refuse; this is a private product store, not a directory import feature.

Use Host's held directories, no-follow regular files, nonblocking locks and durable
create-only publication. Private directories/files use 0700/0600. Host primitives
own OS mechanics; Backup owns record meaning, ordering and recovery. Initialization
is explicit local IO into an existing empty private directory. The implemented
initializer retains `format.pending` before publishing `format`. Interrupted
initialization remains occupied evidence and must not be silently treated as a
new empty store. Reading never initializes it.

The plan is immutable. Only the ledger consumes attempts and settles their results.
Large arguments, envelopes, certificates and replies are create-only evidence
files referenced by length and hash. The manifest closes the payload member set;
references are outside `payload/` and do not change its content commitment.
No separate stage journals, learned-edge files or settlement catalog are added.

Use versioned typed JSON records with unknown/duplicate-field refusal and bounded
collections. Store float bits as fixed-width hexadecimal values, never JSON floats.
Normalize identities before encoding, serialize each record once and retain/hash
those exact bytes with a distinct restart domain. Public plan approval commits to
that original encoding. Reopen verifies original bytes; it never pretty-prints or
regenerates identities. Exact field lists and golden vectors belong with the first
writer/reader implementation, before it can authorize remote effects.

For the ledger, select length-delimited frames: a little-endian u32 payload length,
a bounded JSON transition, and a 32-byte SHA-256 chain digest. The initial digest
binds the original plan bytes under the restart domain; subsequent digests bind
the previous digest, encoded length and exact payload. Each transition has a
monotonic sequence and closed event kind. No general event-bus framework is needed.
The implemented framing reader enforces 16 KiB per frame, 65,536 frames and
128 MiB total, with the original plan's evidence allowance as a further byte cap.
All apply independently; plans and manifests remain separately byte-bounded.
Created and initial status reservation/preparation/submission/settlement transitions
are admitted. These are format limits, not measured product capacity claims.

The initial status workflow validates the full bounded ledger on reopen. A held
session admits its own append prefix; it does not repeatedly scan peer operations.
Partial frames, bad hashes, sequence gaps and invalid transitions block effects.
Keep the original bytes; no truncate-and-retry recovery or log compaction in v1.
Hashes detect corruption but do not defeat a malicious host owner or rollback of
all local evidence. The integration must maintain the stated private-store custody.

## Completion and interrupted publication

`CaptureOperation::complete_capture` is implemented local completion and
reconciliation. It requires an already-settled successful final status; it cannot
sign, submit, poll or settle a missing remote result. It reauthenticates original
capture/payload/final evidence and synchronizes the verified assembly. A retained
lookup must have recorded original-call settlement; unresolved or unused lookup
lanes currently refuse completion. Canic still owns continuous maintenance and
external-effect safety; completion does not rewind obligations.

`PublishReady` is the final ledger event. Its canonical body is
`{"kind":"publish_ready","completed_at_ns":N}` with positive u64 host Unix
nanoseconds, selected once. The existing prefix already binds the immutable plan,
source metadata, exact accepted payloads, verified assembly and final status.
No summary duplicate or separate completion journal is persisted. The original
evidence budget reserves one maximum-size frame and the 1 MiB manifest ceiling
before this event. Every subsequent ledger event refuses, including a second
`PublishReady`. Reopening never invents a new timestamp or allowance.

The existing `ic-backup/local-manifest/1` schema owns serialization and validation.
The writer uses ASCII-sorted members, the original metadata snapshot ID/timestamp,
plan release/module identity and actual payload lengths/hashes. Plan, closed
ledger, metadata arguments and metadata reply commitments are raw SHA-256 of
those exact original bytes, with exact lengths. Metadata arguments come from the
retained signed envelope and the reply from retained certified evidence; neither
is re-encoded. The manifest binds the entire closed ledger, including
`PublishReady`. There is no circular hash or post-publication journal event.
The existing golden schema and exact original-byte regressions cover the writer.

The fixed bundle already holds `payload/`. Create `manifest.pending` exclusively
with private permissions, write/synchronize it, verify its exact bytes and held
identity, then use Host's no-replace publisher to rename it to `manifest.json`
and synchronize the directory. The manifest is the sole completion receipt;
no catalog row, extra completion marker, second payload copy or cleanup is added.
All fallible publication invalidates the live handle until closing custody and
original-record admission succeed. A failure preserves every named file.

On local reconciliation, reauthenticate the original evidence and derive only the
same v1 manifest from the frozen prefix. Missing manifest/staging after readiness
can be created; exact complete staging can be published; an exact visible manifest
can be synchronized and returned. Partial, changed, unsafe, aliased, semantically
equivalent-but-differently-encoded, or simultaneously staged/published files refuse
without replacement or cleanup. A partial ledger tail still refuses reopen.
Local reconciliation may synchronize or publish retained local files; it performs
no remote effect and appends no new event after readiness. Read-only inventory
and inspection remain separate from durability reconciliation.

Inventory may observe a manifest whose publisher lost its return; visibility alone
is not proof of that earlier invocation's success. `complete_capture` returns the
original `BackupSummary`, including its exact manifest hash, after local admission
and synchronization. Existing list/latest and payload verification consume that
same manifest. Standalone complete-bundle admission for restore is still separate.

Legacy review revisited frozen artifact-writer finish/publication and tests for
changed closing custody and acknowledged process death at
`4ae087fad8c23de712aa60bf569bde4b2b932651`. The
`29bf7a51044e72c4bd8cf6dc5cd6cb24d3711b4d` history removed an unnecessary downloaded
stage. Reuse exact retained-byte checks, fail-closed partial recovery and the Host
publisher; do not import the old layout or add another persisted lifecycle stage.
Native tests exercise full/partial readiness frames and publication crash images.
The small real IC journey now reaches manifest publication and existing inventory,
then reconciles after gateway shutdown and revoked live authority. Physical power
loss, maximum-size throughput and hosted macOS remain unqualified.

Target claims remain retained after completion. Explicit `create_capture_after`
authenticates the completed predecessor and replaces its claim to create a new
capture of the same target. Restore has no new bundle; its terminal state, source references and load
qualification remain proposed below. No pruning or remote deletion is implied.

## Concurrency and target ownership

Canic owns the bounded task pool. Each Backup operation is sequential; distinct
network/canister targets can progress concurrently without a store-wide effect
lock. There is no Backup batch plan or claim of consistent fleet capture.

Creation takes the target lock first, then source-bundle custody for restore,
then operation custody. Ordinary advancement holds only its admitted source and
operation locks and checks its retained owner binding. Never acquire the target
lock while holding an operation in the reverse order.

The durable owner remains after completion or process death. An occupied unfinished
or uncertain owner refuses new work. Explicit succession takes the target lock
then the predecessor operation lock nonblockingly, admits its exact published
completion and reauthenticates its retained evidence before replacing the owner. Preserve the prior operation and bundle. Never
remove/replace lock files: those names must continue to identify the same lock.
Read-only terminal admission does not need a current target claim. Explicit local
completion reconciliation may synchronize original files, as described below.

An authenticated failure is not automatically terminal: local closure must prove
no unresolved call or dependent continuation remains. A new owner never refunds
or resets an old operation's authority. Separate stores/hosts remain coordinated
by application maintenance custody, not these local locks.

## Implementation checks

Further local-store implementation must exercise interrupted manifest publication,
corrupt/truncated records, changed paths, target contention and stale owner records.
Inventory cases cover multiple networks/targets, clock ties, unfinished work,
unsupported records, bounded scans and completions appearing during enumeration.
Exact golden bytes and independent digest vectors accompany the format writer.
Real capture/restore still requires the production transport and Testkit journeys
in the operation contract; passing local-store tests cannot substitute for them.

Blob upload/download, remote authentication, funding and retention remain v2
integration work. Add no unused dependency or generic storage abstraction now.

### Completed receipt admission

The implemented admission path now distinguishes an exact published receipt from
readiness or staged publication. Only the former can outlive its original owner
binding. Original plan, closed ledger, retained evidence and payload still pass
their existing checks. Missing, changed or staged-only manifests require the
original owner; they cannot create a receipt under a successor's claim. Reopen
is read-only. Explicit completion reconciliation can still synchronize original
files, but cannot sign, submit, append events or alter the successor's owner.
Explicit claim replacement is described below.

The focused legacy review revisited frozen `4ae087fa`'s
`model/attempt_journal::applied_receipt`, its
`applied_receipt_identifies_direct_or_observation_evidence_after_replay` regression,
and `a860d144`'s original-plan ownership consolidation. The retained lesson is
that an original receipt does not grant new dispatch or release authority. The
new store uses its existing manifest/closed-journal checks; it imports neither
legacy observation graphs nor an independent terminal marker. New native cases
exercise changed-owner replay and unfinished, missing, staged and changed receipt
refusals. Synthetic owner replacement qualifies admission, not atomic succession
or physical power loss. No legacy source was copied.

### Explicit completed-capture succession

`BackupStore::create_capture_after` takes a new plan/approval, the exact predecessor
plan and its historical transport/provider. It requires the same target and a
fresh operation ID. Initial journal capacity is admitted before writes. Under
target-then-predecessor locks, it requires the current owner to match the original
plan and an already-published exact receipt. It reuses `complete_capture` for
historical authentication and durability reconciliation; it cannot finish an
unpublished predecessor or issue a new remote call.

Host FS's held-directory `write_at_validated_with` atomically replaces only the
small owner pointer, with private permissions and parent synchronization. Host owns
its temporary projection and before/after-publication errors; an interruption can
retain staging, which normal strict directory admission refuses without cleanup.
The validated callback checks the original owner and held directory custody while
both locks remain held. The same initial-intent writer then creates the successor.
Original operation, bundle, receipt and lock files are never removed or replaced.
There is no new on-disk schema, handoff journal or generic transaction framework.

This is not a multi-file transaction. A failure after owner publication may leave
the successor absent or partially created. Preserve its claim and any files; only
complete original intent can reopen. No automatic completion of partial creation,
claim rollback or blind replacement retry is provided. A completed predecessor
can still be admitted under the replacement claim. A new capture starts with its
own finite plan, zero reserved calls and independently required live authority.

Focused legacy review inspected frozen `4ae087fa`'s `ops/persistence/journal_lock`
and its exclusion, retained-lock and unsafe-substitution regressions, plus the
`a860d144` ownership consolidation history. Reuse nonblocking exclusion and stable
lock identities; legacy pathname lock creation/cleanup does not own new store
claims. Native regressions cover lock contention, original receipts, initial
budget admission, changed/unfinished/staged predecessors and incomplete successor
images. The real-IC completion fixture now succeeds through succession after
server shutdown with revoked live permission, then replays the original receipt
without extra signatures. This does not qualify physical power loss, maximum-size
throughput or full second-capture effects. No legacy source was copied.

### Read-only completion verification

`CaptureOperation::verify_completion(expected_manifest, transport, provider)`
requires original operation custody and an already-published exact manifest. It
checks the selected hash before reauthenticating original capture, metadata,
accepted data and final-status evidence. The shared assembly reader compares every
member with its certified source bytes and rechecks the tree commitment; its
private verification branch returns before creation, synchronization or publication.
The operation's plan bounds still cap evidence, payload and individual reads.

Completion publication and verification share authentication and manifest decoding.
The verifier cannot finish staging, choose a timestamp or append an event. The
returned `BackupSummary` is passive evidence: a future restore still needs an exact
same-release/target plan, held payload/source custody and durable source references.
A previous verification does not make future reads safe or create live permission.

The focused review inspected frozen `4ae087fa`'s
`ops/persistence/download_journal/ic_snapshot_artifact/verification`, its exact
length/hash and replaced-file regressions, and `9f7b886a824eec67dda8f81792a560436419bb2a`'s
shared verification/upload-reader change. Reuse one owner for exact original-byte
checks and retain admission/closing custody; do not import legacy artifact layouts
or upload authority. Native cases cover expected-hash mismatch, changed member
bytes/length, staged/missing publication, and replay after owner change. The real-IC
fixture verifies before and after succession with the gateway stopped and live
permission revoked. No legacy source was copied; physical power loss and practical
maximum-size throughput remain unqualified.

### Authenticated held capture sources

`BackupStore::open_capture_source(operation, expected_receipt, maximum_payload_bytes,
transport, provider)` composes completion verification with the existing held
payload reader. The selected exact `BackupSummary` binds the original operation,
network, canister, release, module, snapshot, timestamps and manifest. The operation
must belong to the same held store; identical names in another store are insufficient.
Payload admission applies the caller's total byte limit before certificate replay.
Closing checks retain the original payload descriptors through authentication.

`CaptureSource` borrows the capture operation and store: the original operation
lock cannot be dropped and its workflow cannot advance while the source is used.
It exposes only the selected summary and existing bounded `read_data` behavior.
Reads verify at most two original 1 MiB blocks and return at most 1 MiB, with the
existing manifest/directory/file checks. They do not rescan the whole journal or
reauthenticate certificates per extent. Invalid requests leave the reader usable;
IO, integrity or custody refusal invalidates reads. External namespace/content
writers remain excluded by the caller throughout the handle's lifetime.

This is ephemeral source custody, not a restore plan, target claim or durable
source reference. Original capture evidence is authenticated at admission; no
future restore permission, immutable-host guarantee or upload schedule is inferred.
The eventual restore workflow must acquire target custody before source-operation
custody and persist its exact source reference and bounded intent before effects.
No new stored record, hash scheme, block cache, reader or dependency is introduced.

The focused legacy review inspected frozen `4ae087fa`'s
`ops/persistence/download_journal/ic_snapshot_artifact/upload/plan/source`, its
`streaming_bindings_hash_exact_consumed_bytes_once_and_reject_short_or_excess_input`
regression and `9f7b886a824eec67dda8f81792a560436419bb2a`'s shared reader consolidation.
The reused lesson is to retain original custody and derive reads from the same
verified bytes. The existing payload block reader owns that behavior here; legacy
upload plans and destination bindings are not copied ahead of restore planning.
Native source tests cover original-lock retention, wrong selections/stores/budgets,
invalid requests and changed bytes. Existing bounded/cross-block reader regressions
are rerun. The real-IC fixture reads the authenticated source offline before and
after target succession. This does not qualify restore effects or physical power
loss. No legacy source was copied.
