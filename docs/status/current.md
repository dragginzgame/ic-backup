<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-backup/ic-backup-readme-header.svg" alt="IC Backup — Verified backups and safe recovery for Internet Computer apps" width="100%">
</p>

<!-- helper-navigation:start -->
<p align="center">
  <a href="https://github.com/dragginzgame/canic"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/canic.svg" width="18" height="18" alt=""> <strong>canic</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/icydb"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/icydb.svg" width="18" height="18" alt=""> <strong>icydb</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-timers"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-timers.svg" width="18" height="18" alt=""> <strong>ic-timers</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-memory"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-memory.svg" width="18" height="18" alt=""> <strong>ic-memory</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-query"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-query.svg" width="18" height="18" alt=""> <strong>ic-query</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-backup"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-backup.svg" width="18" height="18" alt=""> <strong>ic-backup</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-blob-storage"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-blob-storage.svg" width="18" height="18" alt=""> <strong>ic-blob-storage</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-testkit"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-testkit.svg" width="18" height="18" alt=""> <strong>ic-testkit</strong></a>
</p>
<!-- helper-navigation:end -->


# Current handoff — 2026-10-06

The maintainer reports 0.3.6 pushed. Clean local release commit
`f4b1426b5afac3ad53d3c862e39f784cef7391f9`, annotated tag and exact receipt/parent
checks passed before this handoff update; original receipt source is
`9478fcae1ee6d859908d7602efe75153fab0e2fc`. GitHub main/tag CI confirms that commit
was pushed. Package, lock, receipt and finalized changelog now select 0.3.6; registry
publication was not inspected. The accepted snapshot-upload contract and explicitly
requested ic-metrics integration, tooling adoption and native fixture repair share
one compatible undated 0.3.7 draft. Package
version and receipt stay at 0.3.6; the lockfile only adds registry ic-metrics 0.1.7. Work is
uncommitted. No agent commit, tag, push, publication or live IC effect ran. Release
inspection evidence is under `target/post-036-review`; its `previous-current.md`
preserves the pre-release handoff. The new batch retains its initial handoff and
qualification evidence under `target/snapshot-upload-review`.

The requested issue/tooling refresh on 2026-10-06 inspected the completed
[main CI](https://github.com/dragginzgame/ic-backup/actions/runs/37442582446) and
[tag CI](https://github.com/dragginzgame/ic-backup/actions/runs/37442582576) for that
exact released source. Both Linux jobs passed; both macOS architectures failed in
`non_utf8_tree_names_reject_instead_of_collapsing_path_identity` while creating
the raw `0xff` fixture basename, with `Illegal byte sequence`, before production
checksum admission. Release/tooling/hook/snapshot targets passed on all hosts;
the complete gate is not green. Pending 0.3.7 source remains uncommitted and has
no remote CI result. Raw current job summaries, failed logs, issue discussions and
read-only committed-tooling inspection are retained in
`target/maintenance-037-review.B63Hd3`.

The maintainer then authorized fixes and issue maintenance after Shared Tooling
0.1.6 was committed. The refreshed 44-file snapshot binds exact clean committed
`a37771f1b6b5fc9a88ed6ab3b705bdda35cd8fa3`, matching remote main at inspection.
It includes pinning rules/checker, complete audit methods and host/IC tool setup.
Actual Linux installation and offline checksum/version checks pass. Scoped reasons
retain the three existing exact Cargo constraints; locked selections are unchanged.
CI explicitly provisions local tools; Make/CI/release checks select their bin paths
and never install during ordinary validation. Native macOS setup remains pending.
See [the adoption review](../shared-tooling.md) and
[the tooling owner](https://github.com/dragginzgame/ic-backup/issues/3).

The [macOS fixture repair](https://github.com/dragginzgame/ic-backup/issues/4)
reuses one production tree-name admission owner for checksum/copy traversal.
Direct exact non-UTF-8 rejection is exercised on every Unix host; the filesystem
fixture remains real. Only macOS `EILSEQ` during raw-name creation is admitted;
other hosts/errors fail. Focused Linux artifact cases, Clippy and Rust 1.91 checks
pass. Native macOS confirmation for this source remains pending.
Evidence, previous snapshots/review/handoff and initial tooling-fixture dependency
failure remain under `target/shared-tooling-037-adoption`. The consumer fixture
entry point reuses existing metadata/receipt/real-index/failed-log tests for the
unchanged shared nested-logger regression; all tooling/pinning/installer cases pass.
The final isolated release adapters, runner and hooks also pass
(`release-hooks.log`), along with shell/format/snapshot and installed-tool checks
(`consumer-final.log`). Repeated setup with curl deliberately blocked passes
(`idempotent-offline.log`). CI YAML admission retains the three-host matrix and
places explicit local setup before macOS Bash 3.2 checks and the full gate.
Upstream [0.1.6 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37450707625)
subsequently completed successfully; it supplies no changed-source consumer result.
`qualification.log` derives artifact test membership from the actual registered
binary, checks final passes and unchanged lock selection, and retains current
source identities in `qualified-source.sha256`. Historical upload/metrics evidence
keeps its original pre-adoption scope.
No maintained function, method or type was removed in this adoption/fixture repair.
No full gate, product audit, provider, live IC effect, commit, version/release
transaction, push or publication was performed.

## Pending 0.3.7 source-bound upload contract

The subsequent maintainer-requested `ic-metrics` integration uses the published
0.1.7 dependency with default features disabled, inherited from the root catalog
for the Unix host implementation. The archive checksum matches the lock/index
identity; resolved features/runtime dependencies are empty. No previously selected
package changed, no sibling path/patch was introduced and no IC instruction reader
was enabled. [Dependency review](../ic-metrics-adoption.json) records the exact source.

`DownloadJournalGuard::ic_snapshot_metrics` exposes a copied per-guard
`IcSnapshotLocalMetrics` view. Existing explicit IC-tree verification and upload
metadata/data preparation record successful/rejected local monotonic durations in
nanoseconds through shared `MeasurementSummary` arithmetic. Successfully returned
data contributes actual chunk bytes, including measured zero for empty chunks and
repeated samples for repeated work. Preparation includes its nested verification;
these timings overlap and are not exclusive totals. Guard-local synchronization
preserves Send + Sync, with no lock across filesystem work. Metrics hold no IDs,
serialized history, reset API, global registry or effect/completion authority.
Ordinary create/open starts empty; record reads/replay and metric reads supply no
fresh byte checks or samples. Journals, hashes, returned errors, spending, obligations
and references keep their original owners.

Fresh native sampling/dependency evidence is under `target/ic-metrics-review`:
all registered download-journal cases, including real publication/interruption
recovery and the added diagnostic cases, pass in `journal-final.log`. Tests check
unit separation, success/rejection membership, zero/repeated chunks, duration
clamping, copied views and empty artifact-free reopen. Public artifact/upload
journeys pass (`public-final.log`) while retaining original source/reference bytes
and exhausted pending mutation/observation allowances. Warning-denied all-target/
all-feature Clippy and docs, Rust 1.91 checks, formatting, exact generator checks,
dependency feature admission and the unchanged 23-file shared snapshot pass.
`qualification-check.pl` binds registered passing cases and that integration's source identities.
Earlier upload evidence below retains its pre-metrics scope. The combined 0.3.7
batch is ready for review/configured CI; no full gate, IC effect, transport
measurement or performance-saving claim was introduced. Native macOS remains
unqualified for the new work. That integration removed no maintained symbols.

The subsequent requested metrics review retains shared arithmetic, seven distinct
unit/outcome summaries and the required before/after source custody checks. It removes
unneeded result-type generics, the mutable chunk-size side channel and an unused
checksum return value from private adapters. Successful chunk sizes now come directly
from the fully admitted result. Public APIs, returned errors and check ordering are
unchanged. Two duplicate private tests in `download_journal::metrics::tests` were
removed: `units_outcomes_empty_chunks_and_copied_summaries_stay_distinct` is covered
by the actual guarded filesystem sampling journey;
`duration_exceeds_nat64_is_saturated_without_changing_other_units` is replaced by
`duration_conversion_preserves_nanoseconds_and_clamps_overflow`, which checks zero,
exact nanoseconds and overflow without retesting upstream aggregation. The filesystem
journey also rejects a genuinely missing source tree without adding prepared bytes
or changing retained journal evidence. Fresh cleanup qualification is retained under
`target/ic-metrics-cleanup-review`; earlier integration evidence keeps its original
scope. All currently registered download-journal cases and both public artifact/
upload journeys pass against the final source (`journal-final.log`,
`public-final.log`). Warning-denied all-target/all-feature Clippy and API docs,
Rust 1.91 checks, formatting, unchanged generators and the 23-file shared snapshot
pass. `qualification-check.pl` binds the actual registered passing cases and source
identities; the earlier test-length lint failure remains retained in `clippy.log`.
No full validation gate, native macOS run or IC effect was performed.

`IcSnapshotUploadRequest` encodes same-target official SDK metadata/data arguments
from original source evidence. Metadata retains exact available global bits and
optional timer/hook absence; unavailable globals reject and replacement is always
absent. Data needs a distinct new raw ID and an exact bounded original region or
known chunk, including empty known chunks. Independent Candid/wire/source-binding/
reply-hash fixtures cover every registered case. The source-plan, metadata and tree
checksum binding precedes each new original upload plan and pending reservation;
metadata spending never supplies data spending or permits redispatch.

Explicit guarded metadata/data preparation reuses full retained source plan/journal,
complete Durable selection and exact opt-in IC-tree verification. Data verifies
before/after one bounded no-follow descriptor read, without aggregate buffering or
record/reference changes. File reads retain their original pre-read bounded size;
sequential checks still require integration-owned stable noncooperating custody.
Passive `IcSnapshotUploadAttempt`/reply association rechecks current original
authority, reservation and actual claimed context/target. It authenticates no effect,
writes no receipt, replenishes no spending and leaves lost replies pending. No
upload provider, transport, complete-upload view or release permit was installed.

Native evidence: all registered upload cases pass (`upload-unit-final-2.log`);
the later full binding/reply golden and authority checks pass in
`upload-model-policy-final.log`. Unchanged snapshot owners and real artifact
publication/interruption cases pass (`snapshot-owners-final.log`), as does the
canonical lifecycle empty-tuple owner (`lifecycle-owner-final.log`). Public upload,
artifact, metadata and data journeys pass (`public-owners-final.log`). The new
public upload reopen preserves exact source plan/journal/reference bytes and
independently pending metadata/data spending, including exhausted lost observations.
Tests cover all region/chunk kinds, maximum chunk/ID/globals, exact float bits,
source declaration/evidence drift, invalid/extended wire, and changed/missing/unsafe
source files including real FIFOs. Registry membership is derived from listed and
passing cases in `qualification-check.pl`, rather than an aggregate count.

Warning-denied all-target/all-feature Clippy and API docs, Rust 1.91 checks,
manifest/Rust formatting and all three metadata/data/upload generator checks pass
(`*-final.log`, `generated-final.log`). All previous metadata fixture rows remain
unchanged. Initial test-compilation/lint failures and review evidence are retained.
No full CI/release gate was run. The coherent local contract batch is ready for
maintainer review/configured CI; native tests establish no IC effect or authenticated
complete transfer. No maintained function, method or type was removed.

See [the upload boundary](../extraction-boundary.md#original-source-bound-ic-snapshot-upload),
[generated contract](../contracts/ic-snapshot-upload.json) and
[fresh source review](../ic-snapshot-upload-source.json). Shared Tooling's adopted
23-file snapshot was unchanged during upload qualification; the later 44-file
adoption is described above. Canic review uses committed source
only, with unrelated dirty sibling paths retained and untouched. Next work is narrow
single-call upload provider/accounting and method-specific lost-effect recovery,
then real backend qualification; source/destination attribution, fresh controllers,
application/fence/load/start safety and terminal/reference release remain pending.

## Released 0.3.6 local artifact and tooling batch

- `DownloadJournalGuard::stage_ic_snapshot_artifact` binds the original Created
  artifact to exact metadata target/timestamp and raw wire evidence. Generic tokens
  stay distinct from raw IC IDs; integrations own their authoritative association.
- The private opt-in v1 tree retains exact metadata/request, three region files and
  exact hash-named chunks. Consuming appends reuse bounded coverage and incremental
  checksums. Errors/drop retain partial bytes without journal completion, recreation
  or another read. Complete fresh closed-tree checks retain the expected checksum
  atomically before the existing durable publisher and Durable transition.
- Original ChecksumVerified recovery adopts exact staging/canonical bytes locally.
  Closing root/parent/canonical custody drift can reject after Durable evidence was
  retained; no path repair or cleanup occurs. Ordinary retained replay reads progress
  only. No transfer/spending journal, new schema/hash owner or release permit appears.
- Explicit `verify_ic_snapshot_artifact` admits the full retained original plan,
  unchanged journal and complete Durable selection, then checks one target's fixed
  format, exact metadata/request, region lengths, bounded known chunk hashes and
  closed-tree checksum. No-follow/nonblocking bounded streaming and closing custody
  checks retain all evidence on rejection. Passive checks hold no future byte custody
  or upload permission; ordinary resume stays artifact-free.
- Shared Tooling pins reviewed clean remote HEAD/main
  `cb86188c5956866564de4fb6ec6be67b27981ab9`, exported unchanged across all 23 files.
  Actual failed validation logs survive a failure to copy into their configured
  retained directory. Consumer release checks inspect index and working paths
  independently, including original metadata hidden by restored working bytes.
- Missing-tag and other conditional Git/formatter fixture checks reject explicitly.
  Real private-index tests reuse existing history without creating commits. Public
  Make commands, v1 receipts and prior release-recovery behavior remain unchanged.
- Both artifact and restore-reference FIFO fixtures use the host's `mkfifo -m 600`
  utility and verify actual FIFO type/private permissions. This removes calls to
  Rustix's Apple-excluded `mknodat` without skipping either safety regression.
  Production behavior, artifact format and dependencies are unchanged.

See [the local artifact boundary](../extraction-boundary.md#durable-metadata-bound-ic-snapshot-artifacts),
[tree contract](../contracts/ic-snapshot-artifact.json),
[fresh source review](../ic-snapshot-artifact-source.json),
[shared adoption](../shared-tooling.md) and [shared audit](../shared-tooling-review.json).
No previously maintained function, method or type was removed. The new artifact
format is explicit and opt-in; existing opaque backend artifacts remain unchanged.

## Qualification and current CI

Native Linux evidence is under `target/snapshot-artifact-review`. All 48 registered
local download-journal cases pass (`journal-portable-fifo.log`), including seven new
writer cases: exact private bytes/full metadata, interleaving, missing/mixed/duplicate
coverage, real descriptor IO failure, occupied/changed/unsafe paths, replaced and
closing custody, zero regions/empty chunks, maximum replies/1,024 chunks, and
acknowledged child death during transfer, after checksum retention and after actual
publication. Six fresh verification cases cover exact reopen, unsafe/changed/missing/
extra children (including FIFOs and empty directories), independent checks despite
forged matching journal checksums, original plan/journal/request admission, replaced
or non-durable custody, 1,024 chunks/maximum replies and a nat64 maximum extent.
Public fresh checks and replay (`public-verification.log`) retain original pending
mutation/observation allowances, exact plan bytes and a nonempty source-reference
record. Seven unchanged publisher-owner cases also pass (`publisher-final.log`).
Registries derive membership from actual listed cases; no fixed total grants authority.

Warning-denied all-target/all-feature Clippy and API docs, Rust 1.91 checks, exact
metadata/data fixture regeneration, formatting and diff checks pass. Earlier compile,
fixture and lint failures remain retained, including the initial retained-verification
fixture whose supposedly wrong extent had the original length. Final verifier logs
use `*-verification-final.log`; exact registered membership and source identities are
retained by `verification-evidence-check.pl`. Follow-up Canic review reads the same
committed source; unrelated current dirty sibling paths are retained separately in
`retained-verification-source` and were neither imported nor changed.
These native wire/filesystem/process tests
simulate no IC behavior and qualify no authenticated complete backend transfer.

Final host review found `mknodat` absent under the pinned Rustix 1.1.5 Apple cfg
in both the new artifact test and the older restore-reference test. Cached upstream
source/cfg evidence is retained in `rustix-apple-fifo-api.txt` and
`rustix-apple-fifo-source.txt`. Real private FIFO rejection passes in both owners;
the focused restore-reference log is `layout-fifo-portable.log`. Warning-denied
Clippy and Rust 1.91 all-target checks pass after the fix (`clippy-portable-fifo.log`,
`msrv-portable-fifo.log`). `portable-fifo-evidence-check.pl` binds current registered
membership/source identities; earlier qualification logs and inventories remain
retained. Native macOS execution is still pending. The coherent local artifact and
tooling batch is ready for maintainer review/configured CI; no upload scope was added.

Focused release/tooling checks and all 23 snapshot files pass; 51 executed consumer
cases include conditional rejection and real index/tree boundaries, alongside the
unchanged vendored runner's command substitutes. Evidence and previous snapshot/audit
copies are under `target/shared-tooling-036-review`. The exact original missing-tag
substitute falsely accepted conditional sourcing; that reproduction remains in
`target/snapshot-artifact-review/conditional-tag-reproduction`. Read-only planning
selected 0.3.5 to 0.3.6 during preparation. Those agent checks ran no full CI or
release gate; the maintainer's later release has its own receipt.

Current [main CI](https://github.com/dragginzgame/ic-backup/actions/runs/37442582446)
and [tag CI](https://github.com/dragginzgame/ic-backup/actions/runs/37442582576) for
the exact 0.3.6 release commit were in progress at the upload batch's read-only
inspection, with no completed workflow result or inspected native macOS qualification.
The sole configured workflow is CI; latest exact-commit results are retained in
`target/snapshot-upload-review/release-036-ci.jsonl`; the earlier queued inspection
remains in `target/post-036-review/ci-runs.jsonl`.
No workflow rerun, cancellation or GitHub write occurred.

Earlier [main CI](https://github.com/dragginzgame/ic-backup/actions/runs/37431702050)
and [tag CI](https://github.com/dragginzgame/ic-backup/actions/runs/37431702209) for
0.3.5 failed: Linux passed, while both macOS jobs stopped at the Bash 3.2 completed-tag
fixture and skipped library validation. The shell condition is reproduced and fixed
locally; fresh macOS/remote confirmation remains pending. Existing
[issue #2](https://github.com/dragginzgame/ic-backup/issues/2) owns shared adoption.
The pending 0.3.7 batch has no remote CI result of its own. Read-only issue/PR
review found no open PRs. Existing [issue #2](https://github.com/dragginzgame/ic-backup/issues/2)
still requests native macOS qualification of the adopted release recovery.
The API confirms [issue #1's](https://github.com/dragginzgame/ic-backup/issues/1)
description correction is present and accurate; the issue remains open. No issue
status was changed and no new tracker was created.

## Remaining product boundary

The next product work is explicitly bounded upload providers and their original
per-call accounting/recovery contracts. Local upload payload/byte preparation and
passive replies install no remote reader, runner or default allocation association.
Authenticated snapshot identity/complete backend transfer, fresh permissions and
spending, application/byte/command custody, concrete capture/upload/load reconciliation,
terminal proof and controlled fence/reference release remain independently qualified.
B1/B2 remain incomplete; no transport, CLI or runners are installed. No host has
end-to-end backup/restore qualification; native macOS and real IC/PocketIC remain pending.

## Released 0.3.5 data coverage and prior tooling

0.3.5 introduced metadata-bound data codecs, incremental ephemeral coverage and the
previous 9437bab Shared Tooling snapshot. Its detailed pre-release handoff is retained
at `target/snapshot-artifact-review/previous-current.md`; finalized notes remain in
CHANGELOG. Historical sections below retain their original evidence and limitations,
without supplying current release authority.

## Local 0.3.4 bounded snapshot metadata reads

The new `model::ic_snapshot_metadata` boundary encodes exact canonical target/raw-ID
metadata reads and admits bounded pinned SDK metadata. It reuses the existing
management wire digest without changing the six-method lifecycle/recovery record.
Ordered globals (including unavailable slots), exact floating bits, full nat64
fields, unsigned v128 values, ordered unique chunk identities and optional
source/timer/hook values remain retained. Missing information supplies no upload
default. An actual reserved-marker adapter preserves zero skipped decoder work
without the SDK's arbitrary-value skipping. Read-only views and request/raw-byte
evidence hashes authenticate no origin or freshness and grant no transfer completeness,
spending, outcome, terminal or release. See
[the boundary](../extraction-boundary.md#bounded-ic-snapshot-metadata),
[generated contract](../contracts/ic-snapshot-metadata.json) and
[fresh review](../ic-snapshot-metadata-source.json).

Preserve 1 MiB raw input, 2 MiB decoder work, zero skipped work, 64 type-table entries,
16 KiB headers, 4,096 global slots, 1,024 unique exact 32-byte chunk hashes, 32
certified-data bytes and existing 256-ID-byte/4 KiB arguments. The new ephemeral
request/reply types have no persisted schema, provider, Serde or non-neutral Default.
No existing public/private function, method or type is removed. Full data extents,
authentic capture/transport association, current read access and original per-call
accounting, application/byte/command custody, upload/load/start and controlled
terminal/fence/reference release remain integration-owned.

Fresh native Linux evidence is retained under `target/snapshot-metadata-review`.
Focused unit and public replay cases check independent generated DIDL/SHA-256
fixtures through production and SDK decoders, exact bytes/hashes/optional states,
numeric/float values, limits and malformed/unknown/truncated wire. Local reopen
preserves original plan and journal bytes, pending spent attempts and source
references without observations or settlement (`unit-final.log`, `public-final.log`).
Existing request-owner fixtures qualify unchanged six-method wire identities
(`request-owner.log`, `request-public.log`). Exact registered/passing membership is
retained in the corresponding registry logs and checked in `evidence-check.log`.
Warning-denied package all-target/all-feature Clippy (`clippy-third.log`) and API
docs (`docs.log`), Rust 1.91 all-target/all-feature checks (`msrv.log`), formatting
(`format.log`), all 22 Shared Tooling files (`snapshot.log`) and read-only release
planning (`release-plan.log`, 0.3.3 to 0.3.4) pass. Initial compile/fixture/Clippy
failures are retained alongside corrected validation logs. No actual IC effect is
simulated or performed. Independent fixtures and machine contract regenerate with
`perl scripts/dev/generate-snapshot-metadata.pl`; `--check` verifies exact bytes.

The read-only Canic review pins `9c6460cbe7c6f375eca1c9b607aefd818deec223`,
retaining its clean-at-inspection status and exact committed source copies. It
traces whole-command download/executor/runner completion boundaries; they remain
unchanged. The registry SDK stays pinned at 0.11.0; source SHA-256 and VCS identity
are recorded independently of current interface-document inspection. No source,
dependency, version or lock selection is imported from Canic.

Historical pre-release assessment: the compatible `0.3.4` batch remained
uncommitted; package stayed `0.3.3`. Native
macOS and real backend qualification, concrete capture/upload/load reconciliation
and runners/transport remain pending. No full CI/release gate or release transaction
ran. Earlier sections retain historical pre-release batch evidence.

## Released 0.3.3 durable original-operation restore artifacts

`publish_staged_local_restore_artifact` joins exact original source/operation/artifact
admission with the existing checksum and descriptor-based durable artifact publisher.
It synchronizes and atomically publishes fixed staging bytes to their canonical
original-operation sibling, then re-admits retained originals and canonical bytes.
The returned existing view plus Published/Recovered outcome borrows both layouts,
source journal and exact declarations. Recovery synchronizes the matching canonical
copy without source-tree reads or recopying. Separate
`verify_published_local_restore_artifact` freshly checks canonical bytes without fsync
or inferring earlier durability from a path. See
[the boundary](../extraction-boundary.md#durable-original-operation-restore-artifacts),
[updated contract](../contracts/local-restore-artifact.json) and
[fresh source review](../local-restore-artifact-publication-source.json).

Preserve original 1 KiB requirement/1 MiB plan-manifest-journal IO, 1,024 targets,
256-token-byte and 64 KiB copy/hash-buffer limits. Staging, publication and both
verification paths share original staging-path exclusion. Held identities are admitted
before new publication/verification lock-sidecar creation. Missing/conflicting/unsafe/
changed copies, replaced layouts and original drift reject without replacement,
repair or cleanup. Failure may retain a completed publication; recover its exact
original paths. No record/schema/hash/accounting owner or journal transition appears;
all attempts, fence obligations and source references remain unchanged. Stable
noncooperating parent/byte custody, authentic snapshots/backend completeness,
application safety and upload/load/start/terminal/release authority remain independent.
Ordinary resume and terminal replay do not run these explicit fresh operations.

Fresh targeted Linux evidence under `target/restore-artifact-publication-review`:

- Eleven local artifact cases pass (`unit-final-custody.log`): six new publication
  cases plus five retained staging cases. They check exact private bytes, source-tree
  absence, explicit Published/Recovered outcomes, missing/conflicting/changed/unsafe
  paths, lost replies before/after actual publication, closing original/canonical
  drift, shared contention, and replaced restore roots with no replacement writes.
  Acknowledged native child death before/after actual durable publication recovers
  exact retained paths without cleanup.
- Two public source journeys pass (`public-final-custody.log`), including the existing
  retained-copy case extended through durable publication/recovery and canonical
  verification while preserving exact original journals, manifest, requirement,
  fence obligation, pending spending and source references. Seven existing canonical
  commit-owner regressions pass (`commit-owner.log`), including synchronization faults
  and no-replace publication races. Actual registry/passing membership is retained
  in the three registry logs and `cases.txt`; these are native local cases, not IC
  transfer or snapshot-effect qualification.
- Warning-denied all-target/all-feature Clippy/API docs and all-target/all-feature
  Rust 1.91 checks pass (`clippy-final-custody.log`, `docs-final.log`, `msrv-final.log`).
  Initial Clippy rejected a fixture's unnested error pattern (`clippy-first.log`);
  the assertion now checks the precise error per injected boundary. A repeated
  native fixture run exposed a timestamp-only temporary-path collision
  (`unit-second-failure.log`). The existing shared test-name owner now also uses a
  process-local atomic counter; original helper functions and test cases remain.
- Formatting and all 22 reviewed snapshot files pass (`format-final.log`, `snapshot.log`).
  Read-only `release-plan.log` previews `0.3.2` to `0.3.3`. Exact committed source
  hashes/ranges, deterministic generated JSON, local links, unchanged package/lock/
  receipt and finalized changelog history pass `evidence-check.log`. Current changed
  source/document hashes are retained in `consumer-files.sha256`.

Fresh read-only Canic review binds `5eb85ddf12bde772123b5e0887308c3c045ac669`
with separately retained clean-at-inspection status and committed staging/publication
source copies under `target/restore-artifact-publication-review/source`. It reviews
staging/checksum and canonical publisher ranges; stale-copy deletion, drop cleanup,
upload integration and every Canic consumer remain unchanged and are not imported.
Existing publisher/source provenance and licenses remain retained. No existing
public/private function, method or type is removed; all prior schemas and APIs remain.

Historical pre-release assessment: the complete compatible batch remained
uncommitted under `0.3.3`; package stayed
`0.3.2`. Native macOS and authenticated IC providers, concrete capture/upload/load
reconciliation, complete terminal/application/command-custody admission and controlled
fence/reference release remain pending. No full CI/release gate, release transaction
or live IC effect ran. Earlier sections retain historical pre-release batch evidence.

## Released 0.3.2 exact originally reserved IC recovery observations

`IcObservationRequest` binds the full original plan/authority, exact original
mutation bytes and already reserved canonical status/list bytes to both pending
original attempt IDs. `IcObservationProvider::observe` describes one previously
accounted host replicated observation; no implementation is installed. A bounded
immutable passive response retains both attempts, authority, observation digest,
actual claimed context/target, exact raw reply and opaque evidence. Pure association
rechecks the current journal and all claims, then uses existing status/inventory
decoders. No additional hash, journal, spending owner or persisted schema appears.
See [the boundary](../extraction-boundary.md#exact-originally-reserved-ic-recovery-observations),
[contract](../contracts/ic-observation-port.json) and
[fresh source inspection](../ic-observation-source.json).

Preserve 1,024 total attempts, 4 KiB arguments, 1 MiB raw replies and all existing
snapshot/controller/decoder-work bounds. This is original-mutation recovery, not
a fresh-preflight reservation API. Missing/changed/settled reservations or payloads
reject. Matching writes no receipt and leaves both attempts pending. Lost replies
are not settled Uncertain. Stopped/controllers and zero/one/multiple snapshots prove
no original effect outcome, freshness, permission, drain/load safety or exclusive
attribution. Actual read permissions, authenticated chronology/attribution and proof
of no prior dispatch remain integration-owned. Failure/drop retains spending,
obligations and source references. No automatic reissue, refund, restart, terminal
proof or fence/reference release is admitted.

Fresh targeted Linux evidence under `target/ic-observation-review`:

- Seven new unit cases pass (`unit-final.log`), including exact original payloads,
  authority/budget/class/target/context/attempt/digest drift, absent/settled/replaced
  reservations, chronological attempt/raw-byte/principal bounds, redacted Debug,
  malformed status/list wire and unchanged pending spending for zero/one/multiple
  inventory entries. One public recovery case passes (`public-final.log`) across both
  observations, all four original mutations and replies/three provider failures.
  Exact original journal/plan/argument/reply bytes, durable source references and
  an opaque native obligation marker survive drop/reopen without another callback.
  Native fixtures simulate no IC behavior or actual application fence.
- Existing request, inventory and lifecycle owner cases pass (`request-owner.log`,
  `inventory-owner.log`, `lifecycle-owner.log`). Actual registered test names and
  exact passing membership are retained in the corresponding registry logs and
  `cases.txt`; these are targeted native cases, not real IC qualification.
- Warning-denied all-target/all-feature Clippy and API docs, plus all-target/all-feature
  Rust 1.91.0 checks pass (`clippy-final.log`, `docs.log`, `msrv.log`). Initial Clippy
  failure is retained in `clippy-first.log`: fixtures used constant-size chunk
  iteration and an oversized recovery helper. They now use `as_chunks` and a
  focused reopen helper; production contracts were not relaxed.
- Formatting and all 22 vendored snapshot files pass (`format.log`, `snapshot.log`).
  Read-only `release-plan.log` previews `0.3.1` to `0.3.2`. Source provenance,
  deterministic JSON regeneration, local links, exact finalized history and unchanged
  package/lock/receipt bytes are checked in `evidence-check.log`.

Fresh committed Canic inspection binds `e739ea9ed59d5cc95517585d31aa82286e76b8d0`
with separately retained clean-at-inspection status and exact committed copies under
`target/ic-observation-review/source`. Reviewed executor status/inventory signatures
and runner observation/completion ranges identify the conversions intentionally not
imported. All Canic consumers remain unchanged. No existing public/private function,
method or type is removed; prior schemas, codecs and API behavior remain unchanged.

This compatible batch remains uncommitted under `0.3.2`; package version stays
`0.3.1`. Native macOS and real IC providers, concrete lost capture/upload/load
reconciliation, complete terminal/application/command-custody admission and controlled
fence/reference release remain pending. No full CI/release gate, release transaction
or live effect ran. Earlier sections retain historical pre-release batch evidence.

## Released 0.3.1 exact originally reserved IC mutation port

`IcMutationRequest` joins full original plan/operation authority, immutable attempt
limits and the existing exact canonical IC capture/load/start/stop payload with an
already pending mutation. `IcMutationProvider::submit_mutation` describes one
previously accounted host update; no implementation or runner is installed. A
bounded immutable passive acknowledgement carries actual claimed context/target,
original authority/attempt, exact raw reply and opaque evidence. Pure association
rechecks current reservations and exact claims, then delegates to existing capture/
lifecycle codecs. No second record, codec, digest encoder or spending owner appears.
See [the boundary](../extraction-boundary.md#exact-originally-reserved-ic-mutation-updates),
[typed contract](../contracts/ic-mutation-port.json) and
[fresh source inspection](../ic-mutation-source.json).

Preserve the original 1,024-attempt, 256 raw snapshot-ID byte, 4 KiB argument and
1 MiB raw-reply bounds plus existing decoder-work limits. Observations cannot use
this mutation path. Missing/different original authority/bytes/attempt and pending
observation recovery reject; matching acknowledgements leave the mutation pending.
Reconstruction proves no previous-dispatch exclusion. Actual fresh control/load-origin
permissions, prerequisite/consistency/application safety, complete source/upload
association and exclusive command/byte custody remain integration-owned. Errors,
drop and lost replies retain spent attempts, obligations and source references.
No automatic receipt, retry, refund, restart or terminal/fence/reference release
follows from a raw snapshot ID or canonical empty acknowledgement.

Fresh focused Linux evidence:

- Eight new model/policy cases pass: `target/ic-mutation-unit-final.log`. They cover
  all four original payloads, exact load bytes, authority/budget/target/context/attempt
  drift, absent/replaced/settled/recovering reservations, raw/attempt bounds,
  canonical principals, redacted Debug and method-specific malformed replies.
- One public recovery journey passes across every method and all three provider
  failure variants plus acknowledgement: `target/ic-mutation-public-final.log`.
  Native provider fixtures implement no IC behavior. Exact journal, plan, argument,
  retained raw reply, opaque fixture obligation marker and durable source-reference
  evidence remain unchanged across drop/reopen; a late acknowledgement invokes no
  provider again, and pending observation recovery stays with its original owner.
  The marker is not a qualified application fence. Six existing exact request-owner
  wire/shape/binding goldens also pass: `target/ic-mutation-request-owner.log`.
  These are fifteen distinct registered targeted cases, not live IC qualification.
- Warning-denied all-target/all-feature Clippy and API docs, plus all-target/all-feature
  Rust 1.91.0 checks pass: `target/ic-mutation-clippy-final.log`,
  `target/ic-mutation-docs.log` and `target/ic-mutation-msrv.log`.
- Actual test registries and passing case names agree in
  `target/ic-mutation-cases.txt`. Exact committed source hashes/ranges, deterministic
  contract regeneration, local documentation links, unchanged package/receipt bytes
  and finalized changelog history pass `target/ic-mutation-evidence-check.log`.
  The initial checker wrongly assumed the inspected Canic worktree was clean;
  `target/ic-mutation-evidence-check-first.log` retains that failure. Corrected
  checks distinguish retained dirty state from the exact reviewed committed files.
  Formatting and all 22 vendored snapshot files pass
  `target/ic-mutation-format.log` and `target/ic-mutation-snapshot-check.log`;
  read-only `target/ic-mutation-release-plan.log` previews `0.3.0` to `0.3.1`.
- Initial fixture compilation incorrectly borrowed a model-owned receipt and passed
  an owned authority to the persistence borrow boundary; the first raw-target test
  also supplied an invalid unhyphenated principal. Corrected fixtures preserve the
  canonical owners. Retained failures are `target/ic-mutation-unit-first.log`,
  `target/ic-mutation-unit-second.log`, `target/ic-mutation-public-first.log` and
  `target/ic-mutation-clippy-first.log`.

Fresh Canic inspection binds committed `abeb37ad9e062d730fe84da3539b9b79e4ed1336`
with separately retained dirty worktree evidence and exact source files under
`target/ic-mutation-source.Jpj2ZW`. Canic advanced independently during this batch;
the inspected files still match the retained committed bytes. Its dirty changes
are neither incorporated nor overwritten.
The reviewed executor/stop/start/capture/restore-preparation ranges identify the
existing dispatch and automatic completion conversions; none are imported or
modified. This batch is a local contract refinement using already maintained
wire/plan/journal owners. No existing public/private function, method or type is
removed; all prior schemas and public request/reply behavior remain unchanged.

Work remains uncommitted under the automatically selected compatible `0.3.1` draft.
Native macOS/real IC providers, capture/upload/load reconciliation, complete terminal
artifact/application/command-custody admission and reference release remain pending.
No full CI/release gate, release transaction or live effect ran in this continuation.
Earlier completed batches below retain their historical source and scope.

## Released 0.3.0 release-check isolation fix

The maintainer committed the pending batch at
`505e0994ecbb4b94ec6f7483336e73b7c9b0c477` (`0.3.0` source notes), then reported
release validation failure. Package version remains `0.2.3`, notes remain undated
`0.3.0` and the latest local release tag remains `v0.2.3`. The real release state
directory is empty: preparation did not start, and there is no saved plan to resume.
Review/commit this fix before retrying the normal selected release command.

The retained failure log is
`target/validation-failures/20261005T170113Z-2464655-4-release-check.log`, with exact
failed fixture evidence under `target/shared-release-tests.CqY8hy`. Its dependency
targets passed, but the fixture inherited the enclosing source SHA and Make
command-line overrides. Its standalone gate then tried to record validation under
the real source rather than its mock source and correctly rejected the mismatch.
Earlier standalone qualification did not exercise that enclosing context.

The consumer suite now clears inherited release identity/helper inputs and GNU
Make recursion/override variables in its child process. Fixture calls still supply
their own exact original context. The unchanged shared runner test is invoked with
its own `make` substitute from both Make and macOS system-Bash CI; an intentionally
invalid inherited `RELEASE_MAKE` otherwise exposed that second boundary. The parent
release environment and production source-SHA guard remain unchanged. No functions,
methods or types are removed. The compatible fix extends the same selected `0.3.0`
pending entry, whose command hard cut still owns its minor-release requirement.

Fresh focused evidence under `target/release-context-review.RtbqlD`:

- `reproduction.log` reproduces the exact failure through actual Make with enclosing
  release identities/command-line overrides; failed fixture evidence is retained
  under `target/shared-release-tests.qfMter`.
- `parent-context-final.log` passes all 40 registered consumer cases in
  `target/shared-release-tests.YBYtdc/cases.txt` and the exact shared runner suite.
  This run explicitly supplies source/previous/candidate/date/kind/remote/branch,
  `VERSION`, nested Make/validation depth and an invalid parent helper. The new
  `validation-source-identity` case admits matching source evidence and rejects
  explicit source mismatch without changing metadata or creating a release plan,
  staged payload, tag or push. All Git/Cargo effects are fixture substitutes.
- `shell-check-final.log` and `workflow-check.log` pass Bash/ShellCheck/Perl and
  Actionlint. An intermediate run passed the consumer suite but exposed the
  inherited helper in shared runner tests; `parent-context-validation.log` and
  `target/shared-release-tests.RFLYRS` retain its actual evidence.

Only consumer fixture setup/callers, CI invocation, pending notes and local docs
changed. No product Rust, vendored bytes, package/lock/receipt identity, real release
transaction or upstream repository was changed. Work remains uncommitted; full
CI/release execution and native macOS qualification were not performed here.
Earlier audit and product evidence below retains its original source and scope.

## Latest Shared Tooling refresh and formatting contract

The second read-only upstream check confirmed `HEAD`/`main` at committed
`c0206f1943238e21bd00fbe01658e6a0864c24fa` (`0.1.2`), with clean source checkouts.
The Git-object refresh expands eighteen files to twenty-two: exact standard
hook/installer plus Cargo ownership and Git-hook rules. Automatic numbered
changelog maintenance and root dependency ownership are now committed rules.
The selected undated `0.3.0` remains correct for the complete breaking release/setup
command batch from finalized `0.2.3`; package version remains `0.2.3`.
Previous exact files, manifest, audit JSON and consumer inputs are retained under
`target/shared-tooling-refresh.BOQlFZ`; earlier failures/evidence are unchanged.
See [current adoption](../shared-tooling.md) and [provenance](../shared-tooling-review.json).

`make install-hooks` activates the unchanged shared hook; it was run and local
`core.hooksPath=.githooks` verified separately from qualification. The old
`hooks-install` command and local installer are retired. Exact index exports
auto-format and refresh only selected regular files, reject partial staging and
preserve unrelated tracked/untracked edits. The crate license is an identical
regular copy of the root notice: **stage it with the batch**, because historical
HEAD/index symlinks reject under the standard hook. No functions or types were
removed from the old installer (it declared none); all consumer test functions
remain and now qualify the maintained behavior.

Cargo-sort 2.1.4 is required consistently by Make, setup and CI; formatting sorts
the root/member manifests before Rust, and independent CI/release checks include
both. Prepared metadata passes non-mutating formatting before receipt/staging.
Normalized offline Cargo metadata is identical before/after sorting, all nine
direct dependencies remain inherited and lockfile/receipt bytes are unchanged.
Package file-list admission includes the regular MIT notice; no archive build or
registry upload was performed. No toolchain or MSRV change occurs.

Release preflight/validation-only failures retry the normal target against corrected
current source with a fresh preflight and full gate. Intent is persisted immediately
after successful validation, before preparation. Safe prior early plans and exact
validation sidecars retain unique attempt archives. Any unfinished prepared plan
blocks another increment, even if the current manifest would select a new version;
only exact saved-version resume proceeds. Local adapters retain original-input
checks, backups, dependency selection and all earlier evidence.

Fresh focused Linux evidence:

- The 39 registered consumer release cases in
  `target/shared-release-tests.XDoQUJ/cases.txt` pass through actual Make entry
  points with Git/Cargo substitutes, followed by the exact updated shared runner
  suite: `target/shared-latest-release-first.log`. New cases cover corrected-source
  gate retry, retained early plans/proofs, prepared-plan rejection, numbered notes
  and failed prepared formatting rollback. No real commits/tags/pushes occur.
- The five registered consumer hook cases in `target/hook-tests.63wRAM/cases.txt`
  pass using actual Make, Git indexes, cargo-sort and rustfmt:
  `target/shared-latest-hooks-final.log`. Shared Tooling's own unchanged hook suite
  independently passes from the clean source checkout:
  `target/shared-latest-upstream-hooks.log`. Its inherited/disabled/private hook,
  concurrency/alternate-index and nested-workspace fixtures qualify the exact
  source owner; they are not installed as a second consumer workflow.
- ShellCheck/Bash/Perl, exact snapshot/consumer integrity and rejection/retention,
  format checking and Actionlint pass in `target/shared-latest-shell-final.log`,
  `target/shared-latest-tooling.log`, `target/shared-latest-format.log` and
  `target/shared-latest-workflow.log`. All twenty-two bytes/modes are checked
  against committed Git objects; provenance, registry membership, package identity,
  license bytes and 172 relevant local links pass in `target/shared-latest-provenance.log`.
  CI installs the pinned formatter explicitly and
  retains native Linux/Apple Silicon/Intel jobs plus macOS system Bash 3.2 checks.
- Initial consumer hook qualification selected the unrelated README accidentally;
  corrected fixtures borrow existing source commits without creating new ones.
  Invoking upstream's suite directly against our historical HEAD exposed its license
  symlink; qualification now runs that upstream suite from its own clean source,
  with actual consumer Make/formatter tests locally. A literal shell-fixture string
  produced ShellCheck SC2016 and was replaced with a literal heredoc. These failures
  remain at `target/shared-latest-hooks-first.log`, `target/shared-latest-hooks-second.log`
  and `target/shared-latest-shell-final.log.initial`.

All work remains uncommitted; no sibling is modified, and no product Rust source,
live provider, release/version transaction or registry publication was changed in
this refresh. Native macOS/real release qualification and the full CI/release gate
were not run. Earlier product and tooling evidence below retains its original source.

## Previous Shared Tooling rules and release workflow adoption

The read-only remote query confirmed Shared Tooling `HEAD`/`main` at
`b8537873ac124ad17b30e32aa23e9006a3e6ec21`. The reviewed Git-object export expands
the snapshot from twelve to eighteen files, including common changelog rules,
release contract/runner and helpers/regressions. Exact former files/manifest remain
under `target/shared-tooling-review.ztA5F9`. Newer uncommitted upstream proposals
are separately hashed evidence, never exported as committed policy. See
[adoption](../shared-tooling.md) and [review provenance](../shared-tooling-review.json).

Public patch/minor/major now invoke one exact shared runner with explicit branch/
remote inputs. Exact saved-version recovery uses `release-resume VERSION=X.Y.Z`.
Standalone preparation/stage/commit/push aliases and arbitrary one-shot `release-x`
are removed without compatibility wrappers. This command/semantics hard cut is why
the complete pending batch requires `0.3.0`, despite unchanged Rust APIs/v1 records.
Consumers must select the common one-shot command, then resume its exact saved plan
after interruption; older standalone prepared releases need original identity review.
Package versions/publication remain maintainer-owned; see [the guide](../releasing.md).

Consumer adapters retain original-input validation and metadata backups, preserve
the exact dependency selection, and reuse the existing v1 package receipt. Missing
evidence or changed original inputs rejects before preparation. Only this package's
lockfile version changes during preparation; no Cargo update is called. Ordinary
failure restores originals and retains backups. The shared runner owns the explicit
release index, commit/tag identity and exact atomic branch/tag push with
`--no-follow-tags`. Completed local resume validates original receipt/validation and
tag evidence. Source, plans, build outputs and recovery evidence remain retained.
There is one release workflow; the metadata adapter has no Git mutation functions.

Fresh focused Linux evidence:

- All 34 consumer cases registered in `target/shared-release-tests.4aF2lG/cases.txt`
  pass through actual Make entry points with Git/Cargo substitutes, followed by
  exact vendored runner regressions: `target/shared-rules-release-final.log`.
  Coverage includes all increments/same phase order, exact index/push scope,
  dependency preparation/failure retention, matching pending heading/historical
  preservation, original validation, rollback, lost commit/tag/push replies,
  changed destinations, missing evidence/member drift and completed receipt/tag/HEAD
  drift. All fixture command traces remain retained; no real Git effects or upload
  occur in qualification. Public `make -n` is also proven read-only.
- ShellCheck/Bash/Perl checks and snapshot/consumer integrity, rejection and retained
  evidence regressions pass: `target/shared-rules-shell-final.log` and
  `target/shared-rules-tooling-final.log`. All eighteen snapshot paths pass exact
  source bytes/modes and provenance/link checks. CI adds the exact runner suite to
  the existing macOS system-Bash job; native macOS qualification remains pending.
- Two initial adapter-suite failures exposed an empty-index predicate's return
  status and an out-of-scope publication-lock trap variable. Corrected final runs
  pass; failures remain in `target/shared-rules-release-first.log` and
  `target/shared-rules-release-second.log`, with their complete isolated fixtures.
  An early dry-run of the upstream Make example's `+` recipe attempted a real local
  release lock; the sandbox refused `.git/release-state` creation before validation
  or Git effects. Consumer recipes omit `+`; actual Make fixture tests prove that
  dry-run now dispatches nothing. Initial ShellCheck diagnostics and that refusal
  are retained under the review evidence directory.
- No Rust/dependency source changed in this tooling batch. The previous 47-case
  Rust checks below remain their actual retained evidence; they are not relabelled
  as new runs. No complete CI/release gate, native macOS workflow or real release
  was run. Manifests, lockfile and receipt remain at the released `0.2.3` baseline.

Removed functions in `scripts/release/release.sh`: `tag_absent`, `bump`, `stage`,
`commit_release`, `remote_preflight`, `push_release`. Common runner preflight,
validation/preparation, stage, commit/tag and exact push/reconciliation replace
their release effects; consumer metadata retains only its canonical preparation
and checks. No Rust functions/types or test helpers are removed. The tooling batch
crosses snapshot/governance, release adapters, tests, CI and local documentation;
The current tooling/governance footprint relative to `0.2.3` is about 24 files,
1,900 added and 850 removed lines, including pending documentation propagation
and exact shared exports. Width follows that one command contract. Ownership is simpler with one Git-effects
owner, while retained validation/recovery adds explicit tooling state. Changes
remain uncommitted; no sibling checkout is changed. The stale GitHub description
was corrected and read back only after the maintainer explicitly approved its
exact replacement. Automatic approval review initially rejected that remote
metadata action under audit-only authority; no effect occurred before approval.

## Private original-operation restore artifact staging

The [maintained boundary](../extraction-boundary.md#private-original-operation-restore-artifacts)
adds private staging for exact original opaque operation identities and explicit
retained-copy verification. It reuses complete fresh original-source admission,
the existing descriptor-copy/checksum and journal-lock owners. Fixed direct children
of the held restore layout use 0700/0600 creation permissions. Copied and freshly
checked destination hashes equal the original retained checksum, and retained
original records are re-admitted before return. No new record, digest encoder,
accounting owner or dispatch permit is added. See
[the typed contract](../contracts/local-restore-artifact.json).

Occupied destinations reject without adoption/overwrite/deletion. Copy errors,
changed original admission and drop retain partial/full bytes. Explicit
`verify_staged_local_restore_artifact` admits exact retained originals and copy
bytes without re-reading source trees or copying again. The original trees may
be absent; unchanged metadata remains required. Both views borrow original guards
and preserve spending, unfinished source references and fence obligations. Staging
has no fsync/durable publication or actual backend transfer attestation. Sequential
observations and pathname destination creation require stable noncooperating
destination/byte custody; holding a view does not freeze the filesystem.

Fresh targeted Linux evidence:

- Five new unit cases pass in the 29-case download-owner selection:
  `target/local-restore-artifact-download-owner.log`. They cover exact original
  identity/bytes/private permissions, occupied/unknown-operation rejection, corrupt/
  unsafe/missing retained copies, source changes during copying, changed original
  admission after copying, contention, partial/lost replies and acknowledged process
  death before/after copying. Existing original-source/download/manifest interruption
  and process-death regressions pass in the same selection.
- Eight existing secure artifact owner cases pass:
  `target/local-restore-artifact-copy-owner.log`. Three pure original-source join
  cases and five durable-source owner cases pass:
  `target/local-restore-artifact-source-policy.log` and
  `target/local-restore-artifact-integrity-policy.log`.
- Two public recovery journeys pass: `target/local-restore-artifact-public-final.log`.
  The new journey reopens a retained copy with source trees moved aside, preserving
  exact pending spent attempts, manifest/requirement/journal/fence bytes and source
  references. There are 47 distinct passing targeted cases in this continuation.
- Warning-denied all-target/all-feature Clippy, rustdoc and Rust 1.91.0 checks pass:
  `target/local-restore-artifact-clippy-final.log`, `target/local-restore-artifact-docs.log`
  and `target/local-restore-artifact-msrv.log`. The first five-case unit run passed
  with two unused-import/variable warnings, retained in
  `target/local-restore-artifact-unit-first.log`. The first Clippy run rejected a
  109-line public test (`target/local-restore-artifact-clippy-first.log`); shared
  original-source fixture setup now replaces duplicate inline setup in both public
  cases. Final checks are warning-free; no test failed.
- Formatting/whitespace, all 12 pinned Shared Tooling files, read-only release-plan
  and selected changelog checks pass under the `target/local-restore-artifact-` prefix.
  Exact retained source/consumer references, contract replay, canonical owners and
  local Markdown targets pass `target/local-restore-artifact-provenance-check.log`.
  Manifest, lockfile, release receipt, existing schemas and vendored tooling remain
  unchanged. No complete CI/release gate or release transaction was run.

[Fresh read-only Canic inspection](../local-restore-artifact-source.json) binds
`cb596fc722dad1b0b6fb94c9fd42511920244f57` with separately recorded worktree status.
The exact source is retained under `target/local-restore-artifact-source.TpBbbV`;
generators/reference scan remain under the `local-restore-artifact-` prefix.
Eleven exact reference excerpts identify actual consumers. No product source is
copied; source stale-copy deletion, drop cleanup, upload gating and runner execution
are not imported. Existing public/private symbols remain; original-record admission
is shared inside its owning module. These uncommitted notes now belong to `0.3.0`.

This completes private local copy/recovery admission. Actual authenticated snapshot
upload/extent completeness, application subset/restore safety, current permissions,
command dispatch and full terminal/fence/reference release remain independently
qualified. No paid call, restart or cleanup authority follows. Runners/transport,
native macOS and actual application/IC qualification remain pending.

## Fresh original local restore-source verification

The [maintained boundary](../extraction-boundary.md#fresh-original-local-restore-source-verification)
joins both retained original plans, immutable safety requirement, exact local
download-manifest digest and unchanged guarded journal. Canonical owners admit
same network/release/existing selected IDs, original intent and complete Durable
source coverage. Read-only selected views borrow original snapshot metadata, paths
and checksums. A subset needs separate application safety qualification; differing
source/restore callers remain allowed. See the [typed contract](../contracts/local-restore-source.json).

The opt-in local binding uses the existing complete manifest digest as original
`source_artifacts`; generic integration digests keep their meaning. No new record,
schema, spending owner or authority flag is added. The download guard can now replay
its manifest without reacquiring its held journal lock, still reading records only.
Explicit `verify_local_restore_source` admits retained originals before/after fresh
no-follow checks of every source tree, including those outside the restore subset.
The returned view borrows both layout lifetimes, original plans/requirement and source
journal. It changes no records, references, allowances or obligations and invokes
no provider. Existing 1 KiB requirement, 1 MiB record IO, 1,024-target and 256-token-byte
bounds remain; sequential checks require stable noncooperating byte custody.

Fresh targeted Linux evidence:

- Seven new unit cases pass. Four are in the 32-case download-owner selection:
  `target/local-restore-source-download-owner.log`; three pure join cases are in
  `target/local-restore-source-policy.log`. Coverage includes exact original hashes,
  subset projections, metadata/checksum drift, incomplete/non-Durable or mismatched
  local source bindings, changed unselected bytes, missing/unsafe/changed original
  records, root replacement and manifest contention. Existing manifest/download
  interruption and process-death regressions also pass in that owner selection.
- All five pure durable-source owner regressions pass, including maximum selection:
  `target/local-restore-source-integrity-policy.log`.
- One public recovery journey passes: `target/local-restore-source-public.log`.
  Byte corruption rejects explicit verification while record replay succeeds;
  corrected fixture bytes and drop/reopen preserve exact spent pending attempts,
  manifest/requirement/journal/fence bytes and unfinished source references without
  provider calls. There are 41 distinct passing targeted cases in this continuation.
- All-target/all-feature warning-denied Clippy, rustdoc and Rust 1.91.0 checks pass:
  `target/local-restore-source-clippy.log`, `target/local-restore-source-docs.log`
  and `target/local-restore-source-msrv.log`. The first seven-case run passed with
  an unused test-import warning retained in `target/local-restore-source-unit-first.log`;
  the import was removed and all final checks are warning-free. No test/lint failed.
- Formatting/whitespace and all 12 pinned Shared Tooling files pass:
  `target/local-restore-source-format.log`, `target/local-restore-source-diff-check.log`
  and `target/local-restore-source-shared-tooling.log`. Exact retained sources/consumers,
  contract replay, canonical schema/digest owners and local Markdown targets pass
  `target/local-restore-source-provenance-check.log`.

[Fresh read-only Canic inspection](../local-restore-source.json) binds
`c6da13cc478b76c83eb51f4600f5acd66e47501a` with separately recorded dirty source.
Exact inputs remain under `target/local-restore-source.1QbU5M`; generators and reference
scan are retained with the `local-restore-source-` prefix. No product source is copied;
relocation/mapping, parent ordering, optional-checksum readiness, staging/cleanup and
runner behavior are not imported. Existing public/private symbols remain in place;
common manifest decoding is extracted into its canonical owner and journal equality
admission is shared within its owning module. Changes remain uncommitted.

This completes the local original-source join, not full restore or terminal admission.
It proves no atomic source snapshot, future copied/upload bytes, authenticated capture
or complete backend extents. Application subset safety, effect-boundary permissions,
exclusive dispatch/command custody, remaining IC reconciliation and full terminal
fence/reference release remain independently qualified. Runners/transport and native
macOS/actual application/IC qualification remain pending.

The following two entries describe work now included in released `0.2.3`. Earlier
target logs and source copies are absent in this checkout; their paths below describe
historical results, not freshly retained qualification. Offline caches for the selected
lockfile were explicitly prepared with `cargo fetch --offline --locked`; all new
`local-restore-source-*` inputs and checks above are retained.

## Immutable local download manifests

The [maintained boundary](../extraction-boundary.md#immutable-local-download-manifests)
adds immutable publication of the exact verified local download set. It reuses
`DownloadJournalRecord`, canonical selected-target/Durable checksum policy and
guarded fresh no-follow verification, adding no second schema or progress ledger.
The original plan, exact snapshot token/metadata, derived paths, state and checksum
have a model-owned binary digest. Existing 1 MiB IO, 1,024-artifact and 256-token-byte
bounds are unchanged. See the [contract and independent goldens](../contracts/download-manifest.json).

Private no-replace `download-manifest.json` publication rejects conflicts; an unknown
reply is reconciled explicitly under the expected full digest. Local replay admits
the original retained plan and unchanged original download journal using exclusion.
It never reads artifact trees, invokes providers, reconstructs provenance or changes
progress, allowances or references. Fresh byte verification remains a separate action.
Complete backend extents, authentic snapshots, consistency and full terminal admission
remain integration-owned; sequential checks need stable noncooperating byte custody.

Fresh targeted Linux evidence:

- Seven new unit cases pass in the 28-case download-owner selection:
  `target/download-manifest-download-owner.log`. Coverage includes all-state independent
  binary goldens with maximum metadata/token values, canonical order/checksum identity,
  changed final-tree bytes and original journal, private immutable publication, exact
  replay, contention, missing/unsafe/excessive records, lost replies and acknowledged
  process death before publication and after directory sync.
- All five pure selected-set/Durable policy-owner regressions pass:
  `target/download-manifest-policy-owner.log`.
- The new public manifest replay journey and existing settlement journey pass:
  `target/download-manifest-public.log`. Repeated reopen/replay with artifact trees
  moved aside preserves original journal bytes, unfinished source references and the
  original fence obligation. There are 35 passing targeted cases in this continuation.
- All-target/all-feature warning-denied Clippy, rustdoc and Rust 1.91.0 checks pass:
  `target/download-manifest-clippy-final.log`, `target/download-manifest-docs.log`
  and `target/download-manifest-msrv.log`. Initial cast/test-format lint failures remain
  in `target/download-manifest-clippy-first.log`; corrected code passes.
- Formatting/whitespace and all 12 pinned Shared Tooling files pass:
  `target/download-manifest-format.log`, `target/download-manifest-diff-check.log`
  and `target/download-manifest-shared-tooling.log`. Exact retained sources/consumers,
  independent contract/golden replay and local Markdown links pass
  `target/download-manifest-provenance-check.log`.
- Read-only `release-plan VERSION=0.2.3` reports `0.2.2` -> `0.2.3`:
  `target/download-manifest-release-plan.log`. Changelog admission accepts that selected
  draft. No preparation, full validation gate or real release transaction ran.

[Fresh read-only Canic inspection](../download-manifest-source.json) identifies
`c6da13cc478b76c83eb51f4600f5acd66e47501a`, separately records dirty source and retains
exact inputs under `target/download-manifest-source.Gedhro`. No product source is
copied, no sibling changes occur and earlier provenance/evidence is unchanged.
Framework parent-derived consistency, tool/time provenance reconstruction, completed
receipts and automatic adoption were not imported. The additions were additive
and remove no existing public or private symbols; the public fixture now returns its
existing source declaration for the additional journey. Existing v1 records are unchanged.

The released 0.2.3 baseline includes both this manifest batch and the execution settlement
batch below. These are local retained evidence components, not a complete backup/restore
product. Full manifest/transfer and authenticated effect qualification, actual application
safety and command custody still precede terminal fence/reference release. Application
providers, dispatch admission, remaining IC reconciliation, runners/transport and native
macOS/actual application/IC qualification remain pending.

## Original execution settlement checkpoints

The [maintained boundary](../extraction-boundary.md#original-execution-settlement-checkpoints)
adds an immutable local checkpoint of every original operation's exact chronological
attempt history. Admission reuses canonical execution progress for complete original
plan/context/operation/budget coverage and retained Applied prerequisites, requires
every operation Applied, and matches all history fingerprints. Fingerprints include
negative/uncertain receipts, evidence and consumed reservations; identical final views
cannot hide changed receipts. There is no second spending ledger or completion flag.
See the [strict v1 schema and independent binary goldens](../contracts/execution-settlement.schema.json).

Fixed `execution-settlement.json` has bounded 2 MiB immutable private publication and
exact local replay under exclusive layout custody. Existing 8,192-operation,
65,536 combined original-attempt, 2,048-event and 1 MiB per-journal limits remain.
Bulk authority derivation hashes the original plan once; sequential exact journal
locks bound descriptor use. Callers drop journal guards before admission; Applied
owners reject further transitions. Noncooperating byte custody remains separately
qualified. Lost publication replies reopen exact local evidence without rewriting,
provider calls, fresh artifact verification or replenishing original allowances.

Fresh targeted Linux evidence:

- Fifteen new unit cases pass: `target/execution-settlement-unit-final.log`.
  They cover independent full-history/checkpoint goldens, strict bounded admission,
  receipt drift, missing/pending/uncertain/NotApplied evidence, publication failure,
  contention and acknowledged process death before publication and after directory sync.
- One public replay journey passes after the fixture refactor:
  `target/execution-settlement-public-final.log`. Drop/reopen and repeated replay retain
  exact spent journal bytes, remaining original allowances, fence obligations and
  unfinished source references without provider or IC/application effects.
- Eight plan-owner and ten attempt-journal-owner regressions pass:
  `target/execution-settlement-plan-owner.log` and
  `target/execution-settlement-attempt-owner.log`. There are 34 targeted passing tests
  in this batch, comprising 33 unit cases and one public integration case.
- All-target/all-feature warning-denied Clippy, rustdoc and Rust 1.91.0 checks pass:
  `target/execution-settlement-clippy.log`, `target/execution-settlement-docs.log`
  and `target/execution-settlement-msrv.log`.
- Formatting/whitespace checks and all 12 pinned Shared Tooling files pass:
  `target/execution-settlement-format.log`, `target/execution-settlement-diff-check.log`
  and `target/execution-settlement-shared-tooling.log`. Exact source copies/consumers,
  independent schema/golden replay and local Markdown targets pass
  `target/execution-settlement-provenance-check.log`.

Initial fixture/schema checksum-string failures remain in
`target/execution-settlement-unit-first.log` and
`target/execution-settlement-schema-first-attempt.json`; corrected fixtures reuse
existing checksum record objects. The public-test length lint failure remains in
`target/execution-settlement-clippy-public-first.log`; the extracted fixture helper
passes. Earlier successful runs and independent generators remain retained too.

[Fresh read-only Canic inspection](../execution-settlement-source.json) binds
`c6da13cc478b76c83eb51f4600f5acd66e47501a` and a separately identified dirty working
tree; exact source inputs remain under `target/execution-settlement-source.igm5gK`.
The source's completion counts and command-controlled reference release are not
imported. This batch adds local contracts using existing owners; no existing public
symbols or v1 records were removed or replaced. The additions are now released in 0.2.3.

This checkpoint proves retained original journal settlement only. Full product
terminal admission still needs artifact/manifest/transfer evidence, authenticated
effect attribution and chronology, actual application safety and command quiescence
before fence/reference release. Application codecs/providers, exclusive dispatch
custody, remaining IC reconciliation, runners and transport remain pending. Native
macOS and actual application/IC behavior remain unqualified by these native checks.

The next two entries describe implementation now included in released `0.2.2`.
Their versions and evidence paths record conditions at implementation time. Earlier
target logs/source copies are absent in this checkout; they are historical results,
not freshly retained qualification. Those evidence paths record retention at implementation time.

## Exact application fence acquisition requests

The [maintained boundary](../extraction-boundary.md#exact-application-fence-acquisition-requests)
adds immutable bounded application receiver/method/argument envelopes and a
single-update provider contract under the exact already reserved original mutation.
The nonrecursive payload digest binds raw canonical receiver, update mode and exact
method/argument bytes before the plan/requirement/obligation. Methods retain 1..128
visible ASCII bytes; opaque arguments are bounded to 1 MiB before copying and Debug
excludes their contents. See the
[typed contract and independent goldens](../contracts/fence-acquisition-port.json).

Passive acknowledgement association binds full original authority and mutation
attempt, while retaining the pending original journal unchanged. It produces no
outcome or receipt. Provider admission requires actual whole-unit application
semantics, original bytes/custody, fresh context/permissions, complete prerequisites
and proof of no prior dispatch. Reconstruction after interruption grants no retry.
Acknowledgements/all provider failures retain original obligations, source references
and consumed pending attempts; recovery uses the reserved observation contract.

Fresh targeted Linux evidence:

- Eight new unit cases pass; the focused `fence_` selection also reran related owners:
  32 total passing tests in `target/fence-acquisition-related-unit.log`.
- The new public recovery journey and two existing reconciliation journeys pass:
  `target/fence-acquisition-public.log`. Native fixtures retain exact inputs and
  obligations/references through acknowledgement/failure, reopen and observation
  handoff without a second acquisition invocation. No IC/application effects occur.
- All-target/all-feature warning-denied Clippy passes:
  `target/fence-acquisition-clippy.log`. Earlier cast/test-format and long-test
  failures are retained in `clippy-first.log` and `clippy-second.log` with the same prefix.
- Warning-denied rustdoc and Rust 1.91.0 all-target/all-feature compilation pass:
  `target/fence-acquisition-docs.log` and `target/fence-acquisition-msrv.log`.
- Formatting/diff checks pass; Shared Tooling verifies all 12 pinned files in
  `target/fence-acquisition-shared-tooling.log`. Source revision/checksums, retained
  inputs, exact consumers and independent goldens pass
  `target/fence-acquisition-provenance-check.log`.
- Read-only `release-plan VERSION=0.2.2` reports `0.2.1` -> `0.2.2`; changelog
  admission accepts the selected draft. No preparation or release gate ran.

[Fresh Canic inspection](../fence-acquisition-source.json) is bound to
`071a9c64d7ff71ba7a0a695d24a66de11b651aed` with a separately identified dirty
working tree; exact source copies/generators are retained under
`target/fence-acquisition-*`. Earlier provenance is unchanged. No Canic default
program, command flags, runner, acquisition code or generic fence proof is imported.
The additions were additive; existing public contracts and v1 records were unchanged.
Actual application codecs/providers, exclusive dispatch
admission/custody and terminal-controlled release remain pending before runners.
Native macOS and actual application/IC qualification remain outside these checks.

## Reserved fence acquisition reconciliation

The [maintained boundary](../extraction-boundary.md#reserved-fence-acquisition-reconciliation)
adds a read-only observer contract for an existing pending original acquisition
under its exact reserved observation. Requests/results bind original authority,
budgets, obligation, challenge and both attempt IDs. Pure checks require exact
actual context, full inventory, selected unit and explicitly attributed Active
fence/revisions. Absence alone proves no nonapplication; a lost reply remains
pending rather than becoming settled uncertainty. See the
[typed contract and independent goldens](../contracts/fence-reconciliation-port.json)
and [fresh source inspection](../fence-reconciliation-source.json).

Fresh targeted Linux evidence:

- Ten unit cases pass: `target/fence-reconciliation-unit-final.log`.
- Two public native recovery journeys pass: `target/fence-reconciliation-public.log`.
  Provider failures retain exact spent pending evidence and references across reopen;
  retained late replies need no repeat provider call. Passive fixture claims exercise
  existing guarded receipt transitions without qualifying real application effects.
- Clippy passes for package all-targets/all-features with warnings denied:
  `target/fence-reconciliation-clippy.log`.
- Rust 1.91.0 all-targets/all-features check passes:
  `target/fence-reconciliation-msrv.log`.
- Rustdoc with warnings denied passes: `target/fence-reconciliation-docs.log`.
- Formatting and diff whitespace checks pass; Shared Tooling verifies all 12 pinned
  files in `target/fence-reconciliation-shared-tooling.log`. Source checksums,
  retained copies, exact consumer references and independent binary goldens pass
  `target/fence-reconciliation-provenance-check.log`.

Initial Clippy documentation-formatting and unused-import failures remain in
`target/fence-reconciliation-clippy-first-attempt.log` and
`target/fence-reconciliation-clippy-second-attempt.log`; corrected code passes.
Source inspection copies and independent Perl generators were retained under
`target/fence-reconciliation-*` at implementation time.

The observer installs no authenticated provider or acquisition/release workflow;
views create no automatic receipts, spending, dispatch permits or terminal proof.
Actual original acquisition semantics, authenticated attribution/custody and
dispatch admission remain integration-owned, followed by controlled terminal
release and unresolved IC effects before runners. Native macOS, actual application
fencing and PocketIC/real-IC effects remain unqualified by these checks.

The following entries record work now included in the released 0.2.1 baseline;
their versions and evidence paths describe the conditions at implementation time.

## Explicit release selection and provisional changelog labels

Preparation now lets the explicit release command relabel the single current
future draft, preserving its exact notes and history. The reported `release-minor`
attempt selected `0.3.0` from package `0.2.0` but was blocked by the provisional
`0.2.1` heading. A future draft label now yields to the explicit target; duplicate,
competing, empty, misplaced or dated drafts and existing target sections still
reject. Imported undated history cannot become the new top draft. The final draft
label is resolved against the original manifest before version mutation, under
the existing rollback that restores the original label and all release bytes.
Preparation prints its exact current/target versions before validation.

`make release-check` passed against isolated Git/Cargo/gate substitutes, including
patch/minor/major retargeting, exact preserved notes/history, receipt checks and
gate/update/metadata failure rollback. Evidence is
`target/changelog-retarget-release-check.log`. These tests perform no real commits,
tags, pushes, uploads or cleanup of consumer build/evidence artifacts.
`make shell-check` also passed, including Perl syntax; evidence is
`target/changelog-retarget-shell-check.log`. Read-only admission accepts both
the current `0.2.1` target and explicit `0.3.0` target without changing files.
The manifest remains `0.2.0`, the current changelog stays undated `0.2.1`, and the
existing release receipt is unchanged. This is a consumer-owned release-helper
fix; vendored Shared Tooling bytes and sibling repositories are unchanged.

## At a glance

| Question | Current answer |
| --- | --- |
| Can it perform a complete backup or restore? | No. The transport, runners and CLI remain unimplemented |
| What works today? | Local artifacts, bounded records, journals, plans, immutable manifests/checkpoints, original local restore-source verification, selected IC codecs and pure integration checks |
| What has been qualified? | Native local filesystem, record, policy and process behavior within the evidence described below |
| What remains integration-owned? | Live membership, authority, application consistency, authenticated calls and restored-state acceptance |
| What is the next product boundary? | Full terminal artifact/application/command-custody admission before controlled release, qualified application providers and exclusive dispatch custody, remaining IC reconciliation and runners/transport qualification |

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-backup/ic-backup-product-readiness.svg" alt="Product readiness stages showing the implemented local safety foundation followed by application adapters, authenticated IC transport and runners, and PocketIC and live qualification" width="800">
</p>

The released baseline includes original application fence obligations, acquisition
reconciliation/envelopes, local execution settlement checkpoints and immutable download
manifests. The new draft joins original local restore-source verification without
releasing those obligations or references.
Candid 0.10.37 and management SDK 0.11.0 remain
locked unchanged. The manifest, lockfile, release receipt, earlier provenance and
vendored Shared Tooling bytes are unchanged. Only targeted Linux checks ran for
this continuation; native macOS and actual IC/application qualification remain
unestablished by these checks.

## Original application fence obligations and acquisition recovery

The [maintained boundary](../extraction-boundary.md#original-application-fence-obligation-retention)
now retains exact full-plan/selected-unit capture or restore fence obligations,
original requirements/revisions and an explicit application acquisition operation.
Fixed `fence-obligation.json` has strict v1 1 KiB immutable publication/read with
exact original retained requirements and plans, including both restore/source
layout guards. Weaker lanes, zero acquisition allowance, missing/changed originals,
rebound identities, unsafe/excessive bytes and contention reject. See
[the schema and independent binary goldens](../contracts/fence-obligation.schema.json).

The attempt journal remains the sole spending/reconciliation owner. Pure
`acquisition_progress` joins its exact original operation/context/request/limits
and returns its existing view. Missing evidence cannot mean zero consumption,
lost observations stay pending, Uncertain leaves the mutation unresolved, and
no outcome refunds attempts. Pending, Applied, NotApplied and Uncertain outcomes
retain the original obligation and unfinished source references. An Applied
projection proves no current Active custody or release permission.

Ten focused unit tests and one public native filesystem recovery journey passed.
All-target/all-feature warning-denied Clippy, rustdoc and Rust 1.91.0 compilation
passed. Logs are `target/fence-obligation-unit.log`, `public-final.log`,
`clippy-final.log`, `docs-final.log` and `msrv-final.log`, each with the
`fence-obligation-` prefix. No test or
lint attempt failed in this batch. The public journey preserves exact obligation/
journal bytes and unfinished source references through exhausted lost replies,
every passive settled outcome and drop/reopen. These qualify local contracts only.
Formatting/whitespace checks, all twelve Shared Tooling snapshot files, schema
generator replay, exact inspected source/consumer references and 156 local
Markdown targets also passed; evidence is `target/fence-obligation-format.log`,
`diff-check.log`, `snapshot-check.log` and `evidence-check.log`, each with the same
prefix. Manifest, lockfile, release receipt and vendored baseline remain unchanged.

[Fresh Canic inspection](../fence-obligation-source.json) identifies the same
dirty source HEAD `a2108801d7b2d5802f3e864556dc8781b3b382df`, three exact source
files and their actual text consumers. It confirms retained-reference/terminal
command-custody responsibilities and unreconciled load behavior, supplying no
generic application-fence acquisition or release proof. No source code was copied;
the new contract refines the local product design and reuses existing owners.
Exact read-only inputs, schema/provenance generators and consumer-reference
evidence remain under `target/fence-obligation-*`; earlier provenance is unchanged.

No coordinator implementation, authenticated acquisition reconciliation, fresh
Active proof, dispatch, terminal evidence or controlled fence/reference release
is implemented. Original opaque request semantics, identity and whole-selection
fence custody remain integration-qualified. The next contract work covers actual
coordinator requests and qualified reconciliation, then terminal evidence before
release admission. No full gate, release/version transaction, upload, commit or
live effect ran. This batch was delivered uncommitted under the undated 0.2.1
changelog draft at package 0.2.0; the maintainer subsequently committed it at
`8646589`.

## Original restore/source safety batch

The [maintained boundary](../extraction-boundary.md#original-restore-safety-requirements-and-current-loadstart-checks)
now has immutable same-network/release selected-source declarations and a fresh
`RestoreSafetyProvider` contract for original exact load/start bytes. Required
safety lanes have no generic default. Load needs all selected targets stopped;
start needs its target stopped, every selected restored state accepted for the
exact source and fenced execution qualified. A still-stopping member rejects.
Original source artifacts and fence/membership/external-obligation revisions
cannot silently rebind. Both retained plans and 1 KiB no-replace requirement IO
are checked under unchanged layout exclusion. See
[the schema and independent goldens](../contracts/restore-safety-requirement.schema.json)
and [typed contract](../contracts/restore-safety-port.json).

Sixteen focused unit tests and one public recovery journey passed, with
warning-denied all-target/all-feature package Clippy/rustdoc and Rust 1.91.0
all-target/all-feature compilation. Logs are `target/restore-safety-unit.log`,
`public.log`, `clippy.log`, `docs.log` and `msrv.log`, each with the same
`restore-safety-` prefix. Cases cover strict schema/binary hashes, canonical target
bounds, exact source/context/payload/lane/revision/lifecycle/acceptance denials,
immutable both-layout persistence and unsafe/oversized/rebound files. The public
native fixture preserves exact journal/requirement/outside-source obligation
bytes and unfinished restore references through exhausted pending attempts,
stale/inactive/rebound results, every typed provider failure and drop/reopen.

[Fresh Canic inspection](../restore-safety-source.json) records six working-tree
files and 334 exact text consumer references under HEAD
`a2108801d7b2d5802f3e864556dc8781b3b382df`, with dirty source explicitly recorded.
Fixed-ID/stopped-load checks are adapted; mapping/Root/argv, module-hash-only
verification and status reconciliation are not imported as safety or load receipts.
The source supplies no generic outside-snapshot external-work proof; that contract
is designed from the local product requirements, not claimed as copied qualification.
Exact inspected source bytes remain at `target/restore-safety-source.Z5hWZi/`;
reference inventory and independent contract generation/check evidence remain
under `target/restore-safety-*`. Earlier provenance and licenses are unchanged.

Initial Clippy attempts flagged function length; responsibilities were split and
successful checks rerun. Diagnostics remain at
`target/restore-safety-clippy-first-attempt.log`, `second-attempt.log` and
`third-attempt.log`, each with the same prefix. No full CI/release gate, package
upload, commit or live IC effect ran. The next release version was undecided
at that batch's completion; the current selection is recorded above.
Applications still qualify
authentic complete source/upload association, fresh lifecycle/release/drain,
irreversible-work absence or continuous rewind-independent fence/replay safety,
restored acceptance and prior per-call accounting. No provider, fence acquisition/
release, lost-load settlement, dispatch or terminal/source-reference release is
implemented. These are local contracts, not actual management/application safety.

The entries below retain earlier development evidence. Their draft/version/gate
statements describe original execution before the maintainer's 0.2.0 release.

## Shared Tooling best-practice review and refresh

The current common baseline and all twelve declared tooling files now come from
clean Shared Tooling commit `41e1fd0ba41460bd2127cbf98ac8a4b2b2020d3e`, inspected
read only on 2026-10-05. The reviewed refresh adds `DRAGGINZGAME.md` to the same
checksum/mode manifest as its linked guides and helpers; its source remote is
deliberately recorded with the source checkout's canonical HTTPS spelling.
The upstream exporter read exact committed Git objects, and the completed
snapshot verified offline. No sibling source or Git history was changed.
Previous files and manifest remain at `target/shared-tooling-review.4NA5Aq/previous/`
and `target/shared-tooling-review.4NA5Aq/previous.snapshot`; the export receipt is
`target/shared-tooling-review.4NA5Aq/refresh.log`.

The review found a Linux-only CI configuration and nonportable consumer fixture
helpers. CI now selects Ubuntu 24.04 and macOS 15 on Apple Silicon and Intel, with
host-specific dependencies and GNU Make setup. The macOS jobs explicitly run
tooling with system Bash 3.2 before the common native gate. Consumer release/hook
tests use `shasum` when `sha256sum` is absent and Perl for portable file edits;
the release fixture uses Bash 3.2 subshell identity and positional arguments instead
of BASHPID and empty-array expansion. The upstream verifier includes its own
Bash 3.2 empty-array fix. The local host matrix declares required macOS support
while preserving the absence of native macOS qualification for this revision.

`make shared-tooling-check tooling-check release-check hooks-check shell-check`
passed, including snapshot rejection, retained failure artifacts, release rollback/
retry and hook index preservation. Actionlint passed for the three-host workflow.
The first four focused targets also passed under an isolated PATH with no
`sha256sum`, exercising the real `shasum` path. Logs are
`target/shared-tooling-review-checks.log`, `target/shared-tooling-review-workflow.log`
and `target/shared-tooling-portable-checks.A6zwiZ/checks.log`. Ordinary whitespace
checks passed. These are Linux tooling checks, not native macOS or IC evidence.
No broad CI/release gate or new Rust compilation ran for this refresh.

The public GitHub description review is retained at
`target/shared-tooling-description-review.json`. It still describes extraction
design and contributor instructions rather than the implemented Rust foundations;
[the adoption guide](../shared-tooling.md) contains concrete replacement wording.
Authenticated GitHub CLI access is unavailable, so no issue was filed or metadata
changed. Native macOS CI results and the description correction remain maintainer
actions. No local feedback tracker was created. Product version, earlier SDK
upgrade work, historical provenance and retained failure/recovery evidence remain
preserved; no commit, release transaction, package upload or live IC effect ran.

## Management SDK dependency upgrade

The exact `ic-management-canister-types` pin is now `0.11.0`, upgraded from `0.8.0`.
The official registry index identifies 0.11.0 as the newest non-yanked entry observed
on 2026-10-05. Its cached archive checksum matches that entry and the new lock entry;
inspected source, manifest, changelog and revision metadata also match the archive.
Only this package's version/checksum changed in Cargo.lock; Candid remains 0.10.37.
See [current dependency inspection](../management-types-upgrade.json); the original
0.8.0 source provenance remains unchanged historical evidence.

The six request methods and used snapshot shapes retain their wire fields. Every
original request/reply golden remains byte-for-byte unchanged and passes production
admission. The full SDK status fixture now includes maximum u128 incoming-call
cycles and a distinct allowed status viewer; both new settings remain bounded
skipped work, separate from the required status/controllers and fresh permissions.
No production codec, product v1 record or request/reply hash encoding changed.
The released 0.1.8 API uses SDK DTOs internally and does not expose the lifecycle
status enum; that public API was introduced after the released baseline. The
maintainer selected an undated 0.2.0 draft for this batch. This is a release choice,
not a claim that the SDK upgrade breaks the published API or a version transaction.

All 36 focused cases passed: 26 IC request/reply unit cases, six inventory comparison
cases and four public request/reply/comparison recovery journeys. Warning-denied
all-target/all-feature package Clippy and rustdoc, plus Rust 1.91.0 all-target/
all-feature package compilation passed. Logs are `target/management-types-upgrade-`
`codecs.log`, `comparison.log`, `public.log`, `clippy.log`, `docs.log` and
`msrv-check.log` with that same prefix. The initial MSRV attempt could not run
because the toolchain was absent; its log remains at
`target/management-types-upgrade-msrv.log`. The required toolchain was installed
before the successful retry; installation evidence is
`target/management-types-upgrade-msrv-install.log`. Source/archive comparison files
remain at `target/management-types-source.IDXDP5/`, and the exact registry response
at `target/management-types-upgrade-registry.jsonl`.

These checks qualify native codecs, pure comparison and retained local recovery.
No full CI/release gate, package upload, commit or IC effect ran. The product manifest
remains 0.1.8; the selected undated changelog draft is 0.2.0.

## Retained local ShellCheck lookup correction

The reported `shell-check` failure came from a missing PATH entry, not a script
diagnostic. ShellCheck 0.11.0 already exists at `/home/adam/.local/bin/shellcheck`.
The host's login profile includes that directory, but the inherited non-login
validation environment did not. `make shell-check` now selects the PATH command
first, then an executable `~/.local/bin/shellcheck` only if PATH lookup fails.
It reports the selected command and rejects a missing tool; an available tool's
failure stops validation without another lookup or lint bypass.

Plain Make and the actual shared runner's selected `shell-check` target passed
ShellCheck, Bash syntax and Perl syntax with PATH restricted to `/usr/bin:/bin`.
A fixture PATH command exited 53; its marker and failed Make status verified PATH
precedence and failure propagation. Logs are `target/shellcheck-user-local-validation.log`
and `target/shellcheck-lookup.HTluDY/`. Earlier validation with an explicitly extended
PATH remains at `target/shellcheck-path-validation.log`. Both retained failure logs
remain under `target/validation-failures/` with names
`20261005T125303Z-600509-2-shell-check.log` and
`20261005T125321Z-601273-2-shell-check.log`.

The development quick start and selected changelog describe current lookup behavior.
No dependency download or host-profile edit was needed. The consumer-owned Makefile
changed; vendored scripts and snapshot identity remain intact. No Rust compilation,
full CI/release gate or release transaction ran for this lookup correction.

## Retained Shared Tooling document restoration

The documentation sweep in `c16b909` changed all seven manifest-declared Markdown
files: it added local banners/navigation, removed trailing blank lines and added
consumer prose/tables. The snapshot verifier correctly rejected the first drifted
file, `docs/principles/README.md`. All seven documents now match the existing pinned
Shared Tooling revision `956236a3848c2cfae6ae05f5c77e9c37b01b3366` again. Restoration
used read-only Git objects, verified the complete staged snapshot before copying
and changed neither the manifest nor verifier. No sibling checkout was modified.

Useful local principle navigation, host scope and tooling-snapshot terminology
now live in the consumer-owned documentation index, development guide and adoption
guide. AGENTS.md explicitly excludes manifest paths from branding/documentation
sweeps. `make shared-tooling-check` and `make tooling-check` passed, including exact
bytes/modes, drift/missing/symlink rejection, runner and evidence-retention cases.
Local link and changelog checks passed. Ordinary `git diff --check` reports only
the five restored upstream EOF blank lines; that diagnostic is retained. Scoped
diff checks passed with those checksum-bound EOF lines permitted for the principle
guides, while all other changed files used the ordinary whitespace check.
Original files, pinned restoration bytes and passing logs remain under
`target/shared-tooling-repair.g3TJaJ/`. The reported failure log remains at
`target/validation-failures/20261005T124750Z-588243-0-shared-tooling-check.log`.
No Rust compilation, full CI/release gate or release transaction ran for this
documentation/snapshot correction.

## Focused 0.1.9 draft review

Review of the four selected implementation batches at repository HEAD
`6356c0ff00fd950b9e9f106c335ec7450b56f33c` found no blocking correctness issue in
their maintained local scope. Production owners, bounded wire admission, exact
request/raw-evidence hashing, baseline preservation and original-plan/durable-byte
checks agree with the machine contracts and changelog. Decoded state, candidate
cardinality and local checks remain separate from fresh authority, effect settlement,
transfer completeness and terminal/reference release. No production change was needed.

Fresh offline/locked checks passed: 26 IC request/reply unit cases, six inventory
comparison cases, five integrity-policy cases, seven guarded integrity cases and
five public request/reply/comparison/integrity recovery journeys. The public cases
retain exact evidence, pending exhausted attempts and unfinished restore references.
Logs are `target/0.1.9-review-codecs.log`, `target/0.1.9-review-comparison.log`,
`target/0.1.9-review-integrity-policy.log`, `target/0.1.9-review-integrity-ops.log`
and `target/0.1.9-review-public.log`. All 28 retained Canic source hashes and 273
exact consumer references still match the inspected working-tree bytes; evidence
is `target/0.1.9-review-provenance.log`. The existing Candid/management SDK source
hashes also match retained provenance. Successful tests cleaned only their owned
fixtures; earlier failed/build/recovery evidence remains retained.

This is focused draft review, not the full CI/release gate or IC backend qualification.
This earlier review used the undated `0.1.9` draft; the current draft selection is
recorded above. The package manifest remains `0.1.8`. The maintainer owns source commits and release execution under
[the release guide](../releasing.md). No version transaction, tag, push, upload or
live IC effect ran. Restore-safety/fence and lost-effect contracts, providers,
transport and runners remain unfinished as described below.

## Retained implementation batches

### Fresh local download integrity batch

`policy::download_integrity::validate` borrows the original full plan and download
journal, requiring exact intent, canonical selected coverage and Durable checksums.
Views retain journal-owned snapshot identity and metadata without a new schema,
serialized verified flag or hash encoder. The original pre-capture plan cannot
independently establish the later captured snapshot ID.

`DownloadJournalGuard::verify_durable_artifacts` explicitly admits the persisted
original plan and unchanged held journal before and after no-follow directory
checksum verification. Existing owners retain 1 MiB record IO, 1,024 selected
entries and 256-byte snapshot tokens. Ordinary reopen/resume still reads retained
progress without a fresh check. New typed failures write no evidence, transition
no journal, replenish no attempt and release no reference. Filesystem checks are
sequential observations rather than an atomic tree/set snapshot; integrations own
stable byte custody, authentic capture association, complete backend transfer and
terminal/release admission. No provider, runner, transport or live effect is added.

Eight Canic source sections and 47 exact consumer references were freshly inspected
under the retained dirty source HEAD. Explicit durable-byte verification and exact
coverage are adapted; the old manifest, duplicate artifact metadata, serializable
completion flag, receipt inference, CLI/prune/restore effects are not imported.
See [the contract](../contracts/download-integrity.json) and
[fresh source/consumer provenance](../download-integrity-source.json).

Five pure-policy tests, seven guarded filesystem tests and one public recovery
journey passed. Coverage includes changed original intent/selection, every incomplete
state, maximum target/token limits, missing or replaced plans/journals, changed
published bytes, absent/file/symlink substitutions and unusable custody. Public
recovery preserves exact journal bytes, exhausted pending reservations and unfinished
restore references after successful checks and checksum failure. Native evidence
qualifies local mechanisms, not IC effects or a complete independently usable product.

Warning-denied all-target/all-feature package Clippy and rustdoc, Rust 1.91.0
all-target/all-feature package compilation, formatting/diff, changelog and source/
reference/contract/link checks passed. Logs are `target/download-integrity-policy.log`,
`target/download-integrity-ops.log`, `target/download-integrity-public.log`,
`target/download-integrity-clippy.log`, `target/download-integrity-docs.log` and
`target/download-integrity-msrv.log`. The initial unit compilation rejected a test
cleanup helper moving a borrowed layout; its log is retained at
`target/download-integrity-policy-initial-failure.log`. Explicit ordered fixture
drops fixed it. Successful tests cleaned only owned runtime fixtures; all earlier
failure/recovery artifacts remain retained. No full suite, package/release gate,
version transaction or publication ran.

### Lifecycle reply batch

`model::ic_lifecycle_reply` admits the existing status/stop/start/load methods.
Mutation acknowledgements require the canonical six-byte empty Candid tuple.
Status requires exactly one status/settings/controllers projection, reusing the
upstream state enum and existing `ControllerSet` canonical owner. Explicit empty
controllers remain empty; missing fields and duplicates reject. Bounded visitors
retain at most 10 principals without allocating from untrusted declared lengths.
Raw input is capped at 1 MiB; status work at 2 MiB, skipped work at 64 KiB and
type-table entries at 64. Extra arguments, trailing data and malformed shapes reject.

Unprojected fields are skipped, not retained or qualified. This may admit records
lacking fields outside the projection and is not full SDK status validation.
Read-only kinds retain exact raw checksums plus a separate v1 request/reply digest;
ignored fields and original controller ordering stay in evidence identity. There
is no serialized fresh result, provider, dispatch, receipt or journal transition.
Transport association/authentication, current timing/permissions, continuous fences,
stopped/drained evidence and safe same-release load settlement remain integration-owned.
Neither Stopped/controller values nor acknowledgements settle a pending attempt or
authorize restart, release, retry or new spending. See
[the contract](../contracts/ic-lifecycle-reply.json) and
[fresh provenance](../ic-lifecycle-reply-source.json).

Eight Canic files/sections and 49 exact consumer references were inspected read-only
under the same dirty source HEAD retained below. Typed lifecycle states, required
status/controller projection and pending-observation rejection are adapted. Agent/
CLI calls, optional status defaults, Root/Fleet routing, command-success or
status-equality completion receipts and automatic reconciliation are not copied.
Existing SDK 0.8.0 and Candid 0.10.37 source/registry identities are recorded; the
current primary management interface was also checked read-only.

Ten focused unit cases and one public recovery case passed. Every registered
hand-assembled Candid/hash fixture enters production admission; a separate complete
SDK status exercises bounded skipped fields. Native cases cover required fields,
unknown states, empty/maximum/duplicate controllers, raw/work/type/count limits,
truncations/tuple/trailing rejection and exact target/method/raw load-ID hashes.
The public journey durably retains local load/status wire fixtures and reopens
exhausted pending original reservations. Decoding, wrong-shape/malformed rejection
and changed load association preserve exact journal/evidence bytes and allowances.
It issues no receipts and performs no remote observations or IC effects.

Warning-denied all-target/all-feature Clippy and rustdoc, Rust 1.91.0 all-target/
all-feature package compilation, formatting/diff, changelog admission and fresh
source/reference/dependency/golden/JSON/link checks passed. Logs are
`target/ic-lifecycle-reply-unit.log`, `target/ic-lifecycle-reply-integration.log`,
`target/ic-lifecycle-reply-clippy.log`, `target/ic-lifecycle-reply-docs.log` and
`target/ic-lifecycle-reply-msrv.log`. An initial test enum lacked the Serde derive
needed for its rename attribute; its compiler log remains at
`target/ic-lifecycle-reply-unit-initial-failure.log`. The initial hex-helper lint
failure remains at `target/ic-lifecycle-reply-clippy-initial-failure.log`.
Corrected cases passed. Successful runtime fixtures cleaned only their owned
temporaries; prior failed/build/recovery evidence remains retained. No full suite,
package verification, full CI/release gate or publication ran.

### Snapshot inventory comparison batch

`policy::snapshot_inventory_delta::compare` borrows the exact existing capture and
two inventory reply owners. It requires the closed capture/list methods and exact
canonical target, then linearly compares canonical raw IDs. Every baseline ID must
remain with unchanged timestamp/size; loss and metadata drift reject with typed
errors. The read-only view exposes zero/single/multiple canonical candidates and
borrowed original request/reply evidence. Existing 1,024-entry/256-ID-byte bounds
apply, with no duplicate wire/hash owner or new persisted schema.

Cardinality does not attribute or settle a capture. Another controller could create
one candidate; no candidate does not prove failure. Integrations own original
pre-effect baseline custody, authenticated association, actual observation chronology
and exclusive attribution. No provider, receipt, retry, restart, cleanup, spending
reset or upload reconciliation is introduced. See
[the machine contract](../contracts/snapshot-inventory-delta.json) and
[fresh provenance](../snapshot-inventory-delta-source.json).

Six fresh Canic source sections and 75 exact consumer references were inspected
read-only under the same dirty HEAD recorded below. Baseline preservation and set
difference are adapted; singleton-to-completed-receipt inference and restart/journal
cleanup are not copied. Related upload recovery stays unchanged in Canic and outside
this capture projection. Earlier source provenance remains exact and retained.
The maintained request schema's descriptive Candid selection now matches the
actual `0.10` declaration/0.10.37 lock; historical .35 evidence is unchanged.

Six focused policy cases and one public persistence/recovery case passed. They cover
all candidate cardinalities, exact byte-prefix/raw-order identity, method/target
mismatches, baseline loss at every merge position, timestamp/size drift and maximum
combined bounds. Public recovery retains original baseline/observation bytes and
reopens original exhausted pending mutation/observation reservations. Singleton,
multiple and zero projections, baseline rejection and a denied fresh mutation
preserve exact journal/evidence bytes and original allowances; no receipt is issued.

Warning-denied all-target/all-feature Clippy and rustdoc, Rust 1.91.0 all-target/
all-feature package compilation, formatting/diff checks and fresh source/reference/
JSON/document-link checks passed. Logs are `target/snapshot-inventory-delta-unit.log`,
`target/snapshot-inventory-delta-integration.log`,
`target/snapshot-inventory-delta-clippy.log`, `target/snapshot-inventory-delta-msrv.log`
and `target/snapshot-inventory-delta-docs.log`. Initial test-style lint failures and
an untyped empty-slice assertion compilation failure remain in
`target/snapshot-inventory-delta-clippy-initial-failure.log` and
`target/snapshot-inventory-delta-clippy-empty-assertion-failure.log`; corrected cases
passed. Successful runtime fixtures cleaned only their owned temporary directories.
No full suite/package/full CI/release gate or actual IC effect ran. Transport,
metadata/data transfers, safe lost-effect settlement and runners remain unimplemented.

### Snapshot capture/inventory reply batch

`model::ic_snapshot_reply` decodes exactly one capture descriptor or inventory
vector tied to the caller's borrowed immutable request. Required upstream fields
retain exact 1–256 raw ID bytes and full nat64 timestamp/size. Raw input is bounded
to 1 MiB; decoder work to 2 MiB, skipped work to zero and type-table entries to 16.
Bounded sequence visitors retain at most 1,024 descriptors without allocating from
untrusted lengths. Unknown/missing/wrong fields, malformed data, extra arguments,
trailing bytes and duplicate IDs reject with typed errors and no raw diagnostics.

Inventory views sort exact raw IDs; payload hashes preserve raw ordering and a
separate v1 digest binds the existing request digest plus raw-reply checksum.
The reply contains no target/network/caller/challenge, so the association is
declared, not authenticated. Descriptor metadata is neither transfer completeness
nor a unique lost-capture receipt. No Serde record, reservation, settlement or
dispatch mechanism is introduced; pending attempts and spent limits remain intact.
See [the machine contract](../contracts/ic-snapshot-reply.json) and
[fresh provenance](../ic-snapshot-reply-source.json).

Six Canic source files and 102 consumer references were inspected read-only.
The source still has dirty working-tree material under HEAD
`3978e02d28fea9022c7a6e84a7ec6d6e4a0d4af3`; exact hashes identify inspected bytes.
Raw ID admission, duplicate checks and bounded Candid inventory parsing are adapted.
ICP token readers, optional metadata, agent calls, inventory-delta settlement and
completed execution receipts are not copied. Existing SDK 0.8.0 and actual selected
Candid 0.10.37 source are separately recorded; no dependency or lock update ran.

Native evidence includes every independently assembled DIDL/hash fixture decoded
by production and official DTOs, maximum combined entry/ID bounds, full nat64
values, wire/target hash sensitivity, type/length/truncation/tuple/field rejection
and duplicate identity. A public journey reopens original intent plus exhausted
pending mutation and observation reservations, decodes retained exact bytes,
rejects wrong association and preserves exact journal/evidence bytes and budgets.
It makes no remote observations or receipts. The existing request codec regressions
are also rerun against the current lockfile selection.

Targeted unit/integration, warning-denied Clippy/rustdoc, Rust 1.91.0 library
compilation and formatting/diff checks passed. Logs are
`target/ic-snapshot-reply-unit.log`, `target/ic-snapshot-reply-integration.log`,
`target/ic-snapshot-reply-clippy.log`, `target/ic-snapshot-reply-msrv.log` and
`target/ic-snapshot-reply-docs.log`. An initial integration assertion expected
MutationPending while a separate observation was pending; the corrected typed
ObservationPending assertion passed. The initial integration/lint failure logs
remain under `target/ic-snapshot-reply-*-initial-failure.log`, and its interrupted
fixture remains at `/tmp/ic-backup-public-reply-410-1791184054778669690/`.
No full native test suite, package verification, full CI or live IC check ran.

The following tooling and consistency sections retain prior qualification evidence;
they do not claim fresh broad validation for this batch. Live providers, complete
transfer/remaining response codecs, fencing and safe restore settlement still
precede runners. Independently usable backup/restore remains unimplemented.

### Retained nested release-check correction

The outer validation runner exports `VALIDATION_REPOSITORY_ROOT` for its children.
The dependency-bootstrap fixture inherited the real repository root and dispatched
its simulated Make target there, where the fixture's exact-path substitute rejected
it before Cargo ran. Direct release checks had passed without this parent context.

The consumer-owned release adapter now binds repository, failure-log and GitHub
summary paths to its own fixture. Added success/failure cases seed a different
parent runner context, verify the actual Make gate and require parent summary/log
state to remain unchanged. The tooling adapter similarly owns the upstream runner
test's GitHub summary, keeping simulated failures out of the real parent summary.
Vendored tools, snapshot identity, production release requirements and Rust are
unchanged.

Direct `make release-check` passed. The actual shared runner then passed the
selected `release-check`, `tooling-check` and `shell-check` targets with an explicit
parent failure-log directory and GitHub summary. Only the three real passing
targets appear in that summary; no fixture failure logs escaped into the parent.
Evidence remains at `target/nested-release-check.FPFCKV/`. This was targeted tooling
validation, not full CI or release preparation. Changelog admission and diff checks
also passed.

The original `target/release-tests.QebrYj/` and retained validation logs remain.
A pre-fix reproduction through the shared runner failed identically; its output
is `target/release-check-reproduction.log`, with logs under
`target/release-check-reproduction/` and fixture `target/release-tests.y6HiMu/`.
Earlier failed evidence remains retained too. Successful test fixtures cleaned
only their own temporaries.

### Retained initial shared engineering and tooling batch

The initial [shared adoption](../shared-tooling.md) recorded upstream committed
revision `956236a3848c2cfae6ae05f5c77e9c37b01b3366` and separately hashed dirty
upstream rules. Those were the reviewed working-tree rules retained locally at
that time. The current committed snapshot is described above. AGENTS.md retains
its backup-specific overlay within the mandatory shared baseline. Shared and
local rules agree on GitHub-only feedback tracking and explicit broad-validation
authority. CI does not inherit a mutable sibling checkout.

The local snapshot manifest binds exact committed principles, consumption/host
guides, checksum/snapshot verifiers, validation runner and runner regressions.
Make verifies it before fetching dependencies, then preserves the prior sequential
fail-fast gate with target-labelled failures, timing/result and GitHub summaries,
and full/highlighted failure logs under `target/validation-failures/`. The consumer
adapter checks exact bytes/modes, missing files, symlinks, duplicate/escaping manifest
paths and artifact retention. Vendored files stay unchanged.

Release preparation now labels/dates one top-level unnumbered or selected numbered
draft. It rejects ambiguous/empty/misplaced drafts, preserves imported undated
history and keeps the exact receipt/tag requirements. Registry publication still
delegates directly to Cargo. No package version or backup contract changed.

Targeted checks passed: `make tooling-check`, `make release-check` and
`make shell-check`. The real Make dependency gate was exercised with substituted
Cargo and selected targets, not a full native build or network fetch. Read-only
release planning, snapshot identity/mode checks and diff checks also passed.
An initial new test used a fixed date instead of the generated receipt date;
the corrected test passed. Its failed fixture and trace remain at
`target/release-tests.7mc795/`. Successful isolated test temporaries were removed
by their owners; existing build/package/recovery evidence remains retained.

The consistency qualification below is retained evidence from the prior extraction
batch, not a fresh Rust run for this tooling change. The maintainer's current Candid
dependency declaration is preserved. No broad CI/release gate or native compilation
was requested; product and host qualification boundaries remain as described below.

### Current consistency batch

`model::consistency::ConsistencyRequirementRecord` retains strict v1 `version`,
canonical full original `plan_intent` and explicit `per_canister` or
`application_coordinated` requested guarantee. Missing/unknown fields, other versions
and obsolete flag names reject. A domain-separated binary hash binds original plan
intent and guarantee, not current consistency. Full original intent includes context,
inventory, selection, graph, requests and original allowances.

`create_consistency_requirement` requires the original plan already retained under
layout exclusion, then durably creates fixed `consistency-requirement.json` under
journal exclusion without replacement. Reads require exact original expected
requirement digest and original retained plan. Raw input/canonical output have a
1 KiB bound. Lost local creation replies reconcile through exact retained reads;
missing/corrupt/changed evidence never silently downgrades or recreates original
requirements. No fence or allowance is retained or created by this declaration.

Ephemeral `ConsistencyRequest` binds original requirement/operation, caller-owned
challenge, BeforeCapture/AfterCapture boundary and a 0–1,024 descriptive remote-call
ceiling. Per-canister requests require no expected fence; coordinated requests require
an exact `ApplicationFenceBinding` with retained identity AND original membership
revision recovered by the integration's durable obligation owner. Both fields bind
the request hash. Constructor equality proves neither durable fence retention nor
current Active custody; capture wire/effect admission remains separate.

Model-admitted current observations retain exact request, actually observed canonical
context/full inventory, a nonempty canonical sorted unique 1–1,024 set of inventory-backed
targets with actual Running/Stopping/Stopped states and required opaque stopped/drained
evidence, actual revision, explicit evidence lane, opaque observation evidence and actual
call reporting. Coordinated evidence includes actual Active/Inactive fence state,
identity/revision and required whole-unit write/membership/timer/external-work fencing
and drained-work evidence digests. New request/parameter/result/target/fence/view types
have no Serde/default or persisted authority admission. Only the requirement is persisted.

`ports::consistency::ConsistencyProvider` has a fallible typed signature without an
installed/default implementation. It observes existing obligations; no acquisition/
release API is installed. Pure `policy::consistency::validate` requires exact current
request/context/full inventory/exact selected set/call reporting, every target Stopped
and the original guarantee. Coordinated evidence must have the exact retained Active
fence; both actual and fence revisions equal the original retained revision. This
prevents a changed observation and fence revision from silently rebinding an obligation.
No accepted/expiry/Proven flag or parent/component inference supplies a guarantee.

Integrations qualify actual authenticated state, stop/drain, fresh unique challenge
timing, continuously retained whole-selection fence custody, opaque evidence meaning
and prior per-call accounting. Matching before/after values or sequential stops alone
prove no continuity/distributed checkpoint. Unknown custody fails; known inactive
fences deny. Unavailable/Unsupported reject before effects; Indeterminate retains
spent allowance/evidence and obligations, then stops without retry/reset/release.
Timeout, process death, failure and dropping model values do not release obligations.
The descriptive ceiling grants no paid call/reservation/preflight allowance. Matching
views grant no signing, dispatch, restart, release, capture completion or restore/payment
settlement. Terminal replay invokes no provider. See
[the requirement schema](../contracts/consistency-requirement.schema.json),
[typed current port](../contracts/consistency-port.json) and
[maintained boundary](../extraction-boundary.md).

Fresh native cases cover strict records and independent requirement/request binary
hashes, original plan/allowance sensitivity, challenge/boundary/fence/revision binding,
inactive/non-stopped denial, every member of a multi-target selection, full inventory
drift including an unselected parent, exact selected/lane mismatches, canonical target
aliases/duplicates and 1,024-target admission. Persistence cases cover no downgrade/
replacement, original-plan presence, lock contention, raw bounds, malformed/rebound
records, symlinks and replaced layouts. A public local provider fixture reopens original
requirement and spent pending journals, rejects stale/wrong-fence results and preserves
exact original authority, journal bytes, requirement and integration-owned obligation
fixture bytes across all typed failures/drop. These prove local contracts and recovery,
not actual management/application behavior or continuous fence custody.

Targeted checks passed: 153 unit tests and twelve public integration tests (165 total),
warning-denied Clippy/rustdoc, Rust 1.91.0 all-target/all-feature compilation, formatting,
diff checks and standalone Cargo package verification. The changed unselected-parent
regression also passed in the 13-case focused consistency suite. The archive contains
110 exact current Rust source/test files, the exact IC wire JSON fixture, current README
and exact regular MIT license; verified current archive/source remain under
`target/package/`. Previous snapshot-read archive/source remain retained under
`target/consistency-package-evidence.k2rfq3zw/`, along with earlier snapshot-read logs.
Current logs are `target/consistency-focused-tests.log`, `target/consistency-tests.log`,
`target/consistency-clippy.log`, `target/consistency-msrv.log`,
`target/consistency-docs.log` and `target/consistency-package.log`.
Schema examples/negative cases, independent binary digests, exact fresh source references,
prior Rust/provenance preservation and archive contents passed consistency checks.
No broad CI/release gate ran.

[Fresh consistency provenance](../consistency-source.json) records six inspected Canic
files/sections and 180 exact consumer references. Canic HEAD was
`3978e02d28fea9022c7a6e84a7ec6d6e4a0d4af3` with dirty working-tree source. Exact
hashes identify inspected bytes, not application qualification. Quiescence choices,
request/receipt binding and negative cases are adapted; CrashConsistent/RootCoordinated
names, accepted/Proven flags, expiry and parent-derived application units are not copied.
All source consumers remain unchanged. Earlier tracked provenance and source/lock/license
bytes are preserved, including the prior [snapshot-read provenance](../snapshot-read-source.json).

## Retained boundaries and remaining scope

Snapshot-read contracts retain independent exact list payload/challenge binding, actual
context/target/snapshot visibility and pure controller/public/exact-viewer paths. The
1,024 descriptive call ceiling grants no spending; viewers/controllers are bounded to
10 canonical unique principals. Unknown controllers cannot establish controller access;
independent public/viewer evidence needs no controller projection. Permission evidence
cannot settle a lost observation or replenish original authority. Native snapshot-read
qualification is retained; no metadata/data codec or live provider exists.

Earlier artifact/staging/durable publication, bounded JSON, layout/journal exclusion,
restore reference retention, owned command custody, local download/attempt journals,
canonical inventory/selection, explicit graphs, immutable operation plans and pure
retained-journal progress remain implemented. Their exact bounds/contracts are in
[the maintained boundary](../extraction-boundary.md) and AGENTS.md. Pending paid replies
stay pending, missing journals never mean zero consumption, and allowances never refund/
replenish. Applied prerequisites cannot prove cross-journal actual dispatch chronology
or full terminal/reference-release admission. No prune/reference release exists.

The six-method exact IC host-ingress codec and separate membership/control ports remain.
They bind current evidence to original intent/challenge with pure checks but provide no
live authentication, controller custody, continuity/fence or dispatch permission. Load
origin control, same-ID/same-release safety and external-obligation disposition remain
separate requirements. Every earlier [source provenance](../extraction-boundary.md)
remains retained. This batch adds no dependencies, lock changes, Canic imports, sibling
patches, unsafe code or shared target. Rust remains 2024, development 1.99.0 and MSRV
1.91.0. Maintainers own releases; see [development](../development.md) and
[releasing](../releasing.md).

Full B1/B2 and independently usable backup/restore remain unestablished. Application
fence acquisition/release/uncertain-effect recovery and actual same-release restore
safety still precede runners. Real membership/control/read/consistency/restore-safety
providers, transfer/
response codecs, selected backend snapshot/lifecycle qualification, bounded authenticated
calls and lost create/upload/load reconciliation remain necessary. Prior per-call
observation spending, actual cross-journal chronology, complete execution/restore journals/
manifests, terminal reference release, prune, transport and CLI remain proposed. Follow
[the design](../extraction-design.md) for sequencing. Canic adoption and live effects
need separate instructions; the no-commit rule remains.
