# Current handoff — 2026-10-26

## Released source and ownership

Released Backup is **0.44.3**, `3a6f0fe`. The authorized changelog presentation
[PR #69](https://github.com/dragginzgame/ic-backup/pull/69) is merged at
`528afdda02d0dae0ff7c10b70ec5eaaefa806f08`, and the local checkout is fast-forwarded
to that commit. It adds only the latest release's summary heading, preserving
release identities, dates and older content.

Core owns bounded records, guarded persistence, reconciliation and original-plan
coordination. Agent owns reserved signed transport. Applications retain current
authority, membership, release binding, whole-unit fencing and external obligations.
[Implementation contracts](../implementation-contracts.md) retain product obligations.
The [released handoff](https://github.com/dragginzgame/ic-backup/blob/3a6f0fe3926eba656a4ed4798c321f25963a3f9c/docs/status/current.md)
preserves the prior cleanup, release-adapter and simulator evidence.

## Pending 0.44.4 adoption

Manifest minimums and the contributor release editor now select published
Host/Testkit/Build **0.68.8**, source `7c3b715d07bbeca38b4ba89a60d3bec309de7417`,
and Platform **0.6.19**, source `0502bce432c2478cf86215435864223a22a14a38`.
Shared Tooling is the canonical **0.26.21** snapshot from
`66c91c4412856baa147482e648fc80428db8b179`, matching observed GitHub main.
[The review](../shared-tooling-review.json) records selection and qualification.
Root notes use emoji category sections; older history remains unchanged.

The incoming lock already selected the new IC packages and includes the user's
OpenSSL probe 0.2.2 update. An online Cargo refresh found no further compatible
updates. The lock is byte-identical to task start: SHA-256
`5e51f1c9f7bd8477ff56ec4bae0e47b4fb5596e0b30b233e36ce5cf1c2d68c53`.
Package versions remain **0.44.3**; only notes select the next compatible patch.

All consumed Host/Testkit/Build Rust source is unchanged from 0.68.7; Platform's
Rust source is unchanged from 0.6.18. Its raised Wire metadata minimum is outside
Backup's selected features. Registry archive hashes and committed-source matches
are retained in `/tmp/backup-refresh-published.json`. Agent remains 0.49.3,
management DTO 0.11.0, external Principal 0.1.5 and PocketIC 16.1.0.
No product Rust, feature, wire/storage, authority or recovery changes are made.

Canonical setup reuses authenticated Testkit/editor caches and existing server
assets while retaining earlier tools and evidence. Offline tool admission passes.
Normal releases still start fresh; exact saved recovery uses `release-resume`
with Backup's mandatory receipt checks. This task performs no release.

## Focused evidence

Full locked root metadata and independent normal-consumer graphs pass. Both
consumers select singleton Platform 0.6.19 and Host 0.68.8 without CLI, simulator,
WAT, Wire-schema or CDK runtime features; Core stays Agent-free. Graph evidence:
`/tmp/backup-refresh-graphs/`. Strict all-target/all-feature Clippy passes for both
libraries. Independent Rust 1.91.0 consumers, including public Host error identity
checks, pass at `target/msrv-consumers.jlYSjK/`.

The offline-admitted 0.68.8 editor renders isolated root manifest and lock
candidates, changing only owned versions while preserving foreign packages and
live metadata. Evidence: `/tmp/backup-refresh-editor/`. Dependency inheritance,
boundaries, snapshot integrity and tool admission pass. Logs use
`/tmp/backup-refresh-*`. Prior runtime and recovery evidence remains source-scoped;
simulator and full release-adapter suites are not rerun for unchanged Rust sources.

## Limits and delivery

The merged commit's native Linux/macOS CI was in progress when observed:
[run 40362881639](https://github.com/dragginzgame/ic-backup/actions/runs/40362881639),
source `528afdda02d0dae0ff7c10b70ec5eaaefa806f08`. It does not qualify these
subsequent local adoption edits. No new local full CI, simulator, installed Canic/IC,
package-archive/registry-only, publication or deployment qualification is claimed.
Deployment binding and application acceptance remain consumer-owned. Agent still
contains crypto-bigint 0.5.5 transitively; that earlier concern remains unresolved.
The adoption is left unstaged and uncommitted. No new release, package version,
tag, publication, live IC effect or sibling edit is performed.
