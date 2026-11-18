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

Read-only materialization derives expected member lengths/hashes from original
retained evidence, then reads the actual payload once through the complete closing
tree hash. It compares every expected member, length and digest and retains the
original descriptor and parent/directory checks. An earlier byte-for-byte payload
comparison is redundant on this effect-free path. When creating missing members,
that early comparison remains mandatory: admit **all** existing members before
creating any missing one, then independently hash the complete tree after writing.
No expected commitments or authentication results are persisted as a cache.

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

The read-only payload-pass reduction re-reviewed frozen artifact
`verification/mod.rs::{checksum_child_with, checksum_exact_reader}`, its
`callback_reads_reject_same_length_replacement_and_retain_both_files` regression,
and artifact `tests/mod.rs::changed_unsafe_extra_and_missing_files_reject_before_any_journal_transition`.
History `9f7b886a824eec67dda8f81792a560436419bb2a` consolidates an owning verification
pass while retaining original record and closing namespace checks. Backup keeps
its complete closing SHA-256 comparison and held identities; only the earlier
read-only payload comparison is removed. The owning recorder's corruption matrix
still covers changed/truncated/missing/extra/unsafe files and now also proves that
a corrupt existing heap blocks creation of a missing module. Legacy arbitrary
arrival order and metadata files are outside the fixed new layout; no source was
copied and no historical pass is counted as new qualification.

Fresh reduction evidence: all 15 recorder custody tests, the mutation-across-HTTP
regression and actual-IC 20 MiB capture/restore/interrupted-transfer/offline replay
pass; the latter completes in 90.10 seconds. Strict product Clippy also passes.
Logs and retained failed compile preparation are under
`target/evidence/replay-payload-review/`; before/after offline read counts and
timing limits are recorded in [the restart measurements](../design/restart.md#size-maintained-surface-and-repeated-io).

Repeated phase signing/submission/recovery matrices now live once at
`operation/runtime/tests.rs`; phase-specific geometry, bounded parsing,
metadata bits and exact policy cases remain with their algorithms.
The actual [IC consumer](../../qualification/ic-backup-qualification/tests/capture.rs)
uses 16 KiB extents over a nontrivial module, 64 KiB heap, 128 KiB stable memory
and both empty/nonempty chunks. It independently compares payload bytes and
qualifies interrupted original data settlement; this is fresh IC evidence.
Native synthetic-certificate and pure readback tests do not qualify restore effects.

## 20 MiB qualification — 2026-11-16

`twenty_mib_capture_restore_interrupted_transfer_and_offline_replay` in the same
actual-IC consumer passes with exactly 20 MiB of combined module, heap, stable
memory and chunk bytes. The fixture uses a 64 KiB heap, 19 MiB stable memory,
the existing small module and two chunks: empty and the remaining payload bytes.
One MiB extents produce 23 data units, 28 calls per complete capture and 53 per
complete restore. This is a tested shape, not a guarantee for every 20 MiB input.

The approved profile reserves 22 MiB payload, 128 network requests and signatures,
one observation and 1 GiB evidence per operation. Capture has an 8 KiB request /
4 MiB result ceiling; restore has 2 MiB / 4 MiB. A first attempt with only 20 MiB
payload allowance correctly refused the conservative metadata total of 22,151,205
bytes: each chunk reserves a full extent, including the empty chunk. No allowance
is enlarged after approval or refunded after interruption.

The test loses the original third data reply, closes and reopens the operation,
checks its original request and spending, then imports the surviving certificate
with application permission revoked and no additional signature or network call.
A second import refuses. It completes capture, compares all local payload bytes,
publishes a successor, restores the older source after changing actual stable
memory, verifies full readback/load and stopped state, checks restored memory and
module independently through PocketIC, and starts capture after restore. After
backend shutdown, both completed captures, restore and history replay offline.

Two optimized Linux runs pass in 89.20 and 88.63 seconds. The second warm command
takes 89.58 seconds overall and `/usr/bin/time -v` reports 730,672 KiB maximum RSS
and 908,184 filesystem output blocks. These are test-command/process accounting,
including fixture/backend work, not isolated library memory, a byte-read trace or
a production throughput comparison. The debug attempt spent 482 seconds before
the backend's default idle timeout expired during local source/replay work; its
evidence is retained. Qualification now uses the optimized profile and an explicit
finite 30-minute idle/hard server lifetime for this case. Production code and
operation deadlines are unchanged; repeated replay still has substantial costs.

Evidence: `target/evidence/twenty-mib-{release,warm}-{qualification.log,time.txt}`;
warm original store, journals, certificates and server logs:
`qualification/target/fixtures/capture-Capacity-3805726-1794820857897640283/`.
Failed capacity fixtures are retained too. Production HTTPS, Canic authority and
actual-IC macOS behavior remain separate unqualified boundaries.

The normal-dependency consumer built against the extracted current Cargo archive
also reopens the completed 20 MiB capture and restore after backend shutdown, with
only the original store/IDs/approvals. Its debug-build inspections take 21.95 and
51.62 seconds; `target/evidence/twenty-mib-archive-replay.json` retains both zero
exit statuses. The initial sandbox subprocess-control error is retained separately.
This is archive replay, not a newly published registry install or a live archive
capture/restore run.

Focused review also used frozen legacy `4ae087fad8c23de712aa60bf569bde4b2b932651`:
`src/standalone/transfer/mod.rs`, core `model/ic_snapshot_coverage/tests/mod.rs`
and `workflow/ic_snapshot_restore/tests/mod.rs` (the
`failure_boundaries_retain_pending_or_applied_and_never_repeat_provider` case).
History `a860d14421b9ff1c61c1dce079cdc6a19316bfd0` and
`9f7b886a824eec67dda8f81792a560436419bb2a` explain bounded ownership and verified
upload planning. This fixture adapts their coverage, held-source and original-call
recovery guarantees to the fixed public loop and actual IC. It imports no legacy
source or driver architecture; synthetic historical passes are not this proof.

## Partial Protocol adoption — 2026-11-16

Production snapshot-data, restore allocation and exact empty-result decoding now
use Protocol 0.2.7's selected APIs. Backup's private data/allocation schema wrappers
are removed (61 net production lines). Original byte/header/type/work/allocation
ceilings remain explicit; vector ceilings are 1 MiB for data and 256 for allocation,
and skipped work remains zero. Geometry/hash, nonempty/reused/source snapshot IDs,
step ordering and finite spending remain Backup policy. Original reply bytes and
stored encodings do not change. The affected error variants expose Protocol's
error type, which is a consumer-facing type change for the next release.

Re-reviewed frozen legacy `4ae087fad8c23de712aa60bf569bde4b2b932651`,
`model/ic_snapshot_data/wire/mod.rs`, data tests and
`model/ic_snapshot_upload/tests/mod.rs::bounded_reply_identity_exact_empty_tuple_and_wire_extensions_reject`,
plus migration history `1af7e219c3073395992c316ecaa5e83e74b1aecc`.
These checks prevent oversized allocation, silent field skipping, altered framing
and replacement/source-ID reuse. Existing data tests run against the new decoder;
a re-authored restore regression adds 256/257-byte IDs, every truncated prefix,
trailing bytes and empty tuple versus one-unit-argument refusal. No legacy source
was copied. The fixed-procedure matrix still owns empty/reused/source-ID policy.

This is partial acceptance for
[Platform #11](https://github.com/dragginzgame/ic-platform/issues/11): status,
list/take, metadata and request DTO migration remain. The old production Platform
cohort still supplies those APIs; no replacement management DTO or codec was added.
Qualification signing/proxying uses Platform 0.34.9, while its explicitly named
old-cohort dev dependency is needed only for the product's current public
source-read DTO/identity contract. Remove it when that boundary is ported.

Fresh partial-adoption qualification passes 237 native tests, five documentation
examples, both strict Clippy checks, Rustdoc, all 11 actual-IC journeys (105.69
seconds) and the isolated archive consumer. The actual signer/proxy use Platform
0.34.9 against the existing product recorder, including original-request observation
and full 20 MiB recovery. This is mixed-cohort acceptance, not qualification of
the still-unported new product certificate/metadata/status boundary. Logs remain
under `target/evidence/platform-recheck-current/`; failed preparation and original
IC evidence remain preserved. Neither Platform issue is closed.

### Protocol 0.3.2 continuation

The selected data/allocation/exact-empty production decoders now use Protocol
0.3.2. Its response fixture encoders replace the old generic encoder in the nine
owning data regressions and allocation/acknowledgement cases of the three restore
procedure tests. The latter still exercise empty/reused/source/257-byte IDs and
actual fixed-cursor policy. Independent malformed/truncated/trailing/arity vectors
remain; no round-trip-only replacement or regression deletion occurred.
The `management-snapshot-fixtures` feature is selected only on the test dependency
and is absent from the normal product feature graph. The legacy data/upload
source and test provenance recorded above still applies; no stored format changes.
Protocol's public error identity moves to 0.3, requiring the next minor Backup
release. At that point full status/metadata/request/authentication migration
remained unfinished; the completed port below supersedes this interim state.

## Full Platform port — 2026-11-17

Platform 0.34.11, Protocol 0.3.3 and Types 0.21.2 now own all implemented
management request/reply types and wire traversal. The old Platform dependency
is absent from both workspaces, including tests. `ReadDataArgs`/`ReadKind` expose
explicit `Extent` and `SnapshotChunk` records; metadata uses `SnapshotMetadata`
and globals use bounded `u128` plus exact `Float32Bits`/`Float64Bits`. Backup keeps
schedule geometry, identity/uniqueness, provenance, readback and custody policy.
The private snapshot and metadata schema walkers are removed. Original framing,
header/type/work/allocation/skipping budgets and collection caps remain unchanged.

Focused read-only review revisited frozen legacy
`4ae087fad8c23de712aa60bf569bde4b2b932651`:
`model/ic_snapshot_reply/wire/mod.rs`, its identifier/count/full-u64/malformed
regressions, and `model/ic_snapshot_metadata/wire/mod.rs` with its promotion,
absence, float-bit, collection/hash, unknown-variant and skipped-work tests.
Relevant history: `1af7e219c3073395992c316ecaa5e83e74b1aecc`,
`94b94db89f77968eb4b480ef6d0576d41f6472e2`,
`6220eb7d89b9349eaf3254d256fd283ac75335e3`. Their checks prevent invented
round-trip values, unbounded allocations, ambiguous source identity and changed
numeric state. Those guarantees remain; only their wire owner moves to Protocol.
No legacy source or recovery artifacts were imported.

Retained literal goldens and checksums are unchanged. Normal replies use Protocol
fixture encoders; deliberately malformed schemas and independently decoded
requests use test-only Candid 0.10.40 and Management Canister Types 0.8.0. Full
status fixtures still demonstrate discarded resource statistics, while a separate
authored status-visibility schema demonstrates that visibility grants no controller
authority. No old codec, generic decoder fallback or replacement production DTO
remains. Envelope and certificate fixtures use new Platform construction/signing.

A typed value above 128 bits is now impossible: its former typed scheduling and
readback cases converge on the retained 129-bit raw-wire refusal at metadata
admission. `GlobalRange` is removed; the codec returns `Limit`. Empty snapshot IDs
now return `SnapshotId` after bounded decode; excessive IDs still return `Limit`.
These are explicit pending 0.105.0 public type/error changes, not changes to stored
schemas, authority or spending. The packaged README owns consumer migration.
The handoff records fresh native/IC acceptance and pre-port retained-store replay.

### Direct SDK test dependency retirement — 2026-11-17

[Backup #2 — Remove direct test-only SDK dependencies](https://github.com/dragginzgame/ic-backup/issues/2)
replaces the temporary test declarations above with 68 finite retained binary
inputs (156,998 bytes), their lengths/hashes and original source/schema provenance.
The catalog lives under `capture/tests/wire/tests/fixtures/`; the owning
README pins Backup v0.105.1 and the original Candid 0.10.40 / management SDK 0.8.0
projections. Normal policy setup still uses Protocol fixture encoders; malformed
schemas no longer require runtime SDK types. Literal legacy goldens are unchanged.

Focused review again read frozen legacy
`4ae087fad8c23de712aa60bf569bde4b2b932651` snapshot reply wire/tests and metadata
wire/tests, with history `1af7e219c3073395992c316ecaa5e83e74b1aecc`,
`94b94db89f77968eb4b480ef6d0576d41f6472e2` and
`6220eb7d89b9349eaf3254d256fd283ac75335e3`. Original count-first refusal,
required/optional distinctions, reserved-source strictness and numeric-bit
preservation remain necessary. The changed assumption is finite fixture replay
instead of per-test schema construction; no production algorithm changes.

Status retains full/minimal replies, controller/state/module policy, hostile
counts without element bodies, option promotion/absence, unknown/required fields,
cumulative skipped work, visibility, prefixes/arity/trailing/limits. Metadata
retains 129-bit refusal, unknown source/global/timer/hook, bare/null/reserved and
missing options, required fields and zero-byte unknown fields. Exact take and
seven readback request bytes preserve target/snapshot/kind/offset/size/hash
bindings, independently checked with SDK decoding and re-encoding before removal.
Upload metadata still compares independent exact bytes for integer extremes,
NaN payloads and negative zero. Readback extra/wrong arguments and null versus
empty results retain their refusal cases. Fixture length/digest admission prevents
accidental changes. No case was dropped to satisfy dependency ownership.

Both workspace graphs have no direct consumer Candid/management SDK declarations
of any kind or target. Transitive Candid through Platform and SDK dependencies
owned by native Testkit remain valid; removing those is not this work. The local
SDK exception is retired. New production APIs, wire formats and journals are
unchanged. The handoff owns focused and actual-IC results.
