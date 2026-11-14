# Implemented capture intent

This Unix-only slice persists local intent, not a backup. `CapturePlan::new` is
pure; `BackupStore::create_capture` writes intent; `open_capture` and
`CaptureOperation::inspect` admit original intent without repair. There is no
reservation, dispatch, reconciliation, completion or target-release API. The
[operation design](../design/operation-contract.md) owns the proposed next steps.

## Exact declaration

`CaptureRequest` selects the 32-character lowercase hexadecimal operation ID,
network digest, canonical Platform canister Principal, trust-root digest,
initiator, release, module hash, execution declaration and `CaptureLimitsRecord`.
Direct execution binds the sender. Canic execution binds executor, protocol
digest and authority generation; recording it does not qualify that route.

The strict JSON record uses format `ic-backup/local-capture-plan/1` and rejects
unknown/duplicate fields, unsupported formats and noncanonical identities.
The writer emits compact JSON in the field order frozen by
[the golden plan](../../crates/ic-backup/src/store/tests/fixtures/capture-plan.json).
Reopen retains the exact original bytes, rather than normalizing or re-encoding.
Plan admission is bounded to 1 MiB. The approval commitment is:

```text
SHA256(UTF8("ic-backup/local-capture-plan/1\0") || original_plan_bytes)
```

Each identity and limit is included. Limits declare payload and evidence bytes
in `1..=1 GiB`, extent bytes in `1..=1 MiB` and no larger than payload, network
and signing requests in `1..=8192`, and observations no greater than network
requests. Observations are a subset, not additional authority. These are
structural bounds, not proof that a transport, snapshot or evidence schedule fits.
Provider deadlines, response limits, application admission and metadata-dependent
cost admission are still required before effects. Changing a declaration requires
a different approval; reopening cannot increase an original limit.

## Status preconditions

`CapturePlan::validate_status` checks an already-decoded Platform
`CanisterStatusResult` without allocating, reordering or changing it. The selected
capture profile requires stopped state, the exact approved 32-byte module hash,
and a duplicate-free controller set of at most ten canonical Principals containing
the original direct sender or Canic executor. Ten is this product profile's bound.
An approving initiator, another controller or status viewer cannot substitute for
that management caller. `CaptureStatusError` distinguishes these refusals.

This pure method does not decode or authenticate evidence. The caller must bound
raw decoding and associate the authenticated reply with the original network,
target, request and observation timing; the DTO contains no target identity.
Success does not establish continuous application custody, external settlement,
reservation, dispatch or completion. Resource statistics, migration readiness and
version continuity remain outside this check. No stop or other IC effect occurs.

## Bounded status replies

`CapturePlan::validate_status_reply` reads the required status, controllers and
module hash from one exact Candid argument, then applies the same precondition
policy as `validate_status`. It returns no synthesized full status DTO. Other
fields, including unknown fields, are skipped with bounded traversal; their
presence and semantics are not qualified. Settings/controllers and status must
be present with the required types. An absent or wrongly typed optional module
hash cannot satisfy capture; a present hash must contain exactly 32 bytes.

Limits are 1 MiB raw input, 64 KiB header, 64 type entries, 2 MiB codec work and
2 MiB codec allocation, with a separate cumulative 64 KiB skipped-work allowance.
These are Platform codec counters, not upstream quotas or transport limits.
Controllers are capped at ten before vector allocation. Hash length is checked
before allocating its bytes. Trailing bytes, surplus arguments, unknown status
variants and exhausted bounds refuse through `CaptureStatusReplyError`; policy
failures retain `CaptureStatusError`. Original raw evidence remains caller-owned.

The private argument projection reads labels from Platform's status/settings
schemas. It introduces no replacement management wire declaration or encoder.
Platform owns header parsing, traversal and allocation accounting; the existing
pure policy owns state, module and caller checks. Decoding does not authenticate
network/target/request association, record progress or grant effect authority.

## Files and ordering


Creation requires the caller's exact plan commitment before any write. It uses
Host private held directories, exclusive create-only files, bounded no-follow
regular-file reads and nonblocking native locks. Directories/files use 0700/0600;
hard-linked records refuse. It creates this subset of the proposed store:

```text
targets/<network-digest>/<canonical-canister>/
  lock
  owner.json
operations/<operation-id>/
  lock
  plan.json
  ledger.bin
```

Creation takes the target lock, durably retains the owner record, creates and
locks the operation, then durably retains plan and initial ledger. Owner JSON
contains `format = ic-backup/local-target-owner/1`, `operation_id` and
`plan_sha256`. Its exact bytes must match the original plan, within 1 KiB.
The target lock is released after creation; the operation lock lives with the
returned handle. Different targets have independent locks. Duplicate operation
IDs refuse; a creation race can leave an incomplete second target claim, retained
for original recovery rather than silently removed.

The initial ledger contains one frame, with JSON bytes exactly
`{"sequence":0,"kind":"created"}`. Its encoding is:

```text
seed = SHA256(UTF8("ic-backup/local-ledger/1\0") || original_plan_bytes)
length = little_endian_u32(json_byte_length)
chain = SHA256(seed || length || json_bytes)
frame = length || json_bytes || chain
```

The independent [golden frame](../../crates/ic-backup/src/store/tests/fixtures/capture-created.hex)
uses raw 32-byte digests. The streaming reader admits at most 16 KiB per payload,
65,536 frames and 128 MiB total, further bounded by the original plan's evidence
allowance. Frame lengths and cumulative bytes are checked before allocating a
payload. It retains one frame at a time, validates each chained digest and refuses
any partial tail. Typed `LedgerError` variants distinguish these failures.

Semantic admission still requires exactly the canonical initial created record.
A correctly hashed unknown event, duplicate creation or changed sequence refuses;
no suffix is ignored or interpreted as unused authority. General transition
replay, reservation accounting and append are not implemented. The original golden
bytes are unchanged. This is an unpublished restart format, not a legacy import.

Creation checks that the initial ledger fits the original evidence allowance
before claiming the target. Reopen does not increase that allowance. This bounds
the ledger now; accounting for other evidence files belongs with their writer,
not an assumption that each future file gets the full allowance again.

## Recovery and custody

Dropping a handle or killing its process releases the native lock, not the owner.
A complete operation can reopen only with its original approval, plan, ledger,
owner and lock files. The operation also holds the original ledger descriptor.
Inspection checks held child-directory/lock identities, store format, exact record
bytes and member sets. Ledger admission checks file identity before reading and
rechecks the opened file's length and named file's identity/length afterward.
Replacing the ledger with identical bytes therefore refuses a held inspection. Store root rename follows its
held descriptor, consistently with inventory. These checks assume exclusive
cooperating writers and private-store custody; they are not a hostile-writer fence
or protection against rollback of all local files.

Creation failures retain every partial file and may leave the target occupied.
An existing target lock with a missing owner is incomplete, never a fresh target.
Missing established locks are not recreated beside retained records. Reopen does
not create, synchronize, repair or discard anything. No terminal succession is
implemented: even a successfully created intent keeps its target occupied after
the handle is dropped. There is no cancellation or cleanup escape hatch yet.

## Append integration dependency

The selected Host FS 0.77.2 [capability implementation](https://github.com/dragginzgame/ic-tooling/blob/9d938d2814b637ea40d4ddbae110e87ab402c881/crates/ic-host-fs/src/capability/mod.rs)
opens existing regular files read-only and creates new files read/write. Safe
ledger continuation needs an existing-file writable opener relative to the held
directory, without create/truncate, with no-follow regular-file admission. Backup
must then verify original identity, link count and permissions before writing.
Path-based lock openers are not this capability interface; using a remembered
display path would change continuation behavior after a store-root rename.

The explicitly authorized supplier contribution now implements
`FileEntry::open_regular_file_for_update(forbidden_permissions)` in
`ic-tooling` [c30c0f92](https://github.com/dragginzgame/ic-tooling/commit/c30c0f92c680c918970c9eb06559d1958de0d615). It shares the existing descriptor engine and retains permissions,
no-follow/nonblocking/type admission and held-parent rename behavior. It neither
creates nor truncates and returns a read/write descriptor at offset zero; caller
identity, hard-link policy, locking and seek/write/sync remain explicit. Linux
qualification passed 18 capability and 40 shared reader tests, strict Clippy with
and without capability, and Rustdoc. No macOS or Backup append/recovery claim
follows. The source was pushed to GitHub under the push-only scope recorded in
[b9700c9d](https://github.com/dragginzgame/ic-tooling/commit/b9700c9d7c8a2375943181dbbc4dad22f23f5ad0).
The user now coordinates Tooling delivery. Backup still selects Host FS 0.77.2;
publication and actual consumer adoption remain separate from this fix.

Adopt the published supplier boundary before implementing append. Do not rewrite the
whole ledger, recreate missing records, add a local sibling patch or invent
platform-specific path tricks. The reviewed legacy cached-record helper disables
its session before persistence and installs state only after success. Adapt that
ordering with retained partial frames and acknowledged process-death tests once
append is available. No reservation or lost-write recovery was implemented in the
transfer-planning batch; its [pure schedule](capture-transfer.md) is independent.

## Focused legacy review — 2026-11-14

Reviewed frozen legacy `4ae087fad8c23de712aa60bf569bde4b2b932651`:

- [Attempt persistence](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/ops/persistence/attempt_journal/mod.rs)
  binds reopened journals to exact original authority, reserves durably before
  returning a slot, and invalidates a session after uncertain persistence.
- [Regressions](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/ops/persistence/attempt_journal/tests/mod.rs)
  cover retained identity/limits, failed reservation, unsafe/replaced evidence and
  acknowledged owner death. [The model](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/attempt_journal/mod.rs)
  prevents missing evidence from replenishing attempt authority.
- [Plan simplification history](https://github.com/dragginzgame/ic-backup-legacy/commit/a860d14421b9ff1c61c1dce079cdc6a19316bfd0)
  separates complete-original admission from optional progress projection. Removing
  duplicated projections does not justify omitting original-journal checks.

Adapted the exact-original, no-reset, nonblocking exclusion, retained partials and
acknowledged-death cases in [new native tests](../../crates/ic-backup/src/store/tests/capture.rs).
The missing-owner case reproduced an incorrect new claim before the admission fix.
Golden plan/chain bytes were independently generated with Node's SHA-256, then
compared against the production writer. Tests also cover independent targets,
concurrent claims, ambiguous records, unsafe files and changed custody.

No legacy source was copied. One plan/ledger replaces legacy per-step journals;
there is no generic workflow graph. The next focused review reused the same
attempt persistence/model/tests and plan-ownership history: complete original
admission is still required before any future spending transition. Legacy's
bounded retained-event vector becomes a streaming, length-delimited frame reader.

[Frame and custody regressions](../../crates/ic-backup/src/store/capture/ledger/tests.rs)
cover every truncated initial prefix, chain/plan mismatch, correctly hashed
unsupported events, bounds before body reads, exact frame/count boundaries,
interrupted reads, original IO errors and closing file drift. A separate held
inspection case reproduced same-byte ledger replacement acceptance before the fix.
The frame-count fixture tests framing only; its arbitrary payloads do not qualify
capture transitions. Sparse oversized files are retained as refusal evidence.

The status precondition review also inspected frozen legacy
[controller authority](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/control_authority/mod.rs),
[its regressions](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/control_authority/tests/mod.rs),
[lifecycle observations](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_lifecycle_reply/mod.rs),
[their tests](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_lifecycle_reply/tests/mod.rs)
and [settlement](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_observation/settlement/mod.rs).
These guard exact original caller membership, bounded controller sets and the
separation between a reported state and authenticated effect attribution.
[Codec history 1af7e219](https://github.com/dragginzgame/ic-backup-legacy/commit/1af7e219c3073395992c316ecaa5e83e74b1aecc)
retained those cases while moving lifecycle wire tests to Platform-owned codecs.

The new boundary already receives canonical Platform Principals and distinguishes
initiator, sender and executor in the original plan. It needs no string-based
controller wrapper or sorting. [Re-authored tests](../../crates/ic-backup/src/store/capture/status/tests.rs)
exercise stopped/running/stopping states, canonical duplicates, empty/excessive
sets and the exact ten-controller boundary, sender/executor confusion, visibility,
module mismatch and input preservation. The selected profile adds the approved
module-hash check. Observation wrappers and settlement transitions remain outside
this policy; authenticated observation association and IC effects require their
own work.

The raw-status continuation reviewed the same lifecycle implementation, tests and
codec history, plus its [bounded wire reader](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_lifecycle_reply/wire/mod.rs)
and private field mappings. It retains separately metered skipped work and bounds
before allocating controller elements. The new argument projection uses supplier
schema labels and canonical Principals directly, adds the approved module hash,
and calls the existing policy instead of building a second authority object.

[Eight new wire tests](../../crates/ic-backup/src/store/capture/status/wire/tests.rs)
cover complete supplier results, minimal projections, exact controller bounds,
hostile lengths before payload reads, module option promotion/absence, required
fields, state/caller refusals, cumulative nested skipping and malformed framing.
[Three retained golden replies](../../crates/ic-backup/src/store/capture/status/wire/golden.json)
come from [legacy lifecycle fixtures](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_lifecycle_reply/tests/golden.json);
their exact bytes and SHA-256 values are preserved. Their missing module hashes
cannot satisfy the new capture profile. No legacy implementation was copied.
Acknowledgement, receipt-digest and mutation tests are excluded because this API
performs only status validation, not stop/start/load or authenticated settlement.

Reservation failure, paid effects, receipt
settlement, effect-free terminal replay and power loss are not qualified by this
intent-only slice and must accompany their owning implementation. Native Canic
route round-tripping proves only declaration retention, not executor behavior.
