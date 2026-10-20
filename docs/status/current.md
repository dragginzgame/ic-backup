# Current handoff — 2026-10-20

## Current boundary

Core owns backup records, bounded spending, guarded persistence, reconciliation
and coordination. Agent owns reserved signed transport. Platform owns the external
IC type/codec/SDK boundary; Host owns generic host IO and process custody; Testkit
owns simulator assets and managed processes. Applications supply fresh authority,
consistency, fencing and external-effect settlement. The
[implementation contracts](../implementation-contracts.md) remain mandatory.

## Qualified baseline

`ic-backup` and `ic-backup-agent` **0.34.0**, commit
`f8ba7067d2c3c82beccc7e7c84bab3f88106906a` (`v0.34.0`), passed the complete
[configured CI matrix](https://github.com/dragginzgame/ic-backup/actions/runs/39304958340)
on Linux x86-64, Apple Silicon macOS and Intel macOS. The exact released lock
selects Host 0.30.0, Platform 0.2.2 and Testkit 0.57.0. This is the released
source's receipt, not qualification of later lock selections or working changes.
Development Rust remains 1.99.1; the consumer minimum is 1.91.0. The reviewed
Shared Tooling snapshot remains 0.24.0.

## Pending work

The incoming lock update selects Host Artifacts, FS and Process 0.30.1 and
webpki-root-certs 1.0.10. It is preserved as separate pending work; manifests,
package versions and dependency selections are not edited by this cleanup.

Workflow and checkpoint readers now delegate bounded immutable-record admission
to the existing persistence helper. Domain validation still runs under the same
record lock, with original limits, error ordering and complete-history checks.
Ordinary workflow fixtures prepare complete stages through the production owner
and reopen the original journals. Explicit record-only, missing-journal and
acknowledged-crash cases retain their distinct preparation paths.

Before Platform 0.3 adoption, the full local Linux `make ci` gate passed all 17
targets on the Host 0.30.1 / Platform 0.2.2 lock: both crates' all-target compilation
and strict Clippy, native tests, all 38
Core and 12 Agent PocketIC cases, all 14 Agent HTTP cases, warning-free Rustdoc,
actual Rust 1.91 all-target compilation and independent Core/Agent consumers,
and compilation of both unpacked package archives. Tool admission, dependency
boundaries/inheritance, snapshot integrity, hooks, release-helper fixtures,
formatting and links pass. Native macOS and installed-IC qualification of this
pending graph remain separate from the released matrix's original source/lock.
No public API, wire/storage schema, spending or recovery contract change is
selected. Compatible notes are pending under **0.34.1**; package versions remain
0.34.0. Development and simulator guidance now distinguish implemented generic
coordination from integration-owned application admission.

Task-entry source, pending inputs, hosted CI metadata and per-command evidence are
retained at `/tmp/backup-local-consolidation-cfavf447/`. Full-gate receipts and
task-entry hashes are at `/tmp/backup-host-qualification-bwyxc7tz/`, with per-target
logs/timings under `ci/validation.Yad2iC/`. The gate ran before the documentation-only
follow-up; local link and whitespace checks cover that follow-up. Staging and
unrelated work are preserved. No commit, push, release, publication, deployment or
artifact cleanup is part of this batch.

## Platform 0.3 adoption

Published **ic-platform 0.3.0** is verified against the official Cargo registry
index and its complete archive. The incoming `0.3` catalog requirement and lock
selection are preserved. Internal principal admission uses Platform's owned root
type; explicit conversions retain upstream principals at management DTO and Agent
boundaries. Status wire projections and simulator fixtures use Platform's
contract/external principal exports. Public Backup signatures, canonical text/raw
bytes, independent wire/hash goldens, decoder limits and record schemas remain
unchanged. Required Candid, management DTO 0.11.0 and Agent 0.49.2 remain transitive;
development-only Testkit 0.57 retains Platform 0.2.2 and PocketIC's DTO 0.8.0.

All 11 focused validation targets pass: native all-target builds and strict
Clippy, complete native/HTTP/PocketIC tests, Rustdoc, Rust 1.91 independent
Core/Agent consumers, both unpacked archives, formatting, dependency inheritance
and boundaries, snapshot integrity and links. Six normal/build target trees
exclude simulator/WAT/parser/CDK packages and old Platform; Core remains Agent-free.
A registry-only Rust 1.91 probe executes principal/management-wire checks natively
and compiles for Wasm. This is not native macOS or installed-IC qualification.
Receipts, original pending-file snapshots and provenance are retained at
`/tmp/backup-platform030-2t0aochd/`; target logs/timings are under
`validation/validation.F7jIN3/`. Handoff-only follow-up is checked separately for
local links and whitespace. No missing Platform API or adoption blocker was found.
Unrelated pending work, package versions, staging and Git refs remain unchanged.

## Remaining acceptance

Complete application acceptance still requires integration-owned current
membership/control/read authority, whole-selection fencing, retained source and
external-obligation custody, and terminal evidence. Local all-Applied checkpoints
and test providers do not supply those facts. See the
[implementation contracts](../implementation-contracts.md#application-admission-and-fencing)
and [extraction design](../extraction-design.md) for ownership and unfinished scope.

Read-only Canic inspection at `de4f36d4e3de3f2d7112e558c01f25c69f83b9fe`
confirms its root catalog selects Backup 0.34 and Host FS 0.30. Its CLI
`crates/canic-cli/src/backup/create/mod.rs` still rejects non-dry-run creation with
`LiveCreateUnavailable` before local state resolution. Its existing restore
retention adapter supplies terminal evidence, but does not enable capture. The
next application step is Canic-owned live membership/permission admission and
bounded whole-selection fencing through the existing Backup ports, followed by
consumer qualification. No sibling source or deployment was changed or qualified.

Earlier migration history, source-specific receipts and consumer observations are
preserved verbatim in the [October 19 archive](archive-2026-10-19.md), which links
to the older archives. Historical statements retain their original source and
qualification scope; they are not current dependency or external-issue status.
