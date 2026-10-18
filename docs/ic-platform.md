# Platform dependency boundary

The repository’s Platform migration was completed with published
[ic-platform 0.1.8](https://crates.io/crates/ic-platform/0.1.8). Released Backup
0.31.2 selects Platform 0.1.10; the current lock selects 0.1.13. Their qualification
boundaries are recorded in the [current handoff](status/current.md); the provenance
below describes the original adoption. All production and
test Candid, Principal, SDK and management DTO imports use Platform. There are no
direct declarations of those upstream packages, CDK-family packages or the old
Testkit codec, and no local compatibility re-exports or pin exceptions.

## Ownership and features

The root catalog selects `default-features = false` with `candid` and
`management-canister-types`. Only `ic-backup-agent` adds `agent`; Core’s independent
consumer remains Agent-free. Parser, executor, ic0, certification and ledger
features are not selected.

| Surface | Owner and selection |
| --- | --- |
| Candid contracts and bounded decoding | Platform over Candid 0.10.38 |
| Principal | Platform root export over upstream 0.1.5 |
| All 24 snapshot/status/transfer/upload DTOs | Platform management exports over exactly upstream 0.11.0 |
| Native SDK identities and operations | Platform Agent exports over SDK 0.49.2 with PEM |
| URL/client configuration | Explicit Reqwest dependency; Backup owns middleware, trust, timeout and retry policy |
| Simulator assets and managed processes | Development-only Testkit; Backup owns fixture custody and original evidence |

Backup retains method-specific byte/work/skip/type/header limits, bounded DTO
visitors, strict reserved-marker admission and redacted domain errors. These are
product policy, not unfinished extraction. Empty replies retain exact six-byte
admission. Management calls remain Candid; application protocols owned by IC Wire
retain their existing representation.

Agent preserves canonical receiver/target/arguments and signed ingress, mandatory
retention before one dispatch, original response-versus-pending classification,
reply ceilings and pending spending. No automatic polling, retry, root-key fetch
or replacement identity is introduced. No storage or memory schema changes.

## Dependency enforcement

`make dependency-boundaries-check` uses Shared Tooling 0.24.0’s canonical checker
with `ci/cargo-dependency-boundaries.json`. CI/release validation rejects direct
upstream IC declarations across normal/build/dev/target edges by canonical package
identity, including aliases. The independent Core MSRV consumer separately rejects
transitive Agent reachability; both independent consumers exclude simulator/WAT.

The direct gate does not constrain transitive features, inspect generated/import
paths, discover independent workspaces or automatically recognize new package
names. Review those separately when dependencies or features change. This
repository has two maintained packages, no independent workspace, production Rust generator,
deployed canister or CDK runtime integration. Consumer-fixture generators emit no
old Candid/CDK paths.

Candid and SDK remain necessary production transitives. The complete development
lock retains PocketIC’s management DTO 0.8.0 alongside Platform’s 0.11.0. Testkit
0.54 no longer introduces the standalone codec or CDK family. Simulator packages
do not enter the libraries’ production graphs. Deployment admission belongs to the
consuming application’s exact target/features, successful locked build and artifact
identity.

## Provenance and qualification

The published archive SHA-256 matches the registry index:
`4a3adb6d9785133587b854b66b37dfdd8f7945555bdc59b6d0be26995187484d`.
It records source commit `57fcf8d4f55914dedf0a25e70cec2e8affb3fb82` and publication
at 2026-10-18 10:15:22 UTC. No sibling checkout or Cargo patch supplies the dependency.

The completed 0.1.8 batch passed all 634 native tests, including 34 Core and
12 Agent PocketIC cases, strict Clippy, all-target builds, Rustdoc, Rust 1.91 checks,
both independent consumers and both exact package archives. A separate Rust 1.91
fixture proves all 24 management exports and public Core metadata/status types
are the original 0.11.0 identities; it also compiles for Wasm. Six production target
trees retain Core/Agent isolation without simulator/parser/CDK leakage. Cross-target
inspection and Wasm compilation do not establish native macOS or installed-IC
qualification.

Receipts: `target/msrv-consumers.wKh3ry/`, `target/package-check.e8wOUP/` and
`/tmp/backup-platform-refresh/`. Acceptance is recorded for
[Agent](https://github.com/dragginzgame/ic-platform/issues/9#issuecomment-6248490904)
and [management DTOs](https://github.com/dragginzgame/ic-platform/issues/13#issuecomment-6248761572).
The [adoption archive](status/archive-2026-10-18.md) preserves intermediate releases,
resolved blockers and earlier receipts. The [current handoff](status/current.md)
records the pending batch and validation limits.
