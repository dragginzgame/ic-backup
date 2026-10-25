# Current handoff — 2026-10-24

## Ownership and released source

Released Backup is **0.44.1**, `39094a5`. Core owns bounded records, guarded
persistence, reconciliation and original-plan coordination. Agent owns reserved
signed transport. Applications own current authority, membership, release binding,
whole-unit fencing and external obligations. Host owns generic IO/process/hash,
Platform external IC identities/SDKs, and Testkit simulator assets/processes.
[Implementation contracts](../implementation-contracts.md) retain product obligations.

## Published dependency adoption

Pending **0.44.2** aligns Host Artifacts/FS/Process, Testkit and the Build release
editor minimums to published **0.68.1**, source
`c9c5ef7c44e3d0c3b23c8531a5d43c1800656c0a`. Platform's minimum becomes published
**0.6.15**, source `6dcf54a229201f85775d218b7a9f41809297878f`.
These are the latest published supplying releases at inspection. All five Tooling
Rust source trees match 0.68.0; Platform's Rust source matches 0.6.14. The relevant
release changes select Wire/schema dependencies/features that Backup does not enable.
The dependency adoption itself changes no product Rust, codec, storage, lifecycle,
public type or recovery logic.

The incoming lock already selected the four Tooling 0.68.1 libraries and Platform
0.6.15. It remains byte-identical: SHA-256
`a71c6fa12455a0cf4ae26987b7cc3cd80d59b67418c8dd65e5ff4cc7702bc28e`.
Authenticated archive/source identities are in `/tmp/backup-deps-0681-published.json`.
Agent 0.49.3, management DTO 0.11.0, external Principal 0.1.5 and PocketIC 16.1.0
remain selected. Shared Tooling stays 0.26.14; no snapshot payload changes.
Workspace versions remain 0.44.1. No staging, commit, push, release, publication,
live IC effect or sibling edit is performed.

## Qualification

Linux strict all-target/all-feature Clippy, 525 Core and three Agent unit tests,
actual Rust 1.91.0 all-target/all-feature compilation, dependency inheritance and
boundary checks, formatting and local documentation links pass. Both independent
MSRV consumers pass, including public Host checksum/publication error identities.
Their normal graphs select Host 0.68.1/Platform 0.6.15 and exclude Testkit, PocketIC,
WAT, Clap, Wire/schema packages and Host development features; Core stays Agent-free.
Evidence: `target/msrv-consumers.VHocWF/`.

Canonical setup reused authenticated Testkit/Build CLI assets without rebuilding.
Both offline checks pass, and the selected editor passes isolated metadata/retention
fixtures at `target/shared-release-tests.ixjmk8/`. Existing PocketIC assets and older
or failed tools remain retained. Both authenticated independent candidate-consumer
simulator cases pass: cold capture/restore/acceptance with terminal replay, and
process death after load without redispatch. Build/graph evidence:
`/tmp/ic-backup-agent-independent-build-3617012-0/`; fixture evidence:
`/tmp/ic-backup-agent-pocketic-3617012-{1,2}/`.
Logs: `/tmp/backup-deps-0681-{boundaries,clippy,unit,msrv,consumers,setup,tools-check,editor,simulator}.log`.

## Local cleanup qualification

The pending patch also replaces manual shared-helper fixture copy lists with the
snapshot's declared dependency-closure copier, retaining explicit local roots and
substitutes. Valid public plan fixtures use typed constructors; unit fence fixtures
share obligation construction. Golden generators share independent request framing
and all three check modes reproduce existing bytes exactly. Workflow publication
uses the existing immutable-record owner after bounded encoding. Agent transfer
projection retains original borrows and the distinct metadata/data reply ceilings.
Public signatures, persisted formats and cancellation/recovery contracts are unchanged.

Fresh Linux checks pass: strict all-target/all-feature Clippy, 528 library unit tests,
four public integration journeys, and the authenticated Agent capture/download/upload/
same-ID restore simulator journey. Hook, tooling and complete local release-adapter
fixtures pass, including private-index and retained-proof recovery; ShellCheck passes.
Release fixture evidence: `target/shared-release-tests.L2mTrJ/`; hook evidence:
`target/hook-tests.5ZxVjd/`. Logs: `/tmp/backup-five-{clippy,unit,integration,simulator,hooks,tooling,release,shellcheck}.log`.
The incoming dependency manifests, lockfile and Makefile remain unchanged by cleanup.
No new native macOS, MSRV, installed-IC or full CI qualification is claimed for this batch.

## Limits and remaining integration

This batch does not claim native macOS, installed Canic/IC, new package-archive or
registry-only acceptance, or full local make ci. Agent's signing graph still contains
crypto-bigint 0.5.5; this update supplies no cryptographic correction. The
[Canic guide](../canic-adoption.md) retains application responsibilities and the
requirement to qualify replacement behavior before retiring its existing restore
engine. Direct declarations and normal-graph checks do not admit deployed Wasm
bytes; the deploying application owns target/feature/artifact binding.
Shared Tooling's preceding adoption evidence remains at the
[released handoff](https://github.com/dragginzgame/ic-backup/blob/39094a5/docs/status/current.md).
