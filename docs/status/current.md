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

## Pending 0.26.0 Intel command-custody regression

The current release already includes the license and package-fixture changes
from now-closed Backup PR #49. Its older Intel macOS CI run failed because a
restore-reference test expected quiescence immediately after reaping a child;
the live sidecar still correctly returned `InFlight`. The current test now
retries only that refusal for at most five seconds, checks unchanged reference
bytes on each refusal, and fails if custody never settles. The focused Linux
test passes on the locked graph. Native Intel macOS and the full release gate
remain unqualified for this pending test change.

## Host contribution handoff

Host PRs #152, #114, #125 and #127 merged after their Linux, Intel macOS,
Apple Silicon and MSRV checks passed. PR #112 closed because #114 includes
the Unix no-follow directory opener. Host 0.22.1 is tagged; #127 belongs to
the 0.22.2 draft. [Backup #52](https://github.com/dragginzgame/ic-backup/issues/52)
tracks lock and directory-commit adoption. Backup's directory and HTTP
replacements still require published Host packages and their own consumer
qualification.

## Pending 0.26.0 graph and observation admission

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
checks until a Host package containing the PR is published and selected. Full
release and native macOS qualification remain separate.

## Pending 0.26.0 stream and fixture consolidation

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

## Pending 0.26.0 bounded-model and Host-primitive cleanup

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
replacement errors. The local no-follow opener remains because the published
Host FS 0.21.0 package does not include the opener now merged through
[Host PR #114 — Open files and directories beneath an explicit root](https://github.com/dragginzgame/ic-host-tooling/pull/114).
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
opener from #114 is still absent. Backup-specific checksum, evidence and recovery
policy stays here.

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
