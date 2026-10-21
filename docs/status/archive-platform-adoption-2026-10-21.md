# Platform adoption record — archived 2026-10-21

This is the complete incoming [ic-platform.md](../ic-platform.md) adoption record,
with relative links rebased for this directory. Its version selections, setup
commands and qualification claims describe their recorded source and graph.
Use the current guide and [handoff](current.md) for maintained instructions
and current qualification; historical prose is not a setup or release authority.

---

# Platform dependency boundary

The repository selects published
[ic-platform 0.5.1](https://crates.io/crates/ic-platform/0.5.1), with requirement
`0.5` and empty defaults. Its archive SHA-256 is
`4469fe3e1e7ea4b8cb57db1e32d9906ca95fdb93df5110e4a289352917dc9c76`,
source revision `fe628b35d3a5b33c0d03928975aa16759372aec9`.
Qualification boundaries are recorded in the [current handoff](../status/current.md);
the provenance below preserves the original 0.1.8 adoption. All production and
test Candid, Principal, SDK and management DTO imports use Platform. There are no
direct declarations of those upstream packages, CDK-family packages or the old
Testkit codec, and no local compatibility re-exports or pin exceptions.

## Ownership and features

The root catalog selects `default-features = false` with `candid` and
`management-canister-types`. Only `ic-backup-agent` adds `agent`; Core’s independent
consumer remains Agent-free. Parser, executor, ic0, certification and ledger
features are not selected. Service declaration generation (`candid-service`) and
the CDK entrypoint/runtime surfaces are unused; 0.5 requires no source adapter or
feature expansion. Published archive inspection confirms that principal, Agent,
management exports and bounded decoder behavior are unchanged from 0.4.1. The
Candid contract change only gates unused service macros/printer machinery behind
`candid-service`. Plain `candid` no longer requests the printer, but management
DTOs and Agent still enable upstream Candid defaults in the resolved production
graph; this adoption does not remove the transitive printer.

Platform owns its root `Principal` and `PrincipalError` representations and
uses `MIT AND Apache-2.0` licensing. Backup keeps bounded canonical admission on
the owned principal, converting explicitly into external management DTOs and
Agent calls. Candid status projections use `candid::contract::Principal`;
native SDK fixtures use `principal::ExternalPrincipal`. Both external exports
retain upstream 0.1.5. No Backup public signature exposes the changed root
principal or error identity. Backup does not enable Platform's optional CDK
runtime or macros.

| Surface | Owner and selection |
| --- | --- |
| Candid contracts and bounded decoding | Platform over Candid 0.10.39 |
| Internal principal admission | Platform-owned root `Principal`; existing bounded canonical text/raw bytes |
| External Candid/SDK principals | Platform contract/external exports over upstream 0.1.5 |
| All 24 snapshot/status/transfer/upload DTOs | Platform management exports over exactly upstream 0.11.0 |
| Native SDK identities and operations | Platform Agent exports over SDK 0.49.2 with PEM |
| URL/client configuration | Explicit Reqwest dependency; Backup owns middleware, trust, timeout and retry policy |
| Native simulator client identities | Platform over PocketIC 16.1.0, forwarded by development-only Testkit |
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

`make dependency-boundaries-check` uses Shared Tooling 0.25.9's canonical checker
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
lock retains PocketIC's management DTO 0.8.0 alongside Platform's 0.11.0. Published
Testkit 0.62.1 selects Platform 0.5, consolidating production and development on
0.5.1. Its native call helpers use the same decoder-limit/error identities;
external principal/PocketIC identities and caller-selected limits are unchanged.
No conversion shim or local patch is needed. Production and Testkit share Host 0.62.1
from the consolidated IC Tooling repository. Platform's PocketIC exports
include `RawEffectivePrincipal` without changing the upstream client identity.
Testkit's native CLI brings Clap into the development graph only; no local patch
supplies any dependency.
The selected client preserves original PocketIC identities without a copied DTO,
direct PocketIC declaration or routing fallback. The lock contains neither the
standalone Testkit codec nor CDK-family packages. Simulator packages do not enter
the libraries' production graphs. Deployment admission belongs to the consuming
application's exact target/features, successful locked build and artifact identity.

## Earlier 0.1.14 graph qualification

Platform 0.1.14's registry archive matches the locked SHA-256
`03c2300b6d347bc1c19ba35fcf4c31069134aefe1d4ec5bf7b5a8dfbc411285f`.
Its Candid, Agent and management module sources are unchanged from 0.1.10 and
0.1.13. That graph passed all-target/all-feature compilation, strict Clippy,
actual Rust 1.91 checks and independent Core/Agent consumers, 78 IC codec/model
cases, three Agent unit cases, 14 HTTP cases and 46 Core/Agent PocketIC cases on
Linux. Declaration checks and production-tree inspection preserve Core/Agent
isolation without parser, CDK or simulator reachability. These focused results
do not qualify native macOS, installed-IC or complete application acceptance.
The [current handoff](../status/current.md) records the selected graph and evidence.

## Original adoption provenance and qualification

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
The [adoption archive](../status/archive-2026-10-18.md) preserves intermediate releases,
resolved blockers and earlier receipts. The [current handoff](../status/current.md)
records the pending batch and validation limits.
