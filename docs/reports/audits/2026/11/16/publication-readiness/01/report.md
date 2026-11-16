# Legacy comparison and crates.io readiness audit

Date: 2026-11-16. Requested scope: full comparison with legacy, required features,
and readiness to publish. The maintainer selected approximately 20 MB per
canister (revised from 100 MB on 2026-11-16); use 20 MiB combined
module/heap/stable/chunk payload as the conservative acceptance target, with
explicit shape and call bounds. This amendment changes the capacity requirement,
not the source revision or historical validation results below. The subsequent
fixture repair and its checks are tracked in the [handoff](../../../../../../../status/current.md).

## Verdict

**Not ready to publish as a qualified 20 MiB, supported-host Backup release.**
Hosted native checks fail on both declared macOS architectures. The 20 MiB
capture/restore profile and performance remain unqualified, and the restart has
no configured release/publication workflow. These are Backup-owned work that can
proceed before Canic. Archive assembly and a consumer of that archive do pass.

The selected small direct-controller capture and stopped-restore feature set is
implemented. This is not full legacy feature parity or a working Canic-managed
backup product. Canic manager execution requires coordinated Backup admission/
replay work once the real receipt protocol exists; the current signer/guard
interfaces cannot supply that route by themselves. A deliberately limited,
experimental direct-controller crate could be published earlier after the concrete
release blockers are resolved, but must not claim Canic or 20 MiB qualification.

Audit verdict: **FAIL for current host qualification/release readiness**;
**BLOCKED for 20 MiB operational acceptance and Canic integration**. No new
serious production integrity defect was demonstrated by this inspection. That is
not an exhaustive security, filesystem power-loss or cryptographic proof.

## Identities, method and evidence limits

- Product: clean `af240881239112b8a8a9250b4c37e3f00dc68b19`, version 0.102.0,
  verified on `origin/main`; only this report/handoff are added by the audit.
- Legacy: read-only `4ae087fad8c23de712aa60bf569bde4b2b932651`, existing checkout
  `/tmp/ic-backup-legacy-loc-20261115`; its last release is 0.67.2. The live sparse
  registry index also ends at non-yanked 0.67.2. The crates.io JSON API returned
  HTTP 403; `https://index.crates.io/ic/-b/ic-backup` supplied the current index.
- Local overlay blob: `7c5061a4c3fade4bc8ab93fa2b429a7baba58889` (`AGENTS.md`).
  Product authorities are the restart, operation/local-store/capture/transfer/
  inventory contracts and sibling review. There is no separate adopted audit
  overlay; these explicit product contracts define this run's provisional scope.
- Installed Toko 0.44.4. Methods at immutable revision
  `626da7cd9626cc2e152594862a2d82ea82ef51da`: `audits/README.md`,
  `audits/code-hygiene.md`, `audits/ci-release-integrity.md`, plus the release
  contract `docs/releases.md`. The earlier complexity audit is a different method,
  not a comparable PASS history. The legacy assessment remains supporting evidence.
- Fresh execution: Rust 1.100.0 (`0ef1bc386059596dada8fb65b50425aab2ea637a`),
  Linux x86-64, locked/offline package verification, archive-consumer compilation
  and offline capture/restore inspection. Registry/CI/Canic issue reads are live.
- Reused local evidence: `target/evidence/review-fixes-*.log` (232 native,
  ten actual-IC, five documentation cases); `version-0.102.0-*` verifies final
  metadata and lints. These were not rerun or re-labelled as fresh audit passes.
  No new IC effects, legacy tests, sibling edits, dependency updates, commits,
  uploads, CI reruns or release execution occurred.
- Supplier crypto/codec/Host implementations, application-specific fence semantics,
  production HTTPS and physical power loss remain outside fresh execution.
  Missing 20 MiB and supported-host proofs are gaps, not passing exclusions.

## Feature and guarantee comparison

| Capability | Legacy | Current decision and evidence |
| --- | --- | --- |
| Capture to complete local backup | Core/Agent stages and standalone driver | Implemented fixed status/list/take/metadata/data/final-status/publication loop; actual IC |
| Restore original state | Allocate/upload/readback/load, then application acceptance/start | Allocation through `LoadedStopped` implemented; exact metadata and every byte compared before load; actual IC |
| Snapshot representation | Module, heap, stable memory, globals, certified data, timer/hook, chunk store | Retained, including exact float bits and empty chunks; unsupported/missing round-trip fields refuse |
| Original authority and finite spending | Original catalogs, attempt journals, receipts | Immutable approval/profile, one shared recorder; reservations never refunded or silently renewed |
| Lost replies and restart | Original stage continuation and settlement | Original reply import, one explicit lookup, offline lookup import, separate original time policies, fresh-process continuation |
| Post-response failure | Bounded acknowledgement ownership in failure variants | Received certificate/request ID survives failures; redacted Debug; native and actual-IC failed-write cases |
| Source integrity | Manifest, references and verification passes | Original capture/evidence plus trusted manifest and held payload; substitutions, aliases, extra files and bad coverage refuse |
| History and new captures | Catalog and source-reference machinery | Bounded local list/latest and authenticated succession after completed capture/restore |
| Stop, acceptance, start, external settlement, source release | Explicit application callbacks and lifecycle stages | Deliberately outside Backup; mandatory guard does not implement them; Canic/application owns them |
| Manager-owned applications | Broader low-level composition, no evidence of a completed Canic manager route | Direct-controller route only; manager receipt/executor route remains joint work |
| Signing | Agent identity supplied by caller | External sign-only trait; direct Ed25519/secp256k1; real Canic signer unqualified; other identity families refuse |
| Fleet, arbitrary calls, migration, cross-release/ID restore | Broader low-level Core composition; standalone itself one existing target | Deliberate exclusions; same canister/network/release and original formats only |
| Distribution and consumer qualification | Core + Agent release tooling, candidate and registry consumer modes | One product crate; path consumer qualified; archive consumer passes this audit; release/registry journey not wired |
| Capacity | Standalone fixed eight read/eight write units of up to 1 MiB | Configurable finite limits; full 1 MiB whole chunk qualified; 20 MiB round trip not yet qualified |

Source inspection followed public planning/create/open/advance/import APIs into
intent validation, fixed procedures, common reservation/framing/custody/replay,
metadata/data codecs, held source and completion/succession. No second production
phase engine, automatic retry, default application authority or Canic dependency
was found. Production error paths are typed; panic search hits were fixtures or
constant documentation examples in the reviewed roots. Git contains no symlink
entries. Private fixture keys are outside the product archive's production graph.

Legacy anchors rechecked: Agent `standalone/{mod.rs,README.md,application/mod.rs}`,
`submission/mod.rs`; Core `workflow/qualified_receipt/mod.rs` and
`workflow/ic_snapshot_restore/tests/mod.rs`, especially
`failure_boundaries_retain_pending_or_applied_and_never_repeat_provider` and
`independently_valid_load_plan_cannot_rebind_the_admitted_snapshot_source`;
independent consumer `build/mod.rs` and `scripts/ci/prepare-journey-consumer.sh`.
History consulted includes `f332118eeed000e4615e48155c162a4e113076a8`, together
with the exact original-evidence/ownership revisions already recorded in the
legacy assessment. No legacy implementation or recovery data was copied.

## Findings and smallest next actions

### 1. MEDIUM — both supported macOS jobs fail (Backup qualification owner)

[CI run 43830433645](https://github.com/dragginzgame/ic-backup/actions/runs/43830433645)
is for the exact pushed commit. Scaffold and Linux pass. ARM macOS reports
222 passing / 10 failing tests; Intel macOS reports 211 passing / 21 failing.
The shared HTTP fixture reads return `WouldBlock` at
`crates/ic-backup/src/store/capture/test_fixtures/http.rs:218`; its server joins
then panic. The TLS peer also fails at `advance/tests/http_tests.rs:516`, and
cancellation closure assertions fail. This disproves supported-host readiness;
it does not by itself establish a product HTTP failure.

The listeners are set nonblocking but accepted streams are used as blocking
streams without explicitly selecting their mode. Platform-dependent inherited
socket flags are the leading source-level explanation, not a verified fix.
Smallest correction: explicitly admit accepted stream mode in the shared HTTP,
TLS and affected proxy fixtures, preserving finite read/write timeouts and the
acknowledged cancellation assertions. Run the affected tests and require fresh
passing hosted results on both macOS architectures; do not weaken assertions or
simply relabel the platform unsupported. Failure excerpts are retained under
`target/evidence/publication-audit-macos-{arm,intel}-failure.txt`.

### 2. MEDIUM — the requested 20 MiB profile has no qualification (Backup owner)

`OperationLimits` allows 1 GiB payload and 1 GiB evidence, but those are independent
structural ceilings. `operation/mod.rs::reserve` conservatively reserves the
complete request/result maximum plus four frames for **every** call. Nothing is
refunded after a small result. Restore needs `2*N + 7` management calls, where N
includes the rounded module/heap/stable extents and each whole chunk, including
empty chunks. Observations/signing services need additional allowance.

At the maximum 2 MiB request / 4 MiB result profile, each call reserves 6,357,136
bytes. Even ignoring other reservations, 1 GiB permits 168 calls, hence at most
80 restore data units: at most about 80 MiB with 1 MiB extents. The revised
20 MiB target is below this reservation ceiling; the former 100 MiB target was not.
This is a conservative bound derived from code, not a throughput measurement or an unconditional capacity limit across
all selectable profiles.

A bounded 20 MiB shape can fit mathematically within the existing hard ceilings.
Select its complete-envelope/result profile deliberately, including proof headroom,
metadata, signer overhead, observations, chunk count and region rounding; do not
change existing approved operations. Qualify actual 20 MiB capture, full restore,
independent restored bytes, cold replay and at least one interrupted transfer.
Record elapsed time, peak memory and IO. Historical replay is quadratic in call
history and can dominate this larger journey; prior 1 MiB IO measurements do not
establish 20 MiB performance. Do not promise arbitrary 20 MiB shapes merely from
a byte limit: many tiny/empty chunks can exhaust request/evidence budgets.

### 3. MEDIUM — publication execution/registry acceptance is not configured

Backup currently selects no release/publication entrypoints in its Makefile;
all packages inherit `publish = false`. That flag correctly preserves existing
authorization, but the product has no qualified standard prepare/verify/tag/upload/
registry-consumer path. Legacy's workflow and receipts cannot simply be imported
or treated as qualification of this replacement.

The archive and its normalized graph build successfully. Fresh archive-consumer
build and offline original-IC replay also pass, reducing the package-layout risk.
They do not qualify a fresh registry install or live 20 MiB archive consumer.
Smallest next action: select the current Toko single-product release/publication
mechanics, explicitly bind both locks and the complete selected gate, keep the
two qualification packages non-publishable, then qualify actual archive/registry
admission and recovery. Enable product publication only under publication authority.
No upload was attempted during this audit; the existing 0.102.0 Git checkpoint
is not a registry release.

### 4. MEDIUM — Canic needs a coordinated Backup route, not only adapters

`BackupConnection` contains concrete `BackupHttpTransport`; direct request
admission requires management-canister calls from the original host sender.
The status policy requires that sender to be an actual target controller.
Canic's controlling manager owns managed children; a host controlling the manager
is not thereby a controller of its children. Substituting the signer or guard
cannot authenticate a manager's inner management result or change that topology.

Canic [#5 — bounded sign-only adapter](https://github.com/dragginzgame/canic/issues/5),
[#6 — authenticated manager executor and lookup](https://github.com/dragginzgame/canic/issues/6)
and [#7 — fenced application recovery](https://github.com/dragginzgame/canic/issues/7)
remain open. #6 explicitly calls for coordinated Backup route admission/replay.
The current source/design correctly refuses that route, so this is a product
completion gap, not a reason to add an unused transport trait or permissive fallback.
Smallest next step: design one concrete manager receipt/lookup join with Canic,
then adapt Backup's boundary against an actual consumer. Expect another pre-1.0
API/storage compatibility review. Do not advertise the present crate as a
completed Canic backup integration.

### 5. LOW — the packaged crate has no README or complete onboarding profile

The fresh normalized `target/package/ic-backup-0.102.0/Cargo.toml` contains
`readme = false`; the 98-file archive has no README. The repository has useful
contracts and compiled loop examples, but the registry-facing payload does not
explain the required signer/guard, supported host/size profile, legacy migration,
or what must be retained for disaster recovery.

Add package README metadata and concise consumer guidance, linking existing
contracts rather than creating another architecture manual. Clearly distinguish
local bundle bytes from full restore authority: current restore requires the
same original store's capture evidence and locks, not an arbitrary exported
bundle. State that `LoadedStopped` is not application acceptance or permission
to restart; preserve old readers for old records. This is a documentation fix,
not a request to implement migration or bundle import.

## Deliberate restrictions and operational consequences

No new generalized workflow, CLI, cloud store, compression, fleet coordinator,
automatic start, destructive cleanup or migration is needed to finish this selected
library. Preserve the explicit application boundary. Legacy's acceptance/start/
release callback semantics still need a real owner in Canic; they were removed
from Backup's scope, not proven unnecessary.

Each capture/allocation retains a new remote snapshot and local evidence.
Repeated operation eventually requires externally governed snapshot/storage
retention. There is no pruning/reference-release API. An uncertain or permanently
refused operation keeps its target claim; a fresh operation cannot bypass it.
There is no generic abandonment, repair or orphan adoption. Partial artifacts and
changed custody stay intact. These are intentional integrity choices with real
operator consequences; publication documentation must say so. They are not
permission to delete original recovery obligations manually.

Source-admitted restoration requires original capture operation records and a
trusted original approval; inventory alone is not proof. Preserve the complete
private store and external approval/fence records under the intended disaster
recovery procedure. Portability to a different host/independently reconstructed
store was not newly qualified. The archived-consumer checks reuse original IC
fixtures in place, not a cross-host recovery drill.

## Fresh artifact and authority trace

| Trigger | Inputs and authority | Observed result / boundary |
| --- | --- | --- |
| CI push/PR/manual dispatch | Immutable checkout/action pins, read-only repository token; no publication credentials wired | Linux/scaffold pass; both macOS jobs fail. CI native job does not run the separate actual-IC suite |
| `cargo package --list --locked --offline -p ic-backup` | Clean af24088, root lock/compiler; no registry mutation | 98 files, license and native fixtures included, no README |
| `cargo package --locked --offline -p ic-backup` | Same source, registry-backed suppliers, package verification | Archive created and extracted product compiled successfully |
| Existing consumer source compiled against extracted package | Temporary unpublished audit manifest under target, original lock selections, no Testkit dependency | Public consumer builds against packaged library; no product source edit |
| Archive consumer `capture-inspect` / `restore-inspect` | Original completed actual-IC store, ID and approval only; no backend/signer/guard inputs | Both fresh-process offline inspections pass |
| crates.io publication | No configured adapter and no upload authority exercised | Not attempted; current published index remains 0.67.2 |

Archive SHA-256:
`f59cfb7434f10403da0e938ab38c4f86f1904907ce5d80b0f2222fb77554de27`.
Evidence is under `target/evidence/publication-audit-*`, with the private temporary
consumer under `target/evidence/publication-audit-consumer/`. The first sandboxed
child probe reported EPERM despite a zero status; it is retained as failed evidence
and was not counted. The corrected check also requires no spawn error; its
explicitly admitted rerun produced two clean exits, saved as
`publication-audit-archive-replay-success.json`. No registry dependency was changed.

## Recommended order

1. Correct the native fixture portability issue and obtain green supported-host CI.
2. Select and qualify the 20 MiB profile, with measured recovery/runtime costs.
3. Package the consumer/migration guidance and qualify the publication workflow.
4. Publish only the explicitly supported direct-controller scope, if that is useful
   before Canic; otherwise complete the manager receipt contract first.
5. Implement the Canic signer, executor/fence and corresponding Backup route with
   one real application. Do not freeze a hypothetical manager API beforehand.
