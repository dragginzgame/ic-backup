# Current handoff — 2026-10-21

## Current boundary

Core owns backup records, bounded spending, guarded persistence, reconciliation
and coordination. Agent owns reserved signed transport. Platform owns external
IC types/codecs/SDKs. IC Tooling owns Host, Build and Testkit: generic host
IO/process custody, release metadata rendering and simulator assets/processes.
Applications retain fresh authority, consistency, fencing and external-effect
settlement. The [implementation contracts](../implementation-contracts.md) remain
mandatory; this batch changes no Rust, storage, wire or recovery behavior.

## Current selections

HEAD remains **`38b3b0f` / 0.36.0**. The incoming pending graph selects Host/Testkit
**0.62.1**, Platform **0.5.1**, Candid/derive **0.10.39** and development Metrics
**0.15.0**, with IC Build **0.62.1** as the release editor. Package versions remain
**0.36.0**, notes **0.36.1**. This cleanup preserves incoming manifests and lock
bytes, index, HEAD and all snapshot payloads. Published registry sources supply
dependencies; no sibling checkout or Cargo patch is used.

Shared Tooling remains **0.25.9**, revision
`85aded191d85ac4a8c0e5deab207f6be18895862`, with 119 canonical payloads. The prior
adoption qualifies the runner's refusal to increment again after unconfirmed
committed delivery, preserving exact intent, metadata, receipt, proof and tools.
[The review](../shared-tooling-review.json) distinguishes adoption provenance
from this batch's consumer changes.

## Simplified contributor tooling and guidance

Required setup now orders common host/Cargo tools, locked fetch, Testkit and the
release editor. Offline checks admit host/Cargo, Testkit and editor selections.
The five optional IC executables are outside `install-tools`, `tools-check`, CI
reuse/PATH and release preflight. Manual `make install-ic-tools` and
`make ic-tools-check` remain available; existing bundles and failed evidence stay
retained. Required Testkit admission, original fixture custody and server lifecycle
are unchanged. See [development](../development.md#quick-start) and
[release preparation](../releasing.md).

[Shared Tooling guidance](../shared-tooling.md) and [Platform guidance](../ic-platform.md)
now describe maintained owners/contracts. Complete incoming adoption records,
exact revisions, checksums and historical receipts are preserved with rebased
links in the [Shared archive](archive-shared-tooling-2026-10-21.md) and
[Platform archive](archive-platform-adoption-2026-10-21.md). Manifests, the snapshot
and this handoff own current selections and evidence; compatibility constraints
on external principal 0.1.5 and management DTO 0.11.0 remain explicit.

## Qualification and remaining limits

All **10 focused targets** pass: offline tool admission, snapshot integrity,
dependency boundaries/inheritance, formatting, early strict Clippy, shell,
consumer tooling, documentation links and native tests. All **638 native tests**
pass, including **38 Core + 12 Agent PocketIC** and **14 HTTP** cases. Real setup,
all focused targets and the actual CI offline-reuse step pass with optional IC
commands forced unavailable; no optional command is attempted. The second
consumer reuses exact host/formatter/Testkit identities while downloads refuse.
All **90 consumer release-adapter cases** pass against the published editor;
manual IC routing/refusal, parallel ordering and required-tool failures remain
covered. No real repository Git/release/registry effects occur.

Current evidence: `/tmp/backup-local-simplification-pz964fhf/`, including
`install-tools.log`, `validation/validation.HEb4B1/timings.tsv`,
`ci-reuse-result.json`, `ci-reuse-evidence/`, `release-adapter.log`,
`archive-preservation.json` and the original task-entry files under `before/`.
Release cases/traces remain at `target/shared-release-tests.ixoGB9/`.

Unchanged Rust/graph evidence is reused from
`/tmp/backup-upstream-20261021-_lowx6__/`: 13 passing focused targets, Rust 1.91
independent consumers and exact archives, eight production target trees, and
native/Wasm public principal and all 24 management DTO identity probes.
Independent consumers remain at `target/msrv-consumers.eKuMcC/`, archives at
`target/package-check.9jGJB1/`. Earlier failure/release fixtures and receipts retain
their original source/graph scope. Production excludes simulator, parser/CDK,
WAT, CLI and opt-in filesystem backends; Core remains Agent-free.

The direct boundary gate checks aliases and normal/build/dev/target declarations,
not transitive features or deployed bytes. Candid remains required for management
and SDK contracts, including upstream printer selection; development PocketIC's
DTO identity remains separate. No new full local `make ci`, native macOS,
installed-IC, hosted artifact delivery, release or publication is claimed.

## Remaining constraints and application acceptance

The prior adoption held Shared 0.26.0 for the optional multi-tool installer's
supported macOS awk failure ([#294](https://github.com/dragginzgame/shared-tooling/issues/294)).
This batch does not refresh that snapshot or adopt a local repair. PocketIC pins
thiserror exactly 2.0.18; SHA-256 test oracles retain Host's 0.10.9 line.

Complete application acceptance still requires integration-owned current
membership/control/read authority, whole-selection fencing, retained source and
external-obligation custody, and terminal evidence. Local all-Applied checkpoints
and providers do not establish those facts. See
[application admission](../implementation-contracts.md#application-admission-and-fencing).
No new sibling or application qualification is claimed; earlier Canic observations
retain their recorded identities. Earlier migration history remains in
[the October 19 archive](archive-2026-10-19.md).
