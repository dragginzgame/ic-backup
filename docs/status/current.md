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
Request binding, Host/Types adoption and snapshot preflight were pushed at
[d7bde46](https://github.com/dragginzgame/ic-backup/commit/d7bde46dc5c27d2a9ff231c631ffe17db1b8fcf7).
Development 0.101.0 was pushed at [292d7ca](https://github.com/dragginzgame/ic-backup/commit/292d7cad1f827f83270dbbf60943e0297597af41)
because Host 0.78 changes public digest/error
type identities. Consumers must align Host dependencies; stored records and
recovery authority are unchanged. All packages remain `publish = false`;
no registry publication or release tag is selected.

## Implemented behavior

The [inventory contract](../architecture/local-inventory.md) owns private store
initialization, bounded completion listings and latest backup per target. These
are passive completion declarations. `verify_payload` checks membership, lengths,
hashes and observed custody against a trusted manifest; it does not establish
capture authenticity, future custody or restore authority.
`open_payload` now holds original payload descriptors and an ephemeral 1 MiB block
hash index. Its bounded source reads check exact canister/snapshot/range binding,
selected block bytes and original file/tree custody. IO/integrity failures
invalidate the handle. No new record format, dependency or remote effect is added;
complete capture-evidence/metadata association remains outside this payload reader.

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

Local `decode_uploaded_metadata` now compares bounded uploaded metadata against
admitted capture state: exact global bits/variants, sizes, certified bytes, timer,
hook and complete chunk set. It returns destination provenance separately and
grants no load authority. `begin_readback` now adds bounded ordered comparison of
all destination data against caller-held source slices. It refuses skipped or
repeated positions and same-length corruption; `finish` requires all regions and
named chunks, including empty chunks. This is ephemeral comparison coverage,
not authenticated evidence or a durable receipt. Source data, plan and ledger
remain unchanged.

Snapshot capture, reconciliation, completion publication, full bundle verification,
upload/remote readback/load, CLI and simulator remain unimplemented. This is not yet an
independently usable backup/restore product.

## Dependencies and evidence

Host FS/Artifacts 0.79.0 now resolve from verified registry archives, from supplier
`1a7b1521c56531e25b7bf75e3b79e27fe6ee93a2`. Both hashes match the non-yanked
registry entries and both complete Rust source trees equal 0.78.0. The supplier
release retires HTTP/inspector packages Backup does not use. FS now depends on
Artifacts 0.79; the selected graph contains exactly one version of each.
Only those two lock entries changed against the pre-adoption graph; the incoming
Platform 0.33.3 selection and unrelated versions remain. Root requirements select
0.79 because public digest/error identities cross Backup's API. The existing
unpublished 0.101.0 minor covers this change; no further version bump was made.
Archives, registry rows, prior manifest/lock and disposable/root metadata are under
`target/evidence/host-0.79.0/`; prior 0.78 evidence remains intact.
Locked metadata admits the original feature selection. Host 0.79 qualification
passes 85 focused tests (54 store/inventory/payload/capture, 21 status continuation,
10 ledger), eleven documentation examples, strict package Clippy and
warning-denying Rustdoc. No hosted macOS, authenticated IC or release workflow ran.

The temporary sibling overrides and local exception are removed. No sibling
checkout or shared target is needed. Old local evidence remains under
`target/evidence/local-host-0.77.3/`; the 0.77.3 registry checkpoint evidence is under
`target/evidence/registry-host-0.77.3/`.
[Tooling #358](https://github.com/dragginzgame/ic-tooling/issues/358) retains producer
provenance. Platform 0.33.4/Types 0.20.4 are selected. The `ic-protocol` feature
adds protocol/CBOR and upstream Candid dependencies through Platform; all 74 existing
locked package identities remain unchanged, with 41 new transitive packages.
No Agent, HTTP runtime, Canic control-plane or harness dependency was added.
Feature metadata, the prior lock and independent Node request-hash evidence remain
under `target/evidence/status-call-binding/`. Native supplier identities stay private.

Types 0.20.4 is qualified through the existing Platform dependency.
Its registry archive matches checksum `a8df2753db752bd0c29fa87353b98613465a5f0c5269b6fb8dd0e45ea5c12a22`
and source `343e3ae5a42fd8274330fa2171f5a9d53beab1dc`; source files equal 0.20.3.
Only Types changed against the retained Host-adoption graph. Archive/graph evidence
is under `target/evidence/types-0.20.4/`; no manifest minimum or feature changed.

Incoming Platform 0.33.3 lock changes were preserved. Its cached archive matches
the registry checksum `74c6e194ce3a80487320edef0041e286fd89d6d96e8a9f641811c7e38b856e51`
and source `9035c3f73a9bd8e866737418daa3b741f14e3fad`. Management DTO and Candid codec
source trees equal 0.33.2; registry/metadata evidence is under `target/evidence/platform-0.33.3/`.

The incoming 0.33.4 lock update was retained during GitHub delivery preparation.
Its cached archive matches registry checksum
`959cfca0933238dfb426f3deb356b7d5c27ee594f714cb61eca1a610b3e4ae44`
and supplier `b5c45050e9c4cde46631ae7df4b6e075facf809b`. Snapshot definitions move
from private `native` to `snapshot` modules with unchanged native fields/schema
bytes and portable snapshot exports. The new `certification-verify` feature
supplies historical authentication but is not yet selected by Backup. Current
features and other locked packages remain unchanged; registry and metadata
evidence is under `target/evidence/platform-0.33.4/`.
The combined Host 0.79/Platform 0.33.4 checkpoint passes 84 focused tests
(47 transfer, eight snapshot, 21 status continuation and eight held-source cases),
eleven documentation examples, strict Clippy and warning-denying Rustdoc.
It remains unpublished development version 0.101.0; GitHub delivery grants no
capture/restore qualification or registry publication.

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
owns its adapter, executor receipts and fenced recovery. Platform #162 and #163
are now closed and supplied in 0.33.4; Backup still needs concrete-provider
integration and qualification, and Canic needs its executor protocol. No supplier source changes or
local cryptographic workaround were made. Existing Tooling #358 remains the Host
provenance tracker; no duplicate tooling issue was opened.

Earlier Host/status qualification and legacy history review remain recorded in the
capture contract and d7bde46 handoff. Original plan/frame/reply fixtures remain
unchanged; injected status providers establish ordering, not authentication.
Native fixtures, including failures, remain under `target/local-store-tests/`.

On Types 0.20.4, 57 focused tests passed: 16 capture-intent, 21 status continuation,
eight snapshot, eleven transfer-planning and the golden manifest case. Nine doc
examples, strict package Clippy and warning-denying Rustdoc passed. Original wire,
plan and request fixtures remain unchanged. Snapshot provenance and codec migration
1af7e219 are in the transfer contract. The patch supplies neither #162 nor #163;
both remained open at inspection. No actual IC capture is qualified by these tests.

The local readback batch passes 47 transfer tests (eight metadata and eight payload
readback cases), ten documentation examples, strict package Clippy and
warning-denying Rustdoc on Platform 0.33.3. Provenance, changed optional-state rules,
exact limits and caller custody requirements are in the transfer contract. The
payload comparator reuses the existing decoder and one shared length/hash check;
it adds no dependency, record or remote effect. Full suite/macOS and authenticated
IC journeys were not run.

The held-source batch passes 23 focused native payload tests, including eight new
source cases and the existing legacy closing checks, plus eleven documentation
examples, strict package Clippy and warning-denying Rustdoc. Source reads compose with
ordered readback using synthetic Platform DTOs. Native Linux results do not prove
hosted macOS descriptor capacity or any IC effect. Maximum retained payload
descriptors are 1,027; OS resource failures remain explicit refusals.

Platform's supplier blockers are resolved; concrete-provider work can now assess
the published historical verifier under original trust/time/request bindings.
The next source integration is complete original bundle-evidence admission: bind plan, ledger, raw metadata
and its exact source request to the trusted manifest and payload schedule. The
held payload reader alone does not establish those associations. Keep that work
separate from provider authentication and future restore attempt accounting.

These results do not qualify provider authentication/application safety, actual IC
effects, physical power loss, Canic or hosted macOS. Delivery is GitHub-only;
no release runner, tag or registry publication is selected.
