# Implemented capture transfer validation

`CapturePlan::plan_transfer(snapshot_id, metadata)` produces a borrowed
`CaptureTransfer` using Platform's `ReadCanisterSnapshotMetadataResult`. It is
pure: no filesystem access, reservation, signer or network call. Its lazy
`requests()` iterator produces Platform `ReadCanisterSnapshotDataArgs`, each
bound to the original plan's canister and exact supplied snapshot ID.

Use `CapturePlan::decode_metadata(snapshot_id, raw_reply)` for bounded raw
metadata admission. The caller must still authenticate its association with the
original target/snapshot. Planning does not decode or authenticate
wire bytes. It neither commits a learned schedule to the ledger nor grants
permission to dispatch. Application custody, provider admission, request evidence
and durable accounting remain prerequisites to the effectful workflow.

## Snapshot list and take preflight

`decode_snapshot_list(raw_reply)` returns Platform `Snapshot` values in original
wire order. IDs are opaque nonempty byte strings; duplicates refuse even when
metadata differs. Timestamps and reported sizes retain their full u64 range.
Reported remote size is not the local payload length and is not compared with the
plan's payload allowance. Timestamps alone prove neither freshness nor attribution.

`CapturePlan::take_snapshot_arguments(inventory, capacity)` admits a direct-host
request for the exact plan target, with `replace_snapshot = None`,
`uninstall_code = Some(false)` and `sender_canister_version = None`. The integration
explicitly supplies capacity within 1..=1,024; this is not an IC quota discovered
or guaranteed by Backup. The bounded inventory must have room below that capacity.
The later workflow must bind capacity and authenticated inventory before spending;
a preflight does not reserve remote capacity. Canic execution refuses pending its
sender-version/receipt contract. This helper does not check status or application
safety and grants no signing, submission, deletion, funding or retry authority.

`decode_taken_snapshot(raw_reply, prior_inventory)` admits the exact single take
result only if its ID was absent from the validated prior inventory. It never
infers capture success from a later list difference. An uncertain take still needs
original-request reconciliation; a changed list supplies no retry authority.
Callers retain and authenticate both replies and their original network, target
and request association. These helpers perform no IO, persistence or settlement.

Both decoders use Platform's schema and owned codec: 1 MiB raw reply, 64 KiB header,
16 type entries, 2 MiB work and 2 MiB allocation, with zero skipped-work allowance.
The bounded vector reader admits at most 1,024 snapshots and 256 bytes per ID
before allocation; empty IDs refuse. Unknown/missing fields, wrong types, extra
arguments and trailing bytes refuse. These counters are local codec limits, not
transport limits or upstream Candid quota units. Typed inventory inputs receive
the same count, ID-length and duplicate checks.

## Admitted profile and schedule

The snapshot ID has 1–256 bytes. Metadata must identify a snapshot taken from a
canister, a nonempty module, present timer/hook values and no missing globals.
Metadata from an upload, absent origin/lifecycle information and unavailable
globals return `UnsupportedProfile`; these valid external states are not defaulted.
Globals are limited to 4,096 entries, certified data to 32 bytes and chunks to
1,024 unique 32-byte hashes. A v128 global must fit 128 bits.

Metadata stays immutably borrowed, including chunk wire order, source timestamp,
canister version and exact floating-point bits. There is no JSON conversion or
float comparison. The schedule retains only a bounded sorted chunk-hash list and
region counts. Debug output contains counts, not metadata, snapshot IDs or bytes.

For the immutable original extent ceiling `E`, emit module, heap, then stable
memory, each with ascending offsets and sizes `min(E, remaining)`. Empty heap
or stable memory emits nothing. After the regions, emit each whole stored chunk
once in ascending hash-byte order. A chunk read has no offset/size parameter.
A named empty chunk still requires a read and hash verification.

Checked region-byte sums plus `chunk_count * E` must fit the original payload
allowance. Actual chunk sizes are unknown at this stage. Even the empty-data hash
reserves `E`; recognizing it does not bypass this conservative rule. The data decoder enforces that each actual chunk fits `E`.

Let `N = ceil(module/E) + ceil(heap/E) + ceil(stable/E) + chunk_count`.
Whole-capture management calls are `N + 5`: initial status, list, take, metadata,
data reads and final status. A direct route requires at least `N + 5` network
requests; a Canic executor requires at least twice that count for the two hops.
Both require at least `N + 5` signing preparations in this per-call preparation
contract. These minimum costs must fit the original respective allowances before
any request DTOs are generated. Original observation limits are not extra budget.

These are lower bounds, not full route admission. Authentication, polling,
executor receipts and evidence envelopes may require additional resources.
The helper cannot establish those costs or remaining budget after previous calls.
No claim of actual Canic execution follows from passing its cost calculation.

## Bounded metadata replies

`CapturePlan::decode_metadata(snapshot_id, raw_reply)` returns Platform metadata
only after bounded decoding and the existing transfer admission succeed. It
preserves wire chunk order, timestamps, versions and float bits. It does not
retain a schedule; borrow the returned metadata through `plan_transfer` to inspect
requests and validate data replies. Neither method spends attempts or persists
progress. Successful parsing does not authenticate the target/snapshot association.

Metadata limits are 1 MiB raw bytes, 16 KiB header, 64 type entries, 2 MiB codec
work and 2 MiB codec allocation, with a separate zero skipped-work allowance.
Platform's bounded reader checks vector counts before allocation: 4,096 globals,
1,024 chunks, 32 certified-data bytes and 32 bytes per chunk hash. Original transfer
admission then checks exact hash lengths/uniqueness, v128 range, the capture profile
and plan budgets. These are owned codec counters, not transport limits.

Known bare-value option promotion, null/reserved absence and missing optional
fields retain their meaning. The capture profile rejects unavailable round-trip
values; it does not invent defaults. Unknown fields/optional variants and any
non-reserved source payload refuse, including otherwise zero-byte content.

The private field mapping uses Platform's DTO schemas and labels without a second
management declaration. Its field positions follow the reviewed supplier schema;
retained independent wire fixtures check every field, optional promotion and float
bits. Platform owns binary parsing, type traversal and work/allocation accounting.
No generic codec framework, transport or dependency was added.

## Bounded data replies

`CaptureTransfer::decode_data(ordinal, raw_reply)` decodes one scheduled data
reply using Platform's owned Candid codec and `ReadCanisterSnapshotDataResult`
schema. Ordinals are zero-based within the data schedule. Region bytes must match
the requested size exactly. Whole chunks must fit the original extent ceiling
and match their metadata SHA-256, including empty chunks. Typed refusals retain
Platform codec errors without retaining or printing raw payloads.

The raw reply ceiling is 2 MiB; header 4 KiB; type entries 16; codec work 8 MiB;
allocation 2 MiB; and the vector element ceiling 1 MiB. The vector ceiling is
checked before payload allocation. Exact smaller extent/region constraints are
checked after bounded decoding. Skipped work has a separate zero allowance:
unknown fields, including zero-byte nulls, refuse. Exact single-argument decoding
refuses trailing bytes. These owned codec counters are not upstream Candid quota
units or transport-level response limits.

The result contains only decoded bytes. The caller must retain raw evidence and
authenticate its association with the exact target, snapshot and request. Valid
same-length region bytes do not establish authenticity; chunk hashes are only as
trusted as their metadata. Decoding performs no IO, reservation, progress update,
settlement or complete-coverage check. Effectful capture remains unimplemented.

## Focused legacy review — 2026-11-14

Reviewed frozen legacy `4ae087fad8c23de712aa60bf569bde4b2b932651`:

- [Extent planning](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_snapshot_data/extents/mod.rs)
  admits checked counts against original allowance before allocating extents.
  [Its tests](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_snapshot_data/extents/tests.rs)
  distinguish empty regions from named empty chunks and exercise extreme nat64.
- [Data admission](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_snapshot_data/mod.rs)
  preserves exact target/snapshot association, checked ranges and whole chunks;
  [regressions](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_snapshot_data/tests/mod.rs)
  include empty chunks, wrong metadata and maximum ranges.
- [Metadata admission and tests](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_snapshot_metadata/tests/mod.rs)
  preserve optional meanings, NaN/signed-zero bits and source provenance while
  bounding globals, chunks, certified data and v128 values.
- [History a860d144](https://github.com/dragginzgame/ic-backup-legacy/commit/a860d14421b9ff1c61c1dce079cdc6a19316bfd0)
  added the extreme-size, exact-allowance and empty-region/chunk extent tests.
  Its lesson is count admission before allocation, not reuse of the stage graph.

Re-authored [eleven tests](../../crates/ic-backup/src/store/capture/transfer/tests.rs)
cover coverage/order, exact IDs, conservative bytes, count limits, both routes,
overflow, chunk identities and metadata restrictions. The new implementation
emits requests lazily instead of materializing all extents, sorts a separate chunk
index and includes the fixed five calls in budgeting. Empty modules and missing
round-trip values are deliberately outside the new profile; legacy could retain
those as decoded evidence. No legacy implementation was copied. These pure planning tests do not qualify
persistent coverage or IC effects.

The data-reply slice additionally reviewed the legacy data admission and tests
above, its [bounded wire reader](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_snapshot_data/wire/mod.rs),
and [codec adoption history 1af7e219](https://github.com/dragginzgame/ic-backup-legacy/commit/1af7e219c3073395992c316ecaa5e83e74b1aecc).
It retains exact lengths, whole-chunk hashes, preallocation bounds and independent
skipped-work accounting. The new mapping obtains its schema and field label from
Platform instead of redeclaring a management record. Nine re-authored
[data tests](../../crates/ic-backup/src/store/capture/transfer/data/tests.rs) exercise
those checks, truncation, arity, type errors, unknown fields and the admitted 1 MiB
maximum. Two frozen raw wire vectors retain the legacy type-table ordering cases;
passing these tests is fresh local decoder evidence, not IC qualification.
Legacy reply objects, raw-evidence persistence and authentication machinery are
not copied into this pure helper.

Platform 0.33.0's `management-canister-types` feature supplies all external DTOs
and Nat/Principal identities; no local management declaration or registry Candid
dependency is added. The feature adds locked `num-bigint` 0.5.1, `num-integer`
0.1.47, `num-traits` 0.2.19 and `autocfg` 1.5.1. Supplier source is the registry
package at [Platform dd15c2c8](https://github.com/dragginzgame/ic-platform/tree/dd15c2c8c5f7e407f5aeeef38eb0dce68d43811f/crates/ic-platform).

The metadata slice additionally reviewed the same frozen revision's
[metadata wire reader](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_snapshot_metadata/wire/mod.rs),
[private field mappings](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_candid/mod.rs)
and codec adoption history 1af7e219 above. Eleven re-authored
[metadata tests](../../crates/ic-backup/src/store/capture/transfer/metadata/tests.rs)
cover optional meanings, exact bits, bounds before collection allocation, malformed
wire, source payload refusal and original-plan admission. The two retained wire
fixtures in [golden.json](../../crates/ic-backup/src/store/capture/transfer/metadata/golden.json)
come from legacy's
[golden cases](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_snapshot_metadata/tests/golden.json);
their original SHA-256 values remain checked. Legacy admitted incomplete metadata
as evidence; the public capture helper now refuses it through the existing profile.
Request/evidence digests and legacy reply objects were not copied: they belong to
future authenticated capture and persistence. No IC effects are qualified here.

The list/take slice reviewed frozen legacy's
[snapshot reply admission](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_snapshot_reply/mod.rs),
[wire reader](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_snapshot_reply/wire/mod.rs),
[regressions](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_snapshot_reply/tests/mod.rs)
and codec migration 1af7e219 above. The migration retained bounded ID/count admission
and exact single-argument decoding while moving parsing and separate work/allocation
limits to Platform. The new reader uses Platform's `Snapshot` schema instead of
redeclaring management fields. It retains wire order rather than legacy's sorted
reply representation; uniqueness still compares exact opaque bytes.

Eight re-authored [snapshot tests](../../crates/ic-backup/src/store/capture/snapshot/tests.rs)
exercise maximum counts/IDs and full u64 values, duplicate IDs with changed metadata,
truncated/extra/wrong-shaped replies, hostile declared lengths, unknown/missing/wrong
fields, exact take arguments, capacity refusal and prior-ID reuse. Three raw vectors
in [golden.json](../../crates/ic-backup/src/store/capture/snapshot/golden.json) retain
the original bytes, SHA-256 and decoded values from
[legacy goldens](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_snapshot_reply/tests/golden.json).
Legacy request/evidence digests and reply wrappers are omitted from these pure
helpers; authenticated association and durable settlement belong to the workflow.
No implementation was copied and no IC list/take effects were qualified.

## Platform 0.33 adoption — 2026-11-14

The requested 0.32.3 → 0.33.0 adoption selects the published registry package,
without a local patch. Its Registry protobuf addition is outside Backup's selected
features. Reviewing c90ce9fc → dd15c2c8 found no changes to management DTOs or the
Candid codec; the native graph retains exactly `candid-codec`,
`management-canister-types` and `principal`, with Types 0.20.3. Only Platform's
version and checksum changed in the lock. No Registry, Agent or runtime feature
was enabled, and the Host filesystem capability gap remains independent.

Public management DTOs and codec error types now come from Platform 0.33; Rust
consumers exchanging them must select that compatible line even though external
wire schemas are unchanged. Canonical Principal identity remains Types 0.20.3.
Canic's inspected manifest and lock also select Platform 0.33.0; this is graph
alignment, not qualification of Canic execution or its future Backup adapter.
This dependency adoption did not change the unpublished draft version or
`publish = false`; the later rewrite version decision is recorded in the
[restart direction](../design/restart.md).

The root is the only maintained workspace/lock. Disposable graph admission and
locked root metadata confirmed the selection before package Clippy and all 86
library regressions, covering exact identities, wire bytes, hashes and local
recovery custody. Four public documentation examples also compile. Graph evidence
is retained under `target/evidence/platform-0.33.0/`; no IC effects, supplier suite,
host matrix or release workflow was run.
