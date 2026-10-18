# Current handoff — 2026-10-18

Released baseline: `ic-backup` and `ic-backup-agent` 0.31.0 at
`3317b4e6e49c0fe59e343069f68eb539324c0612` (`v0.31.0`). Package versions remain
0.31.0; the compatible pending batch is 0.31.1. No public API or persisted
schema change is selected. Earlier receipts are retained in the
[October 18 archive](archive-2026-10-18.md) and
[October 17 archive](archive-2026-10-17.md); release metadata retains authority.

## Current boundary

Core owns backup records, bounds, guarded persistence and coordination. Agent owns
one reserved update’s signing, retention and transport. Published Platform 0.1.8
owns the Candid, Principal, management DTO and SDK dependency boundary. Core selects
`candid` and `management-canister-types`; only Agent adds `agent`. There are no
direct upstream IC declarations, compatibility re-exports or pin exceptions.
[The Platform contract](../ic-platform.md) records the selected identities,
enforcement and completed consumer acceptance.

Host owns generic hashing, descriptor IO and process custody; root requirements
remain 0.27.1. A concurrent lockfile edit selects 0.27.3; the prior product
qualification below used 0.27.2. Testkit 0.53.0 owns simulator assets and
managed processes. Backup retains fixture evidence, product DTO visitors, Reqwest
client policy and original journals. Applications own membership, routing, release
identity, consistency and external-effect settlement. A standalone CLI and complete
application runner remain outside the implemented scope.

## Pending implementation batch

- Snapshot region streams use Host hashing writers with the original descriptor,
  extent, coverage, checksum, custody and publication checks.
- Direct SHA-256 use is test-only. Redundant tests and test-only JSON publication
  wrappers are removed; retained regressions exercise the actual owners.
- Both simulator fixtures use Testkit library admission directly. The shell bridge
  is removed; explicit setup, offline admission, digests and managed logs remain.
- Shared Tooling 0.24.0 is exported at exact
  `cab12cd3dfa4fdb34846b25397f0d72881bb9ae0` with 114 verified payloads. Dependency
  roles, grouped formatting, hooks and selected-source release checks agree.
  Direct IC boundaries and independent consumer graph exclusions are enforced.
- The repeat architecture review retires uncalled producer fixtures and the
  generic dependency-checker test dispatch through the canonical exporter. All
  retained shared bytes/modes, consumer tests and actual policy admission remain.
  Active ownership guidance reflects completed Platform adoption, current Host
  APIs and Canic's existing direct Host integration. The
  [repeat review](../reports/audits/2026/10/18/architectural-sediment/03/report.md)
  records scope, findings and deliberate retention.

[The ownership review](../ownership.md) explains deliberately retained adapters
and qualification inputs. Sibling checkouts and unrelated pending work remain
untouched; no removed Core API is restored for downstream compatibility.

## Qualification

The final Platform 0.1.8 batch on Host 0.27.2 passed all 634 native tests, including 34 Core and
12 Agent actual PocketIC cases, strict Clippy, all-target builds, Rustdoc, actual
Rust 1.91 checks and both independent consumers. Both exact package archives build
together. An independent native/MSRV/Wasm fixture proves all 24 management exports
and Core’s public metadata/status projections retain upstream 0.11.0 identities.
Six production target trees preserve Core/Agent isolation without simulator,
parser or CDK leakage. Native macOS and installed-IC qualification remain separate.

Current receipts: `target/msrv-consumers.wKh3ry/`, `target/package-check.e8wOUP/`
and `/tmp/backup-platform-refresh/`. The repeat architecture cleanup retains that
product evidence for its original graph; this cleanup changes no Rust, manifests,
locks or product fixtures. The independent Host 0.27.3 lockfile update is preserved
and still needs its own product qualification; do not relabel the earlier receipts.
Focused `make tooling-check`, formatting, dependency boundaries/pins, locked
metadata, snapshot integrity and local links pass after the selection change.
All 114 retained payloads match the incoming snapshot; the exporter removes only
obsolete owned files. Before-images and new receipts are in
`/tmp/backup-audit-49lj1dvx/`. Product tests are retained evidence for unchanged source.

No full local CI, package version edit, commit, push, release, publication,
deployment or artifact cleanup was performed.
