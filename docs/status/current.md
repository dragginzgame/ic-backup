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


# Current handoff — 2026-10-10

## Current 0.12.1 upload continuation

Continue the compatible 0.12.1 draft from released 0.12.0
`f1bba34b667a724274a65ac9653b4de88f30414f`; package/receipt remain 0.12.0.
`workflow::ic_snapshot_upload::upload_snapshot` now coordinates one exact original
metadata allocation or data write: canonical source/context binding before spending,
complete original reservation/prerequisites, mandatory fresh integration admission
and one provider invocation under the selected journal lock. Re-admit stage/ancestors
and bounded acknowledgement; success remains pending. Explicit independent receipt
qualification, durable reply retention, destination/source custody and application
safety remain integration-owned. No hidden observation, retry/refund, automatic
allocation receipt, schema or terminal/fence/reference release is added.

The actual isolated Testkit caller binds metadata allocation and all exact learned
data requests within original workflow ceilings, retaining replies before explicit
receipts. Complete destination byte checks and lost/malformed metadata/data safe
stops across reopen pass. This is a public one-call owner plus qualified stage
assembly fixture; generic upload/restore orchestration and application-qualified
async Agent/product terminal custody remain incomplete under #29.

Preserve the external Testkit 0.28.1 lock update and explicitly prepare its exact
CLI; offline server admission passes. Host 0.10.1/Metrics 0.3.6/Agent 0.49.2 and
Shared 0.2.10 (95 files) remain selected. All 66 workflow/stage unit cases, seven
existing planned simulator cases and five corrected upload journeys pass on that
graph, as do strict core tests Clippy/rustdoc, both independent Rust 1.88 consumers
and a new public upload API caller. Formatting/snapshot/pins/links/whitespace pass.
The [upload review](../reports/audits/2026/10/10/continuation-0121/02/report.md)
retains final source/log evidence separately from preceding Testkit 0.28.0 proof.
The fixture reference owner correctly rejected an initial directory-as-journal
path before upload; failed logs/fixtures remain retained. No Rust symbol was removed.

Released-source CI has Linux/Apple Silicon success; Intel macOS remains in progress
in [the release run](https://github.com/dragginzgame/ic-backup/actions/runs/37969178975).
Keep native acceptance open. The dirty candidate has no hosted result. No broad
local gate, root Git write, release, production IC effect, sibling edit or recovery
cleanup ran.

## Prior 0.12.1 metadata-stage and formatting continuation

Released 0.12.0 is `f1bba34b667a724274a65ac9653b4de88f30414f`.
Select compatible 0.12.1 for the additive singleton metadata-stage coordinator and
human formatting diagnostics. Package versions and release receipt remain 0.12.0.
`workflow::ic_snapshot_metadata::read_snapshot_metadata` delegates one exact new
read to original spending/fresh admission, requires integration-qualified durable
original bytes and attribution before its explicit Applied receipt, then checkpoints
exact metadata request/raw-reply evidence. Returned response/predecessor feeds the
existing download-plan binding; data-stage/writer preparation stays explicit.
Failures retain replies, spending and Applied/occupied checkpoint history. Pending
or Applied reads never reissue. No new schema, default async Agent bridge or terminal/
fence/reference release follows. The actual Testkit caller uses this owner, with
lost/malformed metadata safe stops across reopen and no successor data stage.

Canonically adopt committed Shared Tooling 0.2.10 at
`43a0dc46cdc3c77e70a68e192561642ed50a3e0f` (95 files), selecting the formatting
reporter explicitly. Success is one line; failures retain stdout/stderr/status and
CI collection. Hook/release/Testkit fixtures include the helper; preserve prepared
tools, Make admission/default help and the post-run release-resume check.

All 58 workflow/stage unit cases, seven real planned simulator journeys, strict
core tests Clippy/rustdoc, both independent Rust 1.88 consumers and a new API caller
pass. Actual hooks/output/preservation, release adapters, Testkit routing, collector
byte checks, formatting/snapshot/pins/links, ShellCheck and workflow lint pass.
The [review](../reports/audits/2026/10/10/continuation-0121/01/report.md) retains
source/log evidence and initial test-only Clippy failures. Preserve incoming Host
0.10.1/Testkit 0.28.0/Metrics 0.3.6/Agent 0.49.2 and the already dirty cc/syn/smallvec
lock changes; no dependency reselect or Rust symbol removal occurred.

The authorized released-source CI retry passes Linux after the original installer
HTTP 500; Apple Silicon also passes. Intel macOS has been retried and remains pending
in [the release run](https://github.com/dragginzgame/ic-backup/actions/runs/37969178975).
This qualifies released 0.12.0 separately from the uncommitted draft. Keep native
acceptance open; next product work remains complete upload/restore orchestration,
application-qualified async Agent integration and product terminal/custody proof.
No broad local gate, root Git write, release, production IC effect, sibling edit or
recovery cleanup ran.

## Prior 0.12.0 direct Host upgrade

Released base remains 0.11.5 at `6b5714aed1b3f69636b037fafebdee03b627a418`.
Relabel the complete pending batch to 0.12.0: the requested direct Host 0.10
upgrade changes the public `PersistenceError::Publication` Rust identity. Consumers
sharing that value must align their direct Host dependency to 0.10. Variant shapes,
v1 records, original limits/spending and private publication barriers are unchanged.
Carry the preceding Shared 0.2.8 tooling work into this minor draft; package and
release receipt stay at 0.11.5.

Keep the incoming 0.10 requirements and select Host 0.10.1 for all four crates.
Use canonical `write_with` for pathname publication; supported Unix callers retain
held-parent `write_at_with`. Preserve serialize-once preflight, 0700/0600 permissions
and synchronized pre-publication/durable-completion barriers. During qualification
an external update selects published Testkit 0.28.0 and Metrics 0.3.5. Keep earlier
0.27.2 results separately; qualify the frozen final selection. The exact 0.28.0 CLI
is prepared through `make install-testkit-server`, with offline server admission
passing. Preserve previous installations and server/recovery evidence. No sibling
patch or production process dependency is introduced.

All 221 focused artifact/persistence/workflow tests and eight actual core/Agent
simulator cases pass. Both-library Clippy, warning-denied rustdoc, both independent
Rust 1.88 consumers and a direct Host 0.10 public-error/pathname-API interoperability
consumer pass. Normal consumers retain disabled optional archive/Wasm features and
exclude Testkit/process/tools. Formatter, snapshot/pin and local-link checks pass.
The [upgrade review](../reports/audits/2026/10/09/host-010/01/report.md) binds exact
published sources, selected graph and retained evidence; the earlier 0.11.6 review
remains historical qualification of its original graph. No Rust function, method
or type was removed.

No full gate, root Git write, version transaction, release, production IC effect,
sibling edit or recovery cleanup occurred. Native macOS acceptance and this dirty
candidate's remote CI remain unqualified. Released 0.11.5 Linux passes; both macOS
jobs remain queued at inspection. Next accepted product work remains #29
complete orchestration and independently qualified application/Agent/terminal custody.

## Prior 0.11.6 tooling and selected Host continuation

Released 0.11.5 is `6b5714aed1b3f69636b037fafebdee03b627a418`.
Select compatible 0.11.6: canonically adopt committed Shared 0.2.8 at
`b2646cde9abbc8861857a4379c683a0c19eba43e`, including the declared execution
companion in the 94-file snapshot. Reject unsafe Make modes before recipes; keep
probe selection local and accept recursive Make arguments. Preserve default help,
prepared formatter admission and the post-run release-resume receipt check. Local
hook, release and Testkit fixtures retain complete companions. Dry-run release
preview now uses `make release-plan`; unsafe modes refuse with no effects.
The sibling's dirty 0.3.0 proposal stays outside this adoption.

Preserve the incoming Host 0.9.7 lock selection, Testkit 0.27.2, Metrics 0.3.4 and
Agent 0.49.2. Published Artifact/FS/Process source is unchanged; Host Tools' hex
cleanup adds no API for Backup to adopt. No safe extra local code retirement was
found. Exact Testkit CLI/server admission, 80 focused unit and eight actual core/
Agent simulator cases, both-library Clippy and independent Rust 1.88 consumers
pass. Hooks, Make admission, release adapters, formatter/snapshot/pin and shell
checks pass; the [review](../reports/audits/2026/10/09/continuation-0116/01/report.md)
retains exact source/graph evidence. Package/receipt remain 0.11.5 and incoming
manifest/lock bytes are unchanged.

The exact released-source CI run has Linux in progress and both macOS jobs queued
at inspection. Keep native acceptance open in #32/#33/#34; this dirty candidate has
no remote result. Next accepted product work remains #29 metadata/capture/upload/
restore orchestration with independently qualified application/Agent admission and
terminal/custody proof. This batch performs no full gate, root Git write, release,
production IC effect, sibling edit or recovery cleanup. No Rust symbol was removed.

## Released 0.11.5 complete data-transfer continuation

Released 0.11.4 is `a585ae1064b1db49cd33f5c02dfe76637bd9f1b9`.
The compatible 0.11.5 draft adds `workflow::ic_snapshot_download::download_snapshot`:
join the exact nonempty metadata-derived data stage and existing private writer,
refuse prior consumption/coverage, reuse single-read durable admission, then require
explicit independently qualified Applied receipts. Append before recording each
receipt and dependent dispatch; finish through existing fresh checksum/durable
publication. Failed qualification/persistence, lost/malformed replies and changed
custody retain spending, partial bytes and bounded returned responses. No partial
read resume, default Agent provider or terminal/reference/fence release is introduced.

The actual planned download caller uses this owner and retains complete publication
and safe-stop cases. Finish #34's redundant formatting declarations; preserve the
shared include and specialized post-run release-resume check. Shared 0.2.6's exact
93-file snapshot is unchanged. Preserve incoming Host 0.9.4; a subsequent external
lock update selects Metrics 0.3.3. Prepare only its locked cache, retain earlier
selection/evidence and qualify the final graph with Testkit 0.27.1/Agent 0.49.2.
Package/receipt remain 0.11.4. The
[review](../reports/audits/2026/10/09/continuation-0115/01/report.md) retains exact proof.
The maintainer committed the implementation as
`d9517a42d685b240933affd58b211364823d5122` during final review; final evidence
updates follow separately. The undated candidate is not a release transaction.

Focused streaming/stage, metrics, both-library Clippy, independent Rust 1.88/public
API, five core and three Agent actual simulator cases pass. Eight Agent HTTP cases
pass with local loopback enabled. Released 0.11.4 Linux passes; both macOS jobs are
queued, so #32/#33 and #34's consumer native acceptance remain open. #25 still owns
installed/downstream Agent trust/identity/custody qualification. #29 retains full
metadata/capture/upload/restore orchestration and application/terminal proof; the
new synchronous core loop does not bypass the async Agent signed-envelope retention
boundary. Final review updates remain local; the agent performed no root Git write,
release, production IC effect,
sibling edit, recovery cleanup or Rust-symbol removal occurred.

## Subsequent Host/Testkit review

The incoming lock now selects published Host 0.9.5/Testkit 0.27.2. Registry and
archive checks match; all 148 published implementation/test/example files are
unchanged from the previously qualified versions. Locked metadata and 27 focused
artifact/JSON-publication cases pass. The exact 0.27.2 CLI is absent, so offline
server admission correctly refuses it. Explicitly prepare with
`make install-testkit-server`, then repeat actual core/Agent simulator qualification.
Earlier simulator/MSRV proof retains its original graph and does not qualify this
selection. Keep existing installations/evidence. Their useful tooling changes are
Shared 0.2.7 Make execution/hook safeguards; deliberate Backup snapshot adoption
remains a separate batch. The [review](../reports/audits/2026/10/09/continuation-0115/01/report.md#subsequent-published-hosttestkit-inspection)
records evidence. No source repair, dependency reselection, tool setup, full gate,
Git write or release was performed in this inspection.

## Testkit 0.27.2 setup completed

The subsequent setup request prepares the exact registry 0.27.2 CLI through
`make install-testkit-server`; receipt/byte checks and existing PocketIC 16.1.0
server admission pass. `make testkit-server-check` now succeeds. Retain all old
selections and failure evidence. The current maintainer commit
`d6be377300b43eff76b3e15c7ac034fedbc5bca6` also selects incoming Metrics 0.3.4.
Five planned core and three Agent actual simulator cases pass on Host 0.9.5 /
Testkit 0.27.2 / Metrics 0.3.4 / Agent 0.49.2. Manifest/lock/snapshot bytes remain
unchanged. [Setup proof](../reports/audits/2026/10/09/continuation-0115/01/testkit-preparation.json)
retains exact executable identity and logs. This resolves the local missing CLI;
full CI, native macOS and fresh standalone MSRV qualification remain separate.
No source repair, dependency reselection, Git write or release occurred.

## Current 0.11.4 capture-step continuation

Released 0.11.3 is `2939a41805ce5c2fa3913162b4028bc970d3d85f`.
The compatible 0.11.4 draft adds `workflow::ic_snapshot_capture::capture_snapshot`:
canonical payload binding, existing durable original reservation, mandatory fresh
integration admission and one provider invocation under selected journal exclusion.
Success remains pending until an independently qualified integration records a
receipt. Lost/malformed replies retain original spending/references and block
recapture or successor stages. The real isolated Testkit journey connects this
capture step to the existing reads and durable artifact publication.

Canonically adopt committed Shared 0.2.6
`ce13a5314916891fd239d9b199b4a91b04775054`, adding its formatting include to the
93-file snapshot. Local formatter recipes retire; consumer fixtures export the
new companion. Keep specialized release routing because `release-resume` checks
the exact receipt/tag/saved validation after the shared runner. Manifest/lock and
package/receipt remain exact 0.11.3, including Host 0.9.3, Testkit 0.27.1 and
Metrics 0.3.2. Both independent Rust 1.88 consumers and focused local checks pass;
the [review](../reports/audits/2026/10/09/continuation-0114/01/report.md) retains proof.

Released 0.11.3 Linux CI passes; both macOS jobs are queued. This candidate is
uncommitted and has no remote result. Keep #29 open for full provider-driven
transfer/upload/restore coordination, actual application/Agent admission and
terminal/custody proof; #32 owns native acceptance. No full gate, root Git write,
release, production IC effect, sibling edit or Rust-symbol removal occurred.

## Current 0.11.3 provider-driven read continuation

Released 0.11.2 is `2f441df35bffb0e9a8d4ea93e826045d0e71424d`.
Keep the compatible 0.11.3 draft and incoming CI fix for #33. Add one callable
snapshot metadata/data read step under exact retained stage/journal evidence:
durable original reservation, mandatory fresh admission, one provider call and
bounded passive association. Retain spending and returned rejection evidence;
success stays pending until an independently qualified integration records a receipt.
The real download fixture now uses this step for metadata and every planned data call.

Canonically refresh the same 92-file snapshot to committed Shared 0.2.5
`04e07b4bf54e7aeb03eb7804a845cee27b7305df`; only literal hook-path handling,
its installer and rule change. The maintainer changed the lock during qualification
from Host 0.9.2/Testkit 0.27.0 to Host 0.9.3/Testkit 0.27.1; preserve that final
selection and Metrics 0.3.2, with earlier graph evidence separate. Explicitly
prepare the new locked cache and Testkit CLI before final offline checks.
Package/receipt stay 0.11.2. Focused evidence and
limitations are retained in the
[review](../reports/audits/2026/10/09/continuation-0113/01/report.md).
Released Linux CI passes; both macOS jobs remain queued. No full gate, release,
Git write, hook activation, sibling edit or production IC effect ran. Keep #29
open for full capture/transfer/upload/restore coordination, independently qualified
application admission and terminal/custody proof; #32 still owns native acceptance.
No Rust function, method or type was removed.

## Pending 0.11.3 CI source qualification

Released 0.11.2 is at `2f441df35bffb0e9a8d4ea93e826045d0e71424d`.
The authorized local fix for [#33](https://github.com/dragginzgame/ic-backup/issues/33)
restricts automatic push CI to main, retaining pull requests and existing manual
exact-ref dispatch. Atomic main/tag releases select one full native matrix;
all job bodies, permissions, matrices and validation gates are unchanged.
The development guide records exact-commit qualification for tag-only releases.

Actionlint, six event/ref selection cases, before/after workflow-body equality,
selected documentation links and whitespace checks pass. Evidence is retained
in `/tmp/ic-backup-ci-033/`. Manifest and incoming dirty lock bytes are preserved.
No Rust symbols were removed. No compilation, full gate, dispatch, commit, push
or release ran. Delivery and actual native CI acceptance remain with #33;
local workflow checks do not establish macOS qualification.

## Current 0.11.2 stage-owned checkpoint continuation

0.11.1 is live at `7492b19d2e5db8c9952c39f045568166f9a1f1c7`. Select compatible
0.11.2 for `ExecutionStageGuard::checkpoint`, which derives the existing exact
predecessor from canonical all-Applied publication and checks retained workflow,
binding/plan and complete ancestor histories before and after publication. The
public learned-stage and real capture/metadata/data callers now use this owner.
Failed post-admission retains checkpoint/journals and returns no predecessor;
learned-evidence qualification and effect permissions remain integration-owned.

44 focused stage/settlement/public/simulator cases, core test/Agent library Clippy,
both independent Rust 1.88 consumers and a standalone new-API caller pass.
Canonically adopt committed Shared 0.2.4
`ffbf665b8481c36b2d9f4d988abec557c3485fa6`: ten companion comments and guidance
change within the same 92-file roster; production helpers/rules/pins are unchanged.
The [review](../reports/audits/2026/10/09/continuation-0112/01/report.md) retains
exact provenance/checks. Preserve manifest/lock bytes, prepared Testkit 0.27.0,
all old/failed evidence and uncommitted work; package/receipt stay 0.11.1.
Released Linux CI passes while macOS/native acceptance remains pending. Keep
#29 open for installed full workflows and application/terminal/custody proof.
No release, Git write, live IC effect or Canic edit follows.

## Current 0.11.1 selected Testkit 0.27.0 preparation

Resolve the reported server-gate failure with explicit `make install-testkit-server`:
the locked 0.27.0 CLI was missing. Both direct offline admission and the actual
validation-runner target now pass. All six core/Agent simulator cases pass with
managed startup on unchanged Host 0.9.2, Testkit 0.27.0 and Metrics 0.3.1 under
Shared 0.2.3. Retain old installations and original failed validation evidence;
manifest/lock/snapshot bytes are unchanged. The
[review](../reports/audits/2026/10/09/testkit-027-setup/01/report.md) records exact
receipts, server bytes and checks. Repeat explicit setup after a Testkit selection
change; ordinary checks install nothing. Keep the 0.11.1 draft uncommitted with
package/receipt at 0.11.0. Local fixture qualification grants no arbitrary application
or terminal/release permission and does not close remaining native acceptance.

## Current 0.11.1 Shared Tooling 0.2.3 adoption

Canonically adopt committed Shared 0.2.3
`ac4549c5ebde497f7db0da5d05d32835112e51de`, superseding the preceding review's
uncommitted-source limitation. The same 92-file selection changes four guidance
files only; engineering rules, executable bytes/modes and tool pins stay exact.
Keep the optional installer suite omitted rather than adding unused production
wrappers. Its owner regression qualifies companion refusals from the complete
committed source; consumer qualification is separate. The
[review](../reports/audits/2026/10/09/shared-023/01/report.md) retains provenance
and focused checks. Preserve all current 0.11.1 work and Host 0.9.2/Testkit 0.27.0
manifest/lock bytes. The previously observed selected Testkit CLI prerequisite
remains separate; this guidance adoption changes no installation or runner.

## Latest Shared/issue review of 0.11.1

GitHub main still selects Shared 0.2.2
`ee48bb37c98c771e77b92fd891f0757d8c1c8b99`; our 92-file snapshot and pin checks
pass. The uncommitted upstream 0.2.3 fix for
[optional installer-fixture companions](https://github.com/dragginzgame/shared-tooling/issues/73)
is excluded; that optional suite is not in our snapshot. No new committed update
or closeable issue was found. Released consumer Linux CI and Shared Linux/lint
jobs pass; both repositories' macOS jobs remain queued.

The incoming selection has changed to Host 0.9.2 and Testkit 0.27.0, with Metrics
0.3.1 retained. Preserve these manifest/lock bytes. On this graph, 36 focused
settlement/stage/public cases pass and the Agent simulator fixture compiles.
The offline server gate correctly refuses the missing selected 0.27.0 CLI;
explicit `make install-testkit-server` preparation is needed before qualifying
actual startup. Earlier 0.26.0 simulator evidence remains separate. Review/check
evidence is retained at `target/shared-issues-review-0111/`. Keep the compatible
0.11.1 draft and uncommitted work; no implicit installation or release follows.

## Current 0.11.1 original-journal checkpoint continuation

0.11.0 is live at `d13823bcbf52649e3935ba8a48b1540b889174e9`. Select compatible
0.11.1 for canonical checkpoint derivation/publication from the exact retained
plan and complete original journals. Replace the simulator's private `settle`
assembly and the public learned-stage caller with that entrypoint; the existing
publisher rechecks full histories before immutable publication. Missing, held,
pending, NotApplied or Uncertain originals refuse without resetting spending.
No record format, outcome, terminal proof or release permission changes.

Canonically adopt committed Shared 0.2.2 handoff
`ee48bb37c98c771e77b92fd891f0757d8c1c8b99`: six changes within the same 92-file
snapshot fix final pin rows without newlines and exact CI-tool publication.
Focused checks pass, including 43 settlement/stage/public/simulator/metrics cases,
both-library and changed-target Clippy, independent Rust 1.88 consumers, installer
fixtures, snapshot/pin/ShellCheck, Testkit admission, formatting and links. The
[review](../reports/audits/2026/10/09/continuation-0111/01/report.md) binds exact
sources and retained logs. Preserve incoming lock bytes (Host 0.9.1, Metrics
0.3.1, Testkit 0.26.0), prior failed release evidence and all uncommitted changes.
Package/receipt remain 0.11.0; no release or Git write follows.

Released 0.11.0 Linux CI passes; both macOS jobs and exact Shared CI remain queued.
Keep #32 open for native adoption and #29 open for full provider-driven workflows,
fresh application authority/fencing and product terminal proof. Canic still owns
its separately authorized application integration; local checkpointing does not
complete that runner or release retained fences/source references.

## Current 0.11.0 missing Testkit selection correction

Committed implementation HEAD is `dfef36774eb778d52615aece3c47e254c7a63a57`,
with package/receipt still 0.10.1 and 0.11.0 pending. The maintainer-selected lock
now contains Testkit 0.26.0 and Metrics 0.3.0; earlier 0.25.5 CLI preparation does
not satisfy that exact selection. Explicitly prepare the locked 0.26.0 CLI/server.
Direct offline check and its actual validation-runner target pass. Missing CLI
admission now names the selected version and explicit setup command, preserving
its original failure status. Release docs include Testkit setup after locked cache
preparation and require repeating it when the selected package version changes.
Routing fixtures and ShellCheck/link checks pass; earlier graph qualification and
the original failed release evidence remain separate. Ordinary validation still
installs nothing. Keep changes uncommitted; no release resume or Git write follows.

## Current 0.11.0 local persistence cleanup

Remove nine private size-check forwarders and call the existing bounded JSON
owner directly with each original model limit. Keep the same admission sites,
error conversions, locking/publication order and recovery checks. All 151
persistence cases, both-library Clippy, Rust 1.88 and formatting/snapshot checks
pass on unchanged direct Host 0.9, Testkit's Host 0.8.10 dev graph, Metrics 0.2.20
and Testkit 0.25.5. The
[review](../reports/audits/2026/10/09/local-size-cleanup/01/report.md) lists every
removed helper and exact evidence. Keep the 0.11.0 draft, versions/receipt at
0.10.1 and all work uncommitted. No public/schema/effect change or native/full
CI qualification follows; prior hard-cut and dependency evidence stays separate.

## Current 0.11.0 Testkit server ownership cut

Select 0.11.0 for the breaking Shared Tooling 0.2.0 handoff committed at
`8140e3dd1b44409d682c721889ab702f438c6a17`. Canonically refresh 92 selected files,
removing both server checkers and their dedicated fixture. Keep the five shared
IC tool pins unchanged and select the exact locked Testkit 0.25.5 CLI through
Shared Tooling's Cargo installer. Explicit `make install-testkit-server` prepares
it; offline `make testkit-server-check` prints Testkit's admitted server path.
Both simulator suites use that route without local version/receipt projection or
shared-bundle fallback, retaining private logs/digests and original spending.

Actual Testkit setup/check and five-tool shared reinstallation pass on Linux;
all 21 old-bundle files remain unchanged. Thirteen core and three Agent simulator
journeys, seventeen original-stage cases, command/refusal/retention fixtures,
Clippy and Rust 1.88 pass; a concurrent direct Host 0.9 selection is separately
qualified on its final graph, retaining Testkit's Host 0.8.10 dev graph, Metrics
0.2.20 and Testkit 0.25.5. Host's exposed Rust error identity is part of this
minor hard cut. The
[review](../reports/audits/2026/10/09/testkit-server-cut/01/report.md) retains exact
inputs, failed attempts and final checks. #32 stays open for committed consumer
Linux/Intel/Apple Silicon acceptance; owner CI is separate. No complete runner or
application/terminal qualification follows. Keep prior stage resume work in this
draft, package versions/receipt at 0.10.1 and all changes uncommitted.

## Current 0.10.2 pinning fix and published graph follow-up

Adopt the committed Shared Tooling 0.1.38 fix at
`926a20606591214ab29faa236b0b584e4857439e`, keeping all 95 selections and excluding
the sibling's dirty VERSION. Multi-document exception catalogs now reject; focused
fixtures, actual pin/snapshot checks and ShellCheck pass. Preserve the incoming
Host 0.8.10/Metrics 0.2.20/Testkit 0.25.5 lock changes. Official registry checksums
and committed package sources match; Host's Rust/member manifests and Metrics
arithmetic are unchanged, so no new runtime offload appears. Testkit's optional
idle lifetime keeps our existing managed-server defaults.

Seventeen stage cases, the public journey, four diagnostics cases and three actual
Testkit download cases pass on the final graph. The
[review](../reports/audits/2026/10/09/updates-0102/01/report.md) retains earlier
Host 0.8.9 results, cache/registry refusals and final qualification separately.
Keep 0.10.2 pending and versions/receipt at 0.10.1. #25/#29 retain their broader
acceptance; the candidate has no matching native CI or installed runner claim.
Work remains uncommitted, without dependency reselection, sibling edits or release effects.

## Current 0.10.2 complete stage resume and tooling follow-up

Released 0.10.1 is local/GitHub main at
`5c4bd9df50d2cf4f954a236ab9b155778b33d645`; select compatible 0.10.2 for current work.
`ExecutionStageGuard::resume` returns the exact retained stage and complete original
child-journal progress, rechecking original ancestor history. Missing/changed/held
evidence rejects without creation, artifact/provider reads or allowance reset.
The public and real Testkit callers use this admission. Seventeen focused stage
cases, the public journey, three actual simulator cases, both-library Clippy and
Rust 1.88 checks pass on unchanged Host 0.8.9/Metrics 0.2.18/Testkit 0.25.4.

Canonically adopt Shared Tooling 0.1.37 at
`dc4fdf0f78928d75b69bbf43b37c690c53a04d1e`, keeping the 95-file selection. Installer
and local hook fixtures pass, including checkout-local tool discovery with wrong
inherited tools and missing/wrong pinned-tool refusal. The
[review](../reports/audits/2026/10/09/continuation-0102/01/report.md) retains focused
evidence. Correct stale README transport claims; the GitHub description is accurate.

Close #30/#31 from released 0.10.0's completed Linux/Intel/Apple Silicon tag CI,
separate from still-pending 0.10.1/new upstream runs. #25/#29 retain complete
transport/installed orchestration/application/restore/terminal acceptance. Work is
uncommitted, versions/receipt remain 0.10.1 and no sibling or release effect occurs.
Earlier entries retain the graphs and snapshots used for their original checks.

## Current 0.10.1 ancestor-history admission

Fix stage admission skipping transitive predecessor history: `create`, `prepare`,
`open` and `layout` now check every ancestor's exact binding/plan, chronological
settlement and original journals. Iterative traversal admits shared ancestors once
with sequential locks and rejects changed/missing history without repair or spending.
Capture/metadata/data simulator stages all use complete journal preparation.
Thirteen focused stage cases, the public journey, three actual Testkit download
cases and both-library Clippy pass on the incoming Host 0.8.9/Metrics 0.2.18/Testkit
0.25.4 graph. The [review](../reports/audits/2026/10/09/stage-ancestry/01/report.md)
retains failures and exact qualification. Keep 0.10.1 pending and versions/receipt
at 0.10.0; no schema/API change, release or Canic adoption follows. Earlier entries
retain the graphs and snapshots used for their original qualification.

## Current 0.10.1 Shared Tooling follow-up

Canonically refresh the unchanged 95-file selection to committed Shared Tooling
0.1.35 at `be550afa57fe9e16872e5110b5cd69c24b4fa9e8`; seven selected files change.
The optional exact Cargo binary/example installer adds profile and receipt checks
without changing default setup. Existing release preflight already prepares the
locked root-workspace cache; explicit offline settings remain authoritative.
Focused installer/command/release-adapter fixtures, ShellCheck, actual offline
tool checks and snapshot verification pass on Linux. Upstream's exact 0.1.35 CI is
in progress at inspection; no candidate-native or real registry-install claim follows.
The [review](../reports/audits/2026/10/09/shared-035/01/report.md) retains scope and
evidence. Preserve all incoming stage work and lock selections; versions/receipt
remain 0.10.0 and work remains uncommitted. The earlier stage qualification below
records the 0.1.34 snapshot used for that earlier batch.

## Current 0.10.1 complete stage-journal preparation

Pushed 0.10.0 is HEAD at `d909fb2f2ecbcc40e8bea920bd3fa45a0a5ea57b`.
Select compatible 0.10.1 for the new `ExecutionStageGuard::prepare` entrypoint:
derive all original child authorities before allocating and durably create every
journal through existing owners before returning. Keep record-only `create`
distinct and reopen read-only. Partial publication, occupied journals and process
death retain originals; no retry fills gaps, and missing evidence never supplies
unused allowance. This advances [#29](https://github.com/dragginzgame/ic-backup/issues/29)
without introducing a schema, spending owner, receipt or installed runner.

Ten stage cases and the public learned-ID journey pass. Three real Testkit planned
download cases now use preparation: complete durable publication, lost reply and
malformed reply preserve original spending, partial bytes and references. Both
libraries' Clippy passes on the released graph. Preserve the incoming Metrics
0.2.18/Testkit 0.25.4 lock update, unchanged Host 0.8.8 and reviewed Shared Tooling
0.1.34. The same fourteen focused cases, Clippy, Rust 1.88 with independent
consumers and strict rustdoc pass on the incoming graph. The agent performs no
dependency selection or snapshot refresh.
The [review](../reports/audits/2026/10/09/stage-preparation/01/report.md) retains exact
commands, source and failures separately. Work remains uncommitted, with package
versions/receipt at 0.10.0 and no sibling, release, publication or live IC effects.

Released 0.10.0 Linux CI passes; Intel/Apple Silicon jobs remain queued.
[#30](https://github.com/dragginzgame/ic-backup/issues/30) and
[#31](https://github.com/dragginzgame/ic-backup/issues/31) retain native acceptance;
new uncommitted preparation has no matching hosted qualification. Fresh application
admission/custody, installed orchestration, upload/restore and terminal/fence/source
reference release remain separate. Earlier handoff entries below describe their
original preparation state rather than current release identity.

## Current 0.10.0 tooling retirement and HTTP fixture repair

Release 0.9.2 is now HEAD and GitHub main at
`f7868ce3f672da5ea1489172fd97e83699fe4f99`. The maintainer explicitly selected
retirement of local fleet reports under
[#30](https://github.com/dragginzgame/ic-backup/issues/30); choose 0.10.0 because
`make cloc-tooling` stops producing reports here. Run it from Shared Tooling.
Package versions and release receipt remain unchanged. Preserve the concurrent
Host 0.8.7 lock update; Metrics 0.2.16 and Testkit 0.25.3 remain selected. Earlier 0.9.2 preparation entries below are historical.

Canonically adopt committed Shared Tooling 0.1.34 at
`3d33cd250fcae7dbe5cabe44b2abd6b2c91a1822` from a clean isolated checkout.
Remove the verified fleet Perl reporter and shell regression, both snapshot rows
and scheduled Make/CI/help references. The narrowed 95-file snapshot matches all
committed bytes/modes; local setup/check/LOC owners remain selected. The retirement
removes 405 measured code LOC. Focused shared commands, workspace LOC, ShellCheck,
archive boundaries and snapshot checks pass on Linux. Exact upstream 0.1.34 CI is queued and released 0.9.2 CI was still in progress
at inspection; no candidate-native claim follows. Focused literal manifest-path
fixtures and actual offline PocketIC alignment pass on the final snapshot.

For [#31](https://github.com/dragginzgame/ic-backup/issues/31), explicitly select
blocking accepted sockets and retain the original three-second capture deadline.
Bound header/body framing and retain exact partial bytes with original IO errors.
Explicit server finish reports errors; Drop preserves a primary unwind and writes
cleanup diagnostics instead of panicking again. Original one-request, routing,
signed body and pending-spending assertions remain. Eight focused HTTP cases,
both libraries' warnings-denied Clippy and Rust 1.88/all-target/independent-consumer
checks pass on Host 0.8.7, along with focused artifact regressions. All four
published archives and 65 Rust files match registry/committed VCS evidence.
Native Intel/Apple Silicon acceptance remains pending; keep #31 and
#30 open until matching committed native qualification.

Released archive integration under
[#28](https://github.com/dragginzgame/ic-backup/issues/28) now has actual hosted
upload/download evidence on all three hosts; #28 is closed. Native archive fixtures cover unusual
names, modes, symlinks and refusal/partial-output behavior; ordinary failed jobs
retain their failure status while archive/upload succeed. Downloaded exact-ID
payloads retain bounded selected roots, private modes and links. Special-name
fixtures and ordinary hosted payloads are separate evidence, not one combined test.
[#25](https://github.com/dragginzgame/ic-backup/issues/25) and
[#29](https://github.com/dragginzgame/ic-backup/issues/29) still own complete Agent,
installed runner and application/terminal qualification. This batch adds no product
effect authority or runner. The
[review](../reports/audits/2026/10/09/shared-033-issues/01/report.md) retains inputs,
failure logs and exact check scope. Work stays uncommitted; no sibling, Git/release,
publication or live IC effects occur.

## Current 0.9.2 Shared Tooling and Host follow-up

Extend the same compatible 0.9.2 draft from released 0.9.1; package versions and
release receipt remain unchanged. Canonically adopt committed Shared Tooling
0.1.32 at `635a39a9dd5f8d021fa9c9196b591e00521a7e02` from a clean isolated
checkout. Eight files change in the unchanged 97-file selection; dirty sibling
0.1.33 dashboard work is excluded. The IC installer reuses complete validated
selections despite comments/row order, retaining original receipts/bundles.
Changed records and malformed/unsafe evidence still refuse. Focused installer
fixtures, ShellCheck, alignment and real offline equivalent-pin admission pass.

Official Host 0.8.5 archives and all committed package source/manifest bytes match.
Its Rust sources are unchanged from 0.8.4, so no additional runtime offload is
justified. Preserve the incoming Host 0.8.5 graph and all prior dirty implementation.
Focused JSON publication/read, inherited custody, original journal, stage/planner
and public stage cases pass on that graph. Three real planned-download Testkit
cases also pass: durable publication, lost reply and malformed reply with retained
original spending/partial bytes. Both libraries pass warnings-denied
Clippy/rustdoc and Rust 1.88 with independent normal consumers. Both exact Host
revisions now have successful Linux and both macOS native CI; Shared Tooling
0.1.32 does too.
This uncommitted consumer candidate has no new native CI evidence.

Testkit's PocketIC setup/check handoff remains open and unpublished. Preserve
the functioning selected matrix, explicit setup and all bundles until that owner
is published and qualified. Cargo-install assessment/fleet dashboard scripts
remain upstream; no extra consumer gate or Metrics feature is introduced.
The [review](../reports/audits/2026/10/09/shared-032-host-085/01/report.md) retains
exact evidence. This batch changes no Rust API, journal, allowance or IC behavior.
Work stays uncommitted; no sibling, Git/release, publication or live IC effects.

## Current 0.9.2 original download planning

The maintainer clarified that 0.9.2 has not been pushed. Release 0.9.1 remains HEAD
at `d6c6082efc7c36de98e6e92ec5b4091de5ca43b4`; extend the existing compatible
undated 0.9.2 draft. Package versions and release receipt remain at 0.9.1.
Preserve incoming Host 0.8.4, Metrics 0.2.16 and TOML/parser lock selections.

`IcSnapshotDownloadPlan` counts complete metadata-derived region/chunk reads with
checked arithmetic before allocation, within the original data stage's finite
allowance. Reuse exact existing codecs, original ordinary plans and stage bindings;
each request has one update and zero observations, with explicit writer dependencies.
Binding checks the exact original metadata stage/request and reply evidence. No
journal, spending owner, new record or automatic receipt is introduced. Oversized
snapshots reject; entirely read-free metadata creates no placeholder operation.

Three real Testkit/PocketIC cases pass: complete capture/download/durable publication
and manifest replay, lost data reply, and malformed data reply. Failure cases stop
with original pending spending, partial Created bytes and source references intact;
dependent admission/recreation refuse and reopen makes no call or new journal.
The driver owns isolated stopped-fixture safety and single-ingress custody only.
It is not an installed generic runner or actual Canic application qualification.

Five planner boundary cases, eleven stage regression cases and the public stage
journey pass. Both libraries pass warnings-denied Clippy/rustdoc and Rust 1.88,
including independent normal consumers. Formatting, local documentation links and shared snapshot integrity
pass. Local Linux evidence is separate from native macOS/full product qualification.
The [review](../reports/audits/2026/10/09/planned-download/01/report.md) retains exact
source, graph, logs and simulator evidence. [#29](https://github.com/dragginzgame/ic-backup/issues/29)
remains open for installed provider-driven orchestration, fresh application admission,
upload/restore and full terminal/release qualification. Work stays uncommitted.

## Current 0.9.2 stage-binding and Host review

Release 0.9.1 is HEAD at `d6c6082efc7c36de98e6e92ec5b4091de5ca43b4`.
Select the compatible next draft 0.9.2 for new local APIs and Host 0.8.4 within
the existing 0.8 requirement. Existing public types and record semantics remain;
package versions and release receipt stay at 0.9.1. Preserve the incoming lock
update from Host 0.8.3 to 0.8.4; no dependency reselection or sibling patch occurs.

Distinct original workflow allocations bind fixed later stages to exact child
plans, original target/context/full inventory and immutable per-stage ceilings.
Only existing child attempt journals own spending. Creation requires exact
predecessor bindings and complete chronological Applied settlements; retained
opaque learned-input commitments do not authenticate snapshot IDs/dimensions or
their derivation. Integrations retain original learned evidence bytes. Fixed
private stage directories refuse replacement; partial preparation stops with
evidence retained. Reopen admits exact originals without journal creation,
budget reset, dispatch, repair or cleanup. Strict v1 schemas and independent binary
goldens are linked from [the contract](../extraction-boundary.md#original-workflow-allocations-and-learned-stage-binding).

Published Host 0.8.4 at `97187b2a46d6f8a6964224a36a133d858ef0d223` matches all
four official registry rows, archive checksums, clean VCS identity and committed
source files. Its existing-lock fast path avoids staging/sync in read-only parents.
The shared opener still creates missing files with general publication permissions;
Backup's existing-only custody and 0600 sidecars retain their current owner.
No further production offload is admitted. Host Process/Tools stay test-only
transitive dependencies through published Testkit, not a restored subprocess backend.

Eleven new model/persistence cases and the public learned-ID/pending journey pass.
Focused existing JSON publication/read, journal reservation, settlement replay and
inherited command-custody cases pass on the selected graph. Both libraries and
independent normal consumers compile on Rust 1.88; final Clippy, rustdoc, formatting
and local documentation links pass. Exact Host 0.8.4 native CI remains queued,
and this uncommitted candidate has no native macOS or full IC workflow qualification.

The [review](../reports/audits/2026/10/08/workflow-host-084/01/report.md) owns exact
checks and CI scope. This batch advances local stage binding under
[#29](https://github.com/dragginzgame/ic-backup/issues/29); authenticated learned
inputs, installed capture/download orchestration, fresh application admission,
complete transfer and terminal/fence/reference release remain separate. Work stays
uncommitted; no release, live IC effect, publication or sibling edit occurs.

## Current 0.9.1 upstream review

Release 0.9.0 is HEAD at `32f091a3cc576dc7e0d8b80aea6a484abf905984`.
The compatible next draft is 0.9.1; package versions and the release receipt stay
at 0.9.0. Rust source remains unchanged. A concurrent lock edit selects Metrics 0.2.15.

Adopt committed Shared Tooling 0.1.29 at
`1a54fb625d6e47efa64c4384808ecbc87be84e7e` through the canonical exporter from a
clean isolated checkout. All 97 files match committed bytes/modes: 15 existing paths change and
the release-source checker plus two CI-installer paths are added explicitly. Initial preflight now reports
all offending staged/unstaged/untracked paths, quotes unusual names and explains
that validation/version preparation have not started for that attempt. Prepared
checks preserve metadata allowances and unchanged original HEAD; committed/resumed
checks require clean source. Git errors remain visible; nothing is auto-staged or
repaired. Real private-index fixtures preserve lock/index/working bytes and HEAD.

Standing issue permission is restricted to `dragginzgame/*`, including adopted
maintenance prompts. Other destinations need explicit destination/action authority.
No external issue write or schedule activation occurs. The Rust-only pin gate
continues without selecting the new optional npm mode. Local full tooling,
release-adapter/runner simulation, declaration, shell and documentation checks
pass on Linux; exact native macOS remains separate.

The official registry index still selects non-yanked Host 0.8.2 for all four
packages; cached archive digests match the original lock and source proof. Host's
read-only local committed 0.8.3 draft adds live child output observation and an unlocked durable
path opener, but neither is published or selected. Backup's existing-only custody
admission and 0600 sidecar policy cannot be replaced by an opener that always
creates missing files/parents. Existing source/runtime evidence retains its original
owner; no dependency downgrade, path patch or extra process owner is introduced.
The consumer-owned PocketIC 16.1 matrix stays unchanged.

Delivered 0.9.0 Linux CI fails SC2015 with Ubuntu ShellCheck 0.9; local 0.11
accepts the same shared expression. Select the reviewed 0.11 binary in Linux CI
through its canonical checksum installer, and prefer a prepared checkout-local
ShellCheck. Actual installer, shell lint, full tooling and final real-index
metadata fixtures pass. No vendored checker edit or lint suppression is added.
Host 0.8.2 now passes all native/MSRV owner jobs. Shared 0.1.29 passes Linux and
lint/security while both macOS jobs remain queued; current consumer native CI
must run on the new committed source before claiming acceptance.

A concurrent external lock update selects Metrics 0.2.15; retain it. Every
published Rust byte is identical to 0.2.14 and exact committed source/archive
proof matches the non-yanked registry row. Empty host features remain selected.
Fresh Rust 1.88 checks compile both libraries and two independent public consumers,
and all five focused diagnostic/distribution cases pass on this exact graph.
Original tooling proof remains separate; no accounting or metrics schema changes.

[The review](../reports/audits/2026/10/08/shared-029/01/report.md) records exact source,
registry, focused checks and observed native CI scope. Full workflow/stage binding,
Canic adoption and terminal/fence/reference release remain separate. Work remains
uncommitted; no sibling edit, release, publication or live IC effect is performed.

## Retained 0.9.0 Shared Tooling 0.1.28 adoption

Adopt committed Shared Tooling `1872ed2c20f6c70689bb2249050b1d673c60bfa0`
through the canonical exporter. All 94 selected files match committed bytes/modes:
17 existing paths change and 15 explicit task/runner-fixture additions close the
new governance roster. The consumer-owned PocketIC 16.1 matrix and exact
original Host 0.8.1 / Testkit 0.25.1 / Metrics 0.2.14 lock were preserved by
the refresh. A subsequent concurrent update selects Host 0.8.2; retain it and
repeat MSRV/independent-consumer, declarations, Clippy and package checks.
Only Host's private macOS cleanup fixture changes; production Rust is identical. No vendored
patch, sibling edit, root Git write, release or publication occurs.

The new common rules separate low qualified MSRVs from development compilers.
Both libraries pass Rust 1.88 all-target/all-feature compilation in a retained
candidate before changing the actual inherited floor. Independent actual consumers
then compile the exact original selected normal dependency graph without Testkit
or dev feature unification. Host/Agent declare 1.88; no dependency downgrade or
ignored floor is used. `make check-msrv` records compiler/Cargo versions and retains
both consumers, and CI explicitly installs/selects 1.88. Development stays 1.99.
The `std`-path rule already matches every maintained Rust source/test/example.

Shared release-runner tests now simulate Git effects, while the separate real-Git
suite remains owner-only. Installed host/IC links reject literal malformed targets
before execution/download. The new maintenance catalog is accessible through
`make tasks`; its fixture uses a substitute CLI. No live agent or schedule is
activated. Existing archive roots, uploader and retention remain unchanged.

Full tooling and release-adapter/simulation fixtures pass locally on Linux.
The [review](../reports/audits/2026/10/08/shared-028/01/report.md) owns source-bound
focused evidence and original preparation failures. Prior 0.1.27 owner CI now
passes; exact 0.1.28 was queued at the last read. Current consumer native macOS,
full workflow/stage binding (#29), actual Canic adoption and terminal/fence/reference
release remain separate. The current draft stays 0.9.0 and package/receipt 0.8.1;
work remains uncommitted. Earlier Testkit/runtime proof retains its original scope.

## Retained 0.9.0 Testkit adoption

0.8.1 is released at `8d1656e7c6c9c29032ad063fe351fc040d316252`.
The maintainer requested Testkit instead of a direct PocketIC dependency.
Both dev dependency declarations and fixture imports now use published Testkit
0.25.1. Managed startup has a 60-second readiness deadline, explicit admitted
server bytes and caller-owned retained 0600 output files. Instance teardown
precedes server cleanup. Exact original calls, reservations, lost-reply behavior,
byte comparisons and fixture application safety stay local; no Testkit snapshot
retry/reset or funding policy is selected.

Preserve the incoming Host artifacts/FS 0.8, Metrics 0.2.14 and PocketIC 16.1
selection. Final Testkit 0.25.1 joins a single Host 0.8.1 graph; process/tools are
dev-only through Testkit. A concurrent patch update
selected Testkit 0.25.1 and all four Host 0.8.1 packages; preserve and freshly
qualify it. All 151 published Rust/original-manifest blobs match non-yanked
official archives and committed source. Testkit and Metrics Rust bytes are
unchanged; Host adds bounded allocation improvements and an unused immediate
path-lock API. Original Backup records/bounds remain unchanged, but the public
publication error's Host identity changes: select the 0.9.0 draft. Package/receipt stay 0.8.1.

Under the shared guide's delegated pin selection, transfer `ci/ic-tools.tsv`
out of the immutable snapshot. Other 79 paths retain exact source/modes at
`db039347d2372b877c1c46dcdd2b5c3aa9412009`. The single consumer matrix pins
PocketIC 16.1's three official archive hashes; other rows/entry points remain.
Explicit preparation and offline alignment pass on Linux. The local tooling
fixture initially omitted this consumer-owned file; retain its failure, copy the
matrix into the isolated consumer and pass the full tooling target. Preliminary
Testkit 0.24 / PocketIC 16.0 proof remains separate from final evidence.

All 158 focused Rust cases, both-library Clippy, Rust 1.91, strict rustdoc and
package checks pass on Linux. Full tooling, shell, formatting, dependency,
documentation, snapshot and exact installed IC admission also pass. Native macOS
acceptance for this uncommitted source, full workflow/stage binding (#29), fresh Canic adapters and terminal/fence/reference
release remain separate. The [review](../reports/audits/2026/10/08/testkit-adoption/01/report.md)
owns exact inputs, graph/features, command logs and qualification limits.
Work remains uncommitted; no sibling edit, root Git write, release, publication
or public IC effect is performed.

## Earlier 0.8.1 upstream refresh

The maintainer's full `tooling-check` failed at consumer commit
`9be366b`: adopted host/IC installer suites called missing `test-tool-evidence.sh`.
The earlier focused qualification did not run those suites. Preserve the original
full log and correct the incomplete selection through committed Shared Tooling
`db039347d2372b877c1c46dcdd2b5c3aa9412009` and explicit companion additions.
The current snapshot has 80 exact source/mode paths. The exporter now refuses
the old incomplete selection before writes. Full `make tooling-check`, shell
and dependency checks pass on Linux, including both installer suites and the
actual shared collector shell body against substituted payloads. Consumer CI
keeps its original archive policy/uploader/retention; the imported action is a
fixture dependency. No Rust, dependency selection, package or recovery contract
changes. [The repair review](../reports/audits/2026/10/08/tooling-companions/01/report.md)
binds original failure and current evidence. This repair remains uncommitted.
The earlier 77-file adoption below retains its original source and test scope.

0.8.0 is released at `04f65896060b1f1cd47fd36906a6d88dbe6c8501`.
The incoming lockfile selected Host artifacts/FS 0.7.1 and Metrics 0.2.13;
preserve it exactly. All 46 published Rust/original-manifest blobs match official
committed source and non-yanked archive checksums. Rust implementation bytes are
unchanged from the prior selected patches; features remain empty. No process
dependency, record, spending, publication or diagnostics owner is added.

Adopt committed Shared Tooling 0.1.27 at
`b866d41041a1986eeec95bde9af4c6ba0853d2e3` through its canonical exporter.
All 77 source bytes/modes match; 32 selected paths change and common rules stay
unchanged. Consecutive uncommitted refresh and edited-input refusal pass in a
retained isolated consumer. The local release bootstrap originally failed under
`CDPATH`; metadata fixtures originally failed with inherited `MAKEFILES`/GNU
flags. Fix local physical root resolution and fixture-only context isolation;
retain both failures and demonstrate the corrected actual commands.

All 158 focused Rust cases, both-library Clippy/Rust 1.91/package checks, release
adapters/runners, validation/snapshot retention, tooling LOC, hooks, formatting,
dependency declarations, snapshot, shell and documentation checks pass on Linux.
Exact-source Shared/Host/consumer native CI remains pending; Metrics CI is in
progress at the last read. Prior Shared 0.1.26 Intel CI timed out under upstream
#71. Those states do not negate local results or qualify native macOS.
The [review](../reports/audits/2026/10/08/upstream-refresh/01/report.md)
owns source-bound inputs, commands, failures and limits.

Correct the public description now that Agent transport is released. Full
orchestration/stage binding remains #29; fresh Canic adoption and terminal/fence/
reference release remain separate. The compatible draft is 0.8.1; package/receipt
stay 0.8.0. Work remains uncommitted; no sibling edit, root Git write, live IC,
release or publication runs. Earlier handoffs below retain their original scope.

## Earlier 0.8.0 direct Agent hard cut

The maintainer selected direct Rust `ic-agent` as the sole product transport.
`crates/ic-backup-agent` implements bounded async preparation and single-update
submission through exact original reserved request owners. It checks current
journals, actual signer and declared context, retains signed ingress identity
before dispatch, and disables retries, redirects, proxies, root fetching and
automatic polling. Pending/error/cancelled/lost replies retain original accounting,
obligations and references. ICP-specific source/probe is retired; historical logs
and independent command-custody contracts remain. No backend fallback is added.

An external update selected Host 0.6.0, crypto-common 0.1.6 and generic-array
0.14.9 after the initial agent graph. It is preserved and independently qualified;
earlier 0.5.2 results retain their exact inputs. Official archives/source match.
Artifacts/FS bytes are unchanged; process adds opt-in group capture and group_error,
which Backup does not select. Exact owner Linux/Intel macOS/Apple Silicon macOS and MSRV CI now pass;
current consumer native macOS still requires its own committed source.

The pending draft is 0.8.0 because the selected transport/command contract is a
hard cut; all prior 0.7.1 notes carry forward. Package/receipt remain 0.7.0. Both
libraries join configured native validation, packaging and explicit publication.
Release preparation derives member manifests/names from selected TOML and binds
both exact manifests, local lock entries and internal registry requirement. The
focused private release fixtures pass; no actual release/Git effects run.

Real local HTTP tests cover exact signed bytes/routing, accepted responses, 429/503,
redirects, malformed/oversized bodies, disconnect and timeout with one request and
unchanged pending journals. Real isolated PocketIC Agent gateway tests cover
capture, exact metadata, complete 1 MiB heap/stable reads, upload/load/start and
same-ID heap/stable/global restoration through an actual Ed25519 controller. Discarded stop replies halt with retained
pending spending; wrong-root authentication fails even though the actual stop
applied. This is transport/fixture evidence, not full runners, generic application
safety, Canic adoption, native macOS acceptance or terminal/reference release.
A subsequent external update selected Host 0.7.0 and Metrics 0.2.12. The final
frozen graph passes fresh consumer qualification with empty selected features.
Remove `ic-host-process`: its sole caller was the retired ICP probe. Host
artifacts/FS and existing descriptor custody retain their separate owners.
All 158 focused Rust cases and 61 release-adapter fixtures pass, along with
Clippy, Rust 1.91, strict Agent rustdoc, both package archives, formatting,
documentation, dependency/snapshot and shell checks. Original metadata, source
hashes, commands and failed/corrected attempts bind the
[final qualification](../reports/audits/2026/10/08/agent-transport/01/qualification.json).
Official source/archive proof matches. Host owner CI passes; Metrics exact-source
owner CI remains queued, while local diagnostic/histogram cases pass. Earlier
0.6.0 results remain bound to their original graph. Full CI/release gates were not run.
[The transport scope](../agent-transport.md) owns current qualification and limits.
All changes remain uncommitted. Prior source/graph evidence below stays historical.

## Earlier 0.7.1 execution admission for downstream integration

Continue the compatible 0.7.1 draft with `read_execution_progress` and guarded
planned mutation/observation reservations. Both use the exact persisted original
plan and complete journal set; missing evidence cannot become zero consumption.
The selected journal remains locked while other journals are admitted sequentially.
Retained Applied prerequisites gate mutation reservations. The existing attempt
owner alone consumes allowance; failures and lost replies preserve spending.
Settlement replay shares the same original-journal reader. No new schema, provider,
terminal/reference release or fresh-authority flag is added.

All 134 affected persistence cases (including five new admission cases), eight pure
progress cases and six public recovery/source/settlement cases pass. Clippy, Rust
1.91, strict rustdoc, formatting, documentation and dependency checks pass.
The existing real ICP/PocketIC routing-refusal case also passes with original pending
spending retained. No direct-agent runtime is installed or qualified. Inputs and
failed/corrected initial results are retained under `target/canic-readiness-071`;
the first failure was a fixture expecting MutationPending where the existing owner
returns ObservationPending.

An external lock update selected Host artifacts/fs/process 0.5.2 during this batch.
It is preserved. All 57 published Rust/manifest blobs match official non-yanked
archives and exact `c7014995bf0890c1df9cd9b9a6ec14ea70f98c6f`. Artifacts/fs and
all original manifests are unchanged; process adds opt-in background handoff,
which Backup does not select. Exact owner native/MSRV CI passes. Fresh source-bound
execution/recovery tests, offline locked metadata and the actual pinned ICP probe
qualify this graph rather than relabelling prior 0.5.1 evidence. Metrics remains
0.2.11 and selected Host/Metrics features remain empty. See the
[review](../reports/audits/2026/10/08/execution-admission/01/report.md).

Shared Tooling 0.1.25 now passes exact all-native/hosted owner CI. Refresh to all
77 canonical paths with explicit archive-helper/regression additions. Consumer CI
archives its original evidence roots before the unchanged pinned v4 uploader,
artifact name and seven-day retention. Real Linux round trips of the actual inline
collector pass for absent roots and early/late failure evidence; names/modes/hidden
files/links survive and Git metadata stays excluded. Actionlint and ShellCheck pass.
Current consumer hosted/native acceptance remains #28; source ownership/qualification
are in the [archive review](../reports/audits/2026/10/08/shared-archive/01/report.md).
Generic orchestration and learned snapshot/extent stage binding now have a concrete
owner in [#29](https://github.com/dragginzgame/ic-backup/issues/29), distinct from
transport #25 and Canic's fresh Fleet/application adapter requirements.

Read-only Canic inspection finds its active worktree already consumes registry
Backup 0.7 artifact/checksum APIs; downstream adoption/qualification remains owned
there and no Canic source is edited. Full executable capture/restore still requires
transport, generic orchestration and fresh Fleet/application preflight. ICP CLI
remains the selected planned backend. Direct Rust `ic-agent` is a long-term
recommendation under review, not an adopted backend. Its official published API
supports exact receiver/effective-target updates without implicit wait, but defaults
include retry behavior that needs explicit bounded qualification. An isolated
exploratory build compiled; the local simulator failed socket admission in the
restricted sandbox and the session was interrupted. That is no transport result,
no root dependency change and no reason to claim a live backend.

## Earlier 0.7.1 upstream adoption qualification

0.7.0 is released. Its exact committed source `f836d5e58170c6b16dba2552c10391666f86ce7e`
passes Linux, Intel macOS and Apple Silicon macOS in
[CI 37753291659](https://github.com/dragginzgame/ic-backup/actions/runs/37753291659),
qualifying the delivered Shared Tooling 0.1.23 selection separately from new work.
Earlier sections below are historical batch handoffs, including their old draft
and package statements.

The compatible next draft is 0.7.1; package/receipt stay 0.7.0. Preserve the incoming
registry Host artifacts/fs/process 0.5.1 lock selection and Metrics 0.2.11. All 54
Host Rust files and three original manifests match official non-yanked archives,
exact source `81f9809861159def2fd0987fcb7961cda4afd969`, and prior 0.5.0 bytes.
Actual selected features remain empty. The owner passes all supported native hosts
and MSRV. There is no new published runtime owner to replace additional Backup
code; retained serialization, descriptor custody, spending and recovery owners stay.

Shared Tooling 0.1.24 at `e9bfdc54c0daefc3dbbdfe091e5665dca5468eb3` is exported
canonically from a clean committed source with all existing 75 paths. Sixteen
paths change and every blob/mode matches. Companion admission and LOC fixes join
the ordinary release-tracking refresh. Static symbolic refs and changed OIDs are
preserved; the same-OID ref-type race remains upstream #62. The new archiver is
unselected while upstream #59's admission correction remains uncommitted.
The upstream hosted Apple Silicon artifact download failed after portable/tool
checks passed; complete owner native/hosted acceptance is not claimed.

All 142 operations and eight public artifact/recovery cases pass on the selected
graph, with Clippy and Rust 1.91. Focused release-adapter/runner, logger/retention,
LOC, exporter/companion, snapshot, shell, dependency, formatting and document checks
pass. Inputs, logs and source/registry/CI proof are in `target/upstream-071-review`
and the linked adoption reviews. No new simulator run relabels earlier graph evidence.
Backup #27 is closed on committed 0.7.0 acceptance. Shared #60/#61 close after the
Intel portable job completes successfully; #59/#62/#63 retain their distinct gaps.
Current dirty consumer
native macOS proof requires its own committed CI. Production transport #25 remains
blocked on pinned ICP 1.6.0 granular routing and independent internal-call/custody
qualification; no new upstream response or release removes those prerequisites.
No product source/API/record changes, Git writes, release or publication effects run.

## Current 0.7.0 canonical IO cleanup and Metrics qualification

The preserved incoming lock now selects registry Metrics 0.2.11, with Host
artifacts/fs/process 0.5.0. No dependency update ran. Its non-yanked official archive
matches the declared checksum and every published source blob matches committed
`69b110b8fbefdac4773eac7631796f9dcb3f41a0`. Metrics Rust is unchanged from 0.2.9/0.2.10;
the actual resolved feature set is empty. Fresh diagnostics/guarded upload checks
qualify this selection rather than relabelling the earlier graph's evidence.

Four private `errno_to_io` functions and their repeated inline conversions are
removed in favor of Rustix's existing `io::Error::from` implementation. Artifact
traversal, directory publication, IC tree verification/upload and layout/journal/
command locks retain native codes and typed failures without a replacement local
wrapper. All 142 affected operations cases, eight public artifact/upload/source/
recovery/settlement cases, configured Clippy and Rust 1.91 checks pass on Linux.
Exact inputs, registered cases, archive/source proof and logs remain under
`target/hard-cut-070-review`; the failed initial selection of a nonexistent public
test target is retained separately, then corrected. No new test mirrors the removed
helper. Earlier PocketIC results retain their actual inputs; no IC effect path changes
and no new simulator or complete gate runs.

The hard-cut review retains independent source-byte/custody checks, immutable v1
record boundaries, serialize-once JSON buffering and same-contract interruption
recovery. It finds no compatibility alias or dual reader in the inspected metrics/
artifact/lock scope. Existing reservations, fence obligations and references retain
their exact owners; no reset, cleanup, provider, terminal or release API is added.
Only production transport [#25](https://github.com/dragginzgame/ic-backup/issues/25)
remains open. Current consumer native macOS delivery remains separate. The draft
stays 0.7.0 for Host's exposed error-type identity; package/receipt stay 0.6.0 and
all work remains uncommitted.

## Shared Tooling 0.1.23 and delivered 0.6.0 inspection

The accepted refresh adopts exact committed Shared Tooling
`0ba0ad00ed94848e54ecc82629b6b7873b7284c0` through the canonical exporter from a
clean isolated source. All 75 prior paths remain; seven change and every blob/mode
matches remote committed source. The baseline/rules are unchanged. Direct release
admission now rechecks payload/index/tag after final hooks and completed resume
observes exact destination identity without repeating effects. PR pagination uses
jq; the consumer still selects direct delivery. Fresh Linux release adapter/private
index, canonical runner, snapshot/logger/retention and shell checks pass. The
optional PR fixture passes on synthetic Git and a GitHub substitute; no actual PR
or merged-source product release is qualified. [The review](../shared-tooling-review.json)
binds source, focused results and native owner scope. The batch extends 0.7.0 and
keeps package/receipt 0.6.0; no repository Git or release effects run.
Exact upstream 0.1.23 CI now passes Linux, both native macOS architectures and
lint/security. The current dirty consumer graph retains its separate native gate.

Exact released 0.6.0 main and tag CI now pass Linux, Intel macOS and ARM macOS at
`8a1152d0a510f34f8daed59f06632306f7cf134e`. Actual registry source was independently
verified. Published directory framing and typed JSON publication therefore have
complete native delivery. The maintainer's normative serialize-once-before-effects
contract explicitly retains the buffer; no memory-saving claim is made. Delivery
evidence is recorded on [#24](https://github.com/dragginzgame/ic-backup/issues/24#issuecomment-6055599951)
and [#26](https://github.com/dragginzgame/ic-backup/issues/26#issuecomment-6055600738),
with a [Canic handoff](https://github.com/dragginzgame/canic/issues/490#issuecomment-6055666405).

ICP's latest release remains 1.6.0. Its maintainer asked about dedicated commands;
[the response](https://github.com/dfinity/icp-cli/issues/811#issuecomment-6055666046)
names exact granular metadata/data read/allocation/write requests and original
per-call accounting, backed by the pinned transfer implementation. Whole-transfer
commands orchestrate several calls. [#25](https://github.com/dragginzgame/ic-backup/issues/25)
still owns actual production transport qualification, independently of routing.

A later concurrent Cargo.lock edit selects Metrics 0.2.10 instead of 0.2.9. It is
preserved; no lock update command ran in this adoption. Earlier Host/PocketIC proof
retains its actual lock identity and does not qualify that new selection. The
current graph needs focused Metrics/runtime qualification and independent native
delivery. Rust implementation and incoming Host requirements remain unchanged;
no broad gate, live IC call, sibling edit or retained-evidence cleanup runs.

## Pending 0.7.0 published Host 0.5 qualification

Released 0.6.0 is the incoming base, `8a1152d0a510f34f8daed59f06632306f7cf134e`.
Its non-yanked official registry row is observed. The maintainer's
incoming manifest/lock edits select registry artifacts/fs/process 0.5.0 with empty
Host features; they are preserved exactly. Package/receipt remain 0.6.0. The
exposed `PersistenceError::Publication` now carries Host 0.5's `NamedWriteError`
Rust identity, so the single numbered next draft is 0.7.0. Consumers exchanging
that value must align direct Host dependencies. No schema, digest, budget,
reservation, record format or byte framing changes.

All three official non-yanked registry archives match their declared SHA-256,
published source `db637fac8b7a9ef62301e1d9009ffeb5ffcd0be7` and all 54 remote
committed Rust blobs. The locked metadata graph is admitted offline after explicit
cache preparation. Gzip/Wasm changes affect disabled features. Closed-writer
executable admission has no JSON-record caller; retained descriptor publication
already uses the canonical owner. Bounded test capture now uses Host's child
owner internally but stays direct-child-only. Group signalling is neither escaped
descendant confinement nor completed quiescence; Backup's one-spawn descriptor
inheritance, original custody records and fresh non-spawning guards remain local.
No duplicate cleanup wrapper, production provider or new mode is introduced.

Fresh Linux checks on this selected graph pass all 142 operations cases, four
checksum-model cases, six public framing cases, four public recovery/source/
artifact/upload targets and all 11 real PocketIC journeys. The actual ICP routing
refusal still retains exact pending exhausted spending and local replay; cleanup
does not settle the update. Configured Clippy, Rust 1.91, the compiled public
example, offline locked standalone packaging, formatting, dependency pins, links,
canonical snapshot and simulator
alignment also pass. The retained local 0.6.0-labelled package contains pending
0.7.0 dependency selections, distinct from the actual registry release. Exact new
source/archive/graph/log/simulator evidence is under `target/host-050-review-3`;
prior qualification keeps its original dependency and source scope. Current
consumer native macOS qualification remains independently required. Exact Host
0.5 owner CI passes Linux, Intel/ARM macOS and MSRV. Released Backup 0.6.0
main/tag pass Linux/ARM macOS; Intel macOS remains in progress at final inspection.
The actual registry archive confirms the published checksum API; this does not
qualify the pending graph. No sibling edit, commit, package version change,
release, push, upload or public IC effect runs.

See [the current Host review](../ic-host-tools-adoption.json) for package proof,
feature and compatibility decisions, and [the PocketIC scope](../pocketic-qualification.md)
for the limited fixture guarantees.

## Earlier 0.6.0 typed JSON publication and checked directory framing

The complete pending batch now selects 0.6.0: adding the exhaustive public
`PersistenceError::Publication` variant changes JSON writer error matching. The
root/member manifests, lock and release receipt remain 0.5.4; no version or Git
transaction runs. Update exhaustive matches and handle Host's typed producer,
cleanup and before/after-publication evidence without inferring a paid retry.
All maintained v1 bytes, budgets, reservations, obligations and references remain
unchanged. The additive checksum API and reviewed Shared Tooling 0.1.21 adoption
below carry into this same minor draft.

JSON create/replace now use the published Host 0.4.6 descriptor publisher beneath
one held syncable parent. The local temporary allocator and hard-link/rename/
cleanup engine are removed. Backup retains 0700 parent creation/link syncs,
0600 files, serializer preflight and crash acknowledgment policy. One encoded
buffer remains required for generic Serialize to run once before filesystem
effects; direct JSON streaming cannot preserve that contract. Production and
crash fixtures use the same adapter/Host engine. A synchronized producer barrier
precedes publication; successful Host completion precedes the durable barrier.

Fresh Linux checks pass: all 129 affected persistence cases, including actual
child death for attempts/downloads/references/manifests/settlement, original
size bounds, private modes, typed native conflicts and changed staging cleanup.
New cases prove serialization once before parents even on failure, retained
original/foreign staging with cleanup evidence, and held-parent publication
after its path moves. The six public framing cases, four public recovery/source/
artifact/upload targets, all 11 real PocketIC journeys, Clippy, Rust 1.91 and
offline locked standalone packaging also pass. Exact source/log/archive and
simulator proof is retained at `target/publication-060`; the local archive remains
version 0.5.4 and is not a published registry artifact. Earlier runs keep their
original identities. No current native macOS result exists for this dirty source.

ICP's latest release remains 1.6.0. Remotely inspected main
`08201a6382559d8eac851509f7984cfe55d0620d` still omits effective routing from generic
calls and signed destinations. Its existing Call/RouteTo owner already supports
the route: [upstream feedback](https://github.com/dfinity/icp-cli/issues/811#issuecomment-6044188605)
proposes an explicit option carried consistently through online/signing/request
status. Fresh actual probe refusal retains pending exhausted original spending.
Routing, bounded internal retry/polling, authenticated association and command
custody remain separate production-transport prerequisites; no unqualified
provider or sibling file edit is introduced. Publication/native delivery and
downstream Canic adoption also remain separate from local implementation.

Current implementation handoffs are posted on
[#24](https://github.com/dragginzgame/ic-backup/issues/24#issuecomment-6044506701),
[#25](https://github.com/dragginzgame/ic-backup/issues/25#issuecomment-6044507064)
and [#26](https://github.com/dragginzgame/ic-backup/issues/26#issuecomment-6044507408).
The [Canic API handoff](https://github.com/dragginzgame/canic/issues/490#issuecomment-6044312449)
requires published/native evidence before separately qualifying downstream custody.
Final inspection confirms both released 0.5.4 main and tag CI succeeded on Linux,
Intel macOS and ARM macOS at the exact incoming commit. These runs do not qualify
the current dirty 0.6.0 draft. All three Backup issues remain open for their stated
delivery, retained-buffer or production-transport requirements; no PR is open.

## Earlier 0.5.5 local framing/tooling qualification, carried into 0.6.0

Released 0.5.4 is the incoming base, `bebd3b5185c8144b5be97e9a9763fee9f0621c6c`;
the official registry index confirms its non-yanked publication. Package/receipt
remain 0.5.4, with one additive 0.5.5 draft. Both manifests and the incoming lock
are byte-preserved: host artifacts/fs/process remain 0.4.6 and Metrics 0.2.8.

The existing `ops::artifacts::checksum_relative_files` is now public and checked.
One I/O-free owner admits exact canonical relative UTF-8 file identities, rejects
malformed/duplicate rows through `DirectoryChecksumError`, and preserves original
PathBuf component ordering and path/NUL/lowercase-digest/newline framing. All
local traversal, durable publication and IC-tree callers use this same owner.
Existing exhaustive `ArtifactError` variants remain unchanged; filesystem callers
retain the non-UTF-8 variant or an InvalidData IO error with the typed cause. No
existing function, method or type is removed. Declared hashes prove no custody,
completeness, synchronization or publication. See [the boundary](../extraction-boundary.md).

The 75-file Shared Tooling 0.1.21 snapshot selects remotely verified committed
main `45e34e92b43edb9543d5b7212774f87f8334079f`. A clean exact-source canonical
export retains all 73 previous paths and adds the optional PR helper/fixture;
all bytes/modes are checked. Consumer release delivery remains explicitly direct.
Patch/minor/major/resume reject unsupported selections before runner state or Git;
fresh merged-source receipt qualification for PR releases is not implemented.
Ordinary contribution PR authority remains distinct. See [the review](../shared-tooling-review.json).

Fresh Linux checks pass: six public framing cases with independent golden bytes,
affected artifact/publication/IC-tree units, four public recovery/source/upload
targets, all 11 real PocketIC journeys, configured Clippy, Rust 1.91, offline
locked standalone packaging, tooling/release/hook/shell and final quality checks.
Exact source, logs, simulator results and local archive proof remain under
`target/continuation-055`. The locally verified archive retains package version
0.5.4 and contains this unpublished draft; it is not the published 0.5.4 archive.
The #26 follow-up adds public composition-through-publication/recovery proof,
including changed staging/canonical byte refusal, and a compiled usage example.
Fresh focused checks and the updated local archive are bound separately under
`target/issue26-followup`; only tests and rustdoc changed, so earlier runtime
journeys retain their original source evidence without an unnecessary rerun.
[PocketIC evidence](../pocketic-qualification.json) keeps older runs under their
original inputs and records this fresh run separately.

Released 0.5.4 exact-source main CI now passes Linux, Intel macOS and Apple Silicon
macOS, completing the previously pending product native delivery. The tag run
passes Linux/Intel with Apple Silicon still queued at this inspection. This
uncommitted batch has no remote CI result. Shared 0.1.21
owner CI passes Linux, Apple Silicon macOS and lint/security; Intel macOS was
cancelled before complete portable qualification. No all-host green gate is
claimed. Contribution-policy and original region-custody/transfer-read adoption
are delivered; streamed JSON publication migration and production transport retain
their own acceptance. Canic framing adoption awaits the actual published API and downstream
qualification. No sibling edit, commit, release/version transaction, push,
publication, public IC call, CI rerun or retained-evidence cleanup occurs.

## Earlier 0.5.4 published host 0.4.6 qualification

Published host 0.4.6 (`0fb05f9e18f032425188d68e1d69317a0f0127d5`) now has
successful exact-source Linux, Intel macOS, Apple Silicon macOS and MSRV owner CI.
Both upstream #18/#19 are closed. The filename fixture observes native rename/
hard-link admission, preserves exact names and typed EILSEQ/cleanup outcomes,
and correctly expects the producer before filesystem-specific publication refusal.
The earlier upstream blockers below are resolved by this release. See
[the host review](../ic-host-tools-adoption.json).

The incoming lock already selected artifacts/fs/process 0.4.6 and is preserved
byte-for-byte, along with both manifests. All three archives/official non-yanked
index checksums and every Rust file match the exact publisher. Production behavior
is unchanged from 0.4.5: artifacts/process Rust is identical, and fs changes only
stream documentation/native tests. Fs's existing compatible minimum 0.4.5 continues
to require its compile repair; all host features remain empty. Shared Tooling
stays the reviewed 0.1.20 snapshot. Metrics stays 0.2.8.

Fresh affected Linux artifact/JSON/IC artifact units, four public recovery/source/
upload targets, configured Clippy, Rust 1.91 and offline locked standalone package
verification pass. Exact source, case/log hashes, source/index/CI responses and
archive evidence stay at `target/host-054-followup`. Product/test Rust and WAT are
unchanged. Earlier 11-case real PocketIC proof retains its actual 0.4.5 graph;
it is not relabelled or rerun for a dependency tests/documentation-only patch.

Current Backup native delivery still requires its own exact-source qualification;
upstream owner CI supplies no consumer execution proof. Publication adoption #24
now records successful host native admission and retains its serializer/private-
parent/crash-barrier migration scope. Checked directory framing #26 and production
transport remain separate work. Package/receipt stay 0.5.3 with one 0.5.4 draft.
No sibling edit, commit, release/version transaction, push, publication, public IC
call, CI rerun or evidence cleanup runs.

## Earlier 0.5.4 Shared Tooling 0.1.20 and host 0.4.5 refresh

The latest remotely verified committed Shared Tooling 0.1.20 snapshot is adopted
at `3ecc48e579f6cf6e6ab01a6645d8a250fc8c6934`. All 68 prior paths remain; five
explicit additions supply contribution rules, the shared IC pin parser and
PocketIC alignment/binary helpers plus their fixture. Exact blobs/modes verify.
Contribution guidance uses scoped explicit PR/commit/release authority; ordinary
work stays uncommitted. No commit, PR, push, version or release is requested or
performed here. CONTRIBUTING and local guides use the same reviewed rules.

The configured gate checks the actual locked PocketIC 16.0.0 client/server pair
after dependency preparation through read-only offline metadata. Canonical tooling,
release/hook/shell and optional checker fixtures pass on Linux, including actual
trailing-slash and symlinked temporary-path contexts. Expanded installer fixtures
retain their substitute scope. Exact 0.1.20 upstream CI passes Linux, both native
macOS architectures and lint/security. Shared #54/#55/#56 are resolved upstream;
consumer delivery remains independent. See [the tooling review](../shared-tooling-review.json).

A final registry/upstream review found published host 0.4.5 at
`93a905b048bcaa2a0aed4214ac2f13f065dc2905`. All three selected registry archives
and every published Rust source match that exact committed publisher. The lock
selects artifacts/fs/process 0.4.5; fs's compatible minimum requires the fixed
release. All other incoming lock entries remain exact, including Metrics 0.2.8.
An incidental Cargo update re-resolution of Windows edges is retained separately
rather than adopted. Explicit offline cache preparation and locked metadata pass.

**The Darwin compile defect is fixed; native delivery is still unqualified.**
Published 0.4.5 uses checked u16 permission admission on Apple, retaining original
bounds and typed before-publication failures. Exact upstream CI compiles and runs
fs tests on both macOS architectures, resolving [host #18](https://github.com/dragginzgame/ic-host-tooling/issues/18).
Both native jobs then pass 57 fs tests and fail the unconditional non-UTF-8 filename
success fixture with EILSEQ. [Host #19](https://github.com/dragginzgame/ic-host-tooling/issues/19)
already owns filesystem-specific fixture qualification; no duplicate issue or
sibling patch is created. Linux/MSRV pass upstream. These results do not establish
native macOS acceptance of this uncommitted consumer draft.

Fresh affected Linux artifact/JSON/IC artifact units, four public source/recovery/
upload targets, all 11 registered real PocketIC journeys, configured Clippy and
Rust 1.91, offline locked standalone packaging and final formatting/declaration/
link/snapshot/alignment checks pass on the reviewed 0.4.5 graph. Evidence stays under
`target/upstream-054-refresh/host-045`; earlier 0.4.3 checks and their original source
hashes remain retained under the parent owner. The new gzip helpers are disabled
and have no current caller. Metrics arithmetic and all product Rust bytes remain
unchanged by this refresh. See [host adoption](../ic-host-tools-adoption.json) and
[exact PocketIC scope](../pocketic-qualification.json).

Typed streamed publication is committed and published, but [consumer #24](https://github.com/dragginzgame/ic-backup/issues/24)
still requires a plan preserving one-pass generic serialization before filesystem
effects, private parent policy and deterministic journal crash barriers, plus native
qualification. The existing publisher stays its sole owner. [Consumer #23](https://github.com/dragginzgame/ic-backup/issues/23)
tracks delivery of locally implemented contribution rules. #21/#22 retain original
native-delivery scopes; #25 and ICP CLI #811 own the production routing/transport gap.
No open Backup PR duplicates this work. New [consumer #26](https://github.com/dragginzgame/ic-backup/issues/26)
requests a checked pure directory-checksum composition API for Canic-owned descriptor
publication; its ownership/compatibility review is recorded, implementation pending.
Matching issues receive evidence rather than duplicate reports or premature closure.

Original incoming files, failed/inconclusive attempts, official index responses,
source/archive proofs and native logs remain retained. The initial crates.io API
refusal is resolved through the official index. Released 0.5.3 main/tag CI passes
all three configured hosts on its original graph; it does not qualify this draft.
Package/receipt remain 0.5.3 and the single draft is 0.5.4. No sibling edit, commit,
release, public IC call, publication, workflow dispatch or evidence cleanup runs.
Full application/Canic/terminal qualification is unchanged.

## Earlier 0.5.4 pinned transport capability review

The compatible draft now has a real pinned ICP CLI capability probe in the
PocketIC target. Published `ic-host-process` 0.4.2 is added only for Unix tests:
it owns executable checksum/version admission, direct argv execution, typed exit
failures, 64 KiB captures and a 60-second deadline. The CLI's portable `ICP_HOME`
override isolates config/data/cache under one retained private directory with a
cleared environment. No production process implementation or transport package
is installed.

Actual ICP 1.6.0 (`18435e1747162447fca231548b74d97e4cf48888`) fails an exact original
binary management stop call on a real owned PocketIC 16 HTTP gateway with HTTP 400
`canister_not_found`. Its generic path cannot retain the effective target distinct
from receiver `aaaaa-aa`. A dedicated status control succeeds on the same endpoint,
trusted root and actual anonymous controller; independent post-call status stays
Running. Original plan/arguments and the reservation precede dispatch. Typed exit
failure leaves unchanged pending/exhausted spending across new-reservation denial
and reopen, with no receipt or reissue. [ICP CLI #811](https://github.com/dfinity/icp-cli/issues/811)
owns the routing fix; [consumer #25](https://github.com/dragginzgame/ic-backup/issues/25)
owns separate backend acceptance. See [the exact scope](../pocketic-qualification.md#pinned-cli-capability-probe).

The incoming graph now selects published host artifacts/fs 0.4.2 and Metrics
0.2.8. Those selections are preserved; only the test-only process entry/edge is
added. Every selected archive checksum and Rust file matches its exact committed
release. New host chunk hashing has no incremental IC coverage caller; retain the
existing copy conversion's original non-IO cause identity. Metrics Rust is unchanged
from 0.2.7. Earlier reviews below retain their actual older source/graphs and are
not current qualification. Current package reviews are
[host](../ic-host-tools-adoption.json) and [Metrics](../ic-metrics-adoption.json).

All registered PocketIC cases, affected artifact/JSON/IC artifact/metrics units and
four public local recovery/source/artifact/upload journeys pass on Linux. Configured
all-target/all-feature Clippy, Rust 1.91 compilation, offline locked packaging,
formatting, dependency declarations, documentation and exact snapshot checks are
retained under `target/icp-backend-054-review`. Initial compile/style failures and
pre-portable-directory probe evidence remain retained. Prior successful checks
are not relabelled as the final source. Host 0.4.2 upstream Linux/MSRV pass; both
macOS jobs still stop at the shared command/TMPDIR fixture before Rust tests.
Metrics 0.2.8 upstream passes all configured hosts. Neither result supplies native
macOS delivery of this uncommitted draft.

Shared Tooling remains the exact reviewed 0.1.18 snapshot. Workspace/package and
receipt remain 0.5.3. No commit, release/version transaction, public IC call, push,
publication, sibling edit or evidence cleanup ran. Production transport still needs
explicit routing, authenticated per-call association, internal retry/polling bounds,
never-dispatched command/descendant custody and interrupted-request qualification.
Application/Canic adapters, executable workflows and terminal/fence/reference release
remain independent requirements before Canic engine replacement.

## Earlier 0.5.4 original snapshot transfer-read qualification

The same compatible 0.5.4 draft now adds the missing ordinary metadata/data read
execution boundary. It borrows existing payloads under the full original plan and
already pending replicated-update reservation. Passive response matching rechecks
exact current authority/attempt/context/target, then delegates to existing codecs.
The existing mutation lane accounts for semantic reads; recovery observations
retain their separate owner. No new counter, persisted schema, hash encoder or
automatic receipt is added. Metadata/data/ID/argument/decoder bounds remain exact.
See [the implemented boundary](../extraction-boundary.md#originally-reserved-snapshot-transfer-reads)
and [issue #22](https://github.com/dragginzgame/ic-backup/issues/22).

The real PocketIC driver now uses this public boundary for all ordinary reads
before explicit isolated-instance success qualification and journal recording.
Actual metadata/data reply-discard cases stop before download or preserve Created
staging and source references, with pending exhausted update accounting, typed
new-reservation refusal and no calls during reopen. Original complete-state and
lost-effect journeys pass on Linux. This is post-effect reply discard, not a
network/process fault or production transport. Fresh permissions, authenticated
snapshot/metadata custody and never-dispatched command proof remain integration-owned.
Production transport/runners, full workflow/terminal admission and Canic adaptation
remain necessary before engine replacement.

Focused read association cases, configured all-target/all-feature Clippy and Rust
1.91 compilation pass. Evidence is under `target/transfer-read-054-review`, including
actual retained simulator originals, traces, raw bytes and safe-stop results.
The initial wrong fixture sequence, sandbox loopback refusal and two Clippy style
failures remain retained. Locked package, documentation, formatting, pins and exact
snapshot checks are recorded there. Native 0.5.4 delivery remains independent;
no old native proof is relabelled. Workspace/package version and receipt remain
0.5.3, incoming lock bytes are preserved, and no commits, release transactions,
sibling writes or evidence cleanup run.

## Earlier 0.5.4 retained region-custody qualification

The maintainer reports that 0.5.3 was pushed accidentally. Its release at
`7660b56c196d2af3d084524bc74bb9dd4982d1ad` contains metadata and handoff changes,
with no product Rust/tooling changes from 0.5.2. Its historical version/date remain;
the changelog now explains the accident under explicit maintainer authorization.
The single next draft is 0.5.4. Workspace/package version and released receipt
remain 0.5.3; no release transaction or commit ran.

The compatible draft stops snapshot artifact appends after detectable original
region-file drift. Before coverage admission or another write, descriptor-relative
no-follow metadata must match all three held regular files and their exact original
covered lengths. No new coverage counter, buffer, open or v1 record appears;
at most six metadata syscalls are added per append. An error consumes the writer
and preserves partial bytes, occupied staging and unchanged Created journal state.
Sequential checks do not fence noncooperating writes or detect same-length byte
changes; final fresh checksum verification remains required. See
[the boundary](../extraction-boundary.md#durable-metadata-bound-ic-snapshot-artifacts)
and [issue #21](https://github.com/dragginzgame/ic-backup/issues/21).

A real filesystem regression reproduces the accepted unrelated write before the
fix and checks replacement, symlink, directory, missing path, truncation and extension
afterward. Affected artifact/JSON/IC artifact cases and the four public local recovery,
source, artifact and upload journeys pass on Linux, alongside configured Clippy and
Rust 1.91 compilation. Evidence and registered case identities are under
`target/continuation-054-review`; the original regression and initial path-argument
compile failure remain retained. Locked package and focused repository checks are
recorded there. Native delivery of this uncommitted draft remains separate.

The incoming lock selecting published host artifacts/fs 0.4.1 is byte-preserved.
Archive checksums and every Rust source match committed release
`ce2dd57cedc5000b44bb6a9ff5194f7d65a42c38`; separate dirty sibling 0.4.2 work is
excluded. Host named external-tool staging does not replace our retained directory
publication, and its consuming HashingWriter API does not simplify these held
descriptors/hash lifetimes. The current [host review](../ic-host-tools-adoption.json)
retains this scope and consumer evidence. Metrics remains the incoming 0.2.7 selection.

Host 0.4.1 upstream Linux/MSRV pass, but both macOS gates stop at a shared command
fixture before Rust tests. [Shared Tooling #56](https://github.com/dragginzgame/shared-tooling/issues/56)
already owns the exact failure and proposed physical TMPDIR normalization. This is
not a Rust failure or native qualification. The adopted Shared Tooling snapshot
remains exact 0.1.18, `a3430b34b32a60f3b245a2b4f7e2f5321556fe56`; no dirty shared
source or vendored patch is adopted.


## Released 0.5.2 native delivery review

Release 0.5.2 is pushed at `e1300abfbaba47d5776a71b5bc1fa005d9079e1c` with tag `v0.5.2`.
Its [main](https://github.com/dragginzgame/ic-backup/actions/runs/37607137734) and [tag](https://github.com/dragginzgame/ic-backup/actions/runs/37607138196) CI runs pass the full configured gate on Linux.
Both Intel and Apple Silicon macOS jobs now pass. Exact completed run/job
responses are retained in `target/continuation-054-review/released-052-ci*.json`.
The selected metrics, release-fixture, shared snapshot and host dependency fixes
are included in that release and their original delivery acceptance is complete.
That source/graph proof does not qualify the 0.5.4 draft above.

The release's parent is `4a1a88bcb2551f821d41ff0b05425a82a327537c`.
Finalization changes only the root Cargo version/lock, changelog date and receipt;
all product Rust, tests, shared snapshot, local tooling and CI are identical to
that committed prepared source. Fresh job/source evidence is retained under
`target/issue-fixes-052-review`. Earlier local target review directories are no
longer present in this checkout; the previous prose below retains their historical
qualification and is not fresh native proof. No new failure, open PR or code repair
is identified. This review creates no commit, version transaction, push, workflow
dispatch, publication or cleanup.

## Retained 0.5.2 local diagnostics and Shared Tooling qualification

Release 0.5.1 is pushed at `0441940772d142400bae17ce60f675a1ef79b739`.
Both exact-source [branch](https://github.com/dragginzgame/ic-backup/actions/runs/37597006011)
and [tag](https://github.com/dragginzgame/ic-backup/actions/runs/37597005993) workflows
now pass Linux, Apple Silicon and Intel macOS. Delivered #13/#14/#16/#17 are
closed under their original scopes; those results do not qualify this draft.
Earlier qualification below retains its original source/dependency identities.

The compatible 0.5.2 draft adds one shared prepared-chunk size histogram to
per-guard diagnostics. Inclusive bounds are zero, 32 KiB, 256 KiB and the 1 MiB
payload ceiling; failure exclusion, zero/repeated samples and empty reopen remain.
The existing summary getter derives from the histogram rather than maintaining
another byte aggregate. Six duration summaries and all v1 accounting/recovery
owners remain intact. Fixed storage grows by 72 bytes, with at most four bound
comparisons and no new clock/allocation/remote call. See
[the package review](../ic-metrics-adoption.json) and
[issue #15](https://github.com/dragginzgame/ic-backup/issues/15).

The incoming published ic-metrics 0.2.7 and host artifacts/fs 0.4.0 selections
are preserved. Each archive checksum matches the incoming lock and every Rust
source matches its clean exact release; Metrics Rust is unchanged from 0.2.6.
The Metrics minimum remains 0.2.4, the first released histogram API.

The host review replaces duplicate IO error mapping with the shared conversion
in raw checksum/copy, bounded JSON reads and exact IC checksum/upload readers.
Native IO identity, typed record size/file-shape errors and original wrapper
identity remain intact. New upstream readers do not change our maintained required
no-follow record read. Shared path locks wait and create parents; our nonblocking
locks, existing-file acquisition and descendant custody stay local. Generic shared
publication and process capture do not replace private crash barriers or owned
command-descriptor inheritance. No function, method or type was removed.
See [the current host review](../ic-host-tools-adoption.json) and
[issue #20](https://github.com/dragginzgame/ic-backup/issues/20). All registered
focused artifact/JSON/IC artifact/metrics cases and four public recovery/source/
upload journeys pass on Linux. Configured Clippy, MSRV and locked package checks
qualify the current graph; native delivery of this draft remains pending.
Evidence is under `target/ic-host-052-review`; the initial missing test import
failure is retained and tests now import their own IO module explicitly.

The draft now adopts the exact 68-file Shared Tooling 0.1.18 snapshot at
`a3430b34b32a60f3b245a2b4f7e2f5321556fe56`. Remote main matches; isolated
committed export and every file/mode verify. The reusable tooling-LOC test now
uses adopted working bytes while committed exporter proof stays upstream,
addressing [Shared Tooling #50](https://github.com/dragginzgame/shared-tooling/issues/50).
Both LOC fixtures pass before commit with enclosing Git/Cargo context and inherited
target selection; the obsolete local `env -u` workaround is removed. Earlier
0.1.16/0.1.17 failures retain their real inputs and results.

The current logger rejects Make options/assignments before gate dispatch, preserves
Make failure status and supports complete retained logs/timings. Changelog finalization
keeps notes with whitespace-bearing draft headings. The current hook/reference and
LOC symlink-output fixes are selected together. Added Cargo tool commands are optional;
only the existing Cargo-sort formatter is required by this library. A substitute-Cargo
installer fixture qualifies command wiring and retained failures, not actual tool/native
installation. The new linked audit addendum closes the governance snapshot.

Actual consumer tooling, release/runner, hook and shell checks pass on Linux.
Formatting, pins, documentation links, workflow lint and exact 68-file snapshot
verification pass. Both LOC cases also pass under this actual enclosing Cargo
workspace with inherited target selection and checkout-contained TMPDIR. Results and exact
unchanged product hashes are under `target/shared-tooling-052-0118-review`;
[the maintained review](../shared-tooling-review.json) records their scope. GitHub
issue review verifies upstream 0.1.18 Linux, native Intel/ARM macOS and lint/security
CI now passes; exact job evidence is under `target/gh-issues-052-review`.
Native delivery of this complete draft
remains independent; see [issue #19](https://github.com/dragginzgame/ic-backup/issues/19).
No shared helper is patched in place and no product Rust, Cargo selection or receipt
is changed by the refresh.

The same 0.5.2 draft now converges release-fixture assertion ownership. Five
runner-only helper functions and their nine generic cases are removed after a
complete function/coverage map. The unchanged canonical suite still owns all
increments, phase order, lost replies, destination custody and fresh-source retry.
One actual Make-to-adapter recovery now checks exact receipt/validation bytes,
original completed version and preserved history/cache. Local lock/metadata failures,
selected older-commit proofs, real index boundaries and publication cases remain.
The retained Git substitute is required by immutable product file-tree evidence;
unused lost-tag/push injection branches are removed without a new mock framework.
See [the ownership map](../release-fixture-ownership.json) and
[issue #18](https://github.com/dragginzgame/ic-backup/issues/18). Focused release,
shell and documentation checks pass on Linux; actual registered local cases
are 69 before and 60 after, with exactly the mapped nine cases removed. The
unchanged canonical suite passes. Source cloc is 770 to 717 shell code lines,
not a performance or production-size claim. Product Rust/manifests/lock/released
receipt and selected snapshot remain unchanged by this cleanup. Native delivery
of the uncommitted batch is pending; evidence is under
`target/release-fixture-052-review`.

Focused native Linux metrics and public guarded upload cases pass, including
deliberate diagnostic-mutex poison through returned successes/failures and empty
reopen. Duration clamping, exact inclusive bounds, overflow routing and independently
saturated total/distribution remain descriptive. All-target/all-feature Clippy and
Rust 1.91 compilation plus the registered unit cases pass. Locked offline packaging,
formatting, dependency inheritance, links and exact snapshot checks pass. Canonical
tooling, actual consumer release/recovery, shared runner, hooks and shell checks pass
on their selected snapshots. Initial oversized-value Clippy and 0.1.16 fixture failures remain
retained; the helper borrows the view and no shared script is patched.

Evidence is under `target/metrics-histogram-052-review` and
`target/shared-tooling-052-0116-review`; prior 0.1.15 evidence remains intact.
The earlier metrics review inspected Metrics 0.2.6 and host artifacts/fs 0.3.3;
its evidence retains those historical selections. Current host and Metrics review
above qualifies the incoming 0.4.0/0.2.7 graph without adding host-process. Native
delivery of this uncommitted draft remains pending. Workspace version and released
receipt remain 0.5.1, and the maintainer's incoming lock is unchanged.
No commits, release effects, broad gate, sibling writes or cleanup ran.
Native delivery of the fixture convergence and installed transport/Canic/application/terminal
qualification remain independent. Review and commit the complete draft, including
this handoff, before release.

## Pending 0.5.1 Clippy correction

The maintainer's configured all-target/all-feature Clippy check exposed five
implicit-reference-to-raw-pointer conversions in lifecycle/capture settlement
unit assertions. The prior PocketIC-target Clippy pass did not cover library
unit tests. Use explicit `&raw const` addresses while preserving exact borrowed
identity checks; no lint suppression or runtime behavior change is introduced.
Fresh `make clippy` now passes all configured targets/features, and all registered
`policy::ic_observation` unit cases pass. Original failure logs, original/corrected
test hashes and fresh check output are retained under
`target/clippy-settlement-051-review`. Other 0.5.1 work remains intact; no version,
release, dependency or sibling edits were made.

## Pending 0.5.1 real single-canister qualification

The accepted integration direction now has an executable test-only PocketIC 16
journey through production codecs, original journals, streamed durable artifact
publication, manifest replay and exact source-bound upload preparers. It checks
complete uploaded bytes/metadata and restores the same canister's heap/global/stable/
certified state and nonempty chunk store after deliberate pre-load changes. A
separately accounted fresh snapshot verifies every byte and restorable metadata
while stopped before restart. Current controller/status checks precede lifecycle
mutations; metadata/status equality never supplies original attribution.

All registered normal and deliberately discarded capture/allocation/data/stop/load/
start reply cases pass on Linux. Qualified load settlement additionally requires
that complete stopped-state check before the explicit existing receipt. Losing the
reserved load-status reply stops safely with both attempts pending: no receipt,
fresh verification capture, retry or restart. Exact pending denials/reopen retain
journal bytes and exhausted allowances. Every other original journal and source
manifest/reference is replayed unchanged without remote calls. Normal uses 53
accounted management ingresses; each settled discard case uses 54 and the pending
load/status case stops at 41, all below the 64-call ceiling. Fixture setup,
chunk-store changes/assertions and application operations are explicit fixture
setup/assertions outside those backup/restore journals.

This is real platform evidence for one inspected no-external-effects fixture,
not a production transport, runner/CLI or Canic adapter. Exclusive simulator custody
qualifies original-request attribution; cardinality/byte equality alone cannot.
Active timer/hook execution, arbitrary metadata configurations, multiple canisters,
process/network failures, application-specific lifecycle recovery/fencing, installed
consumer and full product terminal/reference release remain separate. See
[the exact scope](../pocketic-qualification.md),
[machine review](../pocketic-qualification.json) and
[issue #17](https://github.com/dragginzgame/ic-backup/issues/17).

Fresh real journeys, selected observation/lifecycle/attempt unit and public
recovery regressions, warning-denied Clippy and Rust 1.91 checks pass. Final
formatting, inheritance/pins, local links and the unchanged 58-file shared snapshot
also pass. Original/final sources, registered
cases, tool/dependency identities, actual per-call results and retained initial
private-import/fixture-binding and lint findings are under
`target/pocketic-lifecycle-051-review`. The receipt/reopen assertions have a separate
fixture owner after the initial function-length finding. Prior backend evidence
remains under `target/pocketic-journey-051-review`, with its original source/graph;
all fixture roots stay retained. Native macOS qualification awaits delivered CI.
Production Rust bytes, dependencies, release receipt and package versions remain
exact from the preceding batch. The test-only client still requires internal
`thiserror` 2.0.18; public management 0.11.0 identity is unchanged. The compatible
0.5.1 draft remains uncommitted, with no release or sibling edits. The older
contract-only sections below retain their separately qualified evidence.

## Pending 0.5.1 original capture settlement contract

The accepted follow-up adds passive original capture outcome claims under an
already reserved successful exact list observation. Pure admission reuses the
original full plan/authority/capture bytes, current reservations, existing finite
inventory decoder/digest and canonical full-baseline comparison. Every retained
ID/timestamp/size stays unchanged. Applied explicitly names an independently
attributed new bounded raw ID; zero/one/many candidates infer no outcome. Negative
evidence excludes transient capture/deletion, and actual settled uncertainty is
distinct from a lost observation reply. Lifecycle/status lanes reject.

The borrowed view authenticates no evidence and writes no receipt. A qualified
integration explicitly invokes the existing journal owner; all original consumption,
obligations and references remain retained. Baseline chronology, actual attribution,
fresh permissions/custody, consistency/transfer/backend qualification and terminal
or release admission remain integration-owned. No new provider, schema, hash recipe
or accounting owner is installed. See
[the contract](../contracts/ic-capture-settlement.json),
[committed source review](../ic-capture-settlement-source.json) and
[issue #16](https://github.com/dragginzgame/ic-backup/issues/16).

Fresh focused Linux observation/capture/lifecycle and closed-baseline unit cases
and the public local recovery journey pass, including all claimed capture outcomes,
exact typed denials, changed raw ordering/evidence, baseline drift and spent
allowances through explicit receipt recording/reopen. Selected Clippy, warning-free
API docs and Rust 1.91 checks pass. Final formatting, dependency inheritance,
documentation links and snapshot checks also pass. Original/final hashes, exact registered cases,
commands and retained initial Clippy fixture finding are under
`target/capture-settlement-051-review`. A method reference replaces the initial
redundant test closure without production changes or suppression.

This compatible addition extends the existing 0.5.1 draft and preserves the prior
custody/lifecycle production bytes. The public fixture reuses one explicit journal
receipt/reopen assertion owner for both claims. Dependencies, package versions,
lockfile and released receipt remain 0.5.0. Work is uncommitted, with no current
native macOS result, executable backend, release or live effects.

## Pending 0.5.1 original lifecycle settlement contract

The accepted follow-up adds passive original stop/start/load outcome claims under
an already reserved successful exact status observation. Pure admission reuses
full original authority/mutation bytes, current journal reservation association,
existing finite status decoder/digest, caller challenge and opaque observation
evidence. Capture and inventory lanes reject. Status/controller/code equality
supplies no attribution or outcome; a load claim independently qualifies exact
original restored state. Negative proof excludes transient application; settled
uncertainty cannot be substituted for a lost/unavailable/malformed read reply.

The read-only view performs no IO, provider call, serialization, journal transition,
retry or refund. Only qualified integration code explicitly records the existing
receipt; uncertainty clears only the observation, keeping mutation and all consumed
allowances. Current execution/lifecycle/application safety and permission admission
remain independent. No provider, schema, hash recipe or spending owner is added.
See [the contract](../contracts/ic-lifecycle-settlement.json),
[committed source review](../ic-lifecycle-settlement-source.json) and
[issue #14](https://github.com/dragginzgame/ic-backup/issues/14).

Fresh Linux exact registered observation/settlement cases and public local recovery
pass, including all method/status/claimed outcome lanes, typed identity/evidence
rejection, changed skipped raw metadata, stale attempts, explicit receipts and
exhausted spending/references/obligations through reopen. Selected Clippy, API docs
with warnings denied and Rust 1.91 checks pass. Final formatting, dependency
inheritance, local documentation links and snapshot verification also pass.
Original/final inputs, registered
cases, exact commands and logs remain under `target/lifecycle-settlement-051-review`.
The initial Clippy length finding split passive fixture construction from actual
journal/reopen assertions; no production behavior changed for that correction.

This compatible addition extends the existing 0.5.1 draft and preserves all prior
custody repair bytes. Package versions, dependencies, lockfile and released receipt
remain 0.5.0. New claims/views authenticate no evidence and qualify no actual IC
backend, full restoration, terminal proof or release authority. Current native CI
and delivery remain pending; work is uncommitted, with no release or live effects.

## Pending 0.5.1 command-quiescence release repair

The maintainer released 0.5.0 at `eece44dac79da1bfb36f82ce2cd30d51edd3a9c1`.
The workspace, lockfile, finalized changelog and receipt now select 0.5.0.
Its [main CI](https://github.com/dragginzgame/ic-backup/actions/runs/37583420007) and
[tag CI](https://github.com/dragginzgame/ic-backup/actions/runs/37583420151) now pass
all full Linux, ARM macOS and Intel macOS gates at that exact source. The delivered
fixture ownership and split artifact/digest-ingress changes now have native
qualification in [issue #10](https://github.com/dragginzgame/ic-backup/issues/10) and
[issue #11](https://github.com/dragginzgame/ic-backup/issues/11).
The explicit system Bash 3.2 tooling suites also pass on both native macOS hosts,
qualifying delivered Shared Tooling adoption in
[issue #12](https://github.com/dragginzgame/ic-backup/issues/12).
These released-source results do not qualify the uncommitted 0.5.1 changes or
establish the historical custody failure's precise cause.
Shared Tooling is current
at committed 0.1.13, `e378671d90afa237ff63a4b0e3b9551eb2c222b6`.

The next accepted batch addresses [issue #13](https://github.com/dragginzgame/ic-backup/issues/13).
A deterministic Linux regression retains a duplicate of a non-spawning quiescence
guard's descriptor. Before the change, dropping the guard leaves fresh acquisition
`InFlight` until that copy closes. Explicit unlock on quiescence guard drop now
releases its exclusion while preserving the exact retained sidecar/record. Closing
the old copy after reacquisition cannot release the fresh guard's lock. Dispatched
owner custody continues to close without unlocking, keeping child/descendant
holders authoritative. No retries, grace changes or evidence cleanup hide contention.

[Apple's flock documentation](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/flock.2.html)
supports the shared descriptor-copy mechanism; an unrelated parallel pre-exec fork in the historical macOS failure
remains an inference, not an observed descriptor trace. The new red/green regression
qualifies the demonstrated release defect on Linux. All registered custody cases
(including real owner death/descendants and bounded finish) and public local recovery
pass, with selected Clippy warnings denied, warning-free API docs and Rust 1.91
checks. Final formatting, dependency inheritance, documentation and snapshot
checks also pass. Initial failing evidence and final commands/case registry/inputs remain in
`target/command-quiescence-051-review`. The initial Clippy finding required naming
the now-active private descriptor field without its former underscore prefix.

The compatible fix selects one undated 0.5.1 draft. Package versions, dependencies,
lockfile, receipt, APIs and v1 schemas remain released 0.5.0. This uncommitted repair
has no native macOS CI result; #13 stays open until exact-source qualification.
No commits, tags, pushes, releases, publication, sibling edits or live IC effects.
Full product/backend/application qualification remains independent.

## Retained pre-release 0.5.0 Shared Tooling 0.1.13 workspace-rule adoption

The 58-file snapshot now binds committed Shared Tooling 0.1.13 at
`e378671d90afa237ff63a4b0e3b9551eb2c222b6`, matching remote main.
The previously pending workspace rule is committed and adopted with all linked
baseline/dependency/hook/roster changes. Its exact file is the only selected
addition. IC Backup already conforms: a virtual root with explicit resolver 3,
`crates/ic-backup` as its sole maintained package, inherited metadata/dependencies
and one root lockfile. No paths, identities, dependency graphs or exceptions change.

Original uncommitted 0.1.12 snapshot bytes are retained. The canonical exporter
refused overwriting them, so a clean isolated checkout exported into a disposable
consumer. Reconciliation first checked unchanged originals, then copied only exact
reviewed blobs/modes. Locked offline metadata and the maintained-manifest inventory
exclude the actual Cargo target directory and qualify existing layout independently
of the inheritance checker. Exported governance links and integrity regressions,
formatting, dependency inheritance and final local documentation/snapshot checks pass.
See [the current review](../shared-tooling-review.json) and
`target/shared-tooling-050-0113-review` for source and focused evidence.

Upstream exact-source CI passes Linux and both native macOS portable-regression
jobs. This uncommitted consumer remains locally qualified only; the separate
released macOS custody failure below is unaffected. Prior 0.1.12 release/hook/shell
qualification keeps its original inputs. This policy-only refresh leaves the
existing 0.5.0 draft, product Rust bytes, manifests, lockfile and 0.4.2 receipt
unchanged. No commits, versions, releases, sibling writes or live effects.

## Retained 0.5.0 Shared Tooling 0.1.12 adoption

That reviewed 57-file snapshot bound committed Shared Tooling 0.1.12 at
`33c2a6f0018a94915f819ff219e270500ed5b73b`, matching remote main. Export uses a
clean isolated committed checkout; dirty sibling Rust-workspace rules and other
uncommitted bytes are excluded. The only reviewed file-set addition is the common
governance roster; all its documents and local links resolve in the exported
consumer. No package relocation is indicated by the current virtual workspace
and `crates/ic-backup` placement.

The snapshot verifier now bootstraps hashing without executing inspected helpers.
Local helper-only and helper-plus-payload rejection checks prove the substituted
helper never executes. The release push substitute follows the shared captured-URL
contract with exact branch/tag refs. Actual shared regression cases cover URL
replacement/addition after validation and before push plus exact retry. Consumer
recovery/source/receipt and separate publication admission remain unchanged.
The new baseline permits relevant cross-repository issue actions; sibling file
edits and release effects retain separate authority.

Fresh Linux tooling, release-adapter, shared-runner, hook and shell checks pass.
Final formatting, dependency inheritance, documentation links and exact committed
snapshot byte/mode checks also pass. Upstream exact-source CI passes Linux and both native macOS portable-regression
jobs. Evidence and prior inputs remain under `target/shared-tooling-050-0112-review`;
see [the reviewed adoption](../shared-tooling-review.json). This batch preserves
all pending product Rust bytes, manifests, lockfile and released 0.4.2 receipt.
It stays within the existing 0.5.0 draft and remains uncommitted.

Released 0.4.2 at `654790f374f9923df9020f4812cec65e47cbe3af` has a successful
[main run](https://github.com/dragginzgame/ic-backup/actions/runs/37501973677) and a
separate failed [tag run](https://github.com/dragginzgame/ic-backup/actions/runs/37501973053).
Both tag macOS jobs fail command-custody reacquisition with `InFlight` after child
reap, `lock.finish()` and guard drop. [Issue #13](https://github.com/dragginzgame/ic-backup/issues/13)
owns diagnosis; this tooling update does not repair or qualify that behavior.
No native CI exists for the uncommitted 0.5.0 batch. No commits, tags, push,
release/publication, live IC effects, sibling writes or recovery-evidence cleanup.

## Retained pre-release 0.5.0 split Host Tooling adoption

Accepted #11 now replaces the monolithic registry dependency with
`ic-host-artifacts` and `ic-host-fs` 0.3.0, with optional archive/compression/Wasm
features disabled. Both archives' hashes and every published Rust source match
committed provenance `efd402e0063ccbbf8a143cc970be52ab41b1766d`. Newer
sibling streaming/matching/gzip work is not imported. Every retained external
lockfile entry remains exact; two split packages are added and the retired
monolith/unused dependency branches are removed. No sibling path patch exists.

Raw byte/digest construction delegates to shared `Sha256Digest`. The redundant
private `hash::sha256_hex` is removed; generic IC wire-byte hex formatting remains.
The accepted follow-up removes `model::artifacts::validate_hash` in favor of ASCII
case normalization plus shared digest parsing, preserving exact v1 fields and
original typed malformed/mismatch errors. Layout locking uses canonical raw digest
formatting and retains exact names/contention. IC child verification replaces
manual `Take` accounting with shared bounded hashing and returned byte-count
admission; short/excess input retains `FileShape`, original IO errors pass through
the same Backup error variant and descriptor/path custody checks remain local.
`checksum_reader` retains its public signature and Backup-owned error/record
types, while delegating to canonical `hash_reader(reader, u64::MAX)`. Its behavior
now retries Interrupted reads internally and rejects impossible counts as typed
InvalidData. Because callers could previously use the first interruption as a
termination signal, the complete pending draft becomes 0.5.0 from released 0.4.2.
Callers must own blocking/timeouts. The existing #10 fixture fix is carried into
that one draft; package versions and released receipt remain 0.4.2.

Existing copy/count/collection calls use Host Artifacts and no-follow record reads
use Host FS. Other original IO errors, JSON limits/decoding, normalized v1 checksum
records, local tree framing, 0700/0600 private staging and crash/publication owners
remain intact. No new provider, transport, runner, schema or release API is added.
Fresh Linux selected checksum/copy/JSON/crash/source tests and public recovery/upload
journeys pass. Exact registered case sets match successful executions; selected-package
Clippy warnings denied, API docs warnings denied and Rust 1.91 checks pass.
The manifest-sorting hook fixture uses the new artifact dependency; its actual
regression, Bash/ShellCheck, formatting, dependency inheritance, document links
and exact 56-path shared snapshot checks pass.
Original inputs/archives/source review, lock delta and exact commands
remain under `target/host-tooling-split-050-review`; earlier fixture qualification
retains its original 0.2 dependency graph below rather than being relabelled.
See [the current dependency review](../ic-host-tools-adoption.json).

Fresh follow-up digest/plan/journal/reference, layout exclusion, IC artifact,
restore-copy and four public recovery/upload targets pass, along with selected-package
Clippy, API docs and Rust 1.91 checks. Exact registered cases, original/final inputs,
commands and logs remain under `target/host-tooling-consolidation-050-review`.
The new lock regression initially omitted creation of its parent (the library
fixture helper intentionally returns an uncreated path); corrected setup and its
retained failure/re-run distinguish fixture correction from product behavior.
No dependencies, package versions, public signatures or retained formats change
in this follow-up; the same 0.5.0 draft remains selected.

The further accepted digest-ingress cleanup gives `model::artifacts::canonical_hash`
crate-internal visibility and routes raw plan/attempt identity and history, inventory,
requirement, download and restore-reference fields to that owner. The two private
`canonical_hash` wrappers in attempt journals and operation plans are removed.
Checksum records remain real record boundaries; no new public API, dependency,
format, validation-order or error-conversion change is introduced. Journal reopen
uses the same normalizer before its existing lock/read admission. Its regression
preserves malformed input, contention, uppercase normalization and exact retained
bytes; focused affected model/persistence and seven public recovery targets pass.
Clippy warnings denied, warning-free API docs and Rust 1.91 checks also pass.
Original/current source, registered cases, commands and logs remain under
`target/digest-ingress-050-review`. Earlier qualification retains its original inputs.

Upstream [provenance feedback](https://github.com/dragginzgame/ic-host-tooling/issues/7)
records that both published archives name the locally inspected commit above,
but GitHub returns HTTP 422 for that revision in their declared repository while
remote main remains `0456b40e116c7d070428b070d1c3befc36a9345d`. Archive hashes and local
source comparison remain verified; no public exact-source CI or native proof is
inferred. No shared API gap was found during this consolidation.

Work stays uncommitted; #10/#11 remain open pending delivery and exact-source native
macOS qualification. No commits, tags, push, release/publication, sibling write,
live IC effect or retained-evidence cleanup. Complete backup/restore and actual
authenticated backend/application qualification remain independent work.

## Retained initial 0.4.3 exclusive public fixture roots

The maintainer has released 0.4.2 at clean session HEAD/tag
`654790f374f9923df9020f4812cec65e47cbe3af`; finalized notes and package/lock/receipt
agree. At the initial fixture inspection, the
[main run](https://github.com/dragginzgame/ic-backup/actions/runs/37501973677)
had passed Linux and ARM macOS while Intel still ran. It has since passed; the
separate tag failure is recorded above. Released 0.4.1's
Linux/Intel full gates pass, while ARM fails the fence-reconciliation fixture
AlreadyExists recorded in [issue #10](https://github.com/dragginzgame/ic-backup/issues/10).
The original colliding path/clock value was not observed; do not claim that its
precise cause was reproduced locally.

Every public integration fixture now reserves a fresh directory before writing
children through one test-only owner in `tests/support/mod.rs`. Atomic directory
creation, a process-local sequence and at most 128 candidates admit exclusive
ownership without a clock. Occupied files/directories/links are retained unchanged;
other errors stop with the attempted path, and successful allocation reports its
path. Existing explicit success cleanup remains; no Drop cleanup is introduced.
Eight path-only wrappers are deleted in favor of this owner. The upload fixture's
root helper remains responsible for its artifact child. Library unit-test path
helpers retain their distinct uncreated-path contract and existing sequence owner.

All affected public integration journeys and new occupied-path, concurrent
allocation and bounded/error cases pass on Linux, with selected-target Clippy
warnings denied and Rust 1.91 checks. Registered membership is derived from Cargo's actual cases, not
a fixed aggregate. Prior source, initial compile/caller migration failures, exact
inputs/commands/registry and final logs remain under `target/fixture-roots-043-review`.
These are local filesystem/accounting tests, not IC effects or proof of native
macOS behavior. Fresh native qualification remains required before #10 closes.

At that batch's inspection, the undated 0.4.3 draft was compatible test tooling;
production source, public APIs/schemas, package/lock versions, dependencies and receipt
remained 0.4.2. Its unchanged fixture work is now carried into the single pending
0.5.0 entry above. The original qualification remains tied to its old dependency
graph. Work stays uncommitted; full backup/restore and provider/backend qualification
remain independent work.

## Retained 0.4.2 Shared Tooling 0.1.11 adoption

That 56-file snapshot bound committed Shared Tooling 0.1.11 at
`46c02774a8335cb3949d6f04284c4f53375353c1`, matching inspected remote main.
The baseline and agent maintenance rule are refreshed together; product and
release authority remain local. Canonical logger/finalizer regressions cover
ordinary Rust test names/context and exact large version comparisons. The
consumer failure-retention fixture now checks the new first logger case's
original input, executed child marker and exact status 31. The initial mismatch
and corrected pass remain under `target/shared-tooling-042-0111-review`.

Both native macOS system Bash 3.2 tooling steps pass for released 0.4.1 at
`04da09a5a919bbf9aa56e71eaacaf245107021d4`. Their exact shared formatter owner
is unchanged by 0.1.11. Four overlapping local hook functions are retired after
that qualification: `test_format_and_retry`, `test_unstaged_protection`,
`test_existing_hooks` and `test_symlink_checkout`. Shared refresh, partial-stage
and installer cases own those checks; `test_formatter_failure` retains the
distinct successful-prerequisite/exact-exit-status boundary locally.

Linux's released full gate passes. ARM's later full gate fails with AlreadyExists
in `fence_reconciliation.rs:97`; [issue #10](https://github.com/dragginzgame/ic-backup/issues/10)
owns that independent fixture finding and proposed exclusive-root allocation.
Its production/Rust repair is outside this tooling batch. Intel's full gate and
0.1.11 upstream CI were still running/queued at inspection. No complete native
gate or CI result for this uncommitted batch is inferred.

The single undated 0.4.2 entry covers compatible internal tooling and fixture
convergence; public API, schemas, package/lock versions and released receipt stay
at 0.4.1. Original dirty documentation, snapshot/review, hook source and notes
are retained under `target/shared-tooling-042-0111-review/previous`. No commit,
tag, push, release, publication, sibling source edit or retained-artifact cleanup.
Product provider/backend and complete backup/restore remain independently
unqualified. Fresh Linux logger/retention, shared and consumer release-adapter,
hook, ShellCheck/Perl, document-link, snapshot and non-mutating format checks pass.
Registered hook membership is read from the retained case registry. No broad
Rust/product gate was rerun for this tooling batch. See
[the adoption review](../shared-tooling-review.json) for exact inputs and logs.
Released adoption issues #6–#9 are closed with their native tooling evidence;
#5 records the applied cleanup pending maintainer commit/push, and #10 remains
open for the independent native fixture failure.

## Retained released 0.4.1 and remaining hook convergence

The maintainer has pushed 0.4.1. Clean local HEAD and annotated `v0.4.1`
resolve to `04da09a5a919bbf9aa56e71eaacaf245107021d4`; package/lock versions,
finalized notes and the source-bound receipt agree. Original receipt source is
`85359068adc532fb69fad96b7b65571b727e6074`. No registry upload is inferred.
The committed snapshot remains the exact 56-file Shared Tooling 0.1.10 revision.

[Tag CI](https://github.com/dragginzgame/ic-backup/actions/runs/37498363329) and
[main CI](https://github.com/dragginzgame/ic-backup/actions/runs/37498362801)
are running at that exact release. Tag CI has passed the full Linux gate and
the ARM macOS system Bash 3.2 tooling step. Intel's system Bash tooling step is
still running after a bounded watch; both full macOS gates remain incomplete.
These results do not establish complete native qualification, so owning issues
remain open. The local hook owner review maps overlapping
formatting, partial-stage and installation cases to the shared checker, while
retaining the distinct exact formatter-status failure case. Retirement waits
for actual consumer native coverage; its source/owner-map evidence is retained
under `target/hooks-convergence-042-review`. The proposed retirement passes
syntax and ShellCheck, but is retained only as evidence; the maintained hook
script remains unchanged. Receipt verification, 458 local documentation links
and all 56 snapshot files pass. There is no new pending release entry from this
documentation-only state update.

The handoff and current-source introduction now distinguish released adoption
from its earlier dirty evidence. Source/version/release/Git effects remain
maintainer-owned. Product provider/backend and complete backup/restore remain
independently unqualified. Earlier batch inputs and evidence are retained below.

## Retained 0.4.1 Shared Tooling 0.1.10 failure retention

The requested Shared Tooling 0.1.10 now binds all 56 exact snapshot files/modes to
clean committed `21f3ec3dd97f2968c9f0b08924451bb2f71770d1`, matching remote main.
Private committed export preserves the earlier dirty 0.4.1 work and its real
qualification. Package/lock version, dependency selections and released receipt
remain unchanged at live 0.4.0; the complete compatible batch retains one 0.4.1
draft. See [the current review](../shared-tooling-review.json).

Adopted dependency, validation-runner and release-runner tests retain original
fixtures and report their paths on unexpected failure, preserving test status.
Three consumer checks inject failures after actual fixture creation and admit
exact status/original inputs, including the privately copied validation helper.
All three, normal tooling checks, common release-runner regressions and shell
checks pass on Linux. The initial missed-helper interception and corrected logs
are retained under `target/shared-tooling-041-0110-review`. No function, method or
type was removed; existing CI artifact selection covers the preserved paths.

[Upstream 0.1.10 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37491682760)
passes Linux and fails the new retention test on both macOS hosts. The retained
macOS 15 artifact proves its substitute shebang is concatenated with the next
statement (`/bin/bashcase`), preventing execution. That cloc/portable test is not
used or vendored here; neither are the new sccache installer or tag-delete helper.
This adoption does not imply the whole upstream CI passed. Current consumer
native macOS CI remains pending; earlier 0.1.9 native evidence retains its source.

No Git/version/release transaction, publication, live effect or artifact cleanup.
Actual product transport/backend, authenticated attribution, complete transfer,
application safety and terminal/reference release remain independently pending.
Earlier 0.4.1 batches and their qualifications remain below.

## Retained 0.4.1 formatter, finalizer and fixture convergence

The complete compatible batch retains the single undated 0.4.1 draft from live
0.4.0 (`52532cca4d5276bb67810ffb346aba464d5a719a`). Package versions, locked
dependency selections and released receipt are unchanged. The earlier version/
inheritance adoption is retained below with its actual source and evidence.

Shared Tooling now binds 56 exact files/modes to clean committed 0.1.9 at
`b32d3038c850a7c53470c326b0f7f11263b31669`, matching inspected remote main.
Private committed export preserves the earlier dirty consumer adoption. The
formatter availability helper and regressions are included; existing installer
fixtures and release guidance are refreshed without vendored patches or sibling
writes. See [the current review](../shared-tooling-review.json).

Setup, `fmt` and `fmt-check` share the existing pin and offline prerequisite.
Matching version output with failure status and unavailable rustfmt reject before
formatting. Real hook checks now overlay the current helper/pin files. Release
changelog selection/rewriting delegates to its canonical AWK owner with saved
previous/target/date identities. Imported undated history remains intact after
a package bump. Empty/missing notes no longer gate preparation; duplicate and
noncanonical identities, conflicting candidates and dated targets still reject.
Failed/empty transformations retain candidates without replacing metadata.
Prepared/committed recovery continues exact payload/receipt admission without
refinalization; original backups, rollback and source validation retain owners.
No functions, methods or types were removed.

All 69 registered consumer release cases and common runner pass, including six
new successful/rejected delegation cases and additional identity conflicts. Real
formatting/hook, shared metadata, formatter, installer, snapshot, ShellCheck/Perl
and workflow lint checks pass on Linux. The initial missing fixture-helper failure
and corrected evidence remain under `target/shared-tooling-041-extension`.
Committed installer cases restore exact authenticated archives rather than
repacking under an old digest, retain installer traces and prove intended refusal
payloads ran. [Upstream CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37489483879)
passes Linux and both native macOS hosts at the adopted revision. This dirty
consumer batch has no remote CI result; its native qualification remains pending
with [#7](https://github.com/dragginzgame/ic-backup/issues/7),
[#8](https://github.com/dragginzgame/ic-backup/issues/8) and
[#9](https://github.com/dragginzgame/ic-backup/issues/9).

Work stays uncommitted. No release/version transaction, publication, live effect
or cleanup occurred. Product transport/backend, authenticated attribution,
complete transfer, application safety and terminal/reference release remain
independently unqualified; standalone backup/restore is not implemented.

## Retained initial 0.4.1 tooling and native failure evidence

The maintainer reports 0.4.0 live. Clean session HEAD and annotated `v0.4.0`
resolve to `52532cca4d5276bb67810ffb346aba464d5a719a`; finalized changelog,
workspace/lock version and source-bound receipt agree at 0.4.0. Original receipt
source is `c82b03fdc0fb4865a187f6a1da6f4b2c67611d2f`. The current batch opens one
compatible undated 0.4.1 entry; package versions, dependency inputs and receipt
remain untouched.

Shared Tooling adopts exact committed 0.1.8 at `d957d1f` across 54 files. Remote
main matched at inspection; the read-only sibling then became dirty, so refresh
used a clean private committed checkout. Uncommitted format prerequisite, archive
repair/diagnostics and changelog work was not imported. The existing pin target
now enables Cargo inheritance under the same configured CI/release pipeline.
The adapter replaces duplicate version parsing with the shared offline stable
Cargo/TOML reader while retaining Git selection, numeric release limits and
preparation/receipt/recovery owners. Selected reads export exact original manifests
and library target; helpers resolve from the adapter path. Failed exports remain
retained, and rejected/empty/partial version output writes no metadata.

Shared metadata cases qualify table forms, aliases, independent discovery and
invalid inheritance/overrides/catalogs. Six additional consumer cases qualify
comments, original commit versus invalid working inputs, duplicate TOML, failed
partial/empty output and failed parsing before preparation. All 61 registered
consumer release cases, common runner, pin/metadata, formatting-hook, installer,
snapshot and ShellCheck/Perl checks pass on Linux. The initial helper-path failure
in a real private-index fixture remains retained beside corrected passing logs.
Exact inputs, source/mode identities, prior snapshot/review and fresh logs are under
`target/shared-tooling-041-review`; see [the adoption review](../shared-tooling-review.json).
No functions, methods or types were removed; version parsing moved to its canonical
shared owner, without a second reader or compatibility lane.

Released 0.4.0 [main CI](https://github.com/dragginzgame/ic-backup/actions/runs/37486731118)
and [tag CI](https://github.com/dragginzgame/ic-backup/actions/runs/37486733748)
passed Linux and failed both macOS hosts during system-Bash host-tool fixtures.
The last successful checksum/pin/evidence stages locate the failed command, but no
native installer trace was retained. Source review identifies regenerated archive
bytes under an unchanged digest as a likely fixture integrity cause, not a proven
production failure. [Issue #9](https://github.com/dragginzgame/ic-backup/issues/9)
owns that finding and native rerun. CI now retains temporary test/index/log artifacts
on failure. Uncommitted upstream corrections must first gain a reviewed revision;
native macOS remains unqualified. [Issue #6](https://github.com/dragginzgame/ic-backup/issues/6)
keeps adoption/native evidence, and [#5](https://github.com/dragginzgame/ic-backup/issues/5)
keeps hook replacement coverage. Formatter/changelog adoption remains with its
own issues, not a handoff queue.

Work stays uncommitted. No Git/version/release transaction, package upload, live IC
effect or cleanup was performed. Product provider/backend/authenticated attribution,
complete transfer, application/load/start and terminal/reference release remain
independent qualification work; full B1/B2 and standalone backup/restore remain
unestablished. Prior handoffs retain their actual inputs and scope below.

## Retained pre-release 0.4.0 upload settlement and dependency adoption

The maintainer reports 0.3.9 live. Local release HEAD/tag
`2587259d1c8424e9aac585413c8efbc374c1faeb`, finalized changelog, package version
and original receipt/source `ce5f09945b86190742b3a163bd1c8366611909d1` agree;
the worktree was clean before implementation. The original compatible work selected an undated 0.3.10 draft. The maintainer's
concurrent manifest/lock edits now select Host Tools and Metrics 0.2.0. Metrics is
returned by seven public local-summary getters, changing Rust type identity; the
complete pending draft therefore requires 0.4.0. The selected patch conflict was
reported; the maintainer now confirms continuing the hard cut. Package/lock version and
receipt remain 0.3.9; only dependency selections changed.

Exact data-upload recovery now binds the full original source/upload plan and
immutable allowances to an independently retained destination metadata/data-read
request. The same target/new raw ID and original region/offset/length or known
chunk hash are required, alongside known uploaded metadata and original region
sizes. Both attempts must remain pending and the exact read digest reserved.
No journal or spending owner is added. See
[the implemented contract](../contracts/ic-snapshot-upload-data-observation.json),
[maintained boundary](../extraction-boundary.md#exact-originally-reserved-data-upload-observations)
and [fresh source review](../ic-snapshot-upload-data-observation-source.json).

The single provider signature permits one already accounted exact replicated
data-read update. Canonical existing authority/reservation/claim admission and
redaction are reused. The new passive data response preserves its existing
2 MiB codec bound, admitting a 1 MiB chunk plus wire overhead; status/list responses
keep their released 1 MiB ceiling. Pure association uses the existing bounded
data decoder/digest and projects original-chunk SHA-256 equality. Matching bytes
can predate a write; differing/absent bytes alone cannot establish a negative
outcome. Unknown destination source cannot supply an uploaded default. No receipt,
Applied/NotApplied inference, refund, retry, complete upload or reference release
follows. Authentication, original allocation/write attribution, backend readback,
fresh read permission/timing and never-dispatched custody remain integration-owned.

All 47 registered selected unit cases pass against current production code,
including 11 new data-readback cases and the affected original upload, observation
and data-codec owners. Coverage includes every region and known empty/nonempty
chunk, exact identities/claims, metadata absence/dimension drift, missing/settled
reservations, all typed provider failures, malformed/inexact/excess replies and
maximum data bytes. The public upload journey durably retains destination metadata/
read originals before reservation, preserves both pending attempts through
indeterminate failure and matching/different evidence, and reopens unchanged bytes
without dispatch. Source journals/plans/references and per-guard diagnostics remain
unchanged. Clippy/docs with warnings denied and Rust 1.91 all-target/all-feature
checks pass. Formatting, all 421 maintained local document references and the exact
48-file Shared Tooling snapshot also pass. Registered membership, exact source/
dependency/release identities and fresh logs remain under
`target/upload-data-observation-0310-review`. This is focused
native Linux qualification; native macOS, full CI/package and live IC remain separate.

Continued compatible work adds passive successful-read settlement claims and
`validate_settlement` under the existing data-observation owner. Admission reuses
exact original authority/reservations/actual claims and bounded read decoding, then
matches both attempt IDs, a caller-owned fresh challenge, exact metadata/read/raw
reply digest and unchanged opaque observation evidence. Applied additionally needs
matching original bytes and independent exclusive-write attribution; NotApplied
requires qualified exclusion of transient application/overwrite. Equal preexisting
bytes imply neither outcome. Unresolved requires a settled authenticated successful
read; lost, absent or malformed replies remain pending. See
[the contract](../contracts/ic-snapshot-upload-data-settlement.json),
[boundary](../extraction-boundary.md#exact-data-upload-settlement-claims) and
[fresh source review](../ic-snapshot-upload-data-settlement-source.json).

Pure views perform no IO, calls or automatic receipt. Only a qualified integration
uses the unchanged journal transition. Settled uncertainty clears that observation,
retains the unresolved mutation and refunds no allowance. The public local journey
records synthetic settled evidence solely to qualify durable accounting/reopen,
then rejects repeated settlement and new attempts while retaining every original
source reference, journal/plan and diagnostic. No IC behavior is simulated.

All 54 registered selected unit cases (7 new settlement cases) and the public
upload/reopen journey pass against current production code. Exact membership,
initial fixture/lint failures, fresh final logs and previous local source remain
under `target/upload-data-settlement-0310-review`. Clippy/API docs with warnings
denied and Rust 1.91 all-target/all-feature checks pass. Native macOS, full
CI/package, actual authenticated attribution and backend qualification remain
separate. No maintained function, method or type was removed; duplicated native
response-fixture construction now has one local owner. Earlier readback/tooling
qualification retains its actual prior inputs below and in the retained evidence.

Shared Tooling now adopts exact committed `9f8c7c7` from clean remote/main and
read-only source. All 52 snapshot files/modes verify. Release preparation uses
the shared lockfile transformer with status admission before metadata writes;
actual original/candidate Cargo graphs pass offline/locked, with only the local
package version changed inside retained fixtures. The hook adapter now invokes
the shared checker with current formatter inputs and ordering-only child manifest.
Focused Linux lock/adapter/recovery, hook, installer and snapshot/runner cases,
shell/Perl checks, offline tools, pins, formatting and local document links pass.
Evidence and previous snapshot/review remain under
`target/shared-tooling-0310-adoption`; see
[the adoption review](../shared-tooling-review.json).
[Issue #5](https://github.com/dragginzgame/ic-backup/issues/5) remains open for
native macOS replacement coverage before retiring overlapping hook fixtures.
Three-host CI and explicit system Bash 3.2 checks are wired; existing additional
formatter-status and release/receipt cases remain local. No vendored patches.

The initial 0.1.14 archive at `1e01809` matched committed source and unchanged
0.1.13 Rust bytes. The maintainer then changed dependency inputs during this batch:
Host Tools 0.2.0 at `be7d739`, Metrics 0.2.0 at `8657c35`, and transitive wasmparser
0.261.0. Archive checksums, every Rust source blob and exact revisions match.
Used Host Tools artifact implementations and Metrics summary arithmetic remain
byte-identical to the prior published inputs; unused tool/Wasm APIs and removed
IC adapter do not enter this consumer. Public Metrics type identity still changes.
The current reviews preserve historical dependency qualifications rather than
relabeling them; see [Host Tools](../ic-host-tools-adoption.json) and
[Metrics](../ic-metrics-adoption.json). Toolchains remain unchanged. Released 0.3.9
[main CI](https://github.com/dragginzgame/ic-backup/actions/runs/37476081335) and
[tag CI](https://github.com/dragginzgame/ic-backup/actions/runs/37476081385)
passed on Linux and both macOS hosts. They do not qualify this dirty draft.

Metadata-allocation settlement now admits exact original/current inventories and
independent passive attribution under original reservations and a current challenge.
Closed-baseline comparison has one shared private owner for capture and upload.
Applied names one new bounded non-source ID even among multiple candidates;
NotApplied must exclude transient allocation/deletion. Successful settled uncertainty
remains distinct from lost observations. The public native journey retains baseline/
list declarations before mutation, explicitly records synthetic settled uncertainty,
then reopens with pending mutation, exhausted spending and exact original bytes.
See [contract](../contracts/ic-snapshot-upload-settlement.json),
[boundary](../extraction-boundary.md#exact-metadata-upload-settlement-claims) and
[source review](../ic-snapshot-upload-settlement-source.json).

All 91 registered unique selected unit cases (7 new metadata-settlement cases) and
five public local recovery/source/upload cases pass on the selected 0.2 dependencies.
Focused Clippy/docs with warnings denied and Rust 1.91 all-target/all-feature checks
pass. Formatting, all 444 local document references and the exact 52-file Shared
Tooling snapshot also pass. Exact membership, original/current dependency inputs, published source/archive
identities and logs remain under `target/upload-metadata-settlement-0310-review`.
Earlier 0.1.14 logs retain their own inputs. No maintained function, method or type
was removed; existing public capture comparison is unchanged and shares its baseline
merge internally. Native macOS, full CI/package and actual authenticated IC/backend
qualification remain separate.

The continued hard-cut review exposes the canonical shared summary type through
`ic_backup::ops::persistence::MeasurementSummary`. Consumers can name returned
metrics without a separate dependency; direct Metrics users adopt 0.2. No local
arithmetic wrapper or older minor lane is installed. The seven measured outcomes/
units, method-specific observation response bounds and v1 recovery owners remain
live, rather than obsolete compatibility code. Unfinished spending, source references
and fence obligations retain exact records and cannot be reset during the hard cut.
See [consumer guidance](../development.md#the-04-hard-cut) and
[the Metrics review](../ic-metrics-adoption.json). No function, method or type was
removed. All three registered focused metrics cases and the public source/upload/
reopen journey pass, alongside Clippy/docs with warnings denied and Rust 1.91.
Formatting, 446 local references and all 52 snapshot files verify. Manifest/lock
inputs remain byte-identical to this review's start. Fresh membership, source/input
identities and logs are retained under `target/hard-cut-040-review`.

Work remains uncommitted, with no Git/version/release transaction, publication,
live effect or cleanup. Next product work remains authenticated allocation/write
attribution and qualified settled reconciliation, then actual provider/transport,
selected backend and application/load/start qualification. Full B1/B2 and
independently usable backup/restore remain unestablished. Prior handoffs and their
evidence retain the exact historical scope below.

## Retained pre-release 0.3.9 metadata-upload recovery and shared primitive fixes

The continued compatible batch adds an ephemeral original metadata-upload/list
observation request and single already reserved provider signature. It binds the
full original plan, exact source/upload/list bytes, immutable allowances and both
pending attempt IDs. Existing upload authority, reservation admission, passive
response/error/view owners and bounded inventory decoding are reused. Pure
association performs no IO, spending or settlement. Data uploads and status
observations reject at this boundary. See
[the implemented contract](../contracts/ic-snapshot-upload-observation.json) and
[fresh committed-source review](../ic-snapshot-upload-observation-source.json).

Zero, one or several inventory entries and timestamp/size agreement establish no
exclusive allocation attribution or upload outcome. Canic's singleton-new-ID
success inference, optional metadata and whole CLI process behavior were not
imported. Integrations retain original baseline custody, authenticated chronology,
fresh read permissions and proof of no prior observation dispatch. Failures/lost
replies remain pending; local reopen invokes no provider or implicit reissue and
retains every allowance, obligation and source reference. No schema, digest,
terminal flag, receipt, reference-release API or installed backend is added.

Published `ic-host-tools` 0.1.13 is now the compatible minimum. It validates
impossible reader/writer counts with typed IO errors instead of panicking in the
shared reads/copies; private consumer copies retain partial output and complete
checksum admission. Its archive, all published Rust sources against the release
commit/cache and isolated lockfile change were verified. Every other locked
package is unchanged. The public local `checksum_reader` remains unchanged.
The exact prior 0.1.12 review/lock/manifest and evidence retain their original
scope under `target/host-tools-013-review`; see
[maintained adoption](../ic-host-tools-adoption.json).

All registered selected cases pass against current production code and published
0.1.13: 137 unit cases across affected primitives/recovery owners and five public
local recovery/restore-source/upload journeys. Eight new cases cover exact
metadata/list originals, typed drift/unsupported/missing/settled denials,
bounded malformed inventory and provider failures. The public upload journey
durably retains exact list bytes before reservation, preserves both pending
attempts through indeterminate failure, associates independent zero/one/many
inventories without settlement and reopens unchanged evidence without dispatch.
Source journals/plans/references and diagnostics remain unchanged. A new consumer
copy regression checks invalid stream counts and exact retained partial output.
Warning-denied Clippy/docs and Rust 1.91 all-target/all-feature checks pass.
Formatting, dependency declarations, all 415 maintained local document references
and the exact 48-file Shared Tooling snapshot also pass.
Fresh membership/source/dependency identities and logs remain under
`target/upload-observation-039-review` and `target/host-tools-013-review`.
These are focused native Linux results, not full CI/package, native macOS or
live IC qualification. Earlier batch evidence below retains its original scope.

Shared Tooling remote main remains the exact adopted committed `47cd2cc` revision.
New [ic-backup #5](https://github.com/dragginzgame/ic-backup/issues/5) remains open:
its lockfile/formatting-hook helpers are uncommitted upstream and cannot yet enter
the reviewed snapshot. No manual vendor patch or GitHub write was made. Published
0.1.13 [upstream CI](https://github.com/dragginzgame/ic-host-tools/actions/runs/37461197671)
is queued. Released 0.3.8 main CI passed; its tag CI is still running. These runs
do not qualify this dirty draft. The undated draft remains 0.3.9; package and
receipt remain 0.3.8. Work is uncommitted, with no release, commit, push,
publication, live effect or cleanup.

Next product work remains authenticated original allocation attribution and
method-specific lost data-upload reconciliation before transport/runners and
selected real backend qualification. Full B1/B2 and independently usable backup/
restore remain unestablished.

### Retained earlier 0.3.9 shared primitives and tooling refresh

The requested continued batch refreshes the exact 48-file snapshot to clean,
committed Shared Tooling `47cd2ccaf0e8b428f06e6db0262df76cfc1581de`, matching
remote main. Upstream's canonical `VERSION` is 0.1.6 with undated 0.1.7 notes;
this is the committed 0.1.7 batch, not a finalized-release claim. Core shared
engineering rules remain unchanged. The new verifier centralizes portable digest
generation and rejects failed/malformed backend output. IC installer receipts
reuse it, reject names the line format cannot represent and retain failed
traversal/hash candidates. Tool pins and receipt formats remain unchanged.

`make check-doc-links` selects tracked and non-ignored new Markdown documents,
then delegates local target existence to the exact shared parser. It joins the
configured CI/release gate; native macOS tooling explicitly exercises the new
parser/digest cases. Its contract excludes anchors and remote URL availability.
The shared verification guide and selected focused regressions are now readable
offline. Optional registry/RustSec helpers have no caller to replace; release
adoption retains the consumer's completed-resume receipt check and existing
actual-Make fixtures. No vendored patch or sibling mutation was made. See
[snapshot review](../shared-tooling-review.json) and
[consumer adoption](../shared-tooling.md).

One additional private upload-source read now uses `ic-host-tools::artifact::read_reader`
over the already opened, sought and `take`-bounded descriptor. Pure kind/range
admission, no-follow regular-file checks, exact length and the existing 1 MiB
limit remain in place, alongside full original source/checksum/journal/custody
verification before and after collection. Actual IO errors retain their existing
type; fallible collection replaces the local growing-buffer body. Exact argument
bytes, source references, original spending and per-guard metrics are unchanged.

All registered upload-source cases and the public source/upload/reopen journey
pass against this production reader, covering empty/region/chunk bytes, unsafe or
changed sources, invalid destination/ranges and an exact 1 MiB slice from a larger
region. Clippy and API docs pass with warnings denied, as do Rust 1.91 all-target/
all-feature checks. Fresh logs and original source/review remain under
`target/shared-chunk-read-039-review`. No maintained function, method or type was
removed; the duplicate private `read_to_end` collector was replaced.

Snapshot integrity, all maintained local document links, actual existing local
host/IC tool offline checks, dependency declarations and formatting pass.
Selected parser, digest, IC installer and snapshot/runner regressions and shell/
Perl checks pass on Linux; evidence and previous snapshot/files/review are under
`target/shared-tooling-039-adoption`. These are focused checks, not a full native
suite, package verification or complete CI/release gate. Earlier primitive and
recovery qualification below retains its original scope and evidence.

ic-backup still has no open issues at inspection; no GitHub writes were needed.
At final inspection, released 0.3.8 main CI is running; its tag CI and new
upstream Shared Tooling CI remain queued:
[shared upstream](https://github.com/dragginzgame/shared-tooling/actions/runs/37458968809).
This dirty 0.3.9 work has no remote/native macOS or live IC qualification.
The coherent compatible batch stays in the same undated 0.3.9 draft; package
version and receipt remain 0.3.8. Changes are uncommitted, with no agent release,
Git commit, push, publication or destructive cleanup. Next product contract work
remains method-specific lost-upload observation/reconciliation; transport, runners,
full B1/B2 and standalone backup/restore remain unestablished.

### Retained initial 0.3.9 primitive adoption

Local release HEAD `a9bc43ba845c8b6d3d996dee27259b146e98d3e2`, annotated
`v0.3.8`, finalized changelog and original receipt/parent
`b63d626274c33dcae156353ebd8fa17f43e46db4` agree. Read-only committed-blob
hashes and tag identities were verified before this batch. `release-tag-check`
refused the maintainer's pre-existing dirty manifest; no work was restored to
bypass that guard. Exact-source main/tag CI was queued at inspection:
[main](https://github.com/dragginzgame/ic-backup/actions/runs/37458336677),
[tag](https://github.com/dragginzgame/ic-backup/actions/runs/37458336061).
Registry publication was not inspected. Both prior 0.3.7 CI runs have passed.

The requested host-tools refresh adopts published `ic-host-tools` 0.1.12 at
`b9fe3fdc5a1565360aa194acea2c5182b206d347`. Its archive checksum, every published
Rust source against both the registry cache and committed upstream, and the
isolated lock change were verified. Every other package entry is unchanged.
The compatible minimum is 0.1.12 because the adopted APIs first appear there;
the original dirty 0.1 manifest range is retained with the pre-update lock and
handoff under `target/host-tools-012-review`. The package inherits the dependency
in its ordinary table for portable output admission; filesystem effects still
retain their Unix contracts. No sibling path/patch, toolchain or MSRV change.

Private staging copies now delegate stream copying and SHA-256 capture to the
shared owner with `u64::MAX`, preserving the existing absence of a total artifact
ceiling. They retry interrupted reads and preserve original source/sink IO
errors and partial output without returning a complete checksum on failure.
Local descriptor traversal, 0700/0600 exclusive creation, retained checksum
comparison, tree digest recipe, byte custody and durable publication stay local.
The public `checksum_reader` implementation and behavior are unchanged.

Nine typed record size owners and restore-reference admission now use the shared
bounded writer over `io::sink` to check exact pretty-JSON output budgets without
allocating the whole encoded record. Inclusive original limits, `RecordTooLarge`
with its exact u64 limit, unrelated JSON errors and all durable publication bytes
are retained. This creates no new persisted state or progress/spending owner.
No maintained function, method or type was removed; the duplicate copy loop and
ten allocating size-admission bodies were replaced.

All registered focused artifact/changed persistence cases and public local
recovery/restore-source journeys pass against the production helpers. The fresh
copy cases check known SHA-256 identity, multichunk/empty input, short writes,
interrupted reads and exact IO failures with retained partial output. Output
cases check pretty/escaped/UTF-8 encoding, exact/zero/extreme limits and unrelated
serialization failure; original journal overflow and interrupted publication/
owner-death/reopen cases remain qualified locally. Warning-denied Clippy/docs,
Rust 1.91 all-target/all-feature checks and locked/offline metadata also pass.
Actual registered/passing membership, source/dependency identities and logs are
retained under `target/host-tools-012-review`; see
[the maintained adoption review](../ic-host-tools-adoption.json).
These are native Linux results. No full suite, package verification, broad
CI/release gate, native macOS run or live IC effect ran for this dirty batch.

Committed Shared Tooling remains the adopted 44-file 0.1.6 snapshot at
`a37771f1b6b5fc9a88ed6ab3b705bdda35cd8fa3`; no newer committed refresh exists.
ic-backup has no open issues at inspection. The 0.1.12 upstream CI run is queued
and does not yet supply native qualification:
[upstream CI](https://github.com/dragginzgame/ic-host-tools/actions/runs/37458519632).
No GitHub writes were needed. Earlier evidence remains at its original scope.

The compatible implementation/dependency changes select one undated 0.3.9 draft.
Package version and release receipt remain 0.3.8; public signatures/exhaustive
errors and v1 schemas/digests/bounds remain unchanged. Work is uncommitted, with
no agent commit, release/version transaction, push, publication or cleanup.
Next product work remains method-specific lost-upload observation/reconciliation
before authenticated transport and selected real backend qualification; full
B1/B2 and standalone backup/restore remain unestablished.

## Retained pre-release 0.3.8 handoff

The maintainer reports 0.3.7 live. Local release commit
`1a23d66dd65b1e36e986b8c7d13758cf3c92d193` has original receipt/parent source
`fafdc0ddc201749c6b16aa41bccd84a8f98d3fe9`; read-only `release-tag-check` passed
before beginning this batch. The package, lock and receipt select 0.3.7, and its
changelog is finalized. Registry publication was not inspected. Exact-source
[main CI](https://github.com/dragginzgame/ic-backup/actions/runs/37453307419) and
[tag CI](https://github.com/dragginzgame/ic-backup/actions/runs/37453307321) were
originally queued. The 0.3.8 maintenance review now confirms the complete tag gate
passed on Linux and both native macOS architectures. Main's Linux and ARM64 jobs
passed; Intel was still running at that inspection. This is exact 0.3.7 qualification,
not a result for dirty 0.3.8. Raw summaries/logs are retained under
`target/maintenance-038-review`.

## Pending 0.3.8 shared record reads and single-call upload port

The maintainer authorized bounded JSON record-read adoption, initially from
published `ic-host-tools` 0.1.10 and now selecting compatible 0.1.11. `read_json`
delegates final-component no-follow,
nonblocking regular-file admission and bounded/fallible byte collection to the
shared owner, then uses the unchanged JSON decoder. Local error projection preserves
`RecordTooLarge` with the caller's exact limit, original IO errors and
`InvalidInput` for nonregular files. Empty input remains a JSON EOF error.
Caller-selected parent aliases remain accepted; this is no new confinement claim.
Durable publication, locks, journals, tree hashing and descriptor custody are unchanged.

The dependency is catalog-owned and inherited only for Unix. Ordinary offline
resolution adds ic-host-tools and its transitive packages, including tar 0.4.46;
the earlier unsafe tar selections are absent. Every selection present at this
batch's start is retained, including the maintainer's pre-existing ic-metrics 0.1.9
lock update. Published archive identity matches the registry checksum; no sibling
path/patch or exact-version exception was added. The later 0.1.11 patch publishes
the corrected compatible tar 0.4.46 minimum and changes only ic-host-tools's version/
checksum in our existing graph. All published Rust sources match 0.1.10 byte for
byte; original source/dependency evidence retains its earlier scope.

All registered JSON cases and public local recovery/upload journeys pass against
the shared production reader, including real FIFO/device rejection, final/dangling
symlinks, parent aliases, exact/zero/extreme limits, missing/malformed/empty records,
interrupted durable publication and exhausted original upload spending/reference
reopen. Evidence and pre-adoption source/lock/handoff are under
`target/ic-host-tools-read-review`; see [dependency/source review](../ic-host-tools-adoption.json).
These are native Linux results, not native macOS or IC effect qualification.
Warning-denied all-target/all-feature Clippy and API docs, Rust 1.91 checks,
locked/offline metadata, formatting, unchanged Shared Tooling and dependency
declarations pass against this graph (`clippy.log`, `docs.log`, `msrv.log`,
`metadata-final.json`, `declarations.log`). Qualification derives JSON/public case
membership from the actual registry and passing logs and retains final source
identities in `target/ic-host-tools-read-review/qualified-source.sha256`. No full
suite, package verification, CI/release gate or macOS run was performed.

The requested maintenance review finds no newer committed Shared Tooling revision:
remote/local main still matches the reviewed 44-file 0.1.6 snapshot at
`a37771f1b6b5fc9a88ed6ab3b705bdda35cd8fa3`. Its documentation/release/registry
helpers remain uncommitted and were not adopted. ic-host-tools's new bounded writer
is also uncommitted/unpublished and supplies no maintained API to adopt yet.
Published 0.1.11 passes fresh consumer JSON/public recovery/upload, Clippy, Rust
1.91, docs, locked metadata, formatting, pinning and snapshot checks; evidence is
under `target/maintenance-038-review`. Its upstream native CI fails at installer
tests because `rg` is missing, while MSRV passes. This host-prerequisite failure
does not supply consumer/macOS runtime qualification. Supporting failed logs and
the complete published Rust-source comparison remain retained.

The accepted continuation adds `ports::ic_snapshot_upload::IcSnapshotUploadProvider`
over the released exact `IcSnapshotUploadAttempt`. It returns the existing passive
`IcMutationAcknowledgement` and `IcMutationProviderError`, with no new request,
failure, record, digest, spending or codec owner. The contract permits one exact
originally reserved replicated update only. Fresh controllers/prerequisites,
authentic complete source and exclusive new destination attribution, stable byte/
command custody and proof of no prior dispatch remain integration-qualified.
No batching, hidden retries/observations, installed provider or automatic receipt,
lost-effect settlement, load/start, terminal or reference/fence release is added.

The native port test covers both upload kinds and all shared failure variants using
independent declared accounting; no management behavior is simulated. The real
public source/upload journey now exercises provider refusal/indeterminate failure,
checks exact durable journal/source/reference bytes, and reopens exhausted originals
without any provider reissue. All registered upload cases and the public journey
pass. Evidence is under `target/snapshot-upload-port-review`; the prior handoff and
unchanged Cargo lock are retained there. See [the port boundary](../extraction-boundary.md#single-originally-reserved-ic-snapshot-upload-port),
[contract](../contracts/ic-snapshot-upload-port.json) and
[fresh committed-source review](../ic-snapshot-upload-port-source.json).
Warning-denied all-target/all-feature Clippy and API docs, Rust 1.91 checks,
manifest/Rust formatting, unchanged shared snapshot and dependency declarations
pass (`clippy.log`, `docs.log`, `msrv.log`, `declarations.log`). Qualification
derives upload test membership from actual registered/passing cases and binds the
final source in `qualified-source.sha256`; it does not infer authority from counts.
No full native suite, CI/release gate or native macOS run was performed for this batch.

This is an additive public trait/module, selecting the compatible 0.3.8 draft.
Package, receipt and v1 records/digests/bounds remain at their released identities;
the shared reader dependency and retained pre-existing lock update are described above.
Work is uncommitted; no agent commit, release/version transaction,
push, package publication, live IC effect or destructive cleanup ran. No maintained
function, method or type was removed. Next work is method-specific lost-upload
observation/reconciliation, then an authenticated single-call implementation and
selected real backend qualification; complete B1/B2 and standalone backup/restore
remain unestablished.

## Retained pre-release 0.3.7 handoff

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
