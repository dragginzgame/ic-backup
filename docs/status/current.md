# Current handoff — 2026-11-15

Work lives at `/home/adam/projects/ic-backup`, branch `main`; the former legacy
local path was absent. Frozen legacy remains read-only at `dragginzgame/ic-backup-legacy`.
No original journals, bundles or obligations were imported, migrated or cleaned up.

This 0.101.5 GitHub checkpoint contains durable metadata acquisition, schedule
binding and local payload authentication. Product metadata, both locks and private
qualification references select 0.101.5. Every package remains `publish = false`;
this checkpoint has no registry publication, tag or deployment. The preceding
0.101.4 checkpoint is [203b208](https://github.com/dragginzgame/ic-backup/commit/203b2083009b3aad9b5892a4023fc4ea5ee6d7a5).
Payload dispatch/persistence and complete backup/restore remain unfinished.

## Implemented scope and safeguards

The [inventory contract](../architecture/local-inventory.md) owns private stores,
bounded listings/latest backup per target, manifest-bound verification and held
source reads. Integrity does not authenticate capture or grant restore authority.
The [capture contract](../architecture/capture-intent.md) owns immutable plans,
durable claims and bounded chained ledgers. Uncertain writes invalidate handles;
complete uncertain frames remain spent. Partial frames, orphan evidence and
missing owners/locks refuse without repair or fresh authority.

`advance_status` checks stopped state, approved module and original controller.
`advance_snapshot` authenticates the original list, admits capacity and takes one
new snapshot. Calls retain exact requests, separate original admissions and
reservations within cumulative budgets. Prepared continuation never resigns;
pending continuation invokes no provider; settled replay authenticates locally.
Inspection is not authentication or completion. No replacement or cleanup exists.

`settle_capture_result` records surviving original certified status/list/take
replies under original trust/timing and spending. Identical settled replay is
effect-free; replacement bytes refuse. Rejection and policy-invalid replies may
be retained but never become policy success. Settlement grants no application
authority and advances no dependent work. Orphans/partial tails still refuse.

`observe_capture_certificate` authenticates original requests and proofs under
separate observation windows/caps, preserving unresolved outcomes.
`authenticate_capture_lookup` checks original signatures, sender/request ID,
lookup expiry and the sole request-status path. `StatusHttpTransport::lookup_once`
performs one bounded read-state request and returns exact certificate bytes and
an authenticated projection. This low-level helper requires external reservations.

`advance_lookup` now composes these pieces into one durable lookup per operation.
Its original ledger retains admission, signing/observation reservation, exact
signed request, HTTP reservation and certificate. Signer/network/byte allowances
fit the original plan alongside complete admitted original-call allowances; later
snapshot admission cannot reuse lookup spending. Integration-owned read permission
is checked before new effects. Prepared continuation reuses exact bytes; failed
signing and lost HTTP stay pending without another attempt. `inspect_lookup`
reports lookup spending separately from original status/snapshot counters.
Observed replay reauthenticates locally without a signer, clock or server.
Original event ordinals and bytes are unchanged; old readers refuse new events.

`settle_capture_lookup` now records a terminal retained observation as the original
call result with its distinct observation source and time policy. Cold continuation
reauthenticates that source and applies existing result policy; live admission is
still required before dependent effects. Unresolved observations cannot settle. There is only one lookup even when a plan
permits more. No original mutation is repeated because its reply was lost.

`DirectCaptureProvider` composes bounded external signing and HTTP submission for
status/list/take. Original route/network/root, signer/guard protocols and timing
remain bound; spending never renews. Direct Ed25519/secp256k1 are supported.
Integrations own signer/process custody, exclusive snapshot custody and continuous
application maintenance. Backup owns no keys or Canic runtime.

The [transfer contract](../architecture/capture-transfer.md) owns bounded pure
metadata/data decoding, schedules and upload/readback comparison.
`metadata_read_arguments` encodes the fixed target/snapshot request;
`authenticate_metadata_reply` verifies its direct signature and exact certificate
reply under original trust/time, then applies the existing metadata/transfer
profile and matches the selected take timestamp. It preserves raw request/proof
bytes, performs no effects and proves no original take custody or spending.
`advance_metadata` authenticates the retained original capture, reserves signing
and submission in the original journal, and retains exact request/result bytes.
Current read permission precedes effects; settled replay needs no live signer or
server. Cumulative budgets include original calls and any retained lookup.
Failed signing and lost HTTP replies remain pending without retries; metadata
lookup/reconciliation is not implemented. Inspection is not policy success.
`bind_transfer` authenticates settled metadata and retains one compact versioned
schedule commitment. Repeated binding compares it without writes or remote work;
unknown/conflicting records refuse. This grants no data-attempt authority.
Binding now accepts only transport and original capture verifier; retained
admission and shared replay checks remove the unused metadata signer input.
Local `data_arguments`/`authenticate_data_reply` now bind payload replies to exact
scheduled requests, then apply existing length/hash checks. Complete evidence
still must fit its independent cap: a full 1 MiB payload plus certificate exceeds
2 MiB; live admission must account for proof overhead before spending.
Payload download, completion writing, full bundle-evidence admission,
upload/load/restore and CLI remain unfinished. A snapshot is not a local backup.

## Dependencies and qualification

Platform 0.33.7/Types 0.20.4 and Tooling 0.44.3 are adopted in both workspaces.
Host FS/Artifacts/HTTP remain 0.79.0, Tokio 1.54.1 and Rust/Cargo 1.100.0.
Private qualification retains Testkit 0.106.0/PocketIC 16.1 and its exact
`thiserror` 2.0.18 constraint; product retains 2.0.21. No sibling overrides exist.
Tooling 0.44.3 updates the LOC reporter: default depth zero is totals, depth one
adds crates and depth two modules. Exporter, snapshot/tools/format/link checks and
actual LOC calls pass under `target/evidence/tooling-refresh-latest/`.
[Setup](../development/setup.md) owns supplier provenance and commands.

Latest evidence under `target/evidence/data-auth/` records 141 advance and
48 transfer tests, 23 documentation examples, strict product/private Clippy,
warning-denying Rustdoc and all four existing PocketIC fixtures. Native data
certificates qualify local authentication only; remote payload reads are unqualified. The fourth acquires real metadata, binds its schedule, then
replays exact metadata and binding after server shutdown and signer revocation.
Native cases cover identity/geometry, unchanged spending, conflicting records,
complete/partial uncertain writes and authentication/evidence-budget refusal.

Earlier evidence remains under `target/evidence/metadata-acquisition/`,
`target/evidence/metadata-auth/` and `target/evidence/lookup-settlement/`;
[setup](../development/setup.md) and the implemented contracts retain scope and
counts. Original failed/successful IC bundles remain in
`qualification/target/fixtures/`. The metadata fixture's initial undersized
allowance was corrected to the existing whole-transfer requirement; no product
limit changed. None of these checks qualifies physical power loss, external
signer processes, Canic maintenance fences, trusted HTTPS, hosted macOS or full
backup/restore. No evidence was deleted.

Replay cleanup evidence under `target/evidence/transfer-replay-cleanup/`
records 135 capture-flow tests, 22 documentation examples, strict product/private
Clippy, warning-denying Rustdoc and all four PocketIC fixtures passing.

The local transfer-boundary review found unnecessary signer/admission inputs on
`bind_transfer`; they were removed in favor of shared private replay owners.
The original evidence, commitments and journal encodings are unchanged.
`AGENTS.md` records the user's required post-push drift/complexity/sediment focus.
This local review is not a review of a newly pushed commit.

## Next work

Implement bounded payload dispatch/persistence against the retained transfer
commitment and qualified data authenticator, then local completion. The operation
contract records the proposed sequential read flow; it is not implemented yet.
Establish exact persistence, source custody and interruption contracts before
payload reads. Keep the single
lookup bound unless a concrete use case requires more; no generic retry engine.

The focused post-push review of `203b208` checked committed lookup HTTP, ledger
spending, settlement authentication and affected continuation callers against the
contracts and native/IC evidence. No actionable issue was found in that scope. Canic owns membership, concurrency, routing, release identity, maintenance,
external settlement and reopening; [Canic #1](https://github.com/dragginzgame/canic/issues/1)
covers adapters and executor/fenced recovery. V1 remains one already-stopped
canister, local bundles and same-release restore to the same existing ID, left
stopped. Blob storage remains v2.
