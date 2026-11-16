# Implemented capture transfer validation

The fixed capture procedure privately owns bounded status/list/take/metadata/data
decoding and deterministic geometry. These are algorithms used by the public
`CaptureOperation::advance` loop, not public phase commands or new authority.
See the [intent contract](capture-intent.md) for signing, spending and recovery.

## Snapshot list and take preflight

The Platform-backed list decoder preserves original wire order and exact opaque
IDs, timestamps and sizes. IDs are nonempty, at most 256 bytes and unique; at most
1024 snapshots are admitted. Remote reported size is not local payload length.

A take requires room under original approved capacity 1..=1024 and encodes
`replace_snapshot = None`, `uninstall_code = Some(false)` and no sender version.
It never replaces/deletes existing snapshots or uninstalls code. The authenticated
take ID must be absent from original list; a later list difference cannot settle
an uncertain take. The original request certificate owns attribution.

Both reply decoders admit at most 1 MiB raw bytes, 64 KiB header, 16 type entries,
2 MiB work/allocation each and zero skipped work. Unknown/missing/wrong fields,
excess arguments and trailing bytes refuse. Count/ID limits precede allocation.

## Admitted profile and schedule

The metadata source must be a canister snapshot, with nonempty module, timer/hook
values and available globals. Upload provenance, missing round-trip values and
unrepresentable v128 refuse; values are never defaulted. Maximums are 4096 globals,
32 certified-data bytes and 1024 unique 32-byte chunk hashes.
The take timestamp must match authenticated metadata.

Metadata preserves source/version, original chunk order and exact float bits.
A separate bounded sorted hash index provides deterministic chunk order.
For original extent ceiling E, read module, heap and stable memory at ascending
offsets with sizes min(E, remaining), followed by whole chunks in ascending hash
order. Empty regions emit nothing. A named empty chunk still gets one read and
hash verification.

Checked region totals plus chunk_count * E must fit original payload allowance.
Even a known empty hash reserves E before its actual reply is known. Each actual
chunk must fit E. Let N be region extent counts plus chunks; the complete direct
capture requires N + 5 management calls including both status calls, list, take
and metadata. Signer-network costs and evidence ceilings are admitted separately
by the common recorder; planning lower bounds do not grant spending.

## Bounded metadata replies

Metadata decoding uses Platform schemas: raw ceiling 1 MiB, header 16 KiB, 64 type
entries, 2 MiB work/allocation each and zero skipped work. Counts/byte lengths are
bounded before allocation. Known optional promotion/null/reserved/missing values
retain their meaning, then selected capture policy refuses unavailable state.
Unknown fields/optional variants and non-reserved source payload refuse.
Independent frozen vectors exercise type ordering, exact float bits and optional
meanings. No replacement management DTO or generic codec framework is introduced.

## Bounded data replies

Platform-backed data decoding admits a single exact argument: raw ceiling 2 MiB,
header 4 KiB, 16 type entries, 8 MiB codec work, 2 MiB allocation, 1 MiB vector
ceiling and zero skipped work. Exact region size and whole-chunk SHA-256 are checked
after bounded decoding. Empty chunks remain explicit. These are local codec
counters, independent of complete certificate/result limits.

Every original request and certified result is retained before policy evaluation.
Semantic replay authenticates exact target/snapshot/method/arguments, deterministic
ordinal and complete coverage. Same-length unauthenticated bytes are insufficient.
The combined module hash must match the original approval. There is no separate
transfer binding, data-accepted event, assembly-verified event or schedule file.

## Local artifact assembly

After all payload evidence passes, the procedure records MaterializationStarted.
It writes exact module/heap/stable/chunk files directly into the unpublished
payload directory under original local allowance. Original certified evidence can
create missing members. Exact complete members may be reused; partial, altered,
unknown or unsafe members refuse intact. No cleanup or repair is inferred.

Full module/coverage/payload verification precedes the distinct final status.
PublishReady freezes the timestamp and closed original journal commitment.
The manifest closes the exact member set and commits original intent, metadata
evidence and journal. It is published create-only. Exact complete staging can
reconcile locally; partial/changed staging refuses. Read-only published replay
does no synchronization or publication.

## Restore source scheduling

`open_capture_source` authenticates original completion and holds same-store
capture/payload custody. Its `read_data` checks exact original target/snapshot
and bounded geometry against retained source descriptors.

`plan_restore_transfer` derives same-network/canister/release requests using a new
positive extent limit and exact known chunk lengths. Its base management count
is 2*N + 7, including upload/readback, metadata, load and status checks.
The [source-bound restore](../design/local-store.md#fixed-restore-through-the-shared-recorder)
consumes this geometry through the common call recorder. Exact metadata comparison
is now production code; bounded destination data decoding is shared with capture.
The obsolete standalone readback accumulator and its fixture model are retired.
Their distinct coverage, destination routing, per-region/chunk corruption and
partial-prefix cases exercise the real restore cursor; bounded decoder and source
integrity cases remain at their existing owners. No second readback journal or
payload copy is added.

## Focused legacy review — 2026-11-15

Frozen source: `4ae087fad8c23de712aa60bf569bde4b2b932651`.

| Source/test anchors | Guarantee retained and current owner |
| --- | --- |
| `model/ic_snapshot_data/{mod.rs,tests/mod.rs,extents/tests.rs}` | Checked counts/ranges, empty regions versus named empty chunks, exact IDs and whole chunks; transfer/procedure tests |
| `model/ic_snapshot_metadata/{mod.rs,wire/mod.rs,tests/mod.rs}` | Optional meaning, NaN/signed-zero bits, bounded globals/chunks/data/v128; metadata tests |
| `model/ic_snapshot_reply/{mod.rs,wire/mod.rs,tests/mod.rs}` | Bounded exact list/take IDs and no reused take ID; snapshot tests |
| `model/ic_snapshot_coverage/tests/mod.rs` | Complete extent/chunk coverage and module identity; fixed-procedure tests |
| `model/ic_snapshot_upload/{mod.rs,tests/mod.rs}` | Source/destination distinction and unavailable-state refusal; production metadata tests and actual restore-policy cases |
| `ops/persistence/download_journal/ic_snapshot_artifact/{mod.rs,tests/mod.rs}` | Closed payload tree, corruption/empty-chunk and interrupted completion; recorder custody/runtime tests |

History `a860d14421b9ff1c61c1dce079cdc6a19316bfd0` adds extreme sizes,
exact allowances and empty-region/chunk cases.
Codec migration `1af7e219c3073395992c316ecaa5e83e74b1aecc` preserves separate
work/allocation/skipped-field bounds while using Platform.

The new planner is lazy and uses one fixed order, a separate sorted chunk index
and original finite intent. Empty module and missing lifecycle/global values are
outside the selected round-trip profile; legacy could retain them as evidence.
The old Canic configuration-only route is removed pending its actual executor
contract. Original raw metadata/snapshot golden vectors and hashes remain beside
their owning tests with provenance; no legacy implementation was copied.

Repeated phase signing/submission/recovery matrices now live once at
`operation/runtime/tests.rs`; phase-specific geometry, bounded parsing,
metadata bits and exact policy cases remain with their algorithms.
The actual [IC consumer](../../qualification/ic-backup-qualification/tests/capture.rs)
uses 16 KiB extents over a nontrivial module, 64 KiB heap, 128 KiB stable memory
and both empty/nonempty chunks. It independently compares payload bytes and
qualifies interrupted original data settlement; this is fresh IC evidence.
Native synthetic-certificate and pure readback tests do not qualify restore effects.
