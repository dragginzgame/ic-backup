# What to retain from Backup Legacy

Assessment: 2026-11-14, against `dragginzgame/ic-backup-legacy` commit
`4ae087fad8c23de712aa60bf569bde4b2b932651`. Its handoff identifies 0.67.2 as the
last delivered library release. This is a source/contract review, not a fresh run
of legacy tests or a claim that its published consumers have been exhaustively
inventoried. No product implementation existed at this assessment; the
[current handoff](../status/current.md) records subsequent implementation.

Recommendation: preserve the safety contract and protocol knowledge, replace the
workflow/storage composition for the selected single-canister scope, and expose
only the operations needed by independent consumers and Canic. Do not port either
legacy crate wholesale. “Leave behind” means exclude from the new implementation;
it never means delete legacy artifacts, journals or unfinished recovery obligations.

## What legacy actually delivers

Legacy is more than codecs and disconnected helpers. Its public
[standalone assembly](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup-agent/src/standalone/README.md)
already composes capture, download, durable publication, allocation, upload, load,
application acceptance, start and reference release. It supplies inspection and
cold continuation of original stages. Required application callbacks and a fixed
small transfer profile limit its general usability; those limitations do not erase
the implemented round trip.

Its [qualification guide](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/docs/qualification/pocketic-qualification.md)
describes real simulator ingress, independently built consumers and deliberately
lost replies. Fixture safety applies to inspected fixture programs, not arbitrary
Canic applications. The new implementation must reproduce relevant proofs.
Historical pass counts and release receipts are not new-product qualification.

## Retain the guarantees

| Legacy asset | What the new library needs | Implementation owner |
| --- | --- | --- |
| Original intent and bounded reservations | Persist the exact operation and consume authority before dispatch. Cancellation, missing replies and process death never restore allowance. | Backup operation ledger and workflow. |
| Exact outcome association | Bind sender, target, method, original request and authenticated response. Decode success, matching bytes or a new snapshot ID alone cannot settle uncertainty. | Provider authentication plus Backup admission. |
| One submission, explicit reconciliation | Separate pending effects from observations. Never repeat an uncertain take, allocation, upload or load. Terminal replay performs no network effects. | Backup workflow; route provider performs one call. |
| Full snapshot representation | Preserve module, heap, stable memory, globals, certified data, chunk store and supported lifecycle metadata. Preserve exact bits and optional-field meaning. | Platform DTO/codec plus Backup's product projection. |
| Exact coverage | Reject gaps, overlaps, wrong lengths, duplicate chunks and wrong source associations; distinguish zero-length regions from named empty chunks. | Backup transfer policy. |
| Durable publication and source custody | Admit a closed artifact tree, hash actual bytes, retain partial writes and distinguish visible publication from confirmed durability. Verify the source before destructive use. | Backup store using Host FS/Artifacts. |
| Fresh application authority | Stopped/controller observations do not establish safe state rewind or settle payments. Preserve a mandatory integration-owned admission boundary. | Canic or standalone application integration. |
| Retained failure evidence | Preserve signed requests, raw arguments/replies, pending history, partial bytes and restore-source references. Typed errors must not discard available post-effect evidence. | Backup store and bounded error/result surface. |
| Supplier boundaries | Import external IC types/codecs through Platform, filesystem/hash primitives through Host, and simulator infrastructure through private Testkit qualification. | Existing sibling owners, with Backup-specific policy local. |

The [implementation contracts](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/docs/architecture/implementation-contracts.md)
are the safety checklist. Preserve the properties above rather than every legacy
record name, callback signature, error nesting or numeric constant.

## Adapt the useful mechanisms

| Legacy mechanism | Decision for the new design | Why |
| --- | --- | --- |
| Core model/policy/ops/workflow split | Keep the dependency direction internally; expose opaque operation handles and read-only results. | Pure policy and explicit effects help review. Publishing the entire internal engine is unnecessary for the selected consumer. |
| Data coverage model | Reuse its compact cursors/chunk membership idea; bind durable progress to the new ledger. | Coverage can stay bounded without retaining all payloads in memory. Legacy explicitly says its ephemeral coverage is not a resume journal. |
| Metadata/data admission | Carry forward the rejection cases and exact-value rules using current Platform APIs. Re-derive product limits and golden tests. | Missing globals, NaNs, oversized vectors, overflow and empty chunks are real protocol problems. A second Candid implementation is unnecessary. |
| Direct Agent transport | Retain one shared dispatch implementation, bounded replies, supplied trust and explicit no-retry configuration; adapt request ownership and external signing. | The existing constructor takes an SDK identity. Canic's external signer and manager sender require a broader explicit execution binding. |
| Required application callbacks | Preserve semantic obligations; separate IC authentication from application admission, and leave application execution in Canic. | Consumers should not have to reconstruct transport authentication or embed the legacy stage engine. |
| Capture/download/upload/load sequencing | Keep effect order and verification gates; use fixed product phases. | A single canister needs ordering, not a public dependency-graph framework. |
| Learned metadata and allocated IDs | Commit exact derived facts to the operation ledger before dependent work. | Unknown snapshot size and newly allocated IDs remain real dependencies even after stage-binding files disappear. |
| Artifact completion | Retain an immutable manifest and exact content/evidence commitments; design a new format. | A smaller journal does not make a checksum declaration prove durable completion or authentic provenance. |
| Restore-source references | Retain references from restore creation. Defer reference release and pruning. | `LoadedStopped` cannot establish application acceptance or settled external obligations. |

Concrete source anchors:
[coverage](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_snapshot_coverage/mod.rs),
[metadata admission](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/model/ic_snapshot_metadata/mod.rs),
[shared submission](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup-agent/src/submission/mod.rs),
[Agent configuration](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup-agent/src/lib.rs).
These are behavioral references, not a mandate to copy their implementations.

## Replace the composition machinery

The principal redesign is the relationship between plans, journals and stages.
Legacy supports inventories, dependency-bearing operation rows, stage catalogs,
learned child plans, ancestor bindings, per-operation journals and settlement
checkpoints. Those mechanisms have actual callers and recovery meaning; they are
not dead code. They are nevertheless too broad a public contract for this restart.

The proposed replacement is one immutable plan and one append-only spending ledger
per capture or restore, with separate immutable evidence and a bundle manifest.
A target claim provides local exclusion. Original step identities and learned
results live under the same operation authority. This is a new custody/recovery
contract; it is not a safe in-place optimization of old records.

[Audit 07](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/docs/audits/mega/07/report.md)
measured eight-operation reservation work opening 64 attempt journals, including
56 peer opens, under full-original-set admission. That supports changing the
storage contract for larger transfers. It does not prove end-to-end backup
throughput, nor does it justify deleting peer checks while retaining the old
contract. Benchmark the replacement with real ledger/evidence IO before claiming
it is faster.

Specifically leave out:

- Public inventory forests, descendant-selection helpers and generic dependency
  graphs. Canic supplies one selected target; membership remains application-owned.
- Separate workflow catalogs, stage directories, learned-edge files and checkpoint
  APIs as consumer-managed concepts. Keep their necessary causal commitments inside
  the operation owner.
- Creating a restore catalog before capture. A later restore gets a separately
  reviewed plan bound to its verified source and current authority.
- The standalone driver's fixed eight-read/eight-write profile. Use explicit finite
  limits derived from the actual transfer and provider route. The draft 1 GiB and
  8,192-call ceilings still need measurement; they are not inherited guarantees.
- The broad low-level public model/ops/workflow composition API. Introduce a public
  extension only for a concrete independent or Canic consumer need.

Do not collapse everything into one record or one untyped RPC. Signed intent exists
before a reply; a manifest declares completed artifacts; references protect later
consumers. Their roles survive even if their physical representation changes.

## Leave out of the first product

| Surface | Disposition |
| --- | --- |
| Stop/start orchestration and post-start application readiness | Canic/application-owned. Backup requires a stopped target and returns `LoadedStopped`. |
| Generic opaque application ingress | Keep application RPC, fences, approvals and executor protocol in Canic. Backup's provider supports its closed snapshot workflow. |
| Multi-canister checkpoint consistency | Defer. Repeated single-canister backups do not establish a distributed checkpoint. |
| State-based capture/allocation/lifecycle reconciliation | Defer qualified inventory/status/readback attribution mechanisms. Initially recover authenticated original-request results, otherwise retain uncertainty. This sacrifices some recoverability and must be stated. |
| Terminal application acceptance and source-reference release | Defer from Backup v1; retain references and let Canic own acceptance. Do not equate successful load with safe reopening. |
| Cross-ID/release restore, allocation of canisters, controller changes | Exclude from the chosen same-ID, same-release operation. |
| Snapshot replacement/deletion, pruning, scheduling, remote stores, compression/encryption and CLI | Defer until required independently. Some are legacy design proposals, not delivered features being removed. |
| Generic command backends, arbitrary-directory completion and restore-copy staging | Do not revive them. Several were already retired in legacy; their historical artifacts remain preserved. |
| Legacy schemas, hash domains and compatibility readers | No automatic import. Existing operations continue with their original qualified release/integration. Any migration needs a separate explicit contract. |
| Legacy release adapters, audit backlog and blanket validation roster | Use the adopted Toko baseline and checks for implemented behavior. Preserve legacy release evidence in legacy; do not duplicate its delivery machinery in a scaffold. |

## Avoid false simplifications

[Audit 09](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/docs/audits/mega/09/report.md)
and the inspected submission source show that the typed management ports already
share one network implementation. Replacing six typed adapters with an arbitrary
method/bytes API would lose associations without removing six clients. The new
single transport boundary must retain a closed typed request family and distinct
original-effect/observation authority.

[Audit 08](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/docs/audits/mega/08/report.md)
found no unused persisted file family in its inspected scope and did not establish
external public-API non-use. Its findings are historical: optional selection was
subsequently removed according to the current contracts, and the inspected
[upload session](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/ops/persistence/download_journal/ic_snapshot_artifact/upload/session/mod.rs)
already compares stamps incrementally. Do not schedule those old candidates as
new fixes or claim the old storage files were pointless.

Likewise, a size/mtime stamp detects some changes but is not a content hash.
Changing to held descriptors does not automatically prove immunity to same-user
writes. Any reduction in repeated source verification needs explicit custody,
continued payload commitments, crash behavior and qualification.

## Carry forward tests by failure mode

Re-author focused tests against the new implementation. Preserve independently
specified protocol vectors where applicable, with their provenance; do not make
legacy record hashes the acceptance target for an intentionally new format.

| Test group | Required new proof |
| --- | --- |
| Intent and accounting | Crash after reservation, cancelled await, lost/oversized reply, exhausted observation budget, unchanged original authority after reopen. |
| Bytes and coverage | Exact metadata/float bits, invalid optional values, wrong ID/target, overflow, gap/overlap/duplicate, empty region/chunk, partial download and corrupt source. |
| Filesystem custody | Contention, symlink/replacement, partial evidence, truncated ledger, ENOSPC and publication failure before/after visibility; retain originals. |
| Actual IC effects | Capture/download and upload/readback/load through production transport, with complete independent state checks and lost replies after real ingress. |
| Public usability | A normal-dependency Rust program uses the public library without internal journals or test-only product branches. |
| Canic fit | External signing, actual manager-only controller topology, retained executor results across both network hops, and application fencing before reopening. |

Do not copy test totals, the full old gate matrix, or start/release fixtures for
features deliberately excluded. Keep product unit/HTTP tests with their packages
and real IC tests in private Testkit qualification. Add process and host coverage
for the new storage mechanism rather than assuming legacy's filesystem tests
validate a different ledger.

## Consequence for the current design

The [operation contract](operation-contract.md) already carries most of these
choices. Its new request-status reconciliation, durable extent continuation,
1 GiB profile and manager executor route are proposals beyond the old standalone
assembly's guarantees; they need fresh qualification. The new stricter lifecycle
metadata admission is also a deliberate initial profile restriction, not an
external decoder requirement inherited from legacy.

The next useful implementation work is the exact operation-store contract followed
by a real capture/verify consumer. Keep restore, Canic authority and failure
acceptance visible while doing it. Success is a smaller usable library preserving
the recovery guarantees above, not reproducing the old public surface or merely
collecting new codec helpers.
