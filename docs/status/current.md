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


# Current handoff — 2026-10-16

## Pending 0.27.0 settlement identity and upstream copy/fixture preparation

Lifecycle, capture, allocation and upload-data settlement share a private pure
identity checker for authority, original attempts and challenge. Association and
inventory admission retain their original ordering; domain evidence checks and
public typed errors stay with each caller. All 57 focused settlement cases,
including simultaneous identity/evidence denials without spending changes, pass.
Strict all-target Core Clippy, actual Rust 1.91 library checking, formatting and
Cargo inheritance pass.

[Host PR #165](https://github.com/dragginzgame/ic-host-tooling/pull/165) at
`9ac7503` extracts create-new descriptor copying into Host FS while Backup keeps
directory digest framing. [Testkit PR #109](https://github.com/dragginzgame/ic-testkit/pull/109)
at `47e21f5` exposes the existing authenticated server assets owner to Rust;
the CLI delegates and owns signals. Both contributions are isolated, committed
and submitted; package versions and normal sibling checkouts are unchanged.
Host's three copy cases pass on current Rust and actual Rust 1.88. Testkit's
seven provisioning, one CLI unit and three focused CLI setup cases pass, as do
Rust 1.91 checking, selected strict Clippy and warning-denied rustdoc for both.

The isolated `/tmp/backup-next-consumer` adapter passes ten artifact cases, the
complete Core allocation/upload/load/start and Agent capture/transfer/upload/
restore journeys on managed PocketIC 16.1.0, and strict all-target Core/Agent
Clippy. It retains server digests, original/optimized Wasm, private logs and app
safety. Path-free adoption patches are retained under
`/tmp/backup-three-next.0z997hto/`; both pass apply-check. The normal Backup
manifest and lock keep published selections until the APIs are published.
[Backup #54](https://github.com/dragginzgame/ic-backup/issues/54) and
[#55](https://github.com/dragginzgame/ic-backup/issues/55) own consumer adoption.
Canic's existing public staging adapter remains valid. Upstream native CI is
running; native Backup macOS acceptance and full local `make ci` remain separate.

## Pending 0.27.0 simplification

The generic operation-bound restore-copy staging, publication and verification APIs
and their exports/tests are removed. No Rust caller was found in the inspected
sibling source repositories. IC upload preparation continues reading verified
original snapshot artifacts directly. Existing copies, contracts and provenance
remain evidence; settle unfinished copy-based restores with the original released
tool before adoption or obtain explicit safe disposition from the integration owner.
The public API cut promotes the combined pending notes to 0.27.0; package versions
and release metadata remain unchanged.

Download and restore now use the existing qualified Applied receipt owner alongside
metadata and uploads. The owner no longer imposes stage admission: each caller
retains its original domain-check ordering, including download append before receipt
recording and restore source/requirement checks before and after recording.
The 112 workflow-related unit cases, the remaining public local-source recovery case,
one actual PocketIC allocation/data-upload/load/start journey and all four
lost/malformed load/start journeys pass. Strict all-target Core Clippy,
warning-denied Core rustdoc, formatting, Cargo inheritance and doc links pass.
No full `make ci` or native macOS qualification was run.

The Host Process custody contribution is committed at `c692826` in the isolated
`/tmp/ic-host-backup-custody` checkout. It passes all 118 all-feature Process tests,
its doctest, seven custody cases on Rust 1.88, strict all-target/all-feature Process
Clippy, warning-denied rustdoc, formatting, Cargo inheritance and doc links.
The isolated Backup adapter at `/tmp/backup-custody-consumer` passes ten
custody/model/reference-release cases and strict all-target Core Clippy.
The reviewable adoption patch is retained at
`/tmp/backup-simplify.ok9k0m7r/custody-adoption.patch`; it contains no temporary
path or lockfile overrides. Backup keeps the published registry graph and local
custody implementation until Host publishes the new API. Native macOS acceptance
remains separate. The contribution merged as `2307c39` through
[Host PR #164 — Share inherited command custody with Backup](https://github.com/dragginzgame/ic-host-tooling/pull/164).
[Exact-source Host CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38903068634)
passes Linux, Intel macOS, Apple Silicon and MSRV for `c692826`.
[Backup #53 — Adopt published Host Process inherited command custody](https://github.com/dragginzgame/ic-backup/issues/53)
owns the remaining published-package selection and native consumer acceptance.

Fresh follow-up qualification on the incoming Testkit 0.49.0 locked graph passes
all 112 workflow-related cases, public local-source recovery, the complete planned
allocation/data-upload/load/start journey and all four lost/malformed load/start
journeys. Strict all-target Core Clippy, formatting and Cargo inheritance pass.
Explicit `make install-testkit-server` prepared the missing selected 0.49.0 CLI;
the authenticated 16.1.0 server and prior bundles remain retained. The isolated
custody adapter also passes its ten cases and strict all-target Core Clippy on
this Testkit selection. Backup's manifest/lock edits remain incoming user work.
No full Backup `make ci` or native Backup macOS qualification was run.

## Pending 0.27.0 Host consumer cleanup

Released Backup 0.26.0 already selects published Host FS and Process 0.22.1.
The pending 0.27.0 draft replaces Backup's duplicate no-follow directory opener
with Host FS at layout and IC snapshot artifact callers. Eight layout and 26
snapshot artifact tests pass, as does strict all-target Core Clippy. The Agent
fixture now delegates bounded, required-Content-Length HTTP capture to Host
Process while retaining raw request and JSON error evidence. Its 14 live HTTP
boundary tests, 12 PocketIC transport cases, strict all-target Agent Clippy,
formatting, doc links and Testkit server admission pass. IC snapshot checksum
and upload slice readers now use Host Artifacts' exact-length APIs, retaining
Backup's file-shape and original I/O projections. All 26 snapshot artifact
tests and strict all-target Core Clippy pass after that change. The full release
gate and native macOS consumer checks remain separate. Issues #45 and #46 remain
open for that acceptance; #52 awaits a published Host release containing #127.
The artifact checksum and staging path now also use Host FS's published
root-relative entry opener instead of a local component walk. All 13 focused
artifact tests and strict all-target Core Clippy pass. Backup still owns UTF-8
path admission and artifact error projection. Existing-only sidecar locking and
directory publication remain local until the corresponding Host primitives are
available in a selected published package.

## Released 0.26.0 Intel command-custody regression

The current release already includes the license and package-fixture changes
from now-closed Backup PR #49. Its older Intel macOS CI run failed because a
restore-reference test expected quiescence immediately after reaping a child;
the live sidecar still correctly returned `InFlight`. The current test now
retries only that refusal for at most five seconds, checks unchanged reference
bytes on each refusal, and fails if custody never settles. The focused Linux
test passes on the locked graph. Native Intel macOS and the full release gate
were still unqualified at the time of that focused test change.

## Host contribution handoff

Host PRs #152, #114, #125 and #127 merged after their Linux, Intel macOS,
Apple Silicon and MSRV checks passed. PR #112 closed because #114 includes
the Unix no-follow directory opener. Host 0.22.1 is tagged; #127 belongs to
the 0.22.2 draft. [Backup #52](https://github.com/dragginzgame/ic-backup/issues/52)
tracks lock and directory-commit adoption. Backup now selects published Host
0.22.1 for its directory and HTTP consumer cleanup; native and release-gate
qualification remain separate.

## Released 0.26.0 graph and observation admission

Effect-graph decoding now uses the shared persisted-sequence visitor with an
aggregate edge check before each node is stored. The node, per-node dependency
and total-edge bounds are unchanged. Membership, controller and snapshot-read
admission share ordered request/context and target/call checks; membership still
checks the full inventory before the call ceiling. All 555 Core unit tests,
including the new combined-denial regression, formatting and strict Core
Clippy pass on the present lockfile.

[Merged Host Tooling PR #152 — Add exact-length artifact stream readers](https://github.com/dragginzgame/ic-host-tooling/pull/152)
adds short-EOF and excess-byte admission to the generic bounded readers. Its
35 default-feature Artifacts tests, nine minimal-feature artifact cases and
strict all-feature/all-target Clippy pass in an isolated clean checkout. Backup's
IC snapshot verification and upload readers keep their existing local length
checks in the released code; the pending 0.27.0 draft replaces their local
exact-length checks with selected Host stream APIs. The focused evidence above
was recorded before release and native macOS qualification.

## Released 0.26.0 stream and fixture consolidation

Framed evidence digests now stream through the published Host Artifacts hashing
writer instead of allocating the complete framing buffer. The checksum record
and its JSON schema remain Backup-owned under Host Tooling's consumer-schema
boundary. The independent digest goldens and all 554 Core unit tests pass.
JSON publication uses Host FS's existing post-sync, pre-publication admission
callback for crash qualification, removing Backup's duplicate file sync while
retaining one production/test publication adapter and both crash barriers. All
15 JSON tests, including acknowledged process-death cases, pass. The PocketIC
fixture shares original call setup and lost/settled reply bookkeeping between
mutation and transfer-read paths; their distinct passive validation remains.
Its compile check, both selected lost-reply journeys and one complete planned
download journey pass. Strict all-target Core Clippy passes. The full `make ci`
gate and native macOS acceptance remain separate.

## Released 0.26.0 bounded-model and Host-primitive cleanup

Artifact staging and workflow stages now use published Host FS 0.21.0
private-directory creation in place of their local `0700` helpers. The Host
primitive synchronizes the parent link and retains a created directory after a
failed final admission; Backup preserves the underlying IO kind and phase in
its existing error path. Its CR/LF path refusal applies to selected and nested
directory paths, changing public directory-staging behavior, so the whole pending
batch is selected as 0.26.0. Partial trees remain retained; no journal is reset.
Consistency, restore-safety and fence-reconciliation request
digests now use one local framing builder; independent binary goldens remain
unchanged. Journal and command-custody drops delegate descriptor unlock to
Host FS. Existing-only sidecar admission and one-shot locking remain local
until [Host PR #127 — Add existing lock admission and directory commit primitives](https://github.com/dragginzgame/ic-host-tooling/pull/127)
is published. Backup's distinct lifetime and command-spending policies stay local.

The remaining controller, snapshot-ID, request snapshot-byte and restore-reference
decoders now use the existing bounded sequence reader. Fence-acquisition, IC
request and snapshot-metadata constructors obtain canonical target text and raw
principal bytes from one parse. Layout and artifact custody checks use the
published Host FS directory-admission primitive with the existing typed
replacement errors. The then-selected Host FS 0.21.0 package lacked the opener
from [Host PR #114 — Open files and directories beneath an explicit root](https://github.com/dragginzgame/ic-host-tooling/pull/114);
the pending 0.27.0 cleanup now adopts it from Host FS 0.22.1.
On the present lockfile, all-target Core checking, 554 Core unit tests, nine
selected native integration tests, strict all-target Clippy, formatting, doc
links and the explicit CR/LF directory-staging regression pass. This is focused
local qualification; the full `make ci` gate and native macOS acceptance have
not been run for this batch. The manifest and lockfile already had separate
incoming edits and were left untouched.

## Earlier Host primitive rehearsal

The earlier Host Tooling rehearsal covered three PRs:
[#114](https://github.com/dragginzgame/ic-host-tooling/pull/114) admits
root-relative file and directory descriptors,
[#125](https://github.com/dragginzgame/ic-host-tooling/pull/125) owns bounded
synthetic HTTP request capture, and
[#127](https://github.com/dragginzgame/ic-host-tooling/pull/127) owns existing
lock admission and directory commit mechanics. Their notes selected pending
Host 0.20.2 after tagged 0.20.1. Their refreshed heads had focused local
Host qualification. An isolated Backup copy compiled against the combined unpublished
Host source and passed 13 artifact tests, 7 directory-commit tests, 11 Agent
HTTP boundary tests, all-target checking and strict all-target Clippy. This
was consumer rehearsal, not published dependency qualification. Published Host
FS 0.21.0 now supplies directory identity admission; the root-relative no-follow
opener from #114 was still absent from that earlier package. Backup-specific
checksum, evidence and recovery policy stays here.

## Observation recovery and handoff consolidation

The three Core observation request constructors now read the original pending
mutation and observation attempt IDs through one mutation-first helper. Agent's
three recovery readback lanes share retained submission and response-evidence
projection while keeping their separate methods, authority bindings, domain
prefixes and response types. The current locked graph, including selected Host
0.20.0 crates, passes the complete `make ci` gate with local loopback access:
554 Core unit tests, the native and PocketIC integration suites, 16 Agent HTTP
boundary cases, 12 Agent PocketIC transport cases, strict all-target Clippy,
Rust 1.91.0 consumers and both package archives. Historical handoff entries
now live in the linked October archive below; their original qualification
claims remain historical, not fresh evidence for the current graph.

## Inventory, data-stage and reply-evidence consolidation

Capture and metadata-upload allocation settlement now share closed-baseline
comparison, exact identity matching and explicit new-candidate outcome projection.
Upload retains its separate destination validation after identity matching and
before candidate admission. Download and upload data stages now use one pure
sequential child-plan builder; their source/extent preparation and public typed
errors stay separate. Five IC reply owners use common domain-prefixed checksum
framing with their original domains and field order. No schema, public signature,
spending owner or backend behavior changed. Full `make ci` passes on the locked
graph with local loopback access, including all 554 Core unit tests, 34 PocketIC
snapshot journeys, 12 Agent transport simulator cases, independent wire/evidence
goldens, warning-denied all-target Clippy, Rust 1.91.0 independent consumers and
both packaged-crate archives. The ordinary sandbox denied PocketIC's local bind;
the same complete gate passed with loopback access. Native macOS acceptance
remains separate.

## Released 0.25.1 Agent package license

The tagged 0.25.0 release is at `e258832`. Its [exact-source CI run](https://github.com/dragginzgame/ic-backup/actions/runs/38732636669)
passes Linux, Apple Silicon and Intel macOS. Delivered-source acceptance for
Host descriptor walks, private-parent refusal and the reduced consumer tooling
gate is complete; issues #48, #50 and #51 are closed against that run.
The Agent crate now includes the same MIT license text as Core, with existing
copyright notices retained. This addresses the packaging gap described in
[PR #49 — Include MIT license text in published packages](https://github.com/dragginzgame/ic-backup/pull/49).
Released 0.25.1 already contains the license and the fresh-checkout package
fixture correction. PR #49 duplicated those exact changes and is closed.
The locked offline package check passed: both archives compiled together,
and the Agent archive's `LICENSE` exactly matches the repository text.
Formatting, documentation links and diff checks passed for that batch.
The earlier [focused-validation PR #44](https://github.com/dragginzgame/ic-backup/pull/44)
is closed because released 0.25.0 already includes the Shared Tooling cadence.

## Remaining scope

The retained artifact, journal, workflow and IC evidence boundaries are implemented,
but independently usable backup and restore remain unestablished. Integrations
still own live authentication, application fence and restore-safety qualification,
exact effect attribution and terminal reference release. See the
[extraction design](../extraction-design.md) for the remaining sequence.

Earlier implementation and qualification entries are retained in the
[October 2026 handoff archive](archive/2026-10.md).
