# Backup complexity audit

Date: 2026-11-15. Trigger: the maintainer asked whether the rewrite is becoming
as large as legacy before completing the selected smaller product.

## Verdict and scope

**PASS WITH FINDINGS for the structural review. Four MEDIUM findings require
simplification decisions before extending restore.** This is not a correctness,
security, performance or release qualification verdict. No serious runtime failure
was demonstrated. Substantially smaller end-to-end implementation remains unproved.

Current Rust code is 29,477 LOC: 10,329 production and 19,148 tests/qualification.
Frozen legacy is 44,877: 15,646 production, 29,152 tests and 79 examples. Current
production is 66.0% of legacy; total is 65.7%. These are code lines excluding
comments and blanks, measured with Toko's Rust classifier, including both owning
workspaces. They exclude docs, shell/Perl tooling, dependencies and generated
build artifacts. These are size comparisons, not functionality-normalized costs.
Legacy implements allocation/upload/load/application acceptance/start; current
restore has intent, planning, status preparation/submission and terminal replay.
A complete restore, later-phase reconciliation and Canic adapter are absent.

| Current owner | Production LOC | Tests LOC |
| --- | ---: | ---: |
| Capture | 8,171 | 13,137 |
| Restore | 1,271 | 2,581 |
| Remaining store/inventory/payload/library | 887 | 1,730 |
| Private qualification | 0 | 1,700 |

About 65% of both trees is tests. Removing tests to reach a headline LOC target
would not address the production trajectory. No defensible final LOC or savings
estimate follows from this inspection.

Inspected production flows: local inventory/payload custody; capture admission,
status/list/take, retained-result and lookup settlement, metadata/data acquisition,
journal replay, assembly/completion/source and restore status. Read owning
contracts, qualification composition, representative failure suites and legacy
submission/history. This is a broad structural baseline, not an exhaustive
line-by-line proof of every decoder or filesystem primitive. Supplier internals,
cryptographic correctness, power-loss durability, live deployments, CI/release
integrity and unrelated sibling work are outside this method. Multi-extent IC
throughput is an explicit proof gap, not a passing exclusion.

## Identities and method

- Product HEAD: `dc4bba68bbaeb0966ecefb8335c4ede85d95e125` (pushed 0.101.12).
- Inspected tree includes local 0.101.13 restore direct profile/submission, its
  moved status journal, new tests/qualification and shared transport changes.
  Separate rand 0.10.4 lock changes remain untouched. The saved source hashes and
  Git status identify tracked and untracked audit inputs; diff stat alone would
  misleadingly omit the new untracked files.
- Local `AGENTS.md` Git blob: `0e8f98d81e1580ec1842a978ecc25d12f2a72f5e`.
- Installed Toko 0.44.4; methods read at revision
  `626da7cd9626cc2e152594862a2d82ea82ef51da`: `audits/README.md`,
  `audits/complexity-and-technical-debt.md`,
  `audits/flow-convergence-and-duplication.md` and
  `docs/principles/simplicity-and-maintainability.md`.
- Product authorities: AGENTS, `docs/architecture/{local-inventory,capture-intent,capture-transfer}.md`,
  `docs/design/{restart,operation-contract,local-store,sibling-integration,legacy-assessment}.md`.
  This first broad run uses those existing product contracts as its scope overlay;
  there is no separately adopted audit overlay or comparable previous broad run.
- Legacy read-only reference: `4ae087fad8c23de712aa60bf569bde4b2b932651`, existing
  `/tmp/ic-backup-legacy-loc-20261115`; no new checkout was created.
- Execution: source/history inspection and LOC measurement only, Linux host.
  Compiler is pinned to 1.100.0; no compile/test or IC effect was run for this audit.
  Existing 348 unit/24 documentation/six IC results are historical evidence in
  `target/evidence/restore-submission/`, not fresh audit passes or performance proof.

## State and ownership map

| Axis | Admitted behavior and owner | Invalid combinations / interaction |
| --- | --- | --- |
| Operation | Capture or restore plan/journal; same target/release restore | Different operation authority cannot spend another operation's allowance |
| Execution | Direct sender; a declared Canic executor shape also exists | Direct signed-envelope implementation does not support Canic execution |
| Call progress | Reserved signing, retained preparation, reserved submission, retained result | Pending never becomes a fresh call; authentic rejection is not success |
| Data progress | Ordered ordinal, plus accepted payload identity | No gaps, duplicate acceptance or acceptance inferred from inspection |
| Recovery | Original result import or one lookup for status/list/take | Metadata/data/final-status/restore recovery is not implied by these paths |
| Time/permission | Live admission versus retained historical verification | Cold terminal replay must not require a clock, signer or live guard |
| Custody | Original held locks/files; exact completion; explicit successor | Completed predecessor may survive target owner change; unfinished may not |
| Local assembly | Verify versus reconcile | Reconcile can create missing members before verification, never repair partial bytes |

These axes are constrained; multiplying enum variant counts would invent a
meaningless state count. Test failure modes are not additional product modes.

| Behavior | Owner and carried contract | Consumers / convergence |
| --- | --- | --- |
| Direct signatures and certificates | Platform plus Backup request/CertificateRequirements checks; exact request/proof bytes | Capture and restore already share authentication primitives |
| Network dispatch | Host HTTP and StatusHttpTransport's one-shot call path | Status, snapshot, metadata/data and restore status converge before HTTP |
| Call lifecycle | Capture status slot; snapshot workflow; metadata workflow; data workflow; restore status | Equivalent reservation/preparation mechanics do not yet fully converge |
| Durable authority | Capture ledger apply/candidate; restore status apply; shared Frames | Cold readers reconstruct original authority, workflows spend it before effects |
| Evidence inventory | Phase references, operation check, commit validation and synchronization | Names/reference presence are rediscovered at several sites (F3) |
| Payload authenticity | Metadata/schedule plus data_payload; completed source block checks | Incremental capture repeatedly returns to whole-history validation (F1) |

## Findings

### F1 — MEDIUM: whole-history validation is nested inside incremental progress

Owner: capture custody/replay and data workflow. Disposition: localize; first
simplify budget derivation, then measure the retained-evidence traversal before
changing its safety boundary. Trigger: before growing restore transfer handling.

Evidence:

- `crates/ic-backup/src/store/capture/mod.rs:107` check replays the entire ledger,
  reads/hashes every referenced evidence file and checks assembly custody.
- The same file at line 541, commit_with, invokes check before candidate generation,
  again before persistence and after it; it also synchronizes all earlier evidence.
- `capture/ledger/mod.rs:482` replay calls apply for each frame; apply calls
  additional_budget (line 773), which reconstructs a Vec of all prior admissions
  and sums the entire prefix for each event.
- `capture/advance/data/mod.rs:328` reauthenticates all preceding payload ordinals
  on each advance_data. This occurs even before returning a pending phase.
- `capture/assembly/mod.rs:75` starts with verify_data, then derives metadata and
  traverses payload proofs again. check_assembly can hash the assembled tree.

Consequence: for N accepted extents, previous-payload verification alone sums
N*(N-1)/2 prior visits. Budget reconstruction is O(N^2) within a full journal
replay; performing O(N) such replays across progress produces O(N^3) cumulative
budget-accounting work in that path, before fixed bounds cap N. These are source
loop bounds, not measured wall times. Repeated IO, crypto and sync costs are
unmeasured. The actual-IC fixture explicitly binds one data call/eight bytes
(`qualification/.../tests/metadata/mod.rs:148`); it cannot qualify this scaling.

Smallest safe correction: keep derived budget totals in the in-memory replay
state and add each call's full reservation at its owning transition. Reconstruct
those totals from original events on reopen; no new journal, persisted cache or
refundable counter. Preserve conservative future-take and recovery allowances.
Then instrument bytes read, evidence authentications and sync calls for increasing
ordinal counts, including cold reopen. Separate a full admission boundary from
per-transition checks only with proof of unchanged custody and invalidation.
Do not replace content checks with mtime or skip checks after an await. The current
store requires exclusion of external writers, but held descriptors alone do not
make content immutable. The existing completed CaptureSource illustrates bounded
held reads; it is not proof that mutable capture can use that contract unchanged.

Required regression boundaries: cumulative lookup/data budgets, changed earlier
proof bytes, substituted names/locks, cancellation, full/partial uncertain frames,
closing custody and cold replay. Source inspection establishes unnecessary repeated
budget derivation; removing other validation requires a focused behavior proof.

### F2 — MEDIUM: metadata and data maintain duplicate signed-read lifecycles

Owner: capture read workflow. Disposition: consolidate internal mechanics when
repair is authorized, before copying this pattern into restore upload.

Evidence: `capture/advance/metadata/workflow/mod.rs:22,49,273` and
`capture/advance/data/mod.rs:27,42,277`. Their admission records carry the same
six fields; validation independently checks the same call/expiry/observation rules.
Both independently implement live signer binding, 300-second preparation ceiling,
durable start, timeout, post-await check, envelope decode, exact request admission,
prepared evidence, second read permission, durable submit, HTTP and retained result.

A change to signing limits or preparation semantics therefore requires parallel
edits. Corresponding cancellation, prepared continuation and uncertain-frame tests
also repeat the same lifecycle contract. This is duplicated decision flow, not
merely similar Rust syntax. Data's ordinal/coverage and accepted-payload transition
are real differences; metadata's snapshot derivation is also real.

Smallest correction: one private read-admission validator and prepared-envelope
builder first; then a private, closed shared read-call continuation only if it
removes the duplicate reservation/sign/submit sequence. Keep phase-specific
arguments, prior-evidence policy, event names and result interpretation with their
owners. No public stage engine, dynamic registry, arbitrary-method dispatcher,
extra journal or configuration switch. Preserve existing record encodings; a
public type unification would need a separate API change decision.

Retain native lifecycle boundary cases once at the shared owner plus metadata/data
semantic differences and real-IC calls through both paths. Do not delete tests
merely because names resemble one another or because legacy had similar coverage.

### F3 — MEDIUM: evidence-file membership has multiple semantic owners

Owner: capture persistence. Disposition: consolidate derivation, retain enforcement
at opening and write boundaries. Trigger: before adding another persisted phase.

Evidence: `capture/mod.rs:142-220` enumerates reference validation and directory
membership; `capture/mod.rs:553-642` separately maps events to permitted evidence
names; lines 660-714 enumerate existing files again for synchronization.
Phase-local evidence methods exist for snapshots, lookup, metadata and final
status; data has separate naming logic in `capture/ledger/data/mod.rs:12,87`.

Consequence: a new evidence-bearing phase changes event/state interpretation,
reference hashing, orphan admission, write pairing and synchronization in several
places. Those are valid separate checks, but the identity/presence decision is
reconstructed independently. Merely extracting another IO helper would not fix it.

Smallest correction: derive the expected evidence names/references from retained
State once through one private iterator; use it for hashing, membership and syncing.
Give EventRecord a narrow pending-evidence projection, checked against the candidate
transition, so workflow callers cannot invent a different event/file pairing.
Keep exact hash/length checks, create-only writes, original file identities, ordering,
handle poisoning and unknown-file refusal. This adds no persisted representation.
No need to merge capture and restore journal state machines to obtain this saving.

Required regressions: valid reference inventory, absent/extra/orphan files,
event-to-file mismatch, partial writes and full uncertain frames. Existing
metadata/data corruption and custody suites are evidence anchors to preserve.

### F4 — MEDIUM: phase-specific integration contracts leak product mechanics

Owner: public operation/direct integration boundary. Disposition: narrow before
extending the consumer surface; share F2's lifecycle implementation rather than
building a second facade over all existing contracts unchanged.

Evidence: `capture/advance/direct/mod.rs:23,47,64,118` has a bounded sign request,
separate application guard and composed timing. Metadata/data expose distinct
signer traits, raw observation bytes, exact expiry and repeated admission fields.
The real consumer fixture implements both and constructs TimePolicy itself
(`qualification/.../tests/metadata/mod.rs:21,92,365,378`). Restore's signer returns
a full PreparedStatus while DirectRestoreStatus validates its composed profile
(`restore/direct/mod.rs`; `restore/persistence/status/mod.rs`). Historical
verification still accepts a CaptureProvider even when only authentication is used.

Consequence: an integration must understand several representations of essentially
one bounded direct signing operation and its timing, alongside legitimate distinct
application permissions. Each new upload phase risks another trait/admission pair.
This is present caller friction, not evidence that the Canic route is implemented
or that external callers are currently broken.

Smallest correction: reuse the bounded signing-request approach for direct calls,
with a closed set of product operations and exact derived arguments. Keep generic
crypto signing independent from phase-specific live application admission. Derive
original timing in Backup under explicit policy; retain exact observation bytes.
Keep cold verification separate from live effects without adding another public
provider taxonomy unless actual callers require it. Prototype the smaller contract
against the existing qualification consumer before publishing more phase APIs.

No new package, Canic dependency or alternate execution route. Existing consumers
and persisted commitments need explicit compatibility review before signature or
binding changes. Do not implement a Canic manager route as an audit fix.

## Three change rehearsals

| Plausible change | Expected owner | Present edit spread / blocker |
| --- | --- | --- |
| Change metadata/data preparation admission | Direct signed-read policy | Both workflow validators/timeouts/output assembly; both signer fixtures and phase tests. F2 owns the duplicated decision. |
| Add restore upload allocation | Restore phase policy and its original journal | Current status-specific signer/preparation/provider shape encourages another complete lifecycle and public adapter; F4/F2 should resolve the reusable boundary first. Allocation semantics remain new required work. |
| Add a capture evidence-bearing transition | Capture ledger transition | Event schema/apply plus check's references/membership, commit pairing and sync traversal. F3 removes repeated membership ownership, not independent corruption checks. |

Comparable existing change: local 0.101.13 extends restore status preparation to
submission while already reusing call_certificate, direct request authentication,
certificate requirements and timing arithmetic. Those shared pieces are good;
the remaining per-phase orchestration is the issue, not a need to replace them.

## Legacy lessons and deliberate retention

At frozen legacy, `ic-backup-agent/src/submission/mod.rs:15` routes different
adapters through submit_once: original request admission, context binding, exact
ingress retention, one dispatch, reply retention and indeterminate post-dispatch
failure. `tests/http_boundary/mod.rs:277,375` covers retention refusal, 202/307/429/503,
malformed/oversized response, disconnect, timeout and stale/context refusal.
Current tests cover analogous boundaries; no old pass was counted as a new pass.

History inspected: `94b94db89f77968eb4b480ef6d0576d41f6472e2`, especially
standalone restore/transfer, consolidates allocation reply/data plan/reply binding
owners. Submission history was inspected to identify its provenance. Changed
assumptions: the new implementation has external signing and independently retained
certificates, local bundles and one journal per operation; legacy's SDK-owned
signing and stage/inventory machinery must not be imported wholesale. Reuse the
one-owner lesson, not its whole interface or its LOC count. No legacy code copied.

Keep deliberately:

- Original intent, finite signing/network/observation budgets and no blind retries.
- Distinct pending versus authenticated result versus policy-success/accepted data.
- Independent guards at untrusted decoding, reopening, post-await and publication.
- Separate restore authority/source references and same-ID/same-release policy.
- Local assembly integrity, exact completion and target succession; these deliver
  backup history for a required consumer, not speculative storage abstraction.
- Platform codecs/crypto, Host filesystem/artifacts/HTTP and private Testkit.
- Explicit capture versus restore journals; equivalent framing already converges.
- Phase-specific tests and actual-IC qualification where they prove distinct effects.

Legacy stage catalogs, dependency-bearing operation graphs and per-stage journals
were not recreated. No large compatibility layer or alternate blob backend was
found in the selected flows. The declared unsupported Canic plan shape is not the
major size driver; revisit it with the adapter owner rather than expanding it now.
Pure upload/readback helpers have a named imminent restore consumer; keep them
until the composed restore establishes whether their public exposure is needed.

## Recommended sequence and verification limits

Pause allocation/upload expansion for a bounded simplification batch: evidence
inventory (F3), read admission/lifecycle (F2), and a concrete direct consumer shape
(F4). Independently remove repeated replay budget summation and measure the rest
of F1 before changing custody checks. These are recommendations, not authorized
product edits in this audit. Do not run a framework rewrite or trim tests for LOC.

Acceptance should show fewer independent owners, unchanged exact retained bytes,
unchanged one-shot/cold-replay behavior and a consumer that needs less protocol
assembly. Record production/test LOC after the repair, but do not invent a target
percentage or claim a reduced final system before restore is complete.

Fresh checks: current and legacy Toko LOC reports; source/caller/history inspection;
report/source identity capture and documentation-link validation. No tests rerun:
product code was unchanged, and the existing suites do not answer throughput.
Required future proof: actual multi-extent IC journey and measured IO/verification
cost, focused native custody/uncertainty regressions for each repair, followed by
the applicable existing complete-batch qualification. Physical power loss and
Canic adapters remain unqualified. No commit, push, publication or deployment.
