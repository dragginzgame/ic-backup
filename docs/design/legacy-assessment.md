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

## Consequence for the design at the initial assessment

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

## Implemented replacement comparison — 2026-11-16

This follow-up compares the pending local replacement with frozen legacy
`4ae087fad8c23de712aa60bf569bde4b2b932651`, including its Agent and qualification
consumer. It supersedes the initial assessment's implementation-status statements,
not its recovery obligations. It is a source, contract and existing-test-evidence
review; no production code was changed and neither test suite was rerun. Current
passing logs are under `target/evidence/backup-completion/`: 229 native tests,
seven actual-IC journeys and five doctests. Those passes do not cover every finding
below. Legacy's historical qualification is not a fresh pass on this host.

The four findings below describe the pre-fix review. Their implemented corrections
and fresh evidence are recorded in [the resolution](#review-corrections--2026-11-16).

### Comparable size measurements

Fresh commands: `toko-cloc -d 1 .` and the same command on the frozen legacy
checkout. Counts are Rust code excluding comments/blanks, with zero unclassified
lines; they exclude documentation, vendored tooling and dependency implementations.
The current total includes untracked replacement source, not only the Git diff.

| Scope | Legacy | Current | Reduction |
| --- | ---: | ---: | ---: |
| Repository production, including legacy Agent | 15,646 | 7,822 | 50.01% |
| Repository total, including tests/qualification/examples | 44,877 | 20,095 | 55.22% |
| `ic-backup` crate production only | 13,579 | 7,822 | 42.40% |
| `ic-backup` crate total | 37,062 | 19,117 | 48.42% |
| Repository tests | 29,152 | 12,273 | 57.90% |

The 45% claim is true for total code and repository production, but false for
crate-only production. New Backup includes its direct HTTP transport; legacy
placed that transport and standalone assembly in Agent. Repository production is
therefore the more useful maintained-library comparison. It still is not feature
parity: the exclusions below account for some of the reduction, and the unfinished
Canic adapter/executor/fence will add code outside this repository. No combined
Backup-plus-Canic cost or percentage of savings attributable solely to refactoring
has been established. Against the preserved pre-replacement 10,259 / 29,474
checkpoint, the current reductions are 23.75% production / 31.82% total.

### Capabilities and obligations

| Area | Legacy implementation | Current implementation / disposition |
| --- | --- | --- |
| Capture and restore | Standalone Agent composes catalogs/stages through download, upload, load, acceptance, start and reference release. | Two fixed procedures through exact local publication and `LoadedStopped`; the latter is not application acceptance. |
| Call mechanics | Shared Agent submission plus domain workflows, attempt journals and settlement checkpoints. | One sealed recorder owns reservations, exact request/result custody and explicit observation for both procedures. No parallel old production engine found. |
| Original authority | Original plan/context, attempts and independent result qualification. | Canonical immutable approval binds target/network/release/sender/trust/profile/finite limits; restore additionally binds original capture and manifest. |
| Recovery | Original stage continuation, explicit observations, no blind repeat, pending/Applied evidence. | Per-call continuation, one original-request read-state lookup per call within operation limits, local certificate import and clock-free completed replay. Post-reply retention gap below remains. |
| Snapshot state | Module/heap/stable, globals, certified data, timer/hook and chunk-store representation. | Same supported state is transferred and compared, including float bits and empty chunks; unavailable metadata and empty modules deliberately refuse. Size ceilings are not all usable end to end. |
| Coverage/source integrity | Original metadata binding, contiguous region cursors, chunk membership, source replay and closed artifact verification. | Fixed ordinal schedule, exact requests, module hash, all-region/chunk byte comparison, original source approval and closing file/tree checks. |
| Publication/history | Durable artifacts/manifests and workflow/source-reference records. | One closed operation journal, exact create-only manifest, bounded list/latest, authenticated older sources and completed-target succession. |
| Application lifecycle | Required callbacks plus explicit restore-safety lanes, fence/membership/obligation checks, acceptance/start/release. | Mandatory action guard and original witness commitment; integration must implement the substantive fence/rewind policy. No built-in stop/start, acceptance or release workflow. |
| Signing/execution | Caller-supplied SDK identity and Agent transport; broader low-level application ingress/provider surfaces. | External sign-only interface and direct Ed25519/secp256k1 authentication; delegated/P-256/WebAuthn/canister signatures refuse. Production signer and manager route remain unqualified. |
| Generic composition | Public inventories, selections, dependency plans, catalogs and low-level workflow composition. | One selected existing canister and fixed same-network/release restore. Fleet membership/coordination and arbitrary application calls are outside Backup. |
| Compatibility | Its own released formats and original recovery readers. | Explicit incompatible restart; no old-store reader/migration. Preserved legacy and intermediate readers remain required for their original obligations. |

Relevant current owners reviewed are
[the recorder](../../crates/ic-backup/src/store/operation/mod.rs),
[dispatch and recovery](../../crates/ic-backup/src/store/operation/runtime/mod.rs),
[capture policy](../../crates/ic-backup/src/store/capture/procedure/mod.rs),
[restore policy](../../crates/ic-backup/src/store/restore/procedure/mod.rs),
[restore custody](../../crates/ic-backup/src/store/restore/persistence/mod.rs),
[payload custody](../../crates/ic-backup/src/store/payload/mod.rs),
[metadata comparison](../../crates/ic-backup/src/store/capture/transfer/readback/mod.rs)
and [completion](../../crates/ic-backup/src/store/operation/completion/mod.rs).
Shared failure matrices, exact metadata tests, source replacement tests and the
actual-IC consumer support these findings; fewer test lines alone establish
neither better nor worse coverage. No missing coverage/hash/original-spending
check was found in these reviewed success and rejection paths. This is not an
exhaustive equivalence proof for all legacy APIs.

### Actionable findings

1. **High priority: preserve received evidence on post-reply failure.** In
   `store/operation/runtime/mod.rs`, `run` receives authenticated evidence, then
   performs source/custody checks and fallible retention before returning. Failures
   at lines 373–387 drop the owned response/proof; `OperationAdvanceError` carries
   no recovery bytes. `observe` has the same problem after its HTTP return.
   `capture/advance/http/mod.rs::submit_expected` also drops a received certificate
   on authentication, combined-evidence or final-deadline refusal. A local IO
   refusal before the result file is created can therefore lose the only returned
   certificate even though the IC call executed. Spending stays pending and no
   automatic repeat occurs, but offline outcome recovery loses evidence it had.
   Legacy Core's `workflow/qualified_receipt::retain_reply` and restore
   `AfterReply { acknowledgement, .. }` explicitly preserve the bounded reply;
   `failure_boundaries_retain_pending_or_applied_and_never_repeat_provider` asserts
   this for malformed replies and qualification/receipt failure. This is not a
   claim that every legacy transport failure retained bytes. Current
   `orphan_appearing_during_http_refuses_result_retention_without_repeating` tests
   refusal, but not return of the newly received proof. Smallest correction:
   carry bounded received evidence and original call identity in a redacted typed
   post-reply error when durable retention cannot succeed. Keep failed custody
   closed; do not append into a changed namespace, introduce another journal,
   auto-adopt orphan files or treat unauthenticated bytes as success. Add focused
   retention-failure and late-reply cases, including an actual-IC lost-local-write
   case whose recovery does not depend on the fixture proxy's private copy.

2. **Medium priority: qualify one coherent capture/restore size profile.** The
   local data decoder allows a 1 MiB vector and the extent/chunk declaration allows
   1 MiB, but the reused certificate verifier caps the entire certificate at
   1 MiB, including the reply and proof overhead. A full-sized data vector cannot
   pass that certificate boundary. Restore caps the complete request at 1 MiB and
   subtracts 4,096 bytes before payload admission (`restore/mod.rs:230`), leaving
   at most 1,044,480 payload bytes. Regions can use smaller restore extents;
   whole chunks cannot be split, so an otherwise admitted larger chunk has no
   restore plan under any current connection profile. Capture has no corresponding
   shared round-trip check. Legacy permits 1 MiB data chunks with a 3 MiB signed
   update cap and 4 MiB HTTP body cap. These are confirmed source-level capacity
   differences, not new actual-IC measurements at those boundaries. The current
   actual-IC fixture uses 16 KiB extents and 0-/3-byte chunks. Smallest correction:
   explicitly align data, certificate, complete-evidence and upload-envelope
   ceilings, then qualify boundary-sized whole chunks through capture and restore.
   Preserve finite limits and original approvals; do not advertise nominal 1 GiB
   payload or 1 MiB extent maxima as demonstrated usable capacity.

3. **Medium priority: the isolated consumer/process proof has narrowed.** The
   current [qualification manifest](../../qualification/ic-backup-qualification/Cargo.toml)
   puts Backup, Platform and Testkit together in `dev-dependencies`. Its
   [integration test](../../qualification/ic-backup-qualification/tests/capture.rs)
   uses the public API with production Backup compilation, but is not the separate
   normal-dependency executable/feature graph described by some current prose.
   Reopening after dropping handles and shutting down PocketIC is useful cold-file
   evidence; it is not restarting that client in a fresh process. A separate native
   child-death test does cover acknowledged signing reservation. Legacy
   `independent/build/mod.rs` builds a distinct normal-dependency program excluding
   Testkit/PocketIC, and its consumer runs in child processes. Retain the current
   tests, describe their actual scope, and restore one small isolated consumer
   proof for the selected capture/restore API before claiming equivalent consumer
   and process qualification. Registry-consumer proof remains release work.

4. **Low priority: present-tense documentation and naming drift.**
   [Local inventory](../architecture/local-inventory.md) still says restore and
   full source admission are unimplemented near lines 9 and 144, despite its later
   implemented-source addendum. The acceptance section of
   [the operation contract](operation-contract.md#implementation-acceptance)
   similarly calls restore source admission proposed. Shared restore execution
   still surfaces `CaptureError`/`StatusHttpTransport` and several capture/status
   descriptions. These are one implementation, not duplicate runtime paths, but
   the names obscure ownership. Correct current-status prose and qualification
   terminology together; keep historical sections explicitly dated. Rename shared
   internals/public names only as a deliberate coherent API change, without aliases
   or changing retained format domains just for wording.

The first two findings mean the earlier statement that no Backup-side functional
gaps remained was too broad. This review records them; it does not repair them or
authorize delivery. Canic's independently outstanding signing/executor/fence work
remains with its existing issues, not a new local issue framework.

### Improvement and residual complexity

The architectural improvement is concrete: consumers drive one fixed loop, the
two procedures share custody/spending/recovery, derived schedules are not separately
persisted, and completion has one owner. Original request authentication is local
and replayable, while application permission is checked for new dependent effects.
The reviewed dependency manifests keep Backup independent of Canic runtime/control
plane and keep Testkit outside production. No extra compatibility engine or
speculative provider registry was found. Removing generic catalogs and lifecycle
features also reduces scope; it is not all implementation efficiency.

Some repeated validation is intentional: journal framing/order and semantic replay
check different boundaries; source authenticity and closing file custody must both
survive. Do not delete these merely because conditions resemble one another.
However, each advance still checks retained files and replays the prior call
history, and appends recheck/sync earlier evidence. For fixed-size calls this
accumulates at least quadratic historical work across an operation. Retained IO
measurements show 227,756,038 certified-result bytes reread from approximately
1.1 MiB retained evidence in the earlier paired consumer. The measured 34% result
read / 33% journal read reductions compare two restart implementations with
capture plus status-only restore, not legacy versus today's full restore. There
is no comparable full-journey legacy/current throughput or large-scale benchmark.
The smaller source tree is easier to maintain; faster end-to-end backup and full
production readiness remain unproved.

Additional history inspected for this review: exact revisions
`f332118eeed000e4615e48155c162a4e113076a8` (receipt simplification),
`36789d26768b7b2eac2f5d4e8c29cc11bf80bdde` (receipt ownership change), and the
previously reviewed `d245b0f03a505ba5215d65c10bdebf1890c7dc37` and
`9f7b886a824eec67dda8f81792a560436419bb2a` (original evidence and verification-pass
ownership context). Current/frozen implementations and regression assertions,
rather than commit titles alone, support the comparisons. No legacy source or
test code was copied, and retained readers, journals and failed fixtures were untouched.

### Review corrections — 2026-11-16

All four findings above have now been addressed in the pending local tree:

- Received certificates and original update IDs survive post-response transport,
  authentication, deadline, source/custody and retention refusal in redacted typed
  errors. Local original-reply import remains unchanged; the new
  `reconcile_observation_reply` admits only an already-submitted unresolved lookup
  under its separate original time policy. Native failed-write, changed-custody,
  tamper and closing-deadline cases check retention without renewed authority.
  An actual-IC load writes no local result, then settles offline from the error's
  certificate without consulting the proxy's recovery copy. Changed/orphaned
  namespaces still refuse; no alternate journal or repair path was introduced.
- New profiles allow 2 MiB complete requests, 4 MiB complete results and 2 MiB
  certificates around a 1 MiB data vector. Full 1 MiB whole-chunk capture and
  restore pass on actual IC, including proof/envelope overhead. Old approved
  request/result limits still apply; a native regression rejects the larger proof
  under the original smaller allowance. The 1 GiB declaration ceiling is not a
  demonstrated full-journey capacity or performance claim.
- A separate test-only `ic-backup-consumer` executable uses normal dependencies.
  The fixture builds it alone, checks its normal graph excludes Testkit/PocketIC,
  and runs capture/restore steps, continuations and offline completion inspections
  in fresh processes. Shared fixture authority avoids duplicating signing code.
  The Testkit integration test itself remains a development-dependency consumer;
  neither case proves registry distribution or Canic integration.
- Present-tense inventory, source/restore and qualification claims are corrected.
  Shared public names are `OperationStoreError`, `BackupHttpTransport` and
  `BackupHttpError`, without compatibility aliases. Stored domains and record
  encodings remain unchanged by this correction; the original smaller profile is
  never upgraded during reopen.

The focused legacy evidence-retention and isolated-consumer review used the frozen
revision and exact history listed above. The guarantee was adapted to the current
certificate/recorder boundary; legacy stage architecture, storage and source were
not copied. Existing and failed fixture evidence remains intact.

Fresh evidence is `target/evidence/review-fixes-{native,ic,doctest,clippy,private-clippy,rustdoc,doc-checks}.log`:
232 native tests, ten actual-IC journeys and five documentation examples pass,
along with strict lint/docs and format/link/tooling checks. The post-test changes
only clarified comments/error messages and grouped imports; final Clippy and
Rustdoc cover those edits. Fixture originals, child command logs and the independent
normal graph remain under `qualification/target/fixtures/`. No commit or push occurred.

Fresh `toko-cloc -d 1 .` output is `target/evidence/review-fixes-loc.txt`:

| Scope | Legacy | Corrected current | Reduction |
| --- | ---: | ---: | ---: |
| Repository app/lib classifier | 15,646 | 8,294 | 46.99% |
| Repository total | 44,877 | 20,877 | 53.48% |
| `ic-backup` production | 13,579 | 8,098 | 40.36% |
| `ic-backup` crate total | 37,062 | 19,580 | 47.17% |

The reporter classifies the 196-line qualification executable as app/lib; it is
explicitly test-only, not production Backup. Excluding it, maintained Backup
production is 8,098 lines versus legacy repository production of 15,646 (48.24%
smaller). The raw classifier comparison above includes it conservatively. Tests
are 12,583 lines and unclassified code is zero. These repairs add 782 total lines
to the reviewed tree, mostly regression/consumer evidence and bounded recovery.
The 45% repository target still holds; crate-only production still does not meet
it. Scope exclusions, quadratic historical replay work and unqualified Canic
integration from the review remain material limitations.
