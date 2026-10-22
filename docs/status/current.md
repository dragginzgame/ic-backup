# Current handoff — 2026-10-22

## Ownership and source

Core owns bounded backup records, guarded persistence, reconciliation and workflow
coordination. Agent owns reserved signed transport. Platform owns external IC
identities/codecs/SDKs, including PocketIC clients/DTOs. Testkit owns simulator
assets, managed startup and harness extensions. Applications retain membership,
permissions, consistency, fencing and external-effect settlement. Current
obligations belong to [implementation contracts](../implementation-contracts.md)
and [the record/API boundary](../extraction-boundary.md).

Base is released **0.37.4**, commit **`522aec0`**. Pending adoption notes select
**0.38.0** because exposed Host Rust type identities change across 0.63/0.64;
callers naming or exchanging those types must align their Host dependencies.
Workspace/package versions remain **0.37.4**. No staging, commit, release,
publication, deployment, live IC effect or sibling source edit occurs here.
Production Rust, wire/storage formats, bounds and recovery rules are unchanged.

## Published dependency adoption

Host Artifacts/FS/Process, Testkit and the Build release editor select published
**0.64.0**. Registry metadata confirms availability; cached library archives match
locked checksums and identify published source `036da0cd`. The Host libraries
have no Rust-source changes from 0.63.3, but their Rust identities differ.

[Issue #66 — Align artifact error conversion with published Host 0.64](https://github.com/dragginzgame/ic-backup/issues/66)
is addressed by this pending alignment. Public consumer regressions exercise
actual Host declaration errors through Backup's `From` implementation, preserving
malformed/duplicate `InvalidData` causes and exact non-UTF-8 paths. Canic still
selects published Backup 0.37.4/Host 0.63.3; keep the issue open until the aligned
Backup graph is published. No downstream compatibility bridge is needed.

Testkit no longer forwards PocketIC. Core and Agent fixture imports now use
`ic_platform::pocket_ic` directly, with Platform's `pocket-ic` feature confined to
development dependencies. Testkit still owns explicit server admission/startup;
no wrapper, retry, reset, funding or application policy is introduced. Platform
stays **0.6.7**, with management DTO **0.11.0** and external Principal **0.1.5**.
The simulator remains **16.1.0**. Build and Testkit CLI selections explicitly
retain `cli`; library consumers do not enable it.

A concurrent lock edit selected ICU **2.4** and related transitive packages. It is
preserved. The first native test binary passed, but its validation guard refused
changed inputs. Explicit locked preparation, all graph-dependent product checks
and the entire native suite were restarted against the final lock, SHA-256
`5a5252bf8880f161f954410e4302f2e8e0e9deadd21313c2182b28ea3b984c34`.
No further input changes occurred during that successful qualification.

## Shared Tooling

The canonical exporter adopts committed **0.26.6**, revision
**`4cb9bc907154225a3f7e4c33fd8d75fbeaa125c8`**, verified against remote main.
All **122 payloads** verify. Added roots/companions provide CI/release integrity
guidance and declared script-fixture closure; existing selections remain.
Caller-relative retained logs resist inherited `CDPATH`, including leading-dash
paths. Runtime restrictions and enforced GitHub requirements remain distinct
under the clarified asynchronous CI policy. No new recurring audit or product
gate is introduced. Shared host/Rust/IC executable pins and workflow permissions
are unchanged. See [the review](../shared-tooling-review.json).

## Validation and evidence

Fresh Linux Rust **1.99.1** qualification on the final graph passes **642 tests**
without failures, ignored cases or filters. This includes **536 Core unit tests**,
all **39 Core PocketIC cases**, **14 Agent HTTP cases** and **14 Agent simulator
cases**, including both independent cold-process acceptance/uncertainty journeys.
All-target/all-feature checks, strict Clippy and warning-denied Rustdoc pass.
Actual Rust **1.91.0** passes for both libraries and independent normal consumers.
Both unpacked package archives compile together with exact original selections.
Normal consumer graphs exclude Testkit/PocketIC/WAT and development-only Host/
Platform features; Core also excludes Agent. Guarded source-reference release and
terminal replay remain fixture-qualified, not production application proof.

Snapshot, Cargo inheritance/IC boundaries, shell, formatting, hooks, consumer CI
checkpoints, evidence retention and isolated release-helper checks pass. Focused
log-path probes verify both ordinary and leading-dash paths without touching a
CDPATH decoy. The initial shared Testkit setup found a live ic-jobs installer;
its lock was preserved, and canonical setup succeeded after that owner finished.

Evidence: `/tmp/backup-host-shared-adoption-20261022-4phyjkg6/`. Final product
receipts are `restart-product-checks.log` and `restart-native-tests.log`; earlier
logs retain their original scopes and the input-change rejection. Private fixture
journals, artifacts, raw output and original locks remain under `restart-fixtures`.
The checksum-error follow-up passes three public integration cases, four artifact
unit cases and strict test-target Clippy. Snapshot, tool admission, checksum helper,
formatting and documentation-link checks pass. Its focused evidence is retained
at `/tmp/backup-issue66-20261022-or1b6q0h/`; the earlier complete suite remains
evidence for the unchanged production implementation and dependency graph.
[The prior handoff](https://github.com/dragginzgame/ic-backup/blob/522aec08e81e2369652f61e30108c370838f151a/docs/status/current.md)
retains 0.37.4 cleanup and qualification history; those receipts keep their source
and dependency identities.

Full local make ci, new hosted CI, native macOS and registry-only installed-product
or installed-IC qualification are not run here.
[#29 — Complete original-plan execution and stage binding before Canic runner adoption](https://github.com/dragginzgame/ic-backup/issues/29)
retains application acceptance: current authority, whole-selection fencing,
command/byte custody and external-obligation settlement. This dependency adoption
does not establish those production facts or qualify Canic.
