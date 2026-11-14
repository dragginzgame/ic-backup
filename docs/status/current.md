# Current handoff — 2026-11-14

Work lives at `/home/adam/projects/ic-backup`, branch `main`; the former legacy
local path was absent. Frozen legacy remains read-only at `dragginzgame/ic-backup-legacy`.
No original journals, bundles or obligations were imported, migrated or cleaned up.

The 0.101.0 checkpoint includes held payload sources, snapshot readback and
historical certificate evidence for successful and rejected initial status calls.
It is prepared for the requested GitHub-only delivery after
[b952533](https://github.com/dragginzgame/ic-backup/commit/b95253375961f111df1eb648bd3479240846a13c).
All packages remain `publish = false`. No release tag, registry publication or
deployment is selected. A separate next-version commit follows this checkpoint.

## Implemented scope and safeguards

The [inventory contract](../architecture/local-inventory.md) owns private local
stores, bounded completion listings and latest backup per target. Listings are
passive completion declarations. `verify_payload` checks a trusted manifest's
membership/lengths/hashes; `open_payload` holds original descriptors and an
in-memory 1 MiB block hash index. Bounded reads bind the exact source request and
recheck original file/tree custody. IO/integrity failures invalidate the handle.
At most 1027 payload descriptors are retained; OS resource failures are explicit.
Neither API establishes capture-evidence authenticity or restore authority.

The [capture contract](../architecture/capture-intent.md) owns approved immutable
plans, durable target claims, held original custody and bounded chained ledgers.
Writes invalidate the session before persistence. Uncertain writes require
reopening; complete uncertain frames remain spent, while partial frames and orphans
block continuation. Missing original owners/locks never become fresh authority.
Root rename follows held directories; replacement custody refuses. Reads never
repair, truncate or discard evidence. Terminal replay grants no new effects.

`advance_status` reserves preparation/signing before invoking a provider, retains
exact prepared bytes, reserves one submission and retains authenticated evidence
before settlement. Pending replay never signs or submits; prepared replay uses
original bytes after current admission. Settled replay authenticates locally and
checks stopped state, approved module and original caller/controller membership.
`inspect_status` recovers original admission, stage, reserved counts and request ID
without writes or provider calls; `Settled` alone is no authentication claim.
Platform checks original call content, arguments, sender, expiry and request ID.

The new [certificate helper](../architecture/capture-intent.md#historical-status-authentication)
uses Platform to build and authenticate original replies and certified rejections.
It binds the original root DER hash, effective target, request-status path and
exact reply bytes.
Explicit certificate/delegation time windows are retained before submission and
reused without reading a clock. Strict frame, certificate and tree bounds apply.
Pending/forgotten outcomes and unavailable paths refuse; no digest substitutes
for a certificate. Rejections retain exact bounded diagnostics and settle without
refunds; optional absence must be certified, never inferred from pruning.
The helper provides no live envelope authentication,
transport, current authority or application-continuity admission.

`StatusProvider` remains a trusted integration contract with no concrete qualified
provider. Canic plans round-trip but execution refuses pending its receipt protocol.
The [transfer contract](../architecture/capture-transfer.md) owns pure bounded
metadata/data decoding, deterministic schedules and non-destructive snapshot
list/take preflight. Uploaded metadata comparison preserves exact state/float bits
and separates destination provenance. Ordered local readback requires every extent
and named chunk, including empty chunks, against caller-held source bytes.
These comparisons supply no authenticated remote readback or load authority.

Snapshot capture/reconciliation, completion writing, complete bundle-evidence
verification, upload/readback/load effects, CLI and simulator remain unimplemented.
This is not yet an independently usable backup/restore product.

## Dependencies and current evidence

Host FS/Artifacts 0.79.0, Platform 0.33.4/Types 0.20.4, Toko Tooling 0.43.1 and
Rust 1.100.0 are selected. Everything resolves from the registry; no sibling patch
or shared target is required. Host 0.79's public Rust identities require consumers
to align; the unpublished 0.101.0 minor covers that change. Stored formats and
recovery authority remain unchanged.

Platform's minimum is now 0.33.4 because its `certification-verify` feature supplies
the required historical API. Six crypto packages were added; all existing locked
versions are preserved. Disposable and root locked/offline metadata confirm no
Agent, HTTP runtime, Canic runtime or simulator dependency. Graph inputs, metadata
and validation logs remain under `target/evidence/status-certificates/`.

Eighteen signed-certificate/persistence cases pass, covering exact bindings,
tampering, delegation, historical policy, bounds, cold replay and spent refusal,
plus strict rejection diagnostics and terminal replay. Rejection evidence/logs
are under `target/evidence/status-rejections/`; prior certificate logs remain.
The existing 21 continuation tests also passed during this batch. Strict package
Clippy, warning-denying Rustdoc and thirteen documentation examples pass. Synthetic
certificates use real Platform BLS signatures but do not qualify IC effects,
a live provider, application safety, physical power loss or hosted macOS.
Failed initial fixture/lint logs and all native fixture directories are retained.

Prior archive checks, source identities and qualification remain in the
[checkpoint handoff](https://github.com/dragginzgame/ic-backup/blob/b95253375961f111df1eb648bd3479240846a13c/docs/status/current.md)
and their existing `target/evidence/` directories. Frozen wire/plan/request fixtures
are unchanged. Legacy review, changed assumptions and re-authored test provenance
are recorded in the owning capture and transfer contracts.

## Next work

Continue the single-canister host library: capture an already-stopped canister and
restore to the same network/release/existing ID, leaving it stopped. Canic owns
membership, concurrency, routing, maintenance, external settlement and reopening.
Local bundles are v1; Blob is v2. No scheduler, catalog database or storage framework.

The [direct-provider review](../design/sibling-integration.md#direct-provider-continuation-review--2026-11-14)
now has its local certificate evidence builder/verifier. Next provider work must retain
actual response certificates and qualify bounded one-shot transport, signing,
root/time admission. Lost replies require separately
accounted original-request reconciliation, never hidden polling or retries.
Complete bundle admission must separately bind the original plan, ledger, raw
metadata and source request to the trusted manifest and payload schedule.

[Platform #162](https://github.com/dragginzgame/ic-platform/issues/162) and
[#163](https://github.com/dragginzgame/ic-platform/issues/163) are closed and supplied
in 0.33.4. [Canic #1](https://github.com/dragginzgame/canic/issues/1) owns the executor
receipt/fenced-recovery protocol. Scope remains Backup-only source edits; file
concrete supplier issues rather than editing siblings. No supplier blocker is
claimed for the now-published historical certificate or Wasm snapshot APIs.
