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


# Current handoff — 2026-10-15

## Host primitive extraction pending publication

Three existing Host Tooling PRs now merge cleanly with Host main:
[#114](https://github.com/dragginzgame/ic-host-tooling/pull/114) admits
root-relative file and directory descriptors,
[#125](https://github.com/dragginzgame/ic-host-tooling/pull/125) owns bounded
synthetic HTTP request capture, and
[#127](https://github.com/dragginzgame/ic-host-tooling/pull/127) owns existing
lock admission and directory commit mechanics. Their notes select pending
Host 0.20.2 after tagged 0.20.1. Their refreshed heads have focused local
Host qualification; current hosted native/macOS/MSRV checks are
pending. An isolated Backup copy compiled against the combined unpublished
Host source and passed 13 artifact tests, 7 directory-commit tests, 11 Agent
HTTP boundary tests, all-target checking and strict all-target Clippy. This
is consumer rehearsal, not published dependency qualification. Backup's
production manifests, lockfile and local implementations retain their current
state until the Host APIs are published and adopted from the registry; the
Backup-specific checksum, evidence and recovery policy stays here.

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

## Pending 0.25.1 Agent package license

The tagged 0.25.0 release is at `e258832`. Its [exact-source CI run](https://github.com/dragginzgame/ic-backup/actions/runs/38732636669)
passes Linux, Apple Silicon and Intel macOS. Delivered-source acceptance for
Host descriptor walks, private-parent refusal and the reduced consumer tooling
gate is complete; issues #48, #50 and #51 are closed against that run.
The Agent crate now includes the same MIT license text as Core, with existing
copyright notices retained. This addresses the packaging gap described in
[PR #49 — Include MIT license text in published packages](https://github.com/dragginzgame/ic-backup/pull/49).
PR #49 now merges cleanly with released 0.25.0 and carries the pending 0.25.1
note. Its fresh-checkout package check creates `target/` before allocating a
retained fixture; the same one-line fix is present locally.
The locked offline package check passed: both archives compiled together,
and the Agent archive's `LICENSE` exactly matches the repository text.
Formatting, documentation links and diff checks pass. The package version
remains with the maintainer's release flow.
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
