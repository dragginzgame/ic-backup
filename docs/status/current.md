# Current handoff — 2026-10-20

## Current boundary

Core owns backup records, bounded spending, guarded persistence, reconciliation
and coordination. Agent owns reserved signed transport. Platform owns external
IC types/codecs/SDKs; Host owns generic host IO/process custody; Testkit owns
simulator assets/processes. Applications supply fresh authority, consistency,
fencing and external-effect settlement. The
[implementation contracts](../implementation-contracts.md) remain mandatory.

## Current adoption

Task-entry HEAD is `3d02961` (released package version **0.34.1**). The incoming
catalog and lock already select published **Host Artifacts/FS/Process 0.31.1**
and **Platform 0.4.0**; those selections are preserved. All four registry archives
and unpacked sources match their official Cargo-index checksums. Platform's used
principal, management and Agent sources are unchanged; its Candid module adds an
unused encoder export. Wire bytes, decoder limits, original journals and spending
contracts remain unchanged. Host's public publication-error Rust identity requires
consumers to align Host FS to 0.31; pending notes are **0.35.0**. Package versions
remain 0.34.1. See [Platform ownership](../ic-platform.md).
The current lock additionally selects rustls-webpki 0.103.16. That patch selection
appeared before the product gate; every gate checkpoint matches the current lock.

Shared Tooling is **0.25.4**, committed revision
`db555ab6cf4a93af9a183f18d921d3b0ee97c725`, verified against remote main. No GitHub
release object was available; this is a committed snapshot adoption, not registry
publication. The canonical exporter selected 119 files, updating 28, adding eight
and retiring three obsolete Perl renderer/fixture payloads. No vendored files were
hand-edited. [The review](../shared-tooling-review.json) records provenance.

The release adapter now consumes published IC Build 0.18.0's editor through
explicit setup and offline admission, with an 8 MiB per-document bound. Original
candidates, receipt/source checks and rollback remain local. Shared grouped
formatting defaults replace redundant Make overrides. Source symlink/license
checks and exact archive-license admission are wired into existing gates.
Failure-evidence collection uses the shared orchestrator; failed selector output
is diagnostic only, and redirected parent links do not authorize traversal.

## Qualification and remaining limits

All 16 focused validation targets pass on Linux: prepared tools, source/license
admission, dependency boundaries/inheritance, snapshot integrity, links, shell
checks, tooling/hooks, formatting, native all-target compilation, strict Clippy,
complete native/HTTP/PocketIC tests, Rustdoc, actual Rust 1.91 all-target and
independent Core/Agent consumers, and compilation of both unpacked archives.
This includes all 38 Core and 12 Agent simulator cases and all 14 Agent HTTP cases.
All 78 consumer release-adapter cases pass using the real published editor and
isolated Git/effect substitutes. The unchanged shared runner retains earlier
passing evidence; no release command ran against the real checkout.

Eight locked normal/build target/package trees cover supported native hosts and
Wasm: no old Host/Platform, simulator, WAT, parser, CDK runtime or IC Build tools
enter production; Core remains Agent-free. Required Candid/management DTO 0.11.0
and Agent 0.49.2 remain transitive. Development-only Testkit 0.57 retains Host
0.30.1, Platform 0.2.2 and PocketIC's management DTO 0.8.0. A registry-only Rust
1.91 principal/management probe runs natively and compiles for Wasm; it is not a
Wasm build of either host library. Five cases execute the actual CI collector
step for complete/partial selection and redirected target/tool parents.

Focused CI/tooling checks cover the collector follow-up. Full current `make ci`,
native macOS, installed-IC and hosted artifact upload/download qualification
remain separate. Role comments do not derive future exclusion policy; independent
consumers enforce current simulator/WAT exclusions. This host-library repository
has no application deployment/artifact-binding gate. No missing dependency API
blocked adoption.

Evidence, original dirty-tree snapshots, registry/archive provenance and per-target
logs are retained at `/tmp/backup-shared-host-platform-trcpwxv2/`; the 16-target
receipt is `validation/validation.ILIF3W/`. Earlier receipts at
`/tmp/backup-platform030-2t0aochd/` and
`/tmp/backup-host-qualification-bwyxc7tz/` retain their original source/graph scope.
The old CLI correctly refused the new installer policy; explicit setup admitted
new common, Testkit and editor selections without removing old generations.
Staging, Git refs and unrelated work remain unchanged; no commit, push, release,
publication, deployment or evidence cleanup is part of adoption.

## Remaining application acceptance

Complete application acceptance still requires integration-owned current
membership/control/read authority, whole-selection fencing, retained source and
external-obligation custody, and terminal evidence. Local all-Applied checkpoints
and test providers do not supply those facts. See
[application admission](../implementation-contracts.md#application-admission-and-fencing).

The earlier read-only Canic inspection at `de4f36d4e` found live capture still
refused before state resolution; its existing restore terminal-evidence adapter
was separate. No new sibling inspection, source change or qualification is claimed
here. Migration history remains in the [October 19 archive](archive-2026-10-19.md).
