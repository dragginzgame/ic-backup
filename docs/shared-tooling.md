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


# Shared engineering and tooling adoption

The maintainer requested adoption of Shared Tooling as the top-level Dragginzgame
engineering rules on 2026-10-04. [AGENTS.md](../AGENTS.md) identifies the local
product overlay; [DRAGGINZGAME.md](../DRAGGINZGAME.md) is the reviewed, byte-for-byte
local copy of the upstream engineering rules. Common validation rules apply here;
Shared Tooling's own `AGENTS.md` script-suite commands apply only upstream.

## What this adoption means

Here, a tooling snapshot means a reviewed local copy of Shared Tooling files,
distinct from an Internet Computer canister snapshot or application backup.

| Layer | Responsibility |
| --- | --- |
| Shared Tooling source | Supplies reviewed repository-neutral rules and helper files |
| Vendored snapshot | Pins exact local bytes and executable modes so CI needs no sibling checkout or network access |
| `DRAGGINZGAME.md` | Preserves the reviewed common engineering baseline used by this repository |
| `AGENTS.md` | Adds IC Backup's product-specific architecture, safety and authority rules |
| Local adapters | Connect shared helpers to this repository without editing vendored files |

Upstream changes do not alter this repository automatically. Adoption requires a
reviewed refresh, a normal consumer diff and the relevant local validation.

## Reviewed sources

The current 92-file snapshot selects committed Shared Tooling 0.2.5 at
`04e07b4bf54e7aeb03eb7804a845cee27b7305df`. Canonical refresh from a clean
isolated committed checkout changes only the pre-commit hook, its installer and
the hook rule within the same roster. Hook paths preserve literal trailing newlines;
failed Git configuration reads stop setup without replacing the current selection.
No hook activation, root Git configuration or dependency selection changes.
The complete upstream hook regression and consumer formatting/install fixtures
pass independently. Exact bytes/modes and focused checks are retained in the
[review](reports/audits/2026/10/09/continuation-0113/01/report.md).

### Prior Shared Tooling 0.2.4 review

The prior 92-file snapshot selects committed Shared Tooling 0.2.4 at
`ffbf665b8481c36b2d9f4d988abec557c3485fa6`. Canonical refresh from a clean
isolated source changes ten test companion declarations and the consumer guidance,
with the same complete roster and exact committed bytes/modes. Common engineering
rules, production helper behavior and tool pins are unchanged. Required test
helpers are now declared at more owners; canonical export admits our existing
complete selection. The optional CI installer suite stays omitted. Focused owner
exporter fixtures and actual consumer admission remain separate; the current
[review](reports/audits/2026/10/09/continuation-0112/01/report.md) records both.

### Prior Shared Tooling 0.2.3 review

The prior 92-file snapshot selected committed Shared Tooling 0.2.3 at
`ac4549c5ebde497f7db0da5d05d32835112e51de`. Canonical refresh from a clean
isolated checkout changes four selected documentation/task files and preserves
all selected executable bytes, modes and pins. Common engineering rules remain
unchanged. The optional CI installer fixture now requires explicit selection of
all five wrapper companions; this consumer selects only its production ShellCheck
wrapper/engine and does not vendor that fixture. Focused exporter qualification
uses the complete exact upstream source, retaining that distinction from the
consumer snapshot. Updated CI-health guidance separates queued jobs from code
failures and requires evidence before treating branch/tag runs as equivalent.
It grants no cancellation, workflow change or runner-capacity authority.
The [adoption review](reports/audits/2026/10/09/shared-023/01/report.md) records
exact source/export evidence and consumer checks.

### Prior Shared Tooling 0.2.2 review

The prior 92-file snapshot selected the committed Shared Tooling 0.2.2 handoff at
`ee48bb37c98c771e77b92fd891f0757d8c1c8b99`. Canonical refresh from a clean
isolated committed checkout changes six selected files, preserving the roster,
all pinned IC tools and exact executable modes. The IC installer/checker handles
the final pin row without a newline. CI tool installation publishes to the exact
executable destination, rejects late directories and replaces late symlink entries
without following their targets; Perl is an explicit bootstrap prerequisite.
Rejected publication retains the candidate and conflicting directory contents. Consumer qualification is recorded in the current handoff.

### Released Shared Tooling 0.2.0 adoption

The released 92-file snapshot selects the committed Shared Tooling 0.2.0 handoff at
`8140e3dd1b44409d682c721889ab702f438c6a17`. Canonical refresh removes the two
PocketIC checkers and their dedicated fixture from the selection; selected bytes
and executable modes come from a clean isolated committed checkout. Testkit now
owns server release/assets, provisioning and offline admission. The five-tool
consumer matrix keeps all other reviewed pins unchanged.

The released 0.11.0 hard cut adds explicit `make install-testkit-server` and offline
`make testkit-server-check`, which prints the admitted absolute server path. Cargo
metadata selects the unique locked Testkit version for both libraries' fixtures
and the CLI; Shared Tooling's selected Cargo installer admits the CLI receipt.
Testkit's check supplies server admission, replacing local version equality and
installed-receipt projection. Both simulator suites keep managed custody, private
logs/digests and original spending locally. CI explicitly prepares and checks this
route on Linux and both macOS hosts; current local evidence and native acceptance
limits are retained in the handoff. Ordinary checks install nothing. Reinstall the
shared five-tool bundle explicitly; previous bundles/evidence remain retained.

### Prior Shared Tooling 0.1.38 review

The prior 95-file snapshot selected the committed Shared Tooling 0.1.38 fix at
`926a20606591214ab29faa236b0b584e4857439e`. Canonical refresh changes three selected
files from 0.1.37, matching all committed bytes and executable modes. The sibling's
dirty VERSION preparation is excluded. Dependency exception admission now requires
one complete JSON array; valid existing exceptions remain unchanged. The regression
suite qualifies conflicting first/last documents and preserves manifests, locks,
exception bytes and the private index on rejection. Real consumer pin/snapshot
checks and ShellCheck pass. The [follow-up review](reports/audits/2026/10/09/updates-0102/01/report.md)
separates this adoption and incoming published dependency qualification from the
earlier 0.10.2 graph. Exact upstream CI remains queued at inspection.

### Prior Shared Tooling 0.1.37 review

The prior 95-file snapshot selects committed Shared Tooling 0.1.37 at
`dc4fdf0f78928d75b69bbf43b37c690c53a04d1e`. Canonical refresh from a clean
isolated checkout changes seven selected files, with all committed bytes and modes
matching. The installer now refuses conflicting multi-document receipts, rechecks
installation ancestors after Cargo and preserves the original failed-build status.
Formatting hooks and their adoption checker reuse the original consumer's prepared
tools while continuing to format exact isolated staged inputs.

Focused installer fixtures and local hook/adoption checks pass on Linux. The local
hook fixture qualifies a prepared pinned sorter ahead of a deliberately wrong
inherited tool, plus missing/wrong local-tool rejection without changing the index
or unrelated files. No implicit installation or default tool selection changes.
Exact upstream 0.1.37 CI and the new 0.10.1 release CI are queued at inspection;
the uncommitted 0.10.2 consumer candidate has no matching native run. The
[review](reports/audits/2026/10/09/continuation-0102/01/report.md) separates current
local checks, prior released native acceptance and remaining runner scope.

### Prior Shared Tooling 0.1.35 review

The prior 95-file snapshot selects committed Shared Tooling 0.1.35 at
`be550afa57fe9e16872e5110b5cd69c24b4fa9e8`. Canonical export from a clean
isolated checkout changes seven selected files; all committed bytes and executable
modes match. The new consumer-selected Cargo binary/example installer is optional:
the existing three-tool bundle and aggregate setup/check selections remain unchanged.
Its substitute-Cargo fixtures cover exact profiles, receipts, offline reuse,
corruption refusal, retained failures and concurrent setup.

The reviewed rules permit registry access for authorized dependency preparation;
explicit offline settings remain authoritative. Backup already runs root-workspace
`cargo fetch --locked` during release preflight before offline validation, so no
release adapter change is required. Ordinary checks still install nothing.

Focused installer, command-wiring, release-adapter, ShellCheck, offline installed-tool
and snapshot checks pass on Linux. Upstream's exact 0.1.35 CI is in progress at
inspection; this uncommitted consumer candidate has no matching native CI. Actual
registry binary/example installation is not qualified by substitute fixtures.
Keep the compatible 0.10.1 draft and preserve incoming stage work and dependency
selection. The [review](reports/audits/2026/10/09/shared-035/01/report.md) retains
source proof, check commands and qualification limits. No function, method or type
is removed by this adoption.

### Prior Shared Tooling 0.1.34 review

The prior 95-file snapshot selects committed Shared Tooling 0.1.34 at
`3d33cd250fcae7dbe5cabe44b2abd6b2c91a1822`. Canonical export from a clean
isolated checkout changes eight retained files. The maintainer explicitly selected
retirement of the optional fleet reporter and its regression suite under
[#30](https://github.com/dragginzgame/ic-backup/issues/30): remove both verified
copies and manifest rows, then refresh the narrowed selection. All remaining
source bytes and executable modes match; no vendored helper is patched locally.

Run fleet reports from Shared Tooling. Local `make cloc`, host/IC setup and offline
checks retain their canonical owners. Shared `make cloc-tooling` refuses an absent
selection with its ownership diagnostic; the consumer no longer advertises or
schedules the retired reporter. The measured removed surface is 405 code LOC.
This command change selects the undated 0.10.0 draft from released 0.9.2, without
changing package versions or release metadata.

Focused command wiring, workspace LOC, archive boundaries and ShellCheck pass on
Linux. The exact Shared Tooling 0.1.34 CI is queued at review; this local
consumer candidate has no matching native macOS CI. Preserve the incoming lock's
Host 0.8.7, Metrics 0.2.16 and Testkit 0.25.3 selections. No new runtime offload,
dependency reselection or provisioning handoff occurs. The
[review](reports/audits/2026/10/09/shared-033-issues/01/report.md) separates local
checks from prior released-source hosted evidence and remaining issue acceptance.

### Prior Shared Tooling 0.1.32 review

The prior 97-file snapshot selects committed Shared Tooling 0.1.32 at
`635a39a9dd5f8d021fa9c9196b591e00521a7e02`. Canonical export from a clean
isolated checkout changes eight selected files; the sibling's dirty 0.1.33
dashboard proposals are excluded. All selected bytes and executable modes match.
IC tool reuse now compares the complete validated records across supported hosts,
ignoring comments and row order without rewriting installation pins or receipts.
Changed records, malformed matrices, unsafe links and altered bytes still reject.
Focused installer fixtures and real offline checks of the retained local bundle
pass, including equivalent commented and reordered inputs with unchanged custody.

The baseline clarifies that Shared Tooling owns fleet reports and Metrics owns
measurement arithmetic. No fleet dashboard or sibling scan is added here. The
new Cargo-install assessment is upstream qualification, not a published generic
consumer installer; its scripts/workflow remain unselected. Our native CI already
preserves pushed-commit runs without a cancellation group. Upstream's exact 0.1.32
native CI passes on Linux and both macOS architectures; consumer native CI for
this uncommitted candidate remains separate.

Host 0.8.5's four published archives match their official registry checksums,
clean release VCS identity and committed Rust/package files. All Rust sources
under `crates/` are unchanged from 0.8.4; the release brings the same tooling
reuse fix and no new runtime offload. The incoming lock already selects 0.8.5;
preserve that selection. Focused local IO/custody, spending, stage/planner and
public stage cases pass on the actual graph, along with all three real planned
download Testkit cases, as do both libraries' Clippy/rustdoc
and Rust 1.88/independent consumers. Exact Host 0.8.4 and 0.8.5 native CI now pass
on all three supported hosts. Published Testkit 0.25.3 still lacks the proposed PocketIC provisioning
contract; preserve the functioning six-tool matrix and bundles until the
[coordinated handoff](https://github.com/dragginzgame/ic-testkit/issues/38) is
published and qualified.

The [review](reports/audits/2026/10/09/shared-032-host-085/01/report.md) binds
source, registry, native CI and local check evidence. No dependency reselection,
package/version transaction, schedule, release or sibling edit occurs.

### Prior Shared Tooling 0.1.29 review

The prior 97-file snapshot selected committed Shared Tooling 0.1.29 at
`1a54fb625d6e47efa64c4384808ecbc87be84e7e`. The canonical exporter runs from a
clean isolated checkout, excluding dirty sibling proposals. Fifteen existing
paths change and the release-source checker and two CI-installer files are added explicitly; every source
blob and executable mode matches. The consumer-owned PocketIC 16.1 matrix and
original Host 0.8.2 selections remain unchanged.

Common issue authority now covers only repositories owned by `dragginzgame`.
Other GitHub destinations require explicit authorization for the destination and
intended action; read-only inspection remains allowed. The adopted task prompts
carry that boundary. No issue is posted by this review.

The release adapter delegates dirty-source admission to the shared checker.
Initial preflight allows pending notes only and states that validation/version
preparation have not started when refused. Prepared checks retain exact metadata
allowances and require unchanged original HEAD; committed/resumed checks require
clean source. Actual private-index fixtures verify all three change categories,
quoted unusual names, original Git errors and unchanged index/lock/working bytes.
No function, method or type is removed. Existing receipts, spending and release
recovery remain unchanged.

The delivered 0.9.0 Linux CI uses Ubuntu ShellCheck 0.9 and fails SC2015 in
the unchanged shared checker, while local 0.11 passes. Linux CI now selects the
already-reviewed 0.11 pin through the canonical installer, and local shell-check
prefers a prepared checkout binary. Actual checksum-admitted Linux installation
and shell lint pass. macOS retains its Homebrew setup. Vendored code stays intact.

A concurrent external lock update selects Metrics 0.2.15; retain it. Every
published Rust byte is identical to 0.2.14 and exact committed source/archive
proof matches the non-yanked registry row. Empty host features remain selected.
Fresh Rust 1.88 checks compile both libraries and two independent public consumers,
and all five focused diagnostic/distribution cases pass on this exact graph.
Original tooling proof remains separate; no accounting or metrics schema changes.

Focused Linux release-adapter, runner-simulation, full tooling, shell and Cargo
pin checks pass. The optional npm checker is adopted with its canonical owner but
is not selected by this Rust-only consumer. The sibling dashboard and Shared
Tooling's repository-specific source fixture remain unselected. No schedule is
activated. [The review](reports/audits/2026/10/08/shared-029/01/report.md) binds exact
inputs and retained evidence; native macOS and full CI/release remain separate.

## Earlier selected sources

The earlier 79-file snapshot selects committed Shared Tooling 0.1.27 correction
`db039347d2372b877c1c46dcdd2b5c3aa9412009`. The previous committed consumer
selection omitted the installer suites' new evidence fixture; `make tooling-check`
failed with status 127. Its original log remains retained. Adopt the corrected
companion declarations through the canonical exporter and explicitly add
`test-tool-evidence.sh`, `select-tool-evidence.sh` and the shared failure action.
All selected source bytes/modes match; the incomplete selection now refuses
before manifest replacement. The full `tooling-check` target, shell and dependency
declarations pass on Linux, including the actual collector shell body with fixture
payloads. The shared action is a fixture dependency; consumer CI continues to use
its existing collection roots, pinned uploader and retention policy.
[The repair review](reports/audits/2026/10/08/tooling-companions/01/report.md)
keeps the failure and corrected results separate from earlier focused evidence.

## Consumer-owned IC pins

The current matrix is consumer-owned and outside the immutable snapshot. It
contains exactly the five shared executables, with unchanged official archive
hashes across Linux x86-64, Intel macOS and Apple Silicon macOS. Testkit owns
PocketIC pins and assets; no server row or duplicate client/server policy remains.
Explicit setup selects the new bundle; checks reject old six-tool selections
without rewriting their pins, receipts or binary evidence.

The original matrix-ownership transfer in 0.8.1 and earlier Testkit 0.25.1
qualification retained exact shared pins and server evidence at that time. Those
historical reviews below describe their original inputs, not the current server
selection. CLI and product qualification remain separate from tool provisioning.

## Older snapshots

The earlier 77-file snapshot selected committed Shared Tooling 0.1.27 at
`b866d41041a1986eeec95bde9af4c6ba0853d2e3`. Thirty-two selected paths change;
common baseline/rules remain unchanged. The canonical exporter runs directly
against this repository, preserving the incoming unrelated lockfile update.
Exact source bytes and executable modes match every selected path. A retained
isolated consumer also qualifies consecutive uncommitted refresh and refusal of
edited input without replacing its manifest or unrelated work.

Focused consumer release/validation/snapshot/LOC/hook checks pass under inherited
`CDPATH`. Local entry points use physical absolute roots; release fixtures exclude
inherited Make includes and GNU flags after both original failures were retained.
At that inspection the new shared compact installer-evidence selector was outside
this consumer's selected file set and archive policy. Existing explicit evidence roots and retention
remain selected. Exact upstream 0.1.27 CI and released consumer 0.8.0 CI are still
pending; prior 0.1.26 Intel CI timed out under
[shared #71](https://github.com/dragginzgame/shared-tooling/issues/71).
[The current review](reports/audits/2026/10/08/upstream-refresh/01/report.md)
binds the compatible 0.8.1 draft to its actual inputs and focused Linux results.

The earlier 77-file snapshot selected committed Shared Tooling 0.1.25 at
`eeb72e741199bd8574280eacb3542d8379b912f6`. Exact owner
[CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37762726615) passes
Linux and both native macOS hosts, including hosted evidence round trips.
The canonical exporter explicitly adds the evidence archive helper and regression;
all previous paths remain. With the prior adoption still uncommitted, export ran
in a clean isolated consumer, then exact prior manifest bytes/modes were admitted
before propagating canonical output. No root Git/index or vendored-source patch
was used. Every resulting committed blob/mode matches its 77-file source.

Consumer CI archives its existing explicit fixture/failure roots before the pinned
v4 upload, preserving legal Unix filenames, modes, hidden files and symlinks while
excluding Git metadata. The uploader, artifact name and seven-day retention stay
selected. The canonical archive regression runs in normal tooling and macOS system
Bash checks. Real Linux tar round trips of the actual inline collector cover no
roots, early and late retained failures; actionlint and ShellCheck pass.
[The review](shared-tooling-review.json) binds these inputs. Consumer hosted/native
acceptance remains [#28](https://github.com/dragginzgame/ic-backup/issues/28),
separate from upstream's green run. The same-OID ref-type race remains upstream
#62; no new release/ref authority follows from archiving.

The earlier 75-file snapshot selected committed Shared Tooling 0.1.24 at
`e9bfdc54c0daefc3dbbdfe091e5665dca5468eb3`. The canonical exporter runs from a
clean isolated checkout and preserves the existing selection. Every blob and
executable mode matches source; sixteen paths change. Common baseline/rules are
unchanged. Refresh now checks declared companions before writing any destination,
and the tooling LOC owner includes `bin/` and unborn repositories.

Confirmed direct releases and completed resume refresh the matching configured
upstream when fetch/push destinations agree. The update preserves static symbolic
refs and concurrent changed OIDs; a same-OID ref-type race remains under
[shared #62](https://github.com/dragginzgame/shared-tooling/issues/62). The new
archive helper is deliberately unselected while its path/output admission repair
remains uncommitted under [shared #59](https://github.com/dragginzgame/shared-tooling/issues/59).
Existing direct delivery and consumer failure-collection roots remain selected.

[The adoption review](shared-tooling-review.json) binds focused consumer checks
and exact source evidence under `target/upstream-071-review`. Focused consumer
release-adapter/runner, logger/retention, LOC, exporter/companion, snapshot, shell,
dependency declarations, formatting and documentation checks pass on Linux.
Upstream portable regressions pass on all three native hosts; Intel/Linux complete
jobs and lint/security pass. Apple Silicon's later hosted artifact download fails
under [shared #63](https://github.com/dragginzgame/shared-tooling/issues/63).
Current consumer macOS acceptance remains separate. Published consumer 0.7.0
[CI](https://github.com/dragginzgame/ic-backup/actions/runs/37753291659) passes all
three native hosts for the prior 0.1.23 snapshot. Historical reviews below retain
their original inputs and limitations. This compatible refresh joins the pending 0.8.0 transport hard cut;
package/receipt remain 0.7.0.

The earlier 75-file snapshot selected committed Shared Tooling 0.1.23 at
`0ba0ad00ed94848e54ecc82629b6b7873b7284c0`, matching the remotely reviewed main.
The canonical committed exporter runs from a clean isolated checkout; all prior
paths remain and every exported blob/mode matches the exact remote source.
Seven adopted files change. Common baseline/rules and product policy remain
unchanged. Direct releases recheck payload/index and exact annotated tag after
the final hook, and completed resume verifies observed published identity without
repeating effects. The optional PR helper now aggregates pages with jq instead of
requiring a newer GitHub CLI `--slurp` option.

Focused consumer release-adapter/private-index, canonical direct-runner,
snapshot/logger/retention and shell checks pass. The optional canonical PR fixture
also passes using synthetic Git histories/bare destinations and a GitHub command
substitute; it qualifies the helper, not a real GitHub release or this consumer's
merged-source adapter. Consumer policy remains explicitly direct. Scope and
source identities are in [the review](shared-tooling-review.json) under
`target/shared-tooling-0123-inspection`. Earlier Host runtime evidence remains
bound to its original graph; the observed later Metrics 0.2.10 lock selection is
preserved and needs its own runtime qualification. Package/receipt remain 0.6.0;
this compatible tooling batch joins the existing 0.7.0 draft. No sibling source,
repository Git or release/publication effects run.

Exact upstream 0.1.23 CI passes Linux, Intel macOS, Apple Silicon macOS and
lint/security. This does not supply native consumer proof for the current dirty
0.7.0 graph.

### Retained 0.1.21 review

The earlier 75-file snapshot selects committed Shared Tooling 0.1.21 at
`45e34e92b43edb9543d5b7212774f87f8334079f`, matching remotely verified main.
All 73 prior paths remain; canonical `release-pr.sh` and its fixture are explicitly
added before committed export. Exact blobs and executable modes are independently
checked. The common runner preserves direct recovery and provides an optional
PR release protocol with fresh merged-source qualification; local consumer policy
stays direct. Public release/resume commands reject unsupported delivery before
runner state or Git effects. Ordinary contributions retain their authorized PR
workflow; no actual commit/PR/release action is part of this adoption.

Local direct-runner, adapter/private-index and hook/shell checks pass. Canonical
consumer tooling checks and exact upstream/native results retain actual scope in
[the review](shared-tooling-review.json) under `target/continuation-055`. The PR
fixture ships unchanged, but this direct consumer does not claim its own PR release
qualification. Package/receipt remain 0.5.4 and the current draft is 0.6.0,
selected by the separately reviewed public JSON publication error change.
No sibling edit, package version change or Git/release/publication effect occurs.

Exact upstream 0.1.21 CI passes Linux, Apple Silicon macOS and lint/security.
Intel macOS is cancelled during portable regression, so the owner run does not
establish complete native qualification. No consumer CI result is attributed to
these uncommitted changes.

### Retained 0.1.20 review

The earlier 73-file snapshot selects committed Shared Tooling 0.1.20 at
`3ecc48e579f6cf6e6ab01a6645d8a250fc8c6934`, matching inspected remote main.
A clean isolated checkout exports exact committed bytes and modes; all 68 prior
paths remain. Explicit additions are the contribution rules, shared IC pin parser,
PocketIC alignment/binary checkers and their canonical fixture. The complete
shared governance payload remains readable in the consumer. No shared file is
patched in place and the IC executable matrix stays unchanged.

The reviewed contribution rules now permit scoped commits and branch pushes for
an explicitly requested PR. Ordinary repairs remain local; merging, integration
branch pushes, releases and publication retain separate authority. Local active
prohibitions are reconciled in AGENTS.md and the development/release guides;
[CONTRIBUTING.md](../CONTRIBUTING.md) directs people and agents to the same rules.
This adoption requests no Git or release effects. [The policy review](shared-tooling-review.json)
records the local-fix, PR and selected-release interpretations.

The normal configured gate checks the actual locked PocketIC client against the
reviewed server pin after dependency preparation. This is offline, read-only Cargo
metadata admission through the shared parser; runtime permission, lifecycle and
backend capability qualification remain local. Both existing versions are 16.0.0.
No default endpoint, provider, binary override or implicit installer is added.

Actual consumer tooling, release, hook and shell regressions pass on Linux.
Temporary-path fixtures also pass with trailing-slash and directory-symlink
TMPDIR values. The shared logger retains combined raw failed-target evidence;
Rust setup rejects redirected paths and the canonical selector preserves
historical EOF bytes. Expanded installer/checker fixtures retain their substitute
scope. Exact source [CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37641211708)
passes Linux, both native macOS architectures and lint/security. Consumer native
delivery remains separate, with the published host 0.4.3 Darwin compile blocker
tracked in [Host #18](https://github.com/dragginzgame/ic-host-tooling/issues/18).

Evidence is under `target/upstream-054-refresh`, including exact prior/current
snapshots, exporter proof, source/mode admission and focused results. The same
compatible 0.5.4 draft is maintained; package/receipt stay 0.5.3. Product Rust and
the maintainer's incoming dependency selections are preserved. No sibling write,
commit, release, publication or evidence cleanup runs. [Consumer #23](https://github.com/dragginzgame/ic-backup/issues/23)
owns contribution-policy delivery independently of upstream acceptance.

### Retained 0.1.18 review

The earlier 68-file snapshot selects committed Shared Tooling 0.1.18 at
`a3430b34b32a60f3b245a2b4f7e2f5321556fe56`, verified against remote main.
A clean isolated source exports exact blobs and executable modes; sibling files
are read-only. All 65 previous paths remain, with the linked canister audit
addendum, optional Cargo tool installer and its canonical fixture added.

[The shared Make include](../make/tools.mk) owns common setup/check commands and
LOC reporting. The selected logger rejects options/assignments as validation goals,
retains complete success/failure logs and timing tables when requested, and preserves
Make failures. Shared changelog finalization keeps trailing-whitespace draft notes
attached to their heading. The hook retains exact staged Rust formatting and
conservative symlink handling; its frontend reference adds no frontend to this library.

Canonical LOC fixtures now isolate enclosing Cargo configuration and inherited
target settings. Tooling-LOC consumer tests use adopted working bytes; actual
committed exporter integration stays upstream. This addresses
[Shared Tooling #50](https://github.com/dragginzgame/shared-tooling/issues/50)
and removes our `env -u CARGO_TARGET_DIR` caller workaround. Both consumer LOC
fixtures pass before a commit, with a temporary directory inside the checkout.

The optional `make install-rust-tools` / `make rust-tools-check` commands use the
shared Cargo-sort, derive-sort and Candid-extractor pins. The library's maintained
formatter still needs only Cargo-sort; the extra set is not attached to aggregate
setup/CI, installed implicitly or used to add another formatter. The canonical
installer fixture qualifies locked commands, offline checks and retained failures
using substitute Cargo; it does not prove actual installation/native qualification.

Actual consumer logger/snapshot checks pass on Linux. Release/hook and shell checks
are recorded in [the review](shared-tooling-review.json) under
`target/shared-tooling-052-0118-review`. The exact upstream
[CI run](https://github.com/dragginzgame/shared-tooling/actions/runs/37604299590)
now passes Linux, native Apple Silicon/Intel macOS and lint/security. That exact
upstream qualification does not cover the uncommitted consumer draft, which has no
remote/native delivery result. See
[issue #19](https://github.com/dragginzgame/ic-backup/issues/19).

The compatible 0.5.2 draft retains its product diagnostics and host dependency
batch. This tooling refresh changes no Rust, Cargo manifests/lock or released
receipt. Package version remains 0.5.1. Earlier failed candidate evidence stays
intact; no sibling write, commit, release transaction or publication occurs.

### Retained 0.1.17 review

Committed 0.1.17 at `88f1d70cdf671aefb9507d7a81411ed5daa358b3` matches inspected
remote main. Its only script change from 0.1.16 is the independent LOC fixture:
it now clears inherited target-directory settings and selects exact fixture
manifests even when `TMPDIR` is inside a consumer Git checkout. Both corrections
pass a fresh Linux run with those inherited conditions, without the local caller's
`env -u` workaround. At that inspection the caller kept the workaround for the selected 0.1.15
script; current 0.1.18 adoption removes it.

The fresh exact 66-file candidate verifies all checksums/modes, but the unchanged
tooling-LOC cross-owner fixture still fails on the uncommitted consumer guard path.
The exporter dependency gap from 0.1.16 remains as well. A maintainer reproduction
in [Shared Tooling #50](https://github.com/dragginzgame/shared-tooling/issues/50)
confirms the committed-tree problem even with the exporter explicitly supplied.
That review retained tested 0.1.15 rather than patching shared bytes or committing
a draft to run its tests. The current 0.1.18 consumer fixture resolves the gap.

Candidate/source identities, fresh outputs, failed fixture and the initial
no-checkout destination refusal remain under `target/shared-tooling-052-0117-review`.
Product Rust, Cargo inputs, released receipt and current vendored bytes are unchanged
by this audit. [Exact-source upstream CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37601115116)
was queued at inspection; no new native qualification is claimed. The retained
0.1.16 review below keeps its own original evidence.

### Retained 0.1.16 review

Committed 0.1.16 at `b69507367d45e3db9543359e689e1fcba0467ff4` was reviewed and
exported as an exact 66-file candidate, including its new linked canister audit
addendum. Its logger, changelog whitespace handling, independent-workspace LOC
and optional frontend selection changes are retained under
`target/shared-tooling-052-0116-review`. Rust-only scope needs no npm/Prettier tool.

The new canonical tooling-LOC cross-owner fixture clones the consumer HEAD and
requires committed exporter inputs plus `scripts/distribution/refresh-consumer.sh`.
That exporter is outside the consumer snapshot; an uncommitted guard addition
also fails its Git path selection. The candidate's checksum/governance and prior
fixture stages pass, but the full consumer tooling gate does not. Preserve the
failed fixture and exact candidate without patching shared bytes or committing
source merely to pass a test. That review retained the tested 0.1.15 adoption;
current 0.1.18 resolves the reusable fixture boundary. See
[Shared Tooling #50](https://github.com/dragginzgame/shared-tooling/issues/50).
This is a consumer test-admission gap, not evidence
that the production LOC reporter fails. Current 0.1.16 native upstream CI was
still queued at inspection.

### Retained 58-file adoption

The earlier 58-file snapshot selected committed Shared Tooling 0.1.13 at
`e378671d90afa237ff63a4b0e3b9551eb2c222b6`, matching inspected remote main.
It adds [the Rust workspace rule](../rules/rust-workspaces.md) and refreshes its
baseline, dependency, hook and governance-roster links together. The sole new
selected path is `rules/rust-workspaces.md`. Every roster document and local link
resolves inside the exported consumer.

IC Backup already conforms: the root is virtual with explicit resolver 3, the
sole maintained member is `crates/ic-backup`, metadata and dependencies inherit
from the workspace, and `Cargo.lock` remains at the root. Locked offline Cargo
metadata and a source manifest inventory independently establish these facts.
The inventory excludes Cargo's actual selected build-output directory; retained
scratch projects and historical fixtures remain evidence. No package relocation,
independent workspace or layout exception is needed.

The exporter refused to overwrite the uncommitted 0.1.12 adoption. Its originals
were retained, and a clean isolated checkout exported the reviewed file set into
a disposable consumer. Reconciliation checked every existing destination against
its preserved original, then copied only exact committed bytes and modes. No
vendored patches or sibling edits were made.

Fresh Linux exported-governance/snapshot regressions pass. Locked offline metadata,
formatting, dependency inheritance, document links and exact snapshot checks are
recorded in [the current review](shared-tooling-review.json) under
`target/shared-tooling-050-0113-review`. Upstream's
[exact-source CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37581058940)
passes Linux, both native macOS portable-regression jobs and lint checks. Native
consumer execution remains separate; the earlier released command-custody finding
in [issue #13](https://github.com/dragginzgame/ic-backup/issues/13) is unaffected.

This governance refresh joins the existing 0.5.0 draft without changing package
versions, dependencies, product Rust bytes or the released receipt. The public
repository description remains accurate. Work is uncommitted; no release or live
effect is performed.

### Retained 57-file adoption

The earlier 57-file snapshot selected committed Shared Tooling 0.1.12 at
`33c2a6f0018a94915f819ff219e270500ed5b73b`, matching inspected remote main.
It is part of the existing 0.5.0 draft; package metadata, lockfile and the released
receipt remain 0.4.2. See [the adoption review](shared-tooling-review.json) and
[the current handoff](status/current.md) for exact source and qualification.

The committed exporter ran from a clean isolated checkout of that revision. The
reviewed file set adds only `scripts/distribution/governance-files.txt`; every
listed governance document, including the already selected tag-maintenance guide,
is present and its local links resolve inside a disposable exported consumer.
At that inspection, uncommitted sibling changes, including the proposed Rust
workspace rule, were excluded. The rule is now adopted from committed 0.1.13
above, with the existing package placement preserved.

Snapshot verification hashes inspected bytes without executing the inspected
checksum helper. Consumer fixtures reject helper-only and helper-plus-payload
corruption and prove that the substituted helper never ran. Release dispatch uses
the single captured destination URL, rechecked after validation and before push.
The updated consumer substitute admits the exact URL and branch/tag refspecs;
canonical shared cases cover within-attempt replacement/addition and exact retry.
Existing source, receipt, lost-reply recovery and publication boundaries remain.

The baseline and maintenance rule are refreshed together. Relevant issue creation,
comments, updates, assignment, closure and reopening are authorized across
repositories when justified by evidence. Sibling file edits and release effects
retain their separate authority requirements. Local product, validation and
release restrictions remain in `AGENTS.md`.

Fresh Linux snapshot, release-adapter, shared-runner, hook and shell checks pass.
Upstream's [exact-source CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37511845192)
passes Linux, both native macOS portable-regression jobs and lint checks. This
uncommitted consumer has no native CI result. Released 0.4.2 main CI passes, but
its separate tag CI fails the command-custody test on both macOS hosts; that
finding remains independently owned by
[issue #13](https://github.com/dragginzgame/ic-backup/issues/13).
Source inputs, prior snapshot, export and focused logs remain under
`target/shared-tooling-050-0112-review`. Product Rust source, manifests and lockfile
are unchanged by this tooling batch. No versions, commits, releases or live effects
are performed.

### Retained 56-file adoption

The earlier 56-file snapshot selected committed Shared Tooling 0.1.11 at
`46c02774a8335cb3949d6f04284c4f53375353c1`, matching inspected remote main.
At that inspection, the compatible pending batch selected 0.4.2; package
metadata, lockfile and receipt remained at released 0.4.1. See
[the adoption review](shared-tooling-review.json) and
[the current handoff](status/current.md) for exact local and native evidence.
Uncommitted sibling work is inspected separately and never becomes inherited
policy or a vendored patch.

That review adopted the earlier baseline and maintenance rule together and
retained local product, validation and release restrictions. The current baseline
above governs current issue-reporting authority; this paragraph records only
that earlier adoption.

The shared logger now distinguishes Rust `error::` names and ordinary failure
context from actual diagnostics. The finalizer compares large version components
as exact strings; this repository retains its narrower numeric release bounds.
Both changes retain their canonical shared regressions. The local retention
fixture follows the new first logger case and preserves its exact failure status.

Released 0.4.1's shared formatting checker has passed both native macOS system
Bash 3.2 tooling steps. Its overlapping local refresh, partial-stage and installer
fixtures are removed; the distinct formatter exit-status case remains local.
ARM's subsequent full gate fails in an independent fence-reconciliation fixture,
so passing tooling milestones are not described as complete native CI success.
At that inspection, 0.1.11 upstream CI and the uncommitted consumer's native
checks remained unqualified. Original inputs and fresh focused evidence are retained under
`target/shared-tooling-042-0111-review`.

### Retained 52-file adoption

The earlier 52-file snapshot selected committed Shared Tooling
[`9f8c7c768793f4ce8f25be9e88282c0f63a06e7f`](https://github.com/dragginzgame/shared-tooling/tree/9f8c7c768793f4ce8f25be9e88282c0f63a06e7f),
matching remote main and the clean read-only sibling. `VERSION` is 0.1.7, while
its changelog remains undated and no local v0.1.7 tag was found; the commit owns
this adoption identity. The unchanged Git-object exporter refreshes all declared
files and adds the lockfile transformer, its regressions, actual formatting-hook
checker and the linked tag-maintenance guide. No vendored patch is made.
[The review](shared-tooling-review.json) binds exact files and focused evidence.

Release preparation delegates only the selected local `ic-backup` lockfile version
rewrite to the shared transformer. It checks the child status and complete output
before writing either manifest or lockfile. Selection, original inputs, receipts,
metadata writes and recovery remain consumer-owned. Cargo admits an isolated copy
of the actual original/candidate manifests offline and locked; every external
package entry and all other lockfile bytes remain unchanged.

The hook adapter supplies an ordering-only child manifest and explicitly selected
current Rust, workspace, lockfile and toolchain inputs to the shared checker. It
qualifies actual `fmt`/`fmt-check`, sorting, index refresh/idempotence, lock and
unrelated-edit preservation, partial staging, malformed formatter inputs and
installer conflicts. The existing local cases remain until both native macOS
jobs qualify this replacement, as required by
[the hook rules](../rules/git-hooks.md#installation-and-adoption) and
[issue #5](https://github.com/dragginzgame/ic-backup/issues/5).
The additional exact formatter-exit-status case remains consumer-owned.

Linux transformer, adapter/recovery, hook, snapshot/runner and installer checks
pass, alongside offline installed tool checks, dependency declarations, formatting
and local document links. The configured three-host CI includes the new checks,
with explicit macOS system Bash 3.2 execution. Released 0.3.9 main/tag CI passed
on all hosts; those results do not qualify this uncommitted adoption. Native
macOS replacement coverage remains pending. Evidence and previous files/review
are retained under `target/shared-tooling-0310-adoption`.

Installer status/type failures now reject explicitly. jq/yq and IC versions,
package/lock version, dependencies and MSRV remain unchanged. Ripgrep archive pins
are available through the optional shared interface; our host setup still selects
jq/yq, while system bootstrap supplies ripgrep as described in
[development](development.md#supported-host-scope). The vendored setup guide
also describes upstream's opted-in ripgrep commands. No tag-maintenance helper is
installed or invoked; its guide is retained to keep shared host-document links
complete. The common engineering baseline is unchanged. Optional registry,
RustSec and release-command helpers have no duplicated owner to replace here.
This compatible tooling work extends the complete 0.4.0 draft (originally 0.3.10). No actual release,
Git commit, tag, push, publication, live effect or artifact cleanup ran.

### Retained 47cd2cc adoption

The current 48-file snapshot selects the committed 0.1.7 batch at
[`47cd2ccaf0e8b428f06e6db0262df76cfc1581de`](https://github.com/dragginzgame/shared-tooling/tree/47cd2ccaf0e8b428f06e6db0262df76cfc1581de),
matching remote main and a clean read-only sibling checkout. Upstream's canonical
`VERSION` still reads 0.1.6 and its 0.1.7 notes are undated; this is a committed
source identity, not a new finalized release claim. The exact Git-object exporter
refreshes all declared files; no vendored implementation is locally patched.
[The adoption review](shared-tooling-review.json) binds the files and evidence.

Portable checksum generation now shares one verifier owner with installer
receipts and snapshot export, validates backend output and handles unusual file
names through stdin. IC receipts reject names their line format cannot represent,
and retain failed candidates when traversal or hashing fails. Executable versions,
pins, receipt formats and activation policy retain their existing owners.

The additive `make check-doc-links` uses the exact shared Perl parser with a local
tracked/non-ignored Markdown roster. CI/release gates and native macOS tooling
checks include local target admission and the selected helper regressions. It
checks local file/directory existence, not anchors or remote URLs. Fresh Linux
parser, digest, installer and snapshot/runner fixtures pass; native macOS and
complete configured CI remain separate qualification. No full gate ran locally.

The complete engineering baseline is unchanged. Other new helpers remain optional:
there is no registry-observation or RustSec preparation flow here to replace.
Consumer receipt and original-input checks remain local. The release-command
checker is not installed; this consumer retains its extra completed-resume receipt
check and existing actual-Make fixtures rather than replacing those obligations
with a runner-only fixture. No release/version transaction, publication, Git
commit or cleanup is part of adoption. Extend the compatible pending 0.3.9.

Previous files, manifest and review remain under
`target/shared-tooling-039-adoption/previous`, `previous.snapshot` and
`previous-review.json`, alongside fresh focused logs and upstream diff.

### Retained 0.1.6 adoption

The maintainer authorized adoption of committed Shared Tooling 0.1.6 on 2026-10-06.
That 44-file snapshot selects
[`a37771f1b6b5fc9a88ed6ab3b705bdda35cd8fa3`](https://github.com/dragginzgame/shared-tooling/tree/a37771f1b6b5fc9a88ed6ab3b705bdda35cd8fa3),
matching remote main at inspection and a clean read-only sibling source. The
unchanged Git-object exporter includes all shared audit methods, pinning rules/
checker, parser and IC pins, installers and their focused regressions. Product
records, package/MSRV/lock selections, release commands and receipt owners remain
unchanged. Setup/check targets are compatible additions in pending 0.3.7, under
[#3](https://github.com/dragginzgame/ic-backup/issues/3).

Explicit `make install-tools` prepares pinned jq/yq followed by six IC executables.
Each set activates independently after checksum/version checks; previous and failed
candidates remain retained. Make/CI select local bin paths. `make tools-check` is
offline. CI/release gates also check declarations and scoped existing exact-version
reasons in [the local overlay](../AGENTS.md#qualified-dependency-constraints).
Provisioning adds no transport/provider or IC runtime qualification.

The logger clears checkout/snapshot identity at target dispatch, preserving normal
release selections. Independent fixtures clear inherited Make/release/logger context.
Consumer-owned `scripts/ci/test-release-metadata.sh` reuses existing receipt,
failed-log and real-index cases for the immutable shared regression; Shared
Tooling's changelog-only adapter is not installed. No vendored file is patched.

Actual Linux setup, offline checks, declaration admission and focused shared
tooling/installer regressions pass. Evidence and previous snapshot/review remain
under `target/shared-tooling-037-adoption`; the review record binds file identities
and check scope. Native macOS setup/changed-source CI remains pending. Upstream's
[0.1.6 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37450707625)
subsequently completed successfully; upstream results do not qualify this consumer.
Shared audit methods are available offline; no product audit, report migration or
automatic broad gate was run by adoption.

### Earlier inspected and adopted revisions

A subsequent maintainer-requested update inspection on 2026-10-06 confirmed
remote main at [committed 0.1.5](https://github.com/dragginzgame/shared-tooling/commit/a7efade1a68e43f148252a1a73908a46c4cbe9e9).
That revision was inspected before the current adoption. Its new
dependency-pinning rules require the exact shared declaration checker/jq module,
reviewed parser preparation, exact-scoped compatibility exceptions and CI/release
gate wiring. It also documents locked preparation for every affected workspace
and isolates validation logger checkout/snapshot identity at target dispatch.
Independent fixtures retain their own Make-selection responsibilities.

Executing the exact committed checker read-only against this consumer reports
`command-fds =0.3.3`, `ic-management-canister-types =0.11.0` and
`ic_principal =0.1.5` as requiring compatibility-reason records. Existing source
qualification needs deliberate review under the new exception contract; inspection
does not authorize relaxing requirements or changing locked versions. The published
`ic-metrics` requirement was already compatible rather than exact. At that inspection
no checker, exception or parser installer was adopted. The then-dirty tool changes
were retained separately. Historical evidence is under
`target/maintenance-037-review.B63Hd3`, including
the committed diff, dirty source status, exact checker bytes and its findings.

The inspected upstream repository is
[`dragginzgame/shared-tooling`](https://github.com/dragginzgame/shared-tooling).
The previous committed baseline and tooling revision was
[`cb86188c5956866564de4fb6ec6be67b27981ab9`](https://github.com/dragginzgame/shared-tooling/tree/cb86188c5956866564de4fb6ec6be67b27981ab9),
reviewed on 2026-10-06. A read-only remote query confirmed that exact `HEAD`/`main`;
the read-only sibling checkout was clean. The unchanged upstream Git-object
exporter supplies all twenty-three exact files. The validation runner retains raw
temporary logs if copying them to the configured failure directory fails. Consumer
release admission checks staged and working paths independently, including original
metadata hidden by restored working bytes. Existing candidate/changelog and whole
prepared-index checks remain enforced. Real private-index cases reuse existing
history without creating commits. Conditional Git/formatter substitutes explicitly
reject missing evidence; Linux reproduces the shell condition behind 0.3.5's macOS
completed-tag failure. Native macOS confirmation remains pending.

The release runner reconciles saved intent through normal targets before
selecting another increment. Its late adapters receive `RELEASE_COMMIT`, which may
precede HEAD. Hook refresh preserves formatter failure and canonicalizes checkout
paths. Public commands remain the same; these are compatible recovery corrections.
No vendored file is patched locally. See [the review provenance](shared-tooling-review.json).

The previous `9437bab201bb6071da0bdc4de0336daf553113f5` files, manifest and audit
are retained under `target/shared-tooling-036-review/previous`, `previous.snapshot`
and `previous-review.json`. Current focused Linux tooling evidence remains in that
same directory; snapshot artifact evidence is under `target/snapshot-artifact-review`.

The previous `c0206f1943238e21bd00fbe01658e6a0864c24fa` files, manifest and audit
are retained under `target/shared-tooling-035-review/previous`, `previous.snapshot`
and `previous-review.json`. Those are historical source/qualification paths; native
macOS qualification and complete CI remain separate.

The previous `b8537873ac124ad17b30e32aa23e9006a3e6ec21` files, manifest and audit
record are retained under `target/shared-tooling-refresh.BOQlFZ/previous`,
`previous.snapshot` and `previous-review.json`. That audit's separately hashed dirty
proposals and all earlier evidence remain historical; this revision now commits
the automatic changelog/root dependency rules plus the new hook contract.

The earlier snapshot at `41e1fd0ba41460bd2127cbf98ac8a4b2b2020d3e` and all its
exact files are retained under `target/shared-tooling-review.ztA5F9/previous`;
its manifest is `target/shared-tooling-review.ztA5F9/previous.snapshot`. The
refresh records the same canonical HTTPS source. Product choices stay within
the shared baseline; no exception is claimed for required macOS support,
release ordering, artifact retention or effect authority.

[The snapshot manifest](../.shared-tooling.snapshot) binds the committed source,
exact SHA-256 bytes and executable modes of the vendored principles, consumption
guide, host matrix, checksum verifier, snapshot verifier, validation runner and
runner regressions and common baseline. CI and releases use only these local
files, without fetching Shared Tooling or reading a sibling.

The original adoption pinned tools at
`956236a3848c2cfae6ae05f5c77e9c37b01b3366` and separately reviewed dirty upstream
AGENTS.md bytes with SHA-256
`142be0235013d766114c3036f903f773caabc758e87c2c76d2baa250e2ca1a6d`.
Those are historical source identities, not the current rules. The refresh
retains the previous manifest and files under the evidence directory linked
from [the handoff](status/current.md).

Keep vendored files unchanged. Refresh from a reviewed clean revision using
[the upstream consumption procedure](consuming-snapshots.md), inspect the diff
and rerun affected consumer checks. Keep consumer adapters outside the snapshot;
`scripts/ci/test-tooling.sh`, the Makefile and release helpers are consumer-owned.
Exclude all manifest-declared paths from local banner, navigation and prose
rewrites. Consumer explanations belong here, in [the documentation index](README.md)
or [development](development.md); the manifest continues to bind exact upstream bytes.
Earlier [tooling provenance](tooling-provenance.json) remains historical evidence
for the adapted release/hook helpers, not a Shared Tooling snapshot identity.
The inspected Shared Tooling revision contains no standalone license file; retain
its source attribution rather than assigning it a sibling's license notice.

## Common release workflow adoption

Current adapters verify original validation and receipt source/date/version/member
hashes from the exact selected release commit. Normal targets finish an unchanged
same-kind saved release only; newer committed fixes or another requested kind then
require fresh preflight and complete validation before the next increment. Explicit
resume selects one saved release, with post-completion annotated-tag and original
validation checks even when the shared plan is already complete. No old proof is
rebuilt from current source. Unknown remote history or conflicting intent stops.
See [the maintained release guide](releasing.md) and the adopted
[issue #2](https://github.com/dragginzgame/ic-backup/issues/2).

The initial workflow adoption below records the reviewed 0.3.0 transition.

The maintainer selected pending `0.3.0` because replacing the public release
workflow is breaking before 1.0. Existing pending implementation notes move into
that same entry; package metadata remains `0.2.3`. The public patch/minor/major
entry points invoke the exact vendored runner with explicit branch/remote inputs.
`release-resume VERSION=X.Y.Z` continues its saved source/date/version/destination
and phase. Conflicting selections reject; `make -n` remains read-only because local
recipes omit the upstream example's recursive `+` prefix. The shared runner still
owns all staging, commits, annotated tags and the exact atomic branch/tag push
with `--no-follow-tags`. Consumers retain original metadata, validation evidence
and build artifacts. See [the local release guide](releasing.md).

The obsolete public preparation aliases and separate stage/commit/push workflow
are removed, including arbitrary one-shot `release-x`. There is one current
workflow, without compatibility wrappers. Consumer adapters reuse the existing
v1 package receipt owner and retain original-input validation evidence under the
runner's Git state directory. Bounded package version parsing and exact inherited
member ownership remain unchanged. Preparation edits only the package's lockfile
version; no Cargo update or dependency reselection occurs. Ordinary failures restore
original metadata; interruption evidence/metadata backups remain retained.

Standalone publication still delegates package admission to Cargo, and read-only
version, plan and tag inspection remain available. Resume additionally checks
retained validation/receipt evidence and the exact completed local tag, including
when the shared plan is already complete. Consumer checks use actual Make entry
points with Git/Cargo substitutes; they establish no real release/publication.
Native macOS qualification remains pending. Snapshot integrity, consumer behavior
and actual host/publication evidence are distinct.

The committed rules now require automatic numbered pending headings and an explicit
root Cargo dependency catalog. The selected `0.3.0` remains the next minor from
finalized `0.2.3` because the complete batch breaks public release/setup commands;
compatible slices extend it. All nine direct dependencies already inherit the root
catalog. Cargo-sort 2.1.4 sorts both manifests without changing effective package
metadata, dependency/feature selection or the lockfile. Package version selection
and release transactions remain separately authorized.

`make install-hooks` replaces `hooks-install` and activates the unchanged shared
hook locally. Formatting uses an isolated exact index export, refreshes only selected
files and rejects partial staging while retaining unrelated edits. The member license
is now an identical regular copy of the root notice because the standard hook rejects
symlinks anywhere in its tracked snapshot. The obsolete local installer is removed;
its protected-hook-path obligation is covered by the stricter shared installer.
Both `fmt` and `fmt-check` use the pinned manifest sorter before Rust formatting;
the latter also admits prepared release metadata before staging. Setup installs
tools explicitly and independent CI/release checks never mutate formatting.

## Local choices and evidence

The new [agent maintenance rule](../rules/agent-maintenance.md) defines user-triggered
CI/issue inspection and carries explicit session activation forward. Checks inspect
and report; repairs and GitHub writes retain their separate authority. One current
inspection reviewed all workflows/runs for local HEAD, open issues/discussion and PRs.
The existing description matches the implemented scope; issue #1 remains open with
already-satisfied metadata. No issue was created, commented on or closed.

Backup contracts, v1 records, same-ID/same-release recovery, finite call authority,
target ownership and the targeted-check boundary remain local. Rust stays at
edition 2024, development 1.99.0 and MSRV 1.91.0. The library API, dependency
selection, package version and live-effect authority are unchanged by this tooling
adoption. Its release command hard cut is described above.

The shared baseline and maintainer-provided local tracking instructions require
GitHub issues as the sole record for bugs and follow-up work. For reusable Shared
Tooling feedback, record the reviewed revision, affected
owner/callers/hosts, symptom, focused evidence, smallest proposal and disposition
in the owning repository's issue, then link it from the handoff. Do not create a
competing local feedback list. Standing issue authority covers `dragginzgame/*`
only; verify the owner before writing. Other destinations require explicit
authorization for that destination and action. Upstream file edits remain
separately authorized. This document records adoption, not outstanding issues.

`make shared-tooling-check` verifies the snapshot offline. `make tooling-check`
runs upstream runner cases against the vendored implementation plus local
byte/mode/missing-file/symlink/manifest rejection and evidence-retention cases.
`make release-check` exercises the actual Make gate with substituted Cargo and
checks failed dependency preparation stops later targets while retaining logs.
Fixture-based checks bind their repository, failure-log and GitHub summary paths
locally, including when a parent CI runner supplies a different context. Expected
fixture failures do not alter the parent's retained logs or reported results.
These native tool checks establish no IC behavior. The shared host matrix
describes upstream evidence; [the consumer matrix](development.md#supported-host-scope)
requires macOS 15 on Apple Silicon and Intel alongside Linux. CI covers those
three hosts and separately invokes consumer checks with macOS system Bash 3.2.
Passing native macOS evidence remains pending; Linux checks and configured jobs
alone cannot qualify it. Local release/hook regressions use portable SHA-256
selection and Perl editing; the refreshed verifier handles empty Bash 3.2 arrays.

The stale design-only GitHub description was corrected with explicit maintainer
approval and read back on 2026-10-05: "Host-side Rust foundations for Internet
Computer backup and same-release recovery: local artifacts, journals, plans and
bounded IC codecs. Transport and runners are not yet implemented." Initial
automatic approval review rejected the remote metadata effect under audit-only
authority; the update ran only after the maintainer separately approved its exact
text. No issue, commit, tag, push or package publication was created.

`make ci` preserves target order and stops at the first failure. It adds live
target-labelled errors, a timing/result summary, a GitHub step summary when
available, and full/highlighted failure logs under `target/validation-failures/`.
Consumer build/evidence artifacts survive success, failure and retry.
Full CI remains a separately authorized/configured gate, not routine validation.

## Retained initial Cargo inheritance and version-reader adoption

The initial 0.4.1 snapshot bound all 54 files to committed Shared Tooling 0.1.8 at
`d957d1f8801885c5b69e4a9ef900155f5f2a8a9d`. The refresh used a clean private
checkout after the read-only sibling acquired new uncommitted edits. Those edits
were not copied. [The review](shared-tooling-review.json) retains exact hashes,
previous snapshot/review and fresh Linux consumer evidence.

`make dependency-pins-check` now supplies `--cargo-inheritance`; the same target
already belongs to CI and release validation. Versions and ordinary/dev/build/
target dependencies inherit their Cargo-discovered workspace catalog. Existing
exact-version exceptions remain scoped to their original root declarations.
Nested-workspace discovery grants no governance exception or Canic dependency.

The release adapter delegates TOML projection to the shared offline stable version
reader. It retains release component bounds and owns Git selection: selected commit
reads export exact root/member manifests and their real library target into private
scratch, retaining failed exports. Current metadata never substitutes for selected
original bytes. Helper paths resolve from the adapter, including when inspecting
an older checkout. No dependency resolution, compilation or source mutation occurs.
CI/release run the shared metadata regressions, including aliases, table forms,
bad overrides/catalogs and malformed/failed parsing. Six consumer cases additionally
cover valid comments, selected original versus invalid working files, duplicate
TOML, failed partial output, empty output and failed parsing before any preparation.

Common CI-tool installer entry points are not used here: apt/brew retain that setup,
so no unused installer implementation is copied. Existing hooks keep their required
native replacement qualification. The 0.4.0 Linux CI passed; macOS failed during
host-tool fixtures. New CI failure artifacts preserve temporary fixture/index/log
evidence. Actual archive restoration/native diagnosis remains tracked in
[#9](https://github.com/dragginzgame/ic-backup/issues/9); this adoption does not claim
that native failure fixed or import uncommitted upstream corrections. Formatter and
changelog convergence await reviewed upstream revisions under their owning issues.

## Retained Shared Tooling 0.1.9 adoption

That snapshot bound 56 exact files and modes to committed
`b32d3038c850a7c53470c326b0f7f11263b31669` (0.1.9). Remote main matched the
reviewed clean source; export used clean private source/consumer checkouts to
preserve the earlier uncommitted adoption. No sibling writes or vendored patches
were made. The original 0.1.8 inputs and qualification remain under
`target/shared-tooling-041-review`; the extension and its prior inputs remain
under `target/shared-tooling-041-extension`. See [the current review](shared-tooling-review.json).

The formatter prerequisite has one shared owner, fed by the existing version
pin for setup, `fmt` and `fmt-check`. Current helper/pin files are explicit
formatting-fixture inputs. The adapter delegates changelog selection and heading
rewriting using saved original release identities. It retains independent ledger
corruption checks, source-bound validation, backups, rollback and exact recovery.
Note content remains maintainer-owned rather than an executable release gate.
No functions, methods or types were removed; the existing `changelog` adapter
retains its metadata role while its selection body moves to the shared owner.

Committed host-tool fixtures restore exact archive bytes and check authenticated
payload execution in version/PCRE2 refusals. Installer traces are retained on
failure. Fresh Linux release, hook, metadata, formatter, installer and shell checks
pass; native consumer macOS remains pending under
[#7](https://github.com/dragginzgame/ic-backup/issues/7),
[#8](https://github.com/dragginzgame/ic-backup/issues/8) and
[#9](https://github.com/dragginzgame/ic-backup/issues/9).
Source adoption does not establish that the earlier native failure is resolved.
The complete compatible batch keeps the single 0.4.1 draft and leaves package
versions, dependency selections and the released receipt unchanged.

## Retained Shared Tooling 0.1.10 adoption

The earlier 56-file snapshot bound exact committed
`21f3ec3dd97f2968c9f0b08924451bb2f71770d1` (0.1.10), matching reviewed clean
source and remote main. Earlier dirty work, snapshot/review inputs and source
qualifications remain retained. Export used clean private committed checkouts;
only reviewed snapshot bytes were replaced. No sibling writes or vendored patches.
The latest evidence is under `target/shared-tooling-041-0110-review`; see
[the current review](shared-tooling-review.json).

Applicable dependency, validation-runner and release-runner regression fixtures
now retain original inputs and print retained paths on unexpected failure. The
consumer's actual helpers have focused injected failure/status/input checks through
the existing tooling owner, including the private copied validation helper. Normal
tooling, the common release runner, shell checks and snapshot qualification pass on
Linux. Existing CI artifact retention covers these temporary fixture paths.

The new sccache installer/launcher policy and tag-deletion helper are unused here
and were not added. Nor was the upstream cloc/portable retention test imported:
[0.1.10 upstream CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37491682760)
passes Linux and fails that test on both macOS hosts. Its retained macOS 15 artifact
shows the generated shebang joined to the next statement (`/bin/bashcase`), so the
substitute cannot run. This is evidence for an unused test's fixture-generation
failure, not a failed consumer helper or compiler-cache requirement. The reviewed
adoption is source-bound; consumer native CI remains independently pending.
The complete compatible batch keeps 0.4.1 without package or dependency changes.
