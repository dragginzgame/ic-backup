# Local storage and fleet inventory

Selected v1 scope: local-host bundles, single-canister operations, independent
concurrent targets and read-only backup history. Blob Storage integration is v2.
This is the proposed persistence contract for the [operation design](operation-contract.md),
not an implemented format. It replaces the earlier conceptual storage layout;
no legacy records or unfinished operations are imported.

## Small public surface

Keep one concrete local store in the existing library. No database, persistent
catalog index, storage-provider trait, background worker or scheduler is required.
Canic calls capture/restore and supplies the execution/application providers.

| Operation | Meaning |
| --- | --- |
| `list_backups(filter, limits)` | Read completion metadata for matching local bundles. No payload hashing or network calls. |
| `latest_completed_backup(network, canister, limits)` | Select the most recently captured completed bundle in this store. |
| `inspect(operation)` | Read retained progress, failure or uncertainty separately from completed backups. |
| `verify_bundle(bundle)` | Rehash and admit the complete original artifact under held custody before restore. |

These names are API proposals. Prefer `latest_completed_backup` over the earlier
sketch `latest_verified_backup`: a cheap inventory cannot establish that files
remain intact today. A listed completion means verification succeeded at capture;
restore still requires fresh complete verification and trusted source provenance.
A successful local read never establishes application rewind safety.

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
The implementation must select measured default limits; no paging protocol or
persistent index is introduced for an unmeasured scale requirement.

## One layout, one spending authority

```text
store/
  format
  targets/<network-key>/<canister>/
    lock
    owner.json
  operations/<operation-id>/
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
own OS mechanics; Backup owns record meaning, ordering and recovery. Store creation
is explicit local IO. Interrupted initialization remains occupied evidence and
must not be silently treated as a new empty store. Reading never initializes it.

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
Proposed admission bounds are 16 KiB per frame, 65,536 frames and 128 MiB total;
all apply independently. Plans and manifests remain separately byte-bounded.
These are format-design limits, not measured product capacity claims.

Reopen validates the full bounded ledger once under exclusive custody. A held
session admits its own append prefix; it does not repeatedly scan peer operations.
Partial frames, bad hashes, sequence gaps and invalid transitions block effects.
Keep the original bytes; no truncate-and-retry recovery or log compaction in v1.
Hashes detect corruption but do not defeat a malicious host owner or rollback of
all local evidence. The integration must maintain the stated private-store custody.

## Completion and interrupted publication

For capture, the final ledger transition is `PublishReady`: all required calls
are settled, final stopped-state evidence and complete verified payload commitments
are durable, and the original completion timestamp is retained. No remote work
follows it. It also binds the fields from which the manifest is generated.

Publish the exact manifest create-only after syncing payload and evidence files.
The manifest binds the final ledger length/digest. There is no circular hash:
`PublishReady` binds content/summary fields, while the manifest additionally binds
the completed ledger prefix. Do not add another catalog row or completion marker.
The durably published manifest is the capture completion receipt.

A crash before manifest publication leaves an incomplete capture. A crash after
publication but before its directory sync or return requires local reconciliation:
verify the retained prefix, manifest and payload commitments, synchronize original
files/directories, then return the original receipt. Never recapture or invent a
new completion timestamp. Mere visibility is not proof that the prior call returned
success. Inventory may report such a visible completion declaration; only held
operation admission establishes durable replay or permission to reuse its target.

Restore has no new bundle. Its terminal `LoadedStopped` ledger transition binds
settled load and final status evidence. Source references remain on success and
failure. Create/sync the exact source reference before a restore can dispatch;
a missing reference on reopen refuses effects. No reference release, pruning or
remote snapshot deletion is implied by local completion.

## Concurrency and target ownership

Canic owns the bounded task pool. Each Backup operation is sequential; distinct
network/canister targets can progress concurrently without a store-wide effect
lock. There is no Backup batch plan or claim of consistent fleet capture.

Creation takes the target lock first, then source-bundle custody for restore,
then operation custody. Ordinary advancement holds only its admitted source and
operation locks and checks its retained owner binding. Never acquire the target
lock while holding an operation in the reverse order.

The durable owner record remains after process death. An occupied unfinished or
uncertain owner refuses new work. A new operation may replace that owner only under
the target lock after admitting the prior exact terminal evidence and acquiring
its operation lock nonblockingly. Preserve the prior operation and bundle. Never
remove/replace lock files: those names must continue to identify the same lock.
Terminal replay does not need a current target claim and performs no effects.

An authenticated failure is not automatically terminal: local closure must prove
no unresolved call or dependent continuation remains. A new owner never refunds
or resets an old operation's authority. Separate stores/hosts remain coordinated
by application maintenance custody, not these local locks.

## Implementation checks

The first local-store slice must exercise interrupted manifest publication,
corrupt/truncated records, changed paths, target contention and stale owner records.
Inventory cases cover multiple networks/targets, clock ties, unfinished work,
unsupported records, bounded scans and completions appearing during enumeration.
Exact golden bytes and independent digest vectors accompany the format writer.
Real capture/restore still requires the production transport and Testkit journeys
in the operation contract; passing local-store tests cannot substitute for them.

Blob upload/download, remote authentication, funding and retention remain v2
integration work. Add no unused dependency or generic storage abstraction now.
