# Current handoff — 2026-11-15

Work lives at `/home/adam/projects/ic-backup`, branch `main`; the former legacy
local path was absent. Frozen legacy remains read-only at `dragginzgame/ic-backup-legacy`.
No original journals, bundles or obligations were imported, migrated or cleaned up.

The 0.101.4 checkpoint contains the Tooling refresh, one-shot HTTP, durable lookup
and retained-observation settlement. Product manifests, both locks and qualification
references agree. This checkpoint is selected for the requested GitHub-only push
to `origin/main`, following verified 0.101.3 commit
[d8955cf](https://github.com/dragginzgame/ic-backup/commit/d8955cf).
Every package remains `publish = false`; no tag, registry publication or deployment
is selected.

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
metadata/data decoding, schedules and upload/readback comparison. Download, completion writing, full bundle-evidence admission,
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

Evidence under `target/evidence/lookup-settlement/` records 118 advance tests,
10 ledger tests, 16 custody tests, 19 documentation examples, strict product and
private qualification Clippy, warning-denying Rustdoc and three actual IC fixtures.
Eight new native groups cover exact offline replay, failed/cancelled/invalid
signing, prepared continuation and lost HTTP, cumulative limits, retained damage,
complete/partial uncertain writes at every new frame, later admission respecting
spent allowance, and binding/permission/time refusal before reservation.
Six additional settlement groups cover distinct timing, invalid policy replies,
nonterminal observations, uncertain/partial/orphan settlement writes, conflicting
results and roots, list settlement with revoked live permission, and rejection.
Earlier evidence, including failed runs, remains retained.

The third actual IC fixture observes a real pending take using product journal
reservations, compares its full snapshot DTO and checks unchanged original call
state, snapshots, module and stopped state. After shutdown and signer revocation,
it settles the pending take from the retained lookup proof, reopens again and
replays both the original call and lookup without effects. The earlier two fixtures
still qualify the low-level transport with fixture-owned reservations and an
unchanged journal. Original failed/successful bundles remain under
`qualification/target/fixtures/`. These checks do not qualify physical power loss,
external signer processes, Canic fences, trusted HTTPS sessions, hosted macOS or
full backup/restore.

## Next work

Proceed to bounded snapshot download and local completion using the existing
transfer contract and original snapshot evidence. Establish exact persistence,
source custody and interruption contracts before remote reads. Keep the single
lookup bound unless a concrete use case requires more; no generic retry engine.

The focused post-push review of `d8955cf` found no actionable issue in its delivered
scope. Canic owns membership, concurrency, routing, release identity, maintenance,
external settlement and reopening; [Canic #1](https://github.com/dragginzgame/canic/issues/1)
covers adapters and executor/fenced recovery. V1 remains one already-stopped
canister, local bundles and same-release restore to the same existing ID, left
stopped. Blob storage remains v2.
