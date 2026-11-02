# Current handoff — 2026-11-02

## Shared Tooling 0.27.26

Adopted commit `e84252f5740bacd708359580455ea2b6fd67df38`, independently
selected from GitHub main and exported from a clean isolated checkout. The
canonical preview/refresh retains existing entrypoints and derives **141**
payloads: seven additions, eighteen updates, no removals. Paired source/revision
verification proves exact bytes and executable modes; no payload was patched.
The committed version is 0.27.26; upstream notes remain labelled Unreleased.

The shared Rust **1.99.1** policy now matches Backup's existing floor and pin;
the obsolete local exception is removed. Common executable catalogs, workspace
manifests, lockfile, compiler pin and product Rust are unchanged from task entry.
Platform's ownership guidance preserves the current facade use for external
DTOs, hashing and Agent transport. No new crate dependency, runtime owner or
application deployment obligation was introduced.

Explicit setup adopts the new native-input identity, registers this checkout's
current tool requirements and trims successful compilation scratch after
executable/receipt admission. Common tools and Testkit reuse matching generations;
Build **0.73.1** is rebuilt and admitted. Existing generations and failed evidence
remain retained. No store reclamation, legacy migration, product compiler-cache
maintenance or artifact cleanup was run. Contributors should rerun
`make install-tools`, then `make tools-check`, after adopting this snapshot.

The baseline keeps work/delivery with each requested repository owner. No sibling
source contribution or unsolicited adoption PR was made. Backup retains its
coordinated release group, default released-version mode, local publication and
metadata adapters, and mandatory exact resume receipt check. Optional group
proof reuse and bootstrap observation are not activated.

## Focused qualification

[The adoption review](../shared-tooling-review.json) records provenance,
decisions and evidence at `/tmp/backup-shared-refresh.qlg7i32c/`.

- Explicit setup and complete offline tool admission pass. The existing native
  shared-tool reuse CI step passes locally, including download refusal,
  application flag/wrapper isolation and exact binary reuse.
- Snapshot/source proof, Cargo inheritance and dependency boundaries,
  ShellCheck, new Perl helper syntax, formatting and local links pass.
- Consumer fixtures pass actual Make boundary composition, changed-input and
  graph refusal, snapshot tamper/evidence retention and Testkit routing.
- Release preparation and exact receipt/resume selections pass using isolated
  substitutes. Registered cases remain in `target/shared-release-tests.ljD0Ga`
  and `target/shared-release-tests.0VJDDY`; no real release effects occurred.
- Captured inputs stayed unchanged during qualification. Review/handoff edits
  after those checks receive focused link, JSON, snapshot and diff checks.

This is fresh Linux tooling qualification. No new full repository gate, product
runtime/PocketIC suite or native macOS run was executed for the snapshot refresh.
Earlier full CI remains scoped to its recorded source and locked graph.

## 0.54 delivery source

The maintainer requested deployment of the complete **0.54.0** batch to
`origin/main` and crates.io. This source uses **0.53.6** package metadata and an
undated **0.54.0** draft; the standard minor-release runner owns their finalization.
The generated [release receipt](../release.json), annotated Git tag and registry
observations own the actual delivery result. Existing
Testkit **0.78.0**, arithmetic-only Metrics **0.25.1**, compiler and publication
changes remain intact. The incoming graph includes `cc` **1.11.0** and
`crc32fast` **1.5.4**; this adoption performed no dependency update or downgrade.

[#77 — Adopt Rust 1.99.1 and consolidate compiler qualification](https://github.com/dragginzgame/ic-backup/issues/77)
and
[#76 — Reuse shared Cargo publication attempt mechanics behind local admission](https://github.com/dragginzgame/ic-backup/issues/76)
are included in the authorized delivery batch. Compiler admission records
actual Cargo/rustc identities before native qualification. Independent normal
consumers, archive-pair builds and the Agent journey select the declared compiler;
Linux, Apple Silicon and Intel macOS jobs remain in CI. Publication retains exact
source/arguments/outcomes behind the local release lock and Clippy gate;
interrupted attempts require explicit reconciliation before redispatch.

Earlier qualification remains at `/tmp/backup-issue77.aut8qyhh/` and
`/tmp/backup-054-readiness.336t_398/`. The latter records a passing complete local
`make ci` on its frozen inputs, including both real PocketIC suites, independent
consumers and normalized package archives. The incoming handoff and previous
adoption review are preserved under this refresh's `before/`.

[Hosted CI run 41331009274](https://github.com/dragginzgame/ic-backup/actions/runs/41331009274)
passed all three native hosts for main commit
`fff0b4654844dc53539a726fd319a6af5abbe272` only. Fresh native macOS evidence requires the delivered-source CI run.
The release workflow must perform its complete local gate before version
preparation and atomic branch/tag push, followed by separate registry publication.
No live IC backup/restore or artifact cleanup is part of this delivery. Preserve unfinished journals, referenced artifacts,
original publication attempts and required recovery evidence.

[Implementation contracts](../implementation-contracts.md),
[the Platform boundary](../ic-platform.md) and
[the operator map](../canic-adoption.md#reusable-operator-composition) remain
canonical.
