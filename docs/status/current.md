# Current handoff — 2026-11-14

The restart lives at `/home/adam/projects/ic-backup`, branch `main`. The former
local legacy path was absent. Legacy source/history remain in
`dragginzgame/ic-backup-legacy`; no original journals, bundles or unfinished
obligations were imported, migrated or cleaned up.

The separate private `dragginzgame/ic-backup` repository received the foundation
at `271d44aaab2ae8e9e90a2cbe53de0a42b123e40b`. This batch includes the subsequent
inventory, payload, capture-intent, transfer and status work. The user selected
`0.100.0` for the rewrite and authorized its commit/push to `origin/main` only;
crates.io publication and release tags are excluded. The single package inherits
`publish = false`. The unpublished `0.68.0` draft is superseded, with no automatic
migration of legacy backups or unfinished operations.

## Implemented slice

The Unix `store` API initializes/opens a private store and exposes `list_backups`
and `latest_completed_backup`. The [inventory contract](../architecture/local-inventory.md)
owns exact bounds and custody. Initialization is create-only; interrupted work
is retained and reads never repair. Scans bound entries, cumulative manifest
bytes and returned rows. Malformed nonmatching records also refuse a complete
result. Latest ranks source capture time, host completion time and operation ID.

Inventory reads passive completion declarations, not authenticated evidence.
`verify_payload` checks complete member coverage, exact lengths and hashes against
a caller-trusted manifest commitment and byte limit. It also rechecks original
manifest, payload/file identities and membership before returning. This proves
observed integrity, not future custody, capture authenticity or restore authority.

The [capture-intent contract](../architecture/capture-intent.md) implements pure
immutable plans, exact approval commitments, durable target claims and exclusive
create/open/inspect operations. Missing original owners/locks never become fresh
authority. Process death releases exclusion but retains the original target claim.
The operation holds its ledger descriptor; same-byte replacement now refuses.

The ledger reader validates chained frames with independent 16 KiB payload,
65,536-frame and 128 MiB total bounds, plus the original evidence-byte allowance.
Allocation follows bounds checks; partial tails and unknown transitions refuse.
Creation checks the initial ledger allowance before claiming a target. Only the
canonical created transition is supported; original golden bytes are unchanged.
Pure [transfer planning](../architecture/capture-transfer.md) now derives lazy
Platform data-read DTOs from borrowed metadata: ordered region extents, sorted
whole chunks, checked conservative payload and whole-capture request/signing minima.
It preserves original IDs and metadata/float bits, rejecting unsupported profiles.
`CaptureTransfer::decode_data` now bounds raw replies through Platform's schema/
codec and checks exact region lengths, original chunk size limits and hashes.
`CapturePlan::decode_metadata` now bounds raw metadata collections before
allocation and reuses original transfer admission. It preserves recognized
optional meanings and float bits; neither decoder authenticates replies or
persists progress. `CapturePlan::validate_status` now checks decoded Platform status
for stopped state, the exact approved module hash and original sender/executor
controller membership, with duplicate and ten-controller bounds.
`CapturePlan::validate_status_reply` applies that same policy to raw status bytes,
with controller/hash checks before allocation and a separate cumulative skipped-
work budget. It projects only the required fields through Platform schema labels;
neither method authenticates observations or grants dispatch authority.
There is no attempt reservation, settlement, ledger append, terminal succession,
completion writer, full bundle admission, capture/restore, transport, CLI or
simulator yet. This is not an independently usable backup/restore product.

Published suppliers are Host FS/Artifacts 0.77.2, Platform 0.33.0 (management DTOs),
Serde 1.0.229, serde_json 1.0.154 and thiserror 2.0.21. The Platform feature adds
locked num-bigint 0.5.1, num-integer 0.1.47, num-traits 0.2.19 and autocfg 1.5.1. No sibling patches, Canic runtime dependency or test
harness was added; Platform owns canonical Principal identity through Types 0.20.3.
The requested Platform adoption changes only its lock version/checksum. Selected
management schemas/codecs are unchanged; Registry protobuf stays disabled. Public
DTO/error consumers must use the 0.33 line. Canic's inspected manifest/lock already
select 0.33.0, without claiming execution or adapter qualification.

## Selected scope and next work

The user selected a host-side single-canister library: capture an already-stopped
canister; restore to the same network/release/existing ID and leave it stopped.
Canic owns membership, routing, application settlement, signing and reopening.
Independent targets may run concurrently under its task limit; Backup owns
per-target exclusion. Local bundles are v1; Blob integration is v2. No database,
persistent catalog index, scheduler or generic storage trait is planned.

The [operation contract](../design/operation-contract.md) and
[local-store design](../design/local-store.md) retain the proposed effectful flow:
one authoritative ledger, intent and bounded spending before effects, explicit
lost-reply reconciliation and retained original evidence. Terminal replay must
be effect-free. Structural record limits are not measured provider capacities.

First resolve the [Host append interface](../architecture/capture-intent.md#append-integration-dependency):
its selected capability API cannot reopen an existing file for writing relative
to a held directory. Path-based lock helpers do not supply that interface. Keep
original ledger custody and root-rename behavior; no workaround was introduced.
The user approved the scoped `ic-tooling` change and GitHub-only push. Host's
`FileEntry::open_regular_file_for_update(forbidden_permissions)` implementation
is pushed in [c30c0f92](https://github.com/dragginzgame/ic-tooling/commit/c30c0f92c680c918970c9eb06559d1958de0d615),
with publication scope recorded in [b9700c9d](https://github.com/dragginzgame/ic-tooling/commit/b9700c9d7c8a2375943181dbbc4dad22f23f5ad0).
It passed 18 capability and 40 reader tests, strict Clippy with/without capability
and Rustdoc on Linux. The user now owns Tooling coordination and delivery; this
Backup batch makes no Tooling changes. Host FS 0.77.3 was unavailable at the latest
registry check; Backup still selects 0.77.2. Adopt a published Foundation release
containing the opener before adding admitted reservation/settlement and durable append with
interruption recovery, then capture completion publication and its consumer,
followed by upload/readback/load. Keep writes internal to the qualified workflow;
there is no public arbitrary-event/raw-reservation API. Select concrete application
safety and authenticated execution providers before remote effects. Do not mistake
the framing reader for implemented spending or attempt replay.

The [sibling review](../design/sibling-integration.md) records Canic's actual
manager-only worker controller and external ICP signer. Integration requires a
Canic-owned durable executor receipt protocol and Platform-owned Wasm snapshot
DTOs, currently native-only. Direct-host qualification will not prove that route.
Testkit belongs only in future private qualification. Blob older-owner activation
remains unsupported; original recovery authority cannot be reset.

## Legacy review and validation

The [legacy assessment](../design/legacy-assessment.md) pins
`4ae087fad8c23de712aa60bf569bde4b2b932651`. Focused inventory/payload and attempt
persistence/model/tests/history reviews are recorded in the implemented contracts.
The latest review preserves complete-original admission before any future spending,
using streamed frames in place of legacy's retained event vector. Transfer review
adds exact extent/metadata source, tests and a860d144 history anchors. Data-reply
and metadata reviews add the bounded wire readers and 1af7e219 codec history.
Status review adds controller authority, lifecycle tests and observation settlement;
the intent contract records retained guarantees and the typed boundary. The raw
status continuation adds its bounded wire reader, independent fixtures and codec
history, retaining skipped-work limits without legacy observation wrappers.
No legacy implementation was copied; raw fixtures retain their source and hashes. The same-byte ledger replacement regression failed before the fix.
Follow [AGENTS.md](../../AGENTS.md) for each next subsystem review.

Platform 0.33.0 adoption passed disposable graph admission, full locked root
metadata, strict package Clippy, all 86 library tests, warning-denying Rustdoc and
four documentation examples. The package suite was selected because Platform's
public types cross the store, intent and transfer boundaries. Original golden
manifest, plan, frame and wire bytes/hashes remain unchanged. Graph evidence is
under `target/evidence/platform-0.33.0/`; native fixtures, including failed and
sparse-file cases, remain under `target/local-store-tests/`.

The status continuation passed all 15 focused tests (seven typed and eight raw
reply cases), strict package Clippy, warning-denying Rustdoc and all six public
documentation examples. Three frozen legacy status replies retain exact bytes
and hashes; no new IC qualification is claimed. The prior 86-test package result
remains evidence for the unchanged implementation and was not rerun. No dependency
or lock selection changed in this batch.

Version preparation for the GitHub-only 0.100.0 batch passed locked offline
metadata admission (the sole package remains non-publishable), formatting,
documentation links and the authenticated tooling snapshot. Existing focused
code-validation evidence above is unchanged; no release runner or registry upload
is selected.

These results do not qualify power loss, attempt reservation, authenticated IC
effects, Canic or hosted macOS. Child-process death covers durable intent only.
Rust 1.100.0 and authenticated Toko Tooling 0.43.0 remain selected (51 snapshot
files, nine entrypoints and the active shared formatter hook). Setup evidence
remains under `target/evidence/`; final scoped formatting/link/snapshot checks
accompany this batch. No full IC or release gate ran.
