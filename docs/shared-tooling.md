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

The current 119-file snapshot selects committed Shared Tooling **0.25.4** at
`db555ab6cf4a93af9a183f18d921d3b0ee97c725`, verified against remote `main` on
2026-10-20. This is a committed snapshot selection; no GitHub release object was
available. The canonical exporter ran from an isolated clean checkout: 28
payloads updated, eight added and three retired. Old Perl Cargo renderers and
their generic lock fixture are retired upstream; Backup's release adapter now
uses the published IC Build editor and retains its original-input, candidate,
receipt and rollback checks. [The review](shared-tooling-review.json) records
selection and consumer qualification separately.

Shared owns grouped formatting defaults; redundant local Make overrides are
removed. The two dependency-role headers remain inside the single root catalog.
Setup prepares the common Cargo set, locked Testkit and the explicit release
editor through the canonical installer. Offline checks never repair selections;
prior tool generations and failed evidence remain retained.

`repository-files-check` rejects indexed and maintained working-tree links and
checks all three regular MIT license copies. Archive qualification additionally
checks each prepared package's exact license bytes through the shared owner.
Independent-consumer admission retains its transitive simulator/WAT exclusions.
The hosted failure step delegates selector/archive composition to Shared's
collector. Failed selector output cannot select physical payloads; local logs,
redirected parent links and incomplete-collection diagnostics remain retained.
Five cases execute the actual workflow step locally; hosted artifact upload and
download remain separate qualification.
There is no application deployment/artifact-binding gate in this host-library
repository; consuming applications retain that enforcement.

### Prior Shared Tooling 0.24.0 review

The earlier 114-file snapshot selects committed Shared Tooling 0.24.0 at
`cab12cd3dfa4fdb34846b25397f0d72881bb9ae0`, matching remote `main` on 2026-10-18.
The canonical exporter ran from an isolated clean checkout, retaining every old
entrypoint and adding Shared's selected-source Cargo release metadata checker.
The initial closure updated 26 payloads and added four, with no retirements; the
Platform boundary extension below initially added two unchanged upstream helpers.
The subsequent selection review retires unused producer fixtures without changing
the retained payloads. Pending sibling
edits were excluded. [The review](shared-tooling-review.json) records consumer
qualification separately from snapshot provenance.

The root catalog now carries canonical dependency roles. Grouped sorting is used
by write/check formatting, the tracked hook and prepared release formatting.
Committed release admission uses the same grouped ordering on its selected source;
product receipts and exact resume evidence remain local. Shared owns the sorter
and metadata mechanics, not dependency-role decisions or deployment policy.

[The production graph review](development.md#dependency-roles-and-production-graphs)
covers both host libraries on all supported target triples. The existing
independent-consumer admission also rejects `wat` alongside simulator packages.
No application deployment gate or artifact-to-deployment binding exists here;
those remain required at the consuming application's build/deployment owner.
Role comments, snapshot hashes and formatting do not establish that enforcement.
Package versions, the selected lockfile and prior implementation work are retained.

### Prior Shared Tooling 0.23.0 review

The earlier 124-file snapshot selects committed Shared Tooling 0.23.0 at
`0b351cd2ba2bcd8927b783a5fec89afc8013f708`, verified against GitHub's `main`
on 2026-10-17. The canonical exporter ran from an isolated clean clone, retaining
all existing entrypoints. [The review](shared-tooling-review.json) records 21
updated payloads and one new host-platform observer, with no retirements. A second
preview retains all 124 unchanged; uncommitted sibling work is excluded.

The common host/Cargo catalog is unchanged. The IC catalog selects ICP CLI 1.7.0
with reviewed native-host digests; explicit setup precedes offline admission.
Shared owns normalized host observations, validated installer catalog projections,
bounded CI download retries and the pinned 7.0.2 evidence uploader. Backup keeps
its host/Cargo/IC/Testkit setup order, local release policy and exact resume receipt
check. Removed product-specific commands and the separate Make-fixture copier
were not selected or called here, so this closure needs no replacement for them.

Direct Host Artifacts, FS and Process select registry 0.27.1; native file identity
comes from Host FS. The incoming locked Testkit/Candid 0.52 selection is retained,
and simulator startup uses the explicit Spawn variant with the original timeout,
TTL choices and output files. Product record bytes, bounds and recovery custody
remain local. Qualification is recorded in the current review; native macOS and
hosted consumer evidence remain separate.

### Prior Shared Tooling 0.20.0 review

The earlier 123-file snapshot selects clean committed Shared Tooling 0.20.0 at
`3f87ddc1a0c1616775b2114c42a4adee7376c9b1`, verified against GitHub's
`main` on 2026-10-17. The canonical exporter ran from an isolated committed copy;
uncommitted sibling work was excluded. Existing entrypoints are retained.
[The current review](shared-tooling-review.json) records 44 updated and five new
payloads, with no retirements. Host/Cargo and IC catalogs are unchanged.

Common setup/checks now cover host and Cargo tools. Backup explicitly selects IC
setup/admission in its ordered local lists and publishes the prepared IC PATH.
The order is host, Cargo, IC, then locked fetch and Testkit setup; offline checks
follow host, Cargo, IC and Testkit admission. Consumer routing and release fixtures
use that order, and release fixtures retain the shared stable-version companion.
The consumer include no longer exposes the fleet `cloc-tooling` command.

The baseline adds stable-memory allocation governance; Backup remains a host-side
operator library with no production stable-memory manager or store. No new runtime
dependency or product storage change is needed. Shared Tooling owns general helper
regressions; Backup's existing consumer, release and recovery checks remain selected.
The GitHub description matches implemented host libraries and direct Agent transport,
with full runners still pending. Product versions, dependency selections, license
texts, original artifacts and Git index/refs are unchanged by adoption. Linux
qualification is recorded in the review; native macOS and hosted CI remain separate.

### Prior Shared Tooling 0.15.0 review

The earlier 118-file snapshot selects clean committed Shared Tooling 0.15.0 at
`3bef3f2daec7f64f875397ed4d53d5b9e2e68a08`. The canonical refresh adds
release-input, Rust-source selection, tool-evidence and catalog readers with their
documentation companions. The common baseline now reserves local full suites for
explicit release or validation requests; Backup's active guidance uses focused
checks for ordinary handoff. Common host and IC tool selections are unchanged.
The review recorded 33 updated and nine new snapshot paths with focused
consumer qualification.

### Prior Shared Tooling 0.11.2 review

The earlier 109-file snapshot selected clean committed Shared Tooling 0.11.2 at
`81bdf144482b21f146acf542f8013eefe1dfeba8`. It hardens fixture cleanup
failure reporting and anchors temporary roots to their physical location. The
selected fixture callers were refreshed with the common lifecycle helper; tool
pins and Backup's product commands are unchanged.

### Prior Shared Tooling 0.11.1 review

The earlier 109-file snapshot selected clean committed Shared Tooling 0.11.1 at
`7a7430bd30902dc655a025be61696ce19417c54a`. The canonical refresh adds
the shared release-tag checker and fixture lifecycle helper and retires the
ShellCheck forwarding wrapper. Backup already invokes the common CI-tool
installer directly, so no local command migration is needed. Shared release
recovery and standalone fixtures now use the common helpers. The reviewed tool
catalogs, executable versions and digests are unchanged. Backup's local release
adapter, product Testkit extension and exact resume receipt check remain local.

### Prior Shared Tooling 0.10.0 review

The earlier 108-file snapshot selected clean committed Shared Tooling 0.10.0 at
`ed9c26f015691859f67b50b512a94709dcdc0c6c`, matching the sibling's
`origin/main`. The reviewed common host/Cargo and five IC executable selections
match Backup's former local catalogs exactly. Both catalogs are now immutable
snapshot companions, and common Make, standalone installers, formatter checks,
freshness inspection and the evidence action select those files. Backup retains
compiler, product dependencies and Testkit/PocketIC selection locally. No tool
version or digest changed; explicit setup and native consumer qualification remain
separate from the snapshot refresh.

### Prior Shared Tooling 0.9.13 review

The earlier 107-file snapshot selected clean committed Shared Tooling 0.9.13 at
`35cad65677ff989b5fb24a88d947fda37d2366e8`. The canonical preview
adds the directly referenced `rules/change-reporting.md` and the required
Cargo-tool cache-key helper. The new baseline requires scoped change metrics
and permits separate `cargo fetch --locked` preparation for existing locked
sources; tool installation and release effects retain their own authority.
Installer failures retain their native status and evidence. The committed
source was isolated in a clean local clone because the sibling checkout has
uncommitted 0.9.14 work. Its recorded `origin/main` matches the selected
commit; live GitHub verification was unavailable during adoption.

### Prior Shared Tooling 0.9.0 review

The earlier 105-file snapshot selected clean committed Shared Tooling 0.9.0 at
`aec7a149f581a42b9531fc747b46024fc0314fcf`. Canonical refresh updates
four selected files without adding or retiring payloads. Snapshot refresh and
integrity verification share one manifest reader, which rejects NUL bytes and
offers a syntax-only `--read-manifest` mode; the local LOC helper adds complete
JSON output without changing its human-readable report. Existing v1 manifests,
consumer command routing, product APIs and tool pins remain unchanged. The local
source's recorded `origin/main` names this commit; remote verification was
unavailable during this adoption because GitHub DNS did not resolve. See
[the current review](reports/audits/2026/10/12/shared-090/01/report.md).

### Prior Shared Tooling 0.8.3 review

The released 0.20.2 snapshot selected clean committed Shared Tooling 0.8.3 at
`8cca7e67b03ce6e701b6e7dcec2e9753eb98337e`, verified against remote
main. Canonical refresh updated eight selected files without adding or retiring
payloads. Formatting admission reads the selected Rust tool catalog;
Cargo-tool setup rejects redirected build-observation files before writing
diagnostics, while a successful ripgrep PCRE2 probe stays quiet. The shared
pin selects `cargo-sort-derives` 0.14.1. The source commit subject says
“0.8.2”; its committed `VERSION`, changelog and canonical export identify
0.8.3. See [that review](reports/audits/2026/10/12/shared-083/01/report.md).

### Prior Shared Tooling 0.8.2 review

The earlier uncommitted 0.20.2 draft selected 0.8.2 at
`aaf75a37717357dd62f4b0b15001eaa3ba1629cf`. Its 105-file export retained
the 102 prior files and explicitly added the Cargo tool freshness task, checker
and fixture; the default refresh alone would have omitted the linked task.
The consumer-owned `make check-cargo-tool-versions` is an online read-only
inspection, not installation or offline byte/build admission. See
[that review](reports/audits/2026/10/12/shared-082/01/report.md).

### Prior Shared Tooling 0.8.1 review

The released 0.20.1 consumer snapshot selected 0.8.1 at
`c97f9f9c7b7c706fffdcfc5925cf58759b58093f`, verified against remote main.
Its 102-file export updated five selected paths without retiring any. Snapshot
verification rejects malformed tab-separated fields; Cargo tool installation
retains both exact build-environment observations when activation refuses a
changed identity. Host 0.14.5, Testkit 0.34.3 and Metrics 0.7.1 remained the
selected graph. See [the focused review](reports/audits/2026/10/12/shared-081/01/report.md).

### Prior Shared Tooling 0.8.0 review

The earlier uncommitted draft selected committed 0.8.0 at
`045a6f22a9de497bfb3af90cb2571957384806f2`. Its canonical export recorded
the 102 reviewed payloads as initial entrypoints and updated four selected
paths. All 16 registered `make ci` targets passed on the Host 0.14.5, Testkit
0.34.3 and Metrics 0.7.1 graph. That exact qualification remains in
[the earlier review](reports/audits/2026/10/12/continuation-0201/02/report.md).

### Prior Shared Tooling 0.6.6 review

The earlier uncommitted 0.20.1 phase canonically selected 0.6.6 at
`a15eb9d406d287697df9bc8fa96db8cda6721a71`: three changed selected
paths and the same 102-file roster. Its common formatter pin moved to 0.14.1;
explicit tool setup, `$crate` probes and the full consumer gate passed with
Testkit 0.34.2. That phase's [review](reports/audits/2026/10/12/continuation-0201/01/report.md)
retains the original graph and failed-attempt evidence. A later external lock
change selected Testkit 0.34.3, followed by the 0.8.0 snapshot adoption.

### Prior Shared Tooling 0.6.5 review

The prior 102-file snapshot selected committed Shared Tooling 0.6.5 at
`42f2091768b5f71c98a684d3dab563f408ca2dd9`, verified against remote main.
The clean canonical export updates nine selected paths without changing the roster.
Backup declares the root manifest and lockfile at the existing CI stage loop;
changed/unreadable inputs stop subsequent dispatch and retain identities and logs.
The release fixture exports the runner's required checksum companion. Exact staged
hook context supports prepared receipt checks while preserving indexed pins/compiler
and refusing setup in the export. Existing product prerequisites and exact resume
receipt checks remain. Common tool pins/build identities are unchanged.

Focused consumer checkpoint, tooling and hook tests pass on Bash 5 and genuine
Bash 3.2.57 on Linux. The separately requested four Host crates select 0.14.3;
verified archives match committed producer source, with unchanged Rust and only
normalized package-version changes. Root package versions stay 0.19.2; existing
breaking Metrics notes remain 0.20.0. Native/full-gate evidence is owned by
[that review](reports/audits/2026/10/12/shared-065/01/report.md).

### Prior Shared Tooling 0.6.4 review

The prior 102-file snapshot selects committed Shared Tooling 0.6.4 at
`6fc8eb9a94f83fb0b9d0a26377d4232508cdbe2d`, verified against remote main.
Canonical clean preview/export updates eight selected files and adds the reviewed
`make/consumer.mk`, `make/release.mk` and `docs/make-interface.md` companions.
The [common Make interface](make-interface.md) replaces separate includes and
equivalent local hook/snapshot/release recipes. Ordered Testkit setup/admission,
direct-only delivery and mandatory exact resumed-receipt verification stay local.
The actual isolated fixture exports include the complete closure. Default help,
formatting, hooks/index custody, Testkit routing and release/recovery checks pass;
actual Testkit and hook fixtures also pass on genuine Bash 3.2.57. Every registered
full `make ci` target passes on unchanged inputs and the incoming Host 0.14.2/
Testkit 0.34.0/Metrics 0.6.6 graph, including Rust 1.88 consumers and packaging.
Product APIs, pins and tool build identities stay unchanged; no reinstall is needed.
Notes remain compatible 0.19.2 and package versions stay 0.19.1.
Exact released Backup 0.19.1 CI was cancelled after Linux success; both macOS
jobs were cancelled. Candidate and producer native acceptance remain separate.
See [the current review](reports/audits/2026/10/11/shared-064/01/report.md).

### Prior Shared Tooling 0.6.3 review

The prior 99-file snapshot selects committed Shared Tooling 0.6.3 at
`4fd979d13c32adb3cb2f51f213ebfc4f98339b3d`, verified against remote main.
Clean canonical preview/export changes eight selected files, with no roster change.
Normal direct release retries can archive an unstaged prepare intent after a clean
descendant commit; current base/candidate metadata determines the next requested
increment, with fresh preflight and full validation. Backup preparation only changes
local metadata and retains original backups. Exact resume and later phases keep
their original identity; no real recovery is executed during adoption.

Delivery reconciliation records exact consumer/Shared commits and required native
CI before closing adoption issues or updating fleet trackers. Pending or cancelled
checks leave acceptance open. The incoming Host Artifact/FS 0.14.0 graph retains
unchanged Rust implementation sources but changes the exposed publication-error
identity, selecting pending 0.19.0 notes. Packages remain 0.18.4.
All 16 delivery targets pass on unchanged inputs, including real simulator
suites, strict Clippy, independent Rust 1.88 consumers and paired archives.
The external Host 0.14 publication-error consumer also passes without dev
unification. Source/graph, real index and HEAD remain unchanged. Released 0.18.4
passes Linux; its macOS checks and candidate native acceptance remain pending.
See [the current review](reports/audits/2026/10/11/shared-063/01/report.md).

### Prior Shared Tooling 0.6.2 review

The current 99-file snapshot selects committed Shared Tooling 0.6.2 at
`7f1ca16b0dfa98021d2671017ecb7de767409b0e`. Clean canonical preview/export
updates two guides and the release runner/fixture from qualified 0.6.1. Source
conflicts now retain exact plan/phase/version/commit diagnostics; guarded recovery
requires maintainer authority and positive consumer proof that preparation never
began. Audit/Machete guidance separates standalone arguments and inherited Cargo
launcher context. Backup has no Audit/Machete extension target. The Cargo lock fix,
tool identities, product graph and library behavior remain unchanged. Reuse the
passing 0.6.1 full gate; all five affected 0.6.2 checks pass. A later external
Host 0.13.5 lock refresh has unchanged Rust sources and receives all nine
graph-dependent checks on frozen final inputs. The three phases cover all
16 delivery targets; source, final graph, index and HEAD remain unchanged.
Pending notes remain compatible 0.18.4 with package versions at 0.18.3.
See [the current review](reports/audits/2026/10/11/shared-062/01/report.md).

### Prior Shared Tooling 0.6.1 review

The current 99-file snapshot selects committed Shared Tooling 0.6.1 at
`731909525038c58d6f80b498e80f8630b11c685d`, verified against remote main.
Clean canonical preview/export updates four selected files with no roster changes.
Cargo setup explicitly propagates source preparation, lock, validation, reference
and activation failures, preserving the original status, previous common selection,
foreign locks and failed evidence. Activation requires acquired checkout/store
locks. Padded build counts receive portable numeric checks. Existing tool command
paths, build identities and product contracts remain unchanged. Explicit setup,
offline admission, actual Linux cross-checkout reuse, the new failure regressions
and all 16 delivery targets pass on frozen inputs. Candidate macOS acceptance
remains separate.
Compatible pending notes remain 0.18.4; package versions remain 0.18.3.
See [the current review](reports/audits/2026/10/11/shared-061/01/report.md).

### Prior Shared Tooling 0.6.0 review

The current 99-file snapshot selects committed Shared Tooling 0.6.0 at
`75180980a0c4e4252d8a915aaecfaefd92bf9a4e`, verified against remote main.
Clean canonical preview/export adds the required `cargo-tool-build.sh` companion;
16 existing selected files change from 0.5.0. Cargo tools now reuse shared
generations under exact compiler/native/package/packaged-lock identities and a
standard build environment. Application flags and wrappers stay local. Existing
common and selected command paths remain; explicit setup and offline admission
are required. Testkit still owns server selection, local storage and lifecycle.

CI checks actual common/selected Cargo reuse offline with different application
flags. Canonical evidence selection owns local Cargo records and referenced shared
generations, including admitted local selections on rejected store configuration.
The local collector preserves failure status after archiving those selections;
the shared action now uploads partial evidence before its completeness refusal.
Preserve original generations and failed attempts. Library contracts remain
unchanged, so pending notes select compatible 0.18.3; package versions stay 0.18.2.
See [the current review](reports/audits/2026/10/11/shared-060/01/report.md).

### Prior Shared Tooling 0.5.0 review

The prior 98-file snapshot selects committed Shared Tooling 0.5.0 at
`a306180ce534227fcb98b882fba9a526d5d747e0`, verified against remote main.
Clean canonical preview/export excludes the sibling's uncommitted 0.5.1 work.
It adds the explicit `tool-store.sh` and sourced `test-tool-store.sh` companions;
12 existing selected files change from 0.4.2.

Host and IC setup now reuse authenticated shared bundle generations, with
unchanged repository command paths and independent pinned selections. Cargo
slots and Testkit storage remain local. Old physical local bundles remain
retained but are not active fallback installations. Local CI selects one isolated
store consistently, checks actual native reuse without downloads and delegates
shared evidence selection to the canonical owner. No store is reimplemented,
no payload is manually moved and no sibling checkout is used for tools.
The local CI collector archives independent local logs before shared selection,
retaining diagnostics and the original rejection status when shared collection
fails. The canonical shared action remains unchanged; its separate retention gap
is tracked by [upstream #121](https://github.com/dragginzgame/shared-tooling/issues/121).

The library contract is unchanged, so the pending notes remain compatible
0.18.2. Package versions and Git/release state remain user-owned. See
[the current review](reports/audits/2026/10/11/shared-050/01/report.md).

### Prior Shared Tooling 0.4.2 review

The prior 96-file snapshot selects committed Shared Tooling 0.4.2 at
`8db24059613c63ba9be36d4225fb74f598f49df2`, verified against remote main.
The canonical exporter preview confirms the existing roster needs no companion
additions; a clean isolated checkout supplies all adopted bytes. Fifteen selected
files change from released 0.3.11; two policy documents advance from the
qualified intermediate 0.4.1 export. Common Cargo tools now reuse the selected-tool receipt/byte owner
and activate only a complete set at `.tools/rust/common/bin`. Local CI, hook
fixtures, shell guidance and failed-validation archives follow that contract.
Old installations and failed attempts remain retained. Explicit setup is required;
offline checks neither provision nor repair tools. Selected Testkit CLI paths and
product dependencies remain unchanged. The 0.4.2 policy assigns future per-user
executable storage to Shared Tooling; that store is not implemented. Follow the
current canonical installers and retained `.tools` paths rather than adding a
consumer store or relocating binaries.

The clarified baseline keeps Git/release execution user-owned and classifies
compatibility at the published package boundary. This contributor-only adoption
selects compatible 0.18.2 notes, without changing package versions or the library
contract. See [the current review](reports/audits/2026/10/11/shared-042/01/report.md).

### Prior Shared Tooling 0.3.11 review

The prior 96-file snapshot selects committed Shared Tooling 0.3.11 at
`db143ae9ae140eb66826c413360723d23c078f66`, matching remote main. Canonical
export from an isolated clean committed checkout preserves all existing paths and
excludes the sibling's uncommitted 0.3.12 work. Seven selected files change: Cargo
probes/install/check effects share the selected consumer's directory, host-tool
fixtures normalize physical temporary roots and validation summaries classify
errors consistently. The scoped derive-macro protection guidance applies only to
actual dollar-prefixed derive definitions; Backup currently has none. New upstream
release/frontend fixtures remain unselected; they are not consumer qualification.
The current Host/application-fence batch reuses unchanged product qualification
and reruns affected consumer tooling checks. See
[the current review](reports/audits/2026/10/11/continuation-0180/01/report.md).

### Prior Shared Tooling 0.3.8 review

The prior 96-file snapshot selects committed Shared Tooling 0.3.8 at
`67285b28a98b7c4211ad32de726709d4e87edea4`, verified against remote main and
adopted through canonical clean export. New coordinated-archive guidance matches
Backup's existing paired-package check: Cargo admits both archives, an isolated
consumer selects their exact unpacked paths and unchanged normal dependency graph,
then builds offline/locked and verifies original bytes. Publication retains ordinary
Cargo admission; versions and product manifests are not patched for this check.
Only reviewed committed bytes are exported; the dirty sibling changelog is excluded.
The common baseline requires routine
CI to cancel superseded runs per workflow/ref; Backup's workflow already preserves
the complete host matrix under that group. Cancelled jobs supply no qualification.
The shared runner fixture isolates inherited log/summary paths; formatter fixtures
require explicit completion. Failed Git index-tree observations stop the hook even
when stdout looks valid. The shared Cargo installer now owns exact registry package
selection from `--lockfile Cargo.lock`, including source, ambiguity and drift refusal.
Backup removes its Cargo metadata/JQ selector; local fixtures retain Make routing,
offline refusal and ordered setup ownership. No validation command installs tools.
Mandatory assertions now fail explicitly on Bash 3.2, including in local tooling,
Testkit, hook and release fixtures. A contradicted local command result verifies
failure status and retained original evidence through the actual fixture.

Consumer-owned IC pins select Binaryen 133 with exact committed archive hashes.
That adoption qualified original/optimized Wasm and optimizer identity through
both simulator suites. Current suites install WAT-generated fixture bytes directly;
Shared Tooling owns generic optimizer qualification. Earlier tools, server
installations and failed evidence remain retained. The incoming graph selects
Host 0.12.8, Testkit 0.33.1 and Metrics 0.5.5. The 0.17.0 draft is required by the
public async upload hard cut; package/receipt remain released 0.16.0.
See [the current qualification](reports/audits/2026/10/10/continuation-0170/01/report.md).

### Prior Shared Tooling 0.3.3 review

Released 0.14.2 adopted the same 96-file roster at committed Shared Tooling 0.3.3
`d63f0cfaba8ab2961d6012064adbf051c1898bc1`. Its bounded nesting-depth admission,
runner completion and consumer-owned fixture isolation remain qualified under
[the historical graph](reports/audits/2026/10/10/shared-033/01/report.md).

### Prior Shared Tooling 0.3.2 review

The prior 96-file snapshot selects committed Shared Tooling 0.3.2 at
`c16444bf006f17c5bb4dda5ad070a0f345da9623`, verified against remote main.
Canonical export from an isolated clean committed checkout preserves local product
work and adds the advisory README freshness task required by the maintenance catalog.
Setup checks platform and Rust/Cargo prerequisites before downloading; offline
diagnostics identify exact tool/path/version and repair commands. Shared Cargo
formatting/setup/check recipes preserve Make jobserver descriptors and standalone
tool includes retain execution-mode refusal. Fixtures require explicit completion
before reporting success or removing evidence, including premature zero exits.
Local Cargo/Testkit/release recipes and local fixture boundaries use these same
contracts. README setup now uses the complete existing `make install-tools` owner.
The compatible 0.14.1 draft keeps package versions unchanged. See
[the initial adoption review](reports/audits/2026/10/10/shared-032/01/report.md) and
[final Testkit 0.32 qualification](reports/audits/2026/10/10/shared-032/02/report.md).

### Prior Shared Tooling 0.3.0 review

The prior 95-file snapshot selects committed Shared Tooling 0.3.0 at
`88a73139a0f083344c41a6f6f4b5c3a8aca7dc1d`. Canonical export excludes the dirty
upstream sibling reporter. The complete shared host/IC/Cargo roster is mandatory;
ripgrep/cloc flags and extra Rust aggregate prerequisites are retired. Backup
registers locked fetch then Testkit setup in the ordered local installation list,
and Testkit admission in the local check list. Release preflight and CI use these
aggregates; CI no longer separately installs Cargo-sort or repeats Testkit setup.
Select 0.14.0 because the aggregate command contract changes after released 0.13.0.
Earlier tools, snapshots, receipts and failed evidence remain retained.
The repository description remains accurate. Consumer qualification is recorded
in [the adoption review](reports/audits/2026/10/10/shared-030/01/report.md).

### Prior Shared Tooling 0.2.14 review

The prior 95-file snapshot selects committed Shared Tooling 0.2.14 at
`fd11692f31e7dfd44dcc2ca56634eaeab3569825`. Canonical export adopts selected
Cargo-tool diagnostics, original-release CLI setup before validation and early
offline admission before dependent qualification. Backup routes these through
its existing Testkit setup/check owners, with real parallel Make dependency edges.
The baseline now requires the full documented suite before completed code delivery
and confines removed-symbol inventories to the final Codex response. Shared 0.2.14
also requires missing failed-step logs to remain an explicit diagnosis gap. The
optional upstream CI observer is not selected locally. Repository description
remains accurate: full runners are not yet implemented.

### Prior Shared Tooling 0.2.12 review

The prior 95-file snapshot selects committed Shared Tooling
`a8ba9b461b831846eacf64452e6ddcd2acd000f1`, whose committed version annotation
is 0.2.12. Canonical export preserves the exact roster; the annotation is source
metadata, not release/tag proof. The common baseline now rejects LF/CR directory
names, including ancestors and resolved aliases, while preserving invalid historical
evidence. The verifier/exporter refuse those paths before command substitution can
trim them. Make admission separately probes computed `MAKEFLAGS` and retained
`MFLAGS`, rejecting attempts to erase the latter. Cleared/replaced MAKEFLAGS cannot
hide unsafe recipe modes. Default help, normal parallel/recursive commands,
formatter order and specialized release-resume checks remain. Focused consumer
Make/path, hook/release/Testkit fixtures pass; detailed evidence is in
[the allocation continuation review](reports/audits/2026/10/10/continuation-0122/01/report.md).

### Prior Shared Tooling 0.2.10 review

The prior 95-file snapshot selects committed Shared Tooling 0.2.10 at
`43a0dc46cdc3c77e70a68e192561642ed50a3e0f`. Canonical refresh adds the explicitly
selected `scripts/ci/run-formatting.sh` companion and records the source version.
The existing formatting include prints one success line and retains complete
stdout/stderr on failure, with unchanged status, order and prepared-tool admission.
Local hook/release/Testkit fixtures include the helper; the consumer CI collector
retains its `formatting.*` logs from RUNNER_TEMP and TMPDIR. Preserve default help,
Make execution admission and the specialized post-run release-resume check.
The reviewed snapshot also refreshes shared evidence/guidance without selecting
an unused registry observer or changing tool pins. Detailed qualification is in
[the continuation review](reports/audits/2026/10/10/continuation-0121/01/report.md).

### Prior Shared Tooling 0.2.8 review

The prior 94-file snapshot selects committed Shared Tooling 0.2.8 at
`b2646cde9abbc8861857a4379c683a0c19eba43e`. Canonical refresh from a clean
isolated checkout adds the declared `make/execution.mk` companion and refreshes
hook adoption and release guidance. The parse-time owner rejects Make modes that
skip recipes or hide failures, resolves its probe beside the selected include,
and accepts recursive Make commands carrying arguments. Preserve harmless default
help, pinned formatter admission and specialized post-run release-resume checks.
Local hook, release and Testkit-routing fixtures export the new companion and its
existing probe. No tool pin, hook activation or dependency reselection changes.
The sibling's uncommitted 0.3.0 work is excluded. Focused review and exact provenance
are retained in the [review](reports/audits/2026/10/09/continuation-0116/01/report.md).

### Prior Shared Tooling 0.2.6 review

The prior 93-file snapshot selects committed Shared Tooling 0.2.6 at
`ce13a5314916891fd239d9b199b4a91b04775054`. Canonical refresh from a clean
isolated source adds `make/rust-format.mk` and refreshes consumption/release/hook
guidance. Replace the local formatting recipes with that include; keep pinned
offline admission and portable checkout-local Cargo lookup. Consumer hook,
Testkit-routing and release fixtures export the new companion too. Owner format/
hook suites and actual consumer fixtures pass independently. The baseline and
tool pins are unchanged; no hook activation or dependency reselection occurs.

The optional `make/release.mk` is not selected: this consumer's `release-resume`
must run its exact receipt/tag/saved-validation check after the shared runner.
The include's prerequisites-only composition cannot retain that post-run step.
Shared 0.2.6 explicitly permits specialized direct runner routing with required
post-run checks preserved. Keep the existing specialized route without a wrapper.
Exact provenance and focused evidence are retained in the
[review](reports/audits/2026/10/09/continuation-0114/01/report.md).

### Prior Shared Tooling 0.2.5 review

The prior 92-file snapshot selects committed Shared Tooling 0.2.5 at
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

## Shared IC pin history

Before 0.10.0, the matrix was consumer-owned and outside the immutable snapshot. It
contained exactly the five shared executables. Binaryen selected the committed
133 archive hashes across Linux x86-64, Intel macOS and Apple Silicon macOS; the
other four tool rows retain their earlier pins. Testkit owns
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
three hosts and runs the full consumer gate with macOS system Bash 3.2.
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

## Platform dependency boundary

The same committed 0.24.0 exporter additionally selects
`scripts/ci/check-cargo-dependency-boundaries.sh` and its regression fixture.
The preview added only those two payloads; none of the preceding 128 changed.
`make dependency-boundaries-check` reads Backup's explicit package-identity policy
in `ci/cargo-dependency-boundaries.json`; CI and therefore release validation run
it. The generic real-Cargo alias/build/dev/target and failed-observation matrix
remains with Shared Tooling. Backup runs the canonical checker against its actual
workspace and policy. No shared payload was patched locally.

This gate checks workspace direct declarations, including inactive target and dev
edges. It does not restrict transitive dependencies, interpret feature selections,
inspect generated Rust paths or automatically recognize future CDK package names.
The [Platform adoption review](ic-platform.md) records those separate inspections.


### Consumer qualification selection

The repeat architecture review traces all selected test entrypoints through local
Make, CI, release, hook and fixture callers. Fourteen producer test roots supply
no distinct consumer proof: thirteen have no executable local caller, and the
generic boundary-checker suite duplicates its producer's qualification. The
canonical exporter retires those roots and two unused companions, reducing the
closure from 130 to 114 unchanged payloads at the same 0.24.0 revision. The
[selection receipt](shared-tooling-review.json) records every retired path. Earlier
adoption sections describe their original selections, not the current roster.

Backup retains snapshot integrity, its real dependency policy check, independent
production consumers, prepared native tools and product qualification. It also
retains the shared fixtures called by its local failure-retention checks and
release adapter; their source names are selected dynamically in `test-tooling.sh`.
The actual Make/Testkit routing, hooks, exact release resume and evidence tests
remain local obligations. This follows the
[consumer qualification contract](consuming-snapshots.md#consumer-qualification-selection);
no helper implementation, tool pin or product dependency changes. No gate-duration
improvement is claimed.
