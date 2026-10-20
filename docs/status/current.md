# Current handoff — 2026-10-20

## Current boundary

Core owns backup records, bounded spending, guarded persistence, reconciliation
and coordination. Agent owns reserved signed transport. Platform owns external
IC types/codecs/SDKs; Host owns generic host IO/process custody; Testkit owns
simulator assets/processes. Applications supply fresh authority, consistency,
fencing and external-effect settlement. The
[implementation contracts](../implementation-contracts.md) remain mandatory.

## Current batch

Task-entry HEAD is `e323361` (released package version **0.35.0**). The incoming
manifest and lock select published **Testkit 0.59.0**, **Platform 0.4.1**, Metrics
0.14.9 and **Host Artifacts/FS/Process 0.31.1**. Those input bytes are preserved.
Testkit and Platform archives and unpacked sources match official Cargo-index
checksums. Platform's only source change from 0.4.0 is in its unused account
feature; Backup's principal, management, Candid and Agent sources are unchanged.
See [Platform ownership](../ic-platform.md).

Testkit now shares the production Host/Platform identities, removing the older
development-only Host 0.30.1 and Platform 0.2.2 selections. PocketIC client/server
remain 16.1.0. CLI setup explicitly prepared the locked Testkit 0.59.0 executable
and admitted its authenticated server without removing previous generations.
Its CLI's published packaged lock remains independent from Backup's library lock.
Clap is development-only. Required production Candid/management DTO 0.11.0 and
Agent 0.49.2 remain transitive; PocketIC retains management DTO 0.8.0 in tests.

The release adapter has one complete manifest-and-lock preparation path through
published IC Build 0.18.0. Its tests now check exact local lock changes alongside
comment retention, unchanged members/foreign bytes and selected-commit refusal.
Original candidate rendering, receipt/source admission and rollback are retained.
The independent Core consumer now also compiles the exact Host checksum-error
conversion reported by [#62](https://github.com/dragginzgame/ic-backup/issues/62),
projecting both Host requirements from production metadata rather than pinning
fixture versions. Both Rust 1.91 consumers pass; a retained negative control with
Host Artifacts 0.30.1 rejects the exact conversion. Four checksum/error cases and
six real command-custody cases also pass, alongside shell, formatting and link
checks. Evidence: `/tmp/backup-host-consumer-7j5fm1d2/`, including the focused gate
at `validation/validation.b9EC1K/` and `mismatched-host.log`.
There are no Rust source, public API, wire, storage or spending-policy changes.
Pending notes are **0.35.1** because this batch changes contributor tooling only;
package versions remain 0.35.0.

Shared Tooling now selects committed **0.25.5**, revision
`75c633d71933b64b3b2558c4a5f3ac0679b7ab32`, verified against remote main.
Canonical refresh preserves all 119 payloads, modes and entrypoints; this patch
changes producer-only fixtures outside Backup's selection. Snapshot, tooling,
formatting and link checks pass. Evidence: `/tmp/backup-shared-0255-1o2lv21t/`;
[the provenance review](../shared-tooling-review.json) links the prior selection.

## Qualification and remaining limits

All 78 consumer release-adapter cases pass on Linux using the real published
editor, command substitutes and real private-index boundaries. No real checkout
release command or Git mutation ran. Eight locked normal/build target/package
trees cover Linux, both macOS architectures and Wasm: no simulator, WAT, Clap,
parser, CDK runtime or IC Build tools enter production; Core remains Agent-free.
All 15 focused targets pass on Linux: prepared tools, source/licenses, dependency
boundaries/inheritance, snapshot integrity, links, shell/tooling checks, formatting,
all-target builds, strict Clippy, complete native/HTTP/PocketIC tests, Rustdoc,
actual Rust 1.91 all-target checks and independent Core/Agent consumers, and both
unpacked package archives. This includes all 38 Core and 12 Agent simulator cases
and all 14 Agent HTTP cases. No dependency API blocked adoption.

Evidence and the original dirty-tree snapshot are retained at
`/tmp/backup-testkit-release-cut-5orrht2t/`; the passing release fixture registry is
`target/shared-release-tests.SmHqTG/cases.txt`. The focused gate receipt is
`validation/validation.7KrqAz/timings.tsv` beneath that evidence directory; package
evidence remains at `target/package-check.SRxT1U/`. Previous adoption receipts at
`/tmp/backup-shared-host-platform-trcpwxv2/`,
`/tmp/backup-platform030-2t0aochd/` and
`/tmp/backup-host-qualification-bwyxc7tz/` retain their original source/graph scope.
The unchanged shared runner retains its earlier passing evidence.

Full current `make ci`, native macOS, installed-IC and hosted artifact delivery
qualification remain separate. The earlier inspection of HEAD's
[hosted CI run](https://github.com/dragginzgame/ic-backup/actions/runs/39400318769)
reports Linux success, an Apple Silicon job failure and Intel macOS still running.
The failed check's annotations report hosted-runner communication loss and macOS
ARM capacity constraints. Its job log returns 404 while the run remains open;
no source-level cause or CI fix is established. No issue is closed from this
incomplete matrix. Cross-target tree inspection is not a native or
Wasm compilation result. Independent consumers enforce simulator/WAT exclusions;
role comments do not derive future exclusion policy. This host-library repository
has no application deployment/artifact-binding gate.

## Remaining application acceptance

Complete application acceptance still requires integration-owned current
membership/control/read authority, whole-selection fencing, retained source and
external-obligation custody, and terminal evidence. Local all-Applied checkpoints
and test providers do not supply those facts. See
[application admission](../implementation-contracts.md#application-admission-and-fencing).
The earlier read-only Canic inspection at `de4f36d4e` found live capture refused
before state resolution; its restore terminal-evidence adapter was separate.
No new sibling inspection or qualification is claimed. Earlier migration history
remains in the [October 19 archive](archive-2026-10-19.md).
