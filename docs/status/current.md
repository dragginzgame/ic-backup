# Current handoff — 2026-11-14

Work lives at `/home/adam/projects/ic-backup`, branch `main`; the former legacy
local path was absent. Frozen legacy is read-only at `dragginzgame/ic-backup-legacy`.
No original journals, bundles or obligations were imported, migrated or cleaned up.

The original rewrite was pushed at
[cd3118d](https://github.com/dragginzgame/ic-backup/commit/cd3118d22cee569a3dfa3a5b36b4d2a6cf56380e).
The status workflow/inspection, registry Host adoption and Toko refresh are
checkpointed at [cc592cf](https://github.com/dragginzgame/ic-backup/commit/cc592cfb445fd94d9e93f951c721c965b20e0eb4)
as unpublished 0.100.0. Development version 0.100.1 was pushed at
[d38bb01](https://github.com/dragginzgame/ic-backup/commit/d38bb01b91fdc509d87c13b84101f3a4e27bc90b).
The request-binding, Host adoption and snapshot preflight batches are local and uncommitted. Host
0.78 changes the Rust identities of public digest/error types, so the combined
pending release is now 0.101.0. Consumers must align their Host dependencies;
stored records and recovery authority are unchanged. The working manifest stays
0.100.1 until explicit version preparation. All packages remain `publish = false`;
no registry publication or release tag is selected.

## Implemented behavior

The [inventory contract](../architecture/local-inventory.md) owns private store
initialization, bounded completion listings and latest backup per target. These
are passive completion declarations. `verify_payload` checks membership, lengths,
hashes and observed custody against a trusted manifest; it does not establish
capture authenticity, future custody or restore authority.

The [capture contract](../architecture/capture-intent.md) owns approved immutable
plans, durable target claims, exclusive operations and bounded chained-ledger
admission. Missing original owners/locks never become fresh authority. Root rename
follows held directories; identical-byte ledger replacement refuses held inspection.
Reads never repair or discard partial evidence.

Initial direct-host `advance_status` reserves signing before preparation, retains
exact request bytes, reserves submission before one call and retains authenticated
result/proof before settlement. Writes invalidate the session before persistence;
write/sync/closing failure requires reopening. Complete uncertain frames remain
spent; partial frames and orphans block continuation. Cold pending replay cannot
sign or submit. Prepared replay uses original bytes after current admission.
Settled replay authenticates historical proof locally and applies status policy.

`inspect_status` now returns original admission, retained stage, reserved signing/
network counts and request ID without provider calls or writes. A restarted caller
can recover exact continuation inputs. Counts are reservations, not traffic.
`Settled` alone claims neither authentication nor status success. Views do not
mutate authority, replenish allowances or release the durable target claim.

Prepared status bytes now pass Platform's strict call-content decoder before
retention/submission and during settled replay or inspection. The original sender,
management receiver, method, exact target arguments, declared expiry and recomputed
request ID must match. Mismatches preserve spending/evidence and invoke no next
provider effect. This checks immutable content, not signatures or current freshness.

`StatusProvider` is a trusted integration contract; no concrete provider is
supplied or qualified. Native injected providers prove ordering only. Canic intent
declarations round-trip, but status execution refuses that route pending its
executor protocol. Pure [transfer/decoding](../architecture/capture-transfer.md)
retains Platform identities, metadata/float bits, exact region lengths and chunk
hashes. Typed/raw status policy requires stopped state, approved module and caller
controller membership; these helpers do not authenticate observations.

Pure snapshot list/take admission now bounds replies through Platform, preserves
wire order and rejects duplicate/reused IDs. Direct take arguments require explicit
capacity and preserve installed code and existing snapshots. The take reply alone
identifies its result; a later list difference cannot settle an uncertain call.
No dependencies, records or effectful transitions were added for these helpers.

Snapshot capture, reconciliation, completion publication, full bundle verification,
upload/readback/load, CLI and simulator remain unimplemented. This is not yet an
independently usable backup/restore product.

## Dependencies and evidence

Host FS/Artifacts 0.78.0 now resolve from verified registry archives, from supplier
`f816478db478c1226d0a953658e1ea0fb7ce7338`. Both hashes match the registry index.
Artifacts source is unchanged from 0.77.3; FS removes helpers Backup does not use.
Held-directory capabilities, bounded reads, identity/locks and the retained-file
creation/publication implementations used here remain unchanged. Only the two
Host lock entries changed from the pre-adoption graph; unrelated versions remain.
Archives, prior manifest/lock and selected metadata are under
`target/evidence/host-0.78.0/`. Public Host type identities require the minor bump.

The temporary sibling overrides and local exception are removed. No sibling
checkout or shared target is needed. Old local evidence remains under
`target/evidence/local-host-0.77.3/`; the 0.77.3 registry checkpoint evidence is under
`target/evidence/registry-host-0.77.3/`.
[Tooling #358](https://github.com/dragginzgame/ic-tooling/issues/358) retains producer
provenance. Platform 0.33.2/Types 0.20.4 are selected. The `ic-protocol` feature
adds protocol/CBOR and upstream Candid dependencies through Platform; all 74 existing
locked package identities remain unchanged, with 41 new transitive packages.
No Agent, HTTP runtime, Canic control-plane or harness dependency was added.
Feature metadata, the prior lock and independent Node request-hash evidence remain
under `target/evidence/status-call-binding/`. Native supplier identities stay private.

Types 0.20.4 was already locked on this turn's admission and is now qualified.
Its registry archive matches checksum `a8df2753db752bd0c29fa87353b98613465a5f0c5269b6fb8dd0e45ea5c12a22`
and source `343e3ae5a42fd8274330fa2171f5a9d53beab1dc`; source files equal 0.20.3.
Only Types changed against the retained Host-adoption graph. Archive/graph evidence
is under `target/evidence/types-0.20.4/`; no manifest minimum or feature changed.

Toko Tooling 0.43.1 and Rust 1.100.0 remain selected. The authenticated 51-file,
nine-entrypoint refresh changed only the engineering baseline, not scripts,
catalogs or hooks. Snapshot/tool admission, dependency/LOC callers and locked
metadata passed; evidence is under `target/evidence/toko-tooling-0.43.1/`.

## Next work and qualification

Continue the single-canister host library: capture an already-stopped canister;
restore to the same network/release/existing ID and leave it stopped. Canic owns
membership, routing, maintenance continuity, application settlement, signing and
reopening. Independent targets use its concurrency limit. Local bundles are v1;
Blob is v2. No scheduler, catalog database or generic storage trait is selected.

The [direct-provider review](../design/sibling-integration.md#direct-provider-continuation-review--2026-11-14)
records the next concrete requirement: retain actual certificates and qualify
historical authentication under original trust/admitted time. Legacy's reply
digest is not retained proof; Agent's current-clock verifier cannot be used
unchanged for old receipts. Preserve bounded one-shot transport, no hidden
retry/polling, and separately accounted original-request reconciliation. Canic
still needs its own executor receipt protocol; direct-host proof does not qualify it.
The user selected Backup-only source edits and requested cross-repository issues:
[Platform #162](https://github.com/dragginzgame/ic-platform/issues/162) owns bounded
historical certificate verification; [Platform #163](https://github.com/dragginzgame/ic-platform/issues/163)
owns Wasm snapshot DTOs/codecs; [Canic #1](https://github.com/dragginzgame/canic/issues/1)
owns its adapter, executor receipts and fenced recovery. The first blocks the direct
provider; the latter two concern the Canic route. No supplier source changes or
local cryptographic workaround were made. Existing Tooling #358 remains the Host
provenance tracker; no duplicate tooling issue was opened.

Focused legacy submission, reserved-request and HTTP boundary tests were reviewed
alongside `a860d144` history. Provenance and changed assumptions are in the capture
contract; no legacy implementation was copied. On Host 0.78.0, all 46 inventory/
payload/capture-intent tests, 10 ledger tests and 21 status continuation tests passed,
along with eight documentation examples, strict package Clippy and warning-denying
Rustdoc. The selections exercise retained publication, unsafe/replaced files,
bounded hashing, exact records, locking, root rename and acknowledged owner death.
No source adaptation was needed for the supplier removals. The full library suite
and hosted macOS were not rerun. Original plan/frame/reply fixtures remain unchanged;
status tests use real envelope framing with deliberately unauthenticated signatures.
Native fixtures, including failures, remain under `target/local-store-tests/`.

On Types 0.20.4, 57 focused tests passed: 16 capture-intent, 21 status continuation,
eight snapshot, eleven transfer-planning and the golden manifest case. Nine doc
examples, strict package Clippy and warning-denying Rustdoc passed. Original wire,
plan and request fixtures remain unchanged. Snapshot provenance and codec migration
1af7e219 are in the transfer contract. The patch supplies neither #162 nor #163;
both remained open at inspection. No actual IC capture is qualified by these tests.

These results do not qualify provider authentication/application safety, actual IC
effects, physical power loss, Canic or hosted macOS. Delivery is GitHub-only;
no release runner, tag or registry publication is selected.
