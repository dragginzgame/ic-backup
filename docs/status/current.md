# Current handoff — 2026-11-14

Work lives at `/home/adam/projects/ic-backup`, branch `main`; the former legacy
local path was absent. Frozen legacy is read-only at `dragginzgame/ic-backup-legacy`.
No original journals, bundles or obligations were imported, migrated or cleaned up.

The completed 0.101.0 checkpoint was pushed and remotely verified at
[5292a61](https://github.com/dragginzgame/ic-backup/commit/5292a61317dbbb619699724c4b2197e1ae6f799d).
Development opened as 0.101.1 at
[e3e9c67](https://github.com/dragginzgame/ic-backup/commit/e3e9c670ec066a952b5b56b1371c6e951b9bc197).
The 0.101.1 checkpoint completes HTTP submission, direct-key authentication,
Host HTTP adoption and the Tooling refresh. The user selected GitHub-only delivery
to `origin/main`, followed by the next compatible development version. All
packages remain `publish = false`; no registry publication, release tag or
deployment is authorized. A future breaking change must advance the minor.

## Implemented scope and safeguards

The [inventory contract](../architecture/local-inventory.md) owns private stores,
bounded completion listings/latest backup per target, manifest-bound payload
verification and held payload sources. Source reads recheck original custody and
block hashes; failures invalidate the handle. Inventory declarations and payload
integrity do not authenticate capture evidence or grant restore authority.

The [capture contract](../architecture/capture-intent.md) owns immutable approved
plans, durable target claims and bounded chained ledgers under held original
custody. Writes invalidate before persistence; uncertain writes require reopening.
Complete uncertain frames stay spent; partial frames and orphans block continuation.
Missing owners/locks never grant fresh authority. Root rename follows held
folders; replacement custody refuses. Reads never repair or discard evidence.

`advance_status` reserves signing before provider invocation, retains exact signed
bytes, reserves one submission and retains authenticated evidence before settlement.
Cold pending replay never signs/submits. Prepared replay uses original bytes after
current admission. Settled replay authenticates locally and checks stopped state,
approved module and original caller/controller membership. `inspect_status` reports
original admission/stage/spending without effects; `Settled` alone proves nothing.

The certificate helpers authenticate original replies/rejections against original
root DER, effective target, request ID and exact bytes. Certificate/delegation time
windows are retained before submission and reused without reading a clock. Strict
bounds apply; pending/forgotten/pruned outcomes never become success or refunds.

`authenticate_status_request` now checks direct Ed25519/secp256k1 signatures,
original sender/content/request ID and complete request bounds through Platform.
It reads no clock and changes no bytes. Delegations, sender-info and other identity
forms refuse. [The direct-key contract](../architecture/capture-intent.md#direct-key-status-request-authentication)
owns exact formats, stricter verification, legacy review and qualification limits.
`StatusHttpTransport` calls it before HTTP and provides one bounded submission
with original network/root/target, deadlines, body caps and authenticated results.
It disables retries, redirects, proxies and polling; cancellation never refunds.
Caller futures require Tokio IO/timers. The [HTTP contract](../architecture/capture-intent.md#direct-status-http-helper)
owns transport/runtime details. Signing, original time policy, provider commitments
and application continuity still require a complete qualified `StatusProvider`.
Canic execution refuses pending its executor/receipt protocol.

The [transfer contract](../architecture/capture-transfer.md) owns bounded snapshot
list/take preflight, metadata/data decoding, deterministic schedules and ordered
uploaded-state/readback comparison against held source bytes. Comparisons preserve
exact state/float bits and require every extent/chunk, including empty chunks.
They provide no authenticated remote readback or load authority.
Snapshot capture/reconciliation, completion writing, complete bundle-evidence
verification, upload/readback/load effects, CLI and simulator remain unimplemented.
This is not yet an independently usable backup/restore product.

## Dependencies and current evidence

Host FS/Artifacts/HTTP 0.79.0, Platform 0.33.5/Types 0.20.4, Toko Tooling 0.43.2
and Rust/Cargo 1.100.0 are selected. The incoming Platform 0.33.5 lock selection
was preserved; its manifest minimum remains 0.33.4. No sibling patch, shared
target, Agent or Canic runtime dependency is present. Backup now imports Reqwest
0.13.5 only through Host HTTP and uses its bounded complete-body reader; the local
chunk loop and direct Reqwest dependency are removed. Rustls uses platform
certificate verification and native trust configuration, replacing the earlier
bundled WebPKI roots. IC trust remains separately provisioned. Tokio is 1.54.1.

`target/evidence/host-http-0.79.0/` retains the authenticated published archive,
before/proposed/admitted graphs, compiler identities and consumer check logs.
Archive SHA-256 is `d33de424aa5afc596c0dace915e50c7b03057ff52cad64bd3451f9ee2380c3cc`;
source is `cd85d922bcaba4d09727eb16d00b2e5a4c4f287b`, supplying
[Tooling #360](https://github.com/dragginzgame/ic-tooling/issues/360).
All 62 focused status continuation tests, strict package Clippy, warning-denying
Rustdoc and all 15 documentation examples pass after adoption. This includes
14 HTTP/TLS cases, nine request-signature cases, certificate authentication and
durable replay. Exact/empty body boundaries and a real TLS ClientHello with
plaintext-peer refusal qualify the new reader/TLS selection at the consumer.
Prior request-authentication evidence remains under `target/evidence/status-request/`.
Original requests/failed fixtures remain under `target/local-store-tests/`.
These are native crypto/HTTP/persistence checks, not actual IC acceptance, external
signer custody, application continuity, completed trusted TLS handshakes, power-loss
or hosted-macOS qualification. The full product suite was not selected.

The Tooling 0.43.2 exporter retained all 51 managed files and nine entrypoints
byte-for-byte; only its exact pin, lock entry and snapshot identity changed.
Snapshot/tools admission, locked metadata, dependency listing/update preview and
LOC checks passed. `target/evidence/toko-tooling-refresh/` owns that evidence;
`target/evidence/status-http/` retains the original transport graph/review checks.
Historical supplier/qualification evidence remains in its existing directories and
the [checkpoint handoff](https://github.com/dragginzgame/ic-backup/blob/b95253375961f111df1eb648bd3479240846a13c/docs/status/current.md).
Stored formats, frozen fixtures and original recovery authority remain unchanged.

## Next work

Compose bounded external signing with live time/provider/application admission
using the supplied request, certificate and HTTP helpers. Qualify actual IC/HTTPS
acceptance separately. Lost replies require accounted original-request reconciliation,
never hidden polling/retry. Complete bundle admission must bind original plan,
ledger, metadata and source requests to the trusted manifest and payload schedule.
Canic owns membership, concurrency, routing, maintenance, external settlement and
reopening. V1 remains one already-stopped canister, local bundles and same-release
restore to the same existing ID, left stopped; Blob storage is v2.
[Platform #162](https://github.com/dragginzgame/ic-platform/issues/162) and
[#163](https://github.com/dragginzgame/ic-platform/issues/163) are supplied in 0.33.4;
no supplier gap is claimed for the direct-key profile. [Canic #1](https://github.com/dragginzgame/canic/issues/1)
owns executor receipts/fenced recovery. Source edits remain Backup-only; file
concrete supplier issues when needed rather than editing siblings.
