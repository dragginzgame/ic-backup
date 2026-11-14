# Current handoff — 2026-11-14

Work lives at `/home/adam/projects/ic-backup`, branch `main`; the former legacy
local path was absent. Frozen legacy is read-only at `dragginzgame/ic-backup-legacy`.
No original journals, bundles or obligations were imported, migrated or cleaned up.

The original rewrite was pushed at
[cd3118d](https://github.com/dragginzgame/ic-backup/commit/cd3118d22cee569a3dfa3a5b36b4d2a6cf56380e).
The status workflow/inspection, registry Host adoption and Toko refresh are
checkpointed at [cc592cf](https://github.com/dragginzgame/ic-backup/commit/cc592cfb445fd94d9e93f951c721c965b20e0eb4)
as unpublished 0.100.0. At the maintainer's request, 0.100.1 starts the next
development iteration. This patch increment introduces no new compatibility
break; future incompatible changes must promote the pending version. Delivery
targets the existing `main` branch of `dragginzgame/ic-backup`. All packages remain
`publish = false`; no registry publication or release tag is selected.

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

`StatusProvider` is a trusted integration contract; no concrete provider is
supplied or qualified. Native injected providers prove ordering only. Canic intent
declarations round-trip, but status execution refuses that route pending its
executor protocol. Pure [transfer/decoding](../architecture/capture-transfer.md)
retains Platform identities, metadata/float bits, exact region lengths and chunk
hashes. Typed/raw status policy requires stopped state, approved module and caller
controller membership; these helpers do not authenticate observations.

Snapshot capture, reconciliation, completion publication, full bundle verification,
upload/readback/load, CLI and simulator remain unimplemented. This is not yet an
independently usable backup/restore product.

## Dependencies and evidence

Host FS/Artifacts 0.77.3 now resolve from verified registry archives. Their
archive hashes match the registry index; the held-directory opener is present.
The previously tested local checkout included unused helper removals, so the
registry graph was qualified independently with all 119 tests, eight doctests,
strict Clippy and Rustdoc. Only the two Host lock entries gained registry sources
and checksums; their versions and other selected dependencies were preserved.

The temporary sibling overrides and local exception are removed. No sibling
checkout or shared target is needed. Old local evidence remains under
`target/evidence/local-host-0.77.3/`; current graph/adoption evidence is under
`target/evidence/registry-host-0.77.3/`.
[Tooling #358](https://github.com/dragginzgame/ic-tooling/issues/358) retains producer
provenance. Platform 0.33.2/Types 0.20.3 and management-only features remain
selected; no Agent, runtime, Canic control-plane or harness dependency was added.

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

Focused legacy source/tests and `a860d144` history were reviewed before this work;
provenance is in the existing contracts and sibling review. No legacy implementation
was copied. All 119 library tests, eight documentation examples, strict Clippy and
warning-denying Rustdoc passed on the current graph. Original plan/frame/wire
fixtures remain unchanged. Native fixtures, including failures, remain under
`target/local-store-tests/`; existing compiler/graph evidence stays under `target/evidence/`.

These results do not qualify provider authentication/application safety, actual IC
effects, physical power loss, Canic or hosted macOS. Delivery is GitHub-only;
no release runner, tag or registry publication is selected.
