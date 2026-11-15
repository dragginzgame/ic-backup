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
settlement or complete-coverage check. Durable acquisition composes it as described below.

## Uploaded metadata readback

`CaptureTransfer::decode_uploaded_metadata(raw_reply)` compares uploaded metadata
with the immutable admitted capture source. It returns the decoded destination
metadata only after exact region sizes, ordered global variants/values, certified
data, timer/hook presence and state, and complete chunk-hash sets match. Floating
globals compare IEEE bits: NaN payloads/signaling/sign and positive/negative zero
remain distinct. Missing globals or lifecycle fields never become defaults.

The destination must identify `MetadataUpload`; the source was already admitted
as `TakenFromCanister`. Destination timestamp and canister version are returned
unchanged as separate provenance, with no equality or freshness claim. This follows
the [management metadata contract](https://docs.internetcomputer.org/references/ic-interface-spec/management-canister/#ic-method-read_canister_snapshot_metadata).
Wire chunk order stays unchanged; comparison sorts a bounded temporary hash index
against the source's unique index. Extra, missing, duplicate and malformed hashes
refuse, including omission of a named empty chunk.

The existing metadata decoder owns all raw/header/type/work/allocation and vector
bounds; this entrypoint adds no parser or external DTO. Its comparison index holds
at most 1,024 32-byte hashes. Source plan, source metadata, raw evidence and ledger
remain untouched. `ReadbackMetadataError` identifies the mismatched state category
without including values or payloads in diagnostics.

This is metadata comparison only. The caller must retain/authenticate the exact
network, target, destination snapshot and request association; the reply contains
no canister or snapshot ID. Complete source-bundle verification, independent restore
intent and budgets, full destination payload comparison, current stopped state
and application admission remain prerequisites to load. The local payload comparison
below implements equality checks; effectful readback remains unimplemented. No network,
provider call, spending, settlement, load or capture completion occurs here.

## Ordered payload comparison

`CaptureTransfer::begin_readback(destination_id, metadata_reply)` first admits a
distinct destination ID of 1..=256 bytes and the existing uploaded-metadata checks.
The returned `UploadedSnapshotReadback` retains destination metadata, immutable
source/destination bindings and a comparison counter. It performs no IO or durable
transition. `next_request()` returns the next ordinal and Platform data request
using the original canister and source schedule with the destination snapshot ID.
Repeated inspection returns the same DTO and grants no dispatch authority.

`compare_data(ordinal, source_slice, raw_reply)` requires the next scheduled ordinal.
It validates source length/hash before decoding the destination under the existing
data-reply bounds, then compares exact bytes. All three region kinds reject
same-length corruption. Whole chunks require the original hash and extent cap on
both sides. Failures leave the comparison count unchanged; repeated local checking
does not authorize any new network attempt. One bounded decoded reply is temporary;
neither source nor destination payloads accumulate in this object.

`finish()` consumes the checker and returns destination metadata only after every
scheduled position matches. Empty heap/stable regions emit no request; every named
chunk, including an empty chunk, requires comparison. Skipped, repeated and excess
ordinals refuse. This is complete equality to supplied slices, not an authenticated
or durable receipt. No existing record schema or spending state changes.

The caller must provide each source slice from the held verified bundle at its
original schedule position and authenticate/retain each reply's exact network,
canister, destination ID and request association. A data reply contains none of
those identities. Comparison alone cannot prove that the caller supplied the
right source, that the destination was freshly allocated, or that either snapshot
remained under exclusive custody. [`open_payload`](local-inventory.md#held-payload-source)
now supplies a held payload reader with verified bounded reads; it does not yet
admit the capture evidence or complete metadata/schedule binding required of a
verified bundle. Restore intent/accounting, actual remote reads,
durable coverage and load admission remain future workflow work. The capture
schedule's budget never becomes a restore allowance.

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

The readback slice reviewed the same frozen revision's
[metadata admission](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_snapshot_metadata/mod.rs),
[metadata tests](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_snapshot_metadata/tests/mod.rs),
[upload mapping](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_snapshot_upload/mod.rs)
and [upload tests](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_snapshot_upload/tests/mod.rs),
alongside codec migration 1af7e219. They preserve global order/bits, optional-state
meaning and source association; upload refuses unavailable globals. Legacy allowed
absent timer/hook values to remain absent. The restart instead requires those
values at source admission and exact readback equality, as selected in the operation
design. Eight re-authored [readback tests](../../crates/ic-backup/src/store/capture/transfer/readback/tests.rs)
cover all state categories, NaN/signed-zero cases, chunk-set differences, unchanged
source/provenance, exact collection maxima and malformed/excessive replies.
No legacy stage graph, request wrapper, implementation or IC qualification was copied.

The payload comparison slice reuses lessons from the frozen data and upload
admission/tests and codec migration above: exact destination/source separation,
bounded lengths, whole-chunk hashes and complete coverage are independent checks.
Its eight re-authored [data readback tests](../../crates/ic-backup/src/store/capture/transfer/readback/data/tests.rs)
exercise all regions and chunks, exact IDs, partial extents, empty chunks, missing
coverage, skipped/repeated ordinals, same-length corruption, source/reply refusals
and exact 1 MiB payloads. Source shape/hash validation now has one owner shared
with the data decoder. Legacy request/evidence wrappers and durable upload stage
machinery are not copied into this ephemeral comparison; those safeguards remain
prerequisites for future effectful integration, not guarantees of this helper.

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

## Local metadata request and certified-reply authentication

`CapturePlan::metadata_read_arguments(snapshot)` encodes Platform's
`ReadCanisterSnapshotMetadataArgs` with the plan target and exact opaque snapshot
ID, admitting 1–256 ID bytes before cloning and a 512-byte argument ceiling.
The fixed method is `read_canister_snapshot_metadata`, management receiver
`aaaaa-aa`, with the plan canister as effective routing target. It grants no call
permission and does not establish that a caller-supplied snapshot came from the
original take.

`authenticate_metadata_reply` binds this exact expected request to externally
supplied `PreparedStatus` and `StatusEvidence` (the existing prepared-call/evidence
types). It checks complete request/result bounds, original sender, receiver,
method, arguments, expiry and request ID. Direct Ed25519/low-S secp256k1 signature
checks share the existing owner. The certificate verifier checks original root,
effective target, retained certificate/delegation windows and exact terminal
request-status response. Both response bytes and rejection flag must match the
retained evidence. Rejections and nonterminal outcomes cannot become metadata.

The existing bounded metadata decoder and transfer profile then apply unchanged.
The metadata timestamp must match the selected original take's timestamp; this
is a consistency check, not proof of freshness or provenance. The take's reported
`total_size` is not equated to metadata extents or local payload size. Globals,
floating-point bits, optional state and raw evidence remain unchanged.

This helper performs no clock read, signing, HTTP, journaling or scheduling.
The effectful workflow described below must authenticate the original settled take, preserve
snapshot custody, check current permission and durably reserve signing/network/
evidence allowances before acquisition. It must retain authenticated metadata and
the admitted schedule before data reads. This local helper introduces no public
arbitrary-method interface, call enum variant, persisted format or retry framework.
Private expected-call validation is shared with existing status/list/take verifiers;
their public contracts and event encodings remain unchanged.

Focused legacy review reread the frozen
[metadata request/reply owner](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_snapshot_metadata/mod.rs)
and [identity/wire regressions](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_snapshot_metadata/tests/mod.rs),
plus [codec migration 1af7e219](https://github.com/dragginzgame/ic-backup-legacy/commit/1af7e219c3073395992c316ecaa5e83e74b1aecc).
They separate effective target from management receiver, preserve exact snapshot
arguments, bound metadata before allocation and treat decoded declarations as
unauthenticated. The new helper reuses canonical Platform identity/codec and
Backup's signature/certificate owners. Legacy digest wrappers and request objects
are unnecessary for this local slice and were not copied. The existing decoder
retains strict optional-field and allocation lessons from that migration.

Six [native test groups](../../crates/ic-backup/src/store/capture/advance/tests/metadata_tests.rs)
cover both keys and exact float bits; ID/timestamp bounds; signed substitution of
method/receiver/target/arguments/sender; signature/proof/root/window and byte limits;
unsupported profile/payload size; and certified rejection/nonterminal outcomes.
Real signatures over synthetic certificates qualify local authentication only.
The original three PocketIC fixtures check the shared verifier's capture/recovery
callers. The fourth fixture qualifies metadata acquisition as described below;
none qualifies a complete downloaded backup.

## Durable metadata acquisition

`CaptureOperation::advance_metadata` requires a settled original snapshot and
replays its original status/list/take authentication before admitting metadata.
The derived snapshot ID, timestamp and reported total size are retained with
`CaptureMetadataAdmissionRecord`; a caller cannot substitute a separate snapshot.
The admission binds the existing call bounds, transport, signer protocol, read
permission witness, expiry and certificate observation windows. Integrations own
continuous application maintenance and exclusive snapshot custody.

The original ledger records `MetadataStarted`, `MetadataPrepared`,
`MetadataSubmissionReserved` and `MetadataSettled`. Signing is reserved before
invocation; `metadata-request.bin` retains exact signed bytes before one bounded
HTTP submission; `metadata-result.bin` retains certified evidence before metadata
policy is applied. The common ledger budget counts complete admitted original
calls, metadata and any lookup against the immutable signing/network/evidence
limits. Metadata cannot reuse allowance already committed to lookup. Existing
event bytes remain unchanged; older readers refuse the new events.

`CaptureMetadataSigner::admit_read` checks current permission before signing or
submission. Prepared continuation reuses the exact request after fresh admission.
Failed, cancelled or invalid signing stays `SigningPending`; uncertain HTTP stays
`SubmissionPending`. Neither repeats, refunds spending or starts another attempt.
Metadata reconciliation through read-state or surviving replies is not implemented.
Complete uncertain frames remain spent after reopening; partial frames and orphan,
changed or missing evidence refuse without cleanup or repair.

Settled replay checks the original capture, request, expiry, certificate windows,
root and metadata policy locally without live admission, signing or HTTP. Certified
rejections and unsupported metadata remain retained but cannot produce `Ready`.
`inspect_metadata` reports only metadata phase and additional reservations; original
status/snapshot views are unchanged. A settled phase alone is not policy success.
No durable data schedule, payload read or completed local backup is produced.

Focused legacy review used the frozen revision's
[artifact persistence owner](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/ops/persistence/download_journal/ic_snapshot_artifact/mod.rs),
[verification owner](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/ops/persistence/download_journal/ic_snapshot_artifact/verification/mod.rs)
and [journal regressions](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/ops/persistence/download_journal/tests/mod.rs).
Exact metadata/request retention, timestamp association, independent reported size
and refusal after incomplete writes carry forward. Backup's existing original
journal and custody checks replace the legacy separate download journal; no
legacy cleanup, retry framework or source code was copied.

Eight new native test groups exercise reservation ordering, exact offline replay,
permission/binding/budget refusal, lookup spending, prepared continuation, lost
outcomes, all four uncertain-write boundaries and evidence damage. The fourth
PocketIC fixture acquires real metadata, verifies unchanged snapshot/canister
state, then replays exact metadata after shutdown and guard revocation. Evidence
is retained under `target/evidence/metadata-acquisition/`; this does not qualify
full payload capture, external signer processes or Canic maintenance adapters.

## Durable schedule binding

`CaptureOperation::bind_transfer` explicitly binds the schedule after metadata is
settled. It shares private original-capture and retained-metadata authentication
with `advance_metadata`, without calling that effectful entrypoint. Its only inputs
are the bound transport and original capture authentication provider. Admission
comes from the retained journal; no metadata signer is accepted. Metadata signing,
live read admission and HTTP are never invoked. Before settlement it refuses
without advancing metadata acquisition.

One `TransferBound` event retains a version-one commitment, exact data-call count
and conservative payload ceiling in the original journal. There is no separate
schedule file or duplicate decoded metadata. The commitment hashes the following
binary concatenation with SHA-256: domain `ic-backup/transfer-schedule/1\0`,
32-byte original encoded-plan digest, 32-byte metadata-request evidence digest,
32-byte metadata-result evidence digest, u32 snapshot-ID length and original ID,
u32 extent size, three u64 region sizes in module/heap/stable order, u32 chunk
count and sorted 32-byte hashes, u32 data-call count and u64 payload ceiling.
Integers are little-endian. The version fixes the existing region/extent/chunk
ordering algorithm. Computing this descriptor is bounded by the chunk limit;
it never materializes or enumerates all data requests. Exact original metadata
wire order and floating-point bits remain bound through the result evidence.

The first bind appends and syncs one local frame within the original evidence
budget. Identical calls reauthenticate and compare the commitment without writes.
Unknown versions refuse journal admission; changed geometry, evidence or policy
cannot silently replace the retained binding. A complete uncertain frame survives
reopen; partial frames refuse. `inspect_transfer` is only a read-only projection,
not authentication. Binding grants no additional signing/network allowance and
makes no claim that remaining provider costs fit; each future data attempt still
needs its own cumulative budget admission and fresh application permission.
There are no payload reads, progress records or completion effects in this slice.

The focused legacy review revisited the artifact verification and journal tests
linked above, plus [plan-ownership history a860d144](https://github.com/dragginzgame/ic-backup-legacy/commit/a860d14421b9ff1c61c1dce079cdc6a19316bfd0).
Those checks keep original evidence admission separate from progress projections.
This implementation preserves exact source association and uncertain-write
refusal using the existing journal; it does not import the legacy second journal,
artifact-tree checksum or mutable progress graph before payload files exist.
No source code was copied. New native cases cover original-evidence and geometry
binding, canonical chunk ordering, pre-settlement refusal, unchanged spending,
cold replay, conflicting/unknown records, partial/complete uncertain frames and
authentication/evidence-budget refusal. The real metadata fixture now binds its
one-request, eight-byte schedule and verifies unchanged offline replay.

## Scheduled data request and certified-reply authentication

`CaptureTransfer::data_arguments(ordinal)` encodes Platform's fixed
`read_canister_snapshot_data` request under a 1 KiB argument cap. It derives the
exact target, snapshot and region offset/size or chunk hash from the existing
zero-based schedule, without iterating preceding extents or accepting arbitrary
ranges. Out-of-range ordinals refuse before request construction.

`authenticate_data_reply` checks the exact direct-key signed request and original
certificate root/time policy, including receiver, method, sender, expiry, request
ID and scheduled arguments. The retained reply bytes and rejection flag must
match the certified terminal outcome. Only then does the existing bounded decoder
check region length or whole-chunk ceiling/hash. A valid signature and certificate
for another same-length extent cannot satisfy this ordinal. Original requests,
proofs and response bytes are not rewritten. Status, metadata and data share the
private certificate evidence comparison; there is no public arbitrary-call API.

This authenticates one supplied result, not its source metadata provenance,
original reservations, current snapshot custody or local persistence. It performs
no IO, clock read, signing, HTTP, settlement or progress update. Callers must retain
and authenticate source metadata and its schedule binding separately. These are
local helpers reused by durable acquisition and complete coverage verification below.

Payload limits and retained-evidence limits are independent. The original result
record contains both response bytes and the certificate containing that response.
The 2 MiB complete-evidence ceiling therefore cannot admit a full 1 MiB payload
plus framing/proof overhead. A 512 KiB region with the qualified certificate fits;
this is not a guarantee for every smaller user-selected evidence cap. Future live
admission must account for complete bounded proofs before spending; it must not
widen limits or alter an already-bound extent size after a refused read.

Focused legacy review used the frozen
[data request owner](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_snapshot_data/mod.rs),
[transfer-read owner](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_snapshot_transfer_read/mod.rs),
[response regressions](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/policy/ic_snapshot_transfer_read/tests/mod.rs)
and the a860d144 history linked above. Preserve original payload association,
separate bounds and the distinction between validated bytes and transfer success.
Use existing Platform DTOs and Backup's shared verifiers instead of copying
legacy request/digest/response wrappers or its operation graph. No code was copied.

Six native groups exercise both supported keys, every region and chunk kind,
empty chunks, signed identity/range substitution, wrong ordinals, damaged
signature/proof/time/outcome, certified malformed payloads, rejection/pending
outcomes and independent complete-evidence bounds. Existing IC fixtures regress
shared certificate callers; they do not qualify remote payload acquisition.

## Durable sequential payload acquisition

`CaptureOperation::advance_data(ordinal, admission, transport, signer, original)`
acquires one exact zero-based data ordinal against an existing `TransferBound`.
`CaptureDataAdmissionRecord` retains the original call limits, transport and
signer identities, read-authority witness, expiry and certificate windows.
`CaptureDataSigner` is a bounded sign-only integration: its permission check is
local and effect-free; it owns process custody and cannot dispatch or retry.

The workflow reauthenticates original capture/metadata, recomputes the retained
schedule binding and verifies preceding accepted payloads before dependent work.
No ordinal can be skipped, and the next ordinal requires the previous one to be
accepted. A prior accepted ordinal can be replayed without advancing later work.
The existing journal's cumulative limits count all original calls, metadata,
lookup and every data admission, including five maximum-size frames per data
read. Reopening never resets signer, network or evidence spending.

The five events are `DataStarted`, `DataPrepared`, `DataSubmissionReserved`,
`DataSettled` and `DataAccepted`. The first reserves signing before invocation;
`data-NNNN-request.bin` retains exact signed bytes before the submission reservation.
`data-NNNN-result.bin` retains the complete certified result before payload policy.
`DataAccepted` records validated byte length and SHA-256, including a valid empty
chunk, only after exact scheduled response authentication. Ordinals are canonical
four-digit decimal names within the original 8,192 signing/network ceiling.
Evidence files are create-only, held and rechecked through the existing custody
owner; extra, missing, altered or orphan files refuse without repair or cleanup.
Existing event bytes are unchanged; older readers refuse the added data events.

Payload bytes remain inside certified result evidence. There is no separate
payload file or per-extent journal. `Ready` returns bytes decoded from that evidence,
not a completion receipt. On replay both the certificate and recorded byte identity
must match. An accepted marker alone cannot authorize another read. A valid
certificate carrying malformed or unsupported payload remains settled and blocks
progress; it cannot become accepted. `inspect_data` reports phase, original
admission and additional reservations without authenticating coverage.

Prepared continuation reuses exact bytes after fresh read permission. Failed,
cancelled or invalid signing remains pending and consumed; lost HTTP remains
pending and consumed. Neither retries. A settled valid result can be accepted
locally after interruption. An already accepted result replays without a clock,
metadata/data signer, live guard, HTTP or journal mutation. Complete uncertain
frames survive reopen with their original spending; partial frames refuse.
Data lookup/surviving-reply reconciliation is not implemented.

Before new signing or submission, admit room for the scheduled payload ceiling
plus 8 KiB reply overhead, the existing 1 MiB certificate cap and five framing
bytes under the original complete-result limit (at most 2 MiB). Region ceilings
use the exact scheduled size; chunks use the original extent ceiling. Accepted
raw replies also enforce that overhead bound. Full 1 MiB extents therefore refuse
before spending, rather than changing ranges or widening limits. Actual response,
certificate, Candid and cumulative evidence limits still apply independently.

This slice qualifies small sequential transfers, not maximum-size throughput.
It reauthenticates the preceding accepted prefix and rechecks retained files;
large-transfer performance needs measurement before claiming practical support
at the declaration ceilings. Closing custody is checked around the prefix, not
by rescanning the whole operation separately for every preceding payload.
No cache, second authority index or generic retry engine is introduced.

Focused legacy review revisited the transfer-read model, policy and regressions
linked above, its
[workflow](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/workflow/ic_snapshot_transfer_read/mod.rs)
and a860d144 consolidation history. Preserve original reservation association,
retained replies on policy failure and separate validation from completion. Adapt
those lessons to the existing single journal rather than importing legacy stage
catalogs, qualified receipts or per-operation journals. No code was copied.

Eight new native groups qualify sequential acquisition/replay, signing failure and
cancellation, lost HTTP, exact prepared continuation, invalid certified payload,
pre-effect binding/permission/proof limits, all five uncertain-write boundaries,
cumulative budget exhaustion, retained evidence damage and false accepted markers.
The fourth IC fixture acquires the real eight-byte Wasm payload and replays it
exactly after server shutdown and signer revocation, preserving the journal and
original canister state. Final status and completion now extend that journey;
restore remains unfinished.

## Complete retained payload verification

`CaptureOperation::verify_data` requires every bound ordinal to be accepted,
reauthenticates original capture/metadata and the exact transfer binding, then
reuses each retained request/certificate/payload check in schedule order. Exact
region lengths and named chunk hashes remain the existing decoder's responsibility;
no duplicate coverage map is added. The combined Wasm SHA-256 must match the
immutable plan. Host Artifacts' bounded `HashingWriter` streams into a sink;
only one bounded decoded reply is retained at a time. Actual payload bytes must
fit the original schedule ceiling, including unknown-length chunks.

Opening and closing custody checks surround verification. Incomplete acceptance,
invalid evidence or a wrong module hash refuse without writes, allowance changes,
cleanup or renewed authority. Success returns the actual payload byte count,
not a persisted receipt: there are no output files, final status check or completed
backup. Verification is repeatable offline using the original provider's replay
validation; it does not call signing, live admission or HTTP.

Legacy review at `4ae087fad8c23de712aa60bf569bde4b2b932651` covered
`model/ic_snapshot_coverage` and its gap/overlap/duplicate/empty-chunk regressions,
plus `ops/persistence/download_journal/ic_snapshot_artifact` and its finish-time
coverage/custody checks. History `94b94db` consolidated those implementations.
Keep complete coverage distinct from durable publication. The fixed new schedule
already owns order and ranges, so do not import legacy interleaving maps, writer
stages or a second progress journal. No source was copied.

## Local artifact assembly

`CaptureOperation::assemble_data` first runs complete authenticated coverage/module
verification, then retains `{"kind":"assembly_started"}` in the original ledger.
Admission reserves two maximum-size frames within the original evidence budget
before filesystem effects. No new signing, HTTP or observation allowance is spent.
The operation ID selects `bundles/<operation-id>/payload/`; staged files occupy
their final location, without a second copy, relocation protocol or manifest.
Inventory counts this unfinished bundle but omits it from completed results.

The fixed member set is `module.wasm`, `heap.bin`, `stable.bin`, followed by
`chunk-<lowercase-sha256>.bin` members from the bound schedule. Empty heap/stable
files and named empty chunks are retained. The source schedule owns ranges/order;
assembly only groups those ordinals by member. At most 1,027 members and original
payload bytes are admitted. Host owns private directories, exclusive 0600 files,
held no-follow reads, file identities and hashing. There is no whole-snapshot
buffer; scratch holds bounded replies/comparison bytes and bounded member handles.

All existing files must match complete authenticated source bytes before creating
any missing file. Partial, altered, extra, unsafe or hard-linked files refuse
without overwrite, append, deletion or a new staging location. A complete file
surviving before verification can be synchronized and reused. Missing files can
be created only before the verification event; after verification, any missing
file or directory refuses reopen instead of reconstruction. An unclaimed bundle
at the selected ID also refuses. The integration excludes noncooperating namespace
and content writers throughout; sequential held checks are not a filesystem fence.

New files are written from the original certified payloads, synchronized and read
back against source-derived exact lengths/hashes. The payload, bundle and parent
are synchronized; retained ledger synchronization also applies on replay. Closing
file/directory custody checks surround hashing. Filesystem uncertainty invalidates
the live operation handle. Reopen preserves partial bytes and consumed authority;
there is no automatic recovery for a partial member in this version.

`{"kind":"assembly_verified","sha256":[...]}` retains the 32-byte local tree
commitment after successful file verification and synchronization. This compact
commitment introduces no second member manifest. Hash input is ASCII
`ic-backup/assembled-payload/1` plus NUL, followed in ASCII filename order by each
u32 little-endian name length, UTF-8 name bytes, u64 little-endian member length
and raw 32-byte SHA-256. The two-extent `helloworld` module with empty heap/stable
has independent golden commitment
`71f4aa3d4064e1e45191510033efb8118152f5c8e4bb41f40ee883bef04e43dc`.
Existing ledger bytes are unchanged; older readers refuse the new events.

Verified assembly replays by authenticating original evidence and checking exact
files, without journal appends, signing or HTTP. Success returns payload bytes;
it does not assert final canister state, publish `manifest.json`, release target
custody or grant restore permission. `PublishReady` and completion publication are implemented separately under
[the local-store contract](../design/local-store.md#completion-and-interrupted-publication). Small fixtures qualify this
path; repeated source/file checking is not a maximum-size performance claim.

Legacy review at `4ae087fad8c23de712aa60bf569bde4b2b932651` covered
`ops/persistence/download_journal/ic_snapshot_artifact` and its partial-drop,
changed/unsafe/missing-file, empty-region/chunk and custody regressions. History
`29bf7a51044e72c4bd8cf6dc5cd6cb24d3711b4d` removed a redundant downloaded transition.
Preserve exact byte checking and partial-file retention; do not import that old
journal/layout or add a downloaded stage. Unlike its streaming remote writer,
this assembler starts with all certified reads already retained, allowing complete
unrecorded members to be compared and reused offline. Native regressions exercise
that changed recovery contract; no legacy source was copied. Actual IC
qualification assembles the eight-byte Wasm and replays after server shutdown.
