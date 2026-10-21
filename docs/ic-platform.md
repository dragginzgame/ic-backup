# Platform dependency boundary

Published `ic-platform` owns Backup's external IC types, Candid codecs and Agent
SDK boundary. [Cargo.toml](../Cargo.toml) and [Cargo.lock](../Cargo.lock) own the
selected requirement and resolved version. [The current handoff](status/current.md)
records source verification, qualification and limits. No sibling checkout or
local Cargo patch supplies dependencies.

## Ownership and features

The workspace disables defaults and selects `candid` and
`management-canister-types`. Only Agent adds `agent`; an independent Core consumer
remains Agent-free. Parser, executor, ic0, certification, ledger, service generation
and CDK runtime features are unused. Production and test imports use Platform;
no local compatibility re-exports, direct upstream declarations or pin exceptions
remain. Application protocols owned by IC Wire keep their existing representation.

| Surface | Owner and maintained constraint |
| --- | --- |
| Candid contracts and bounded decoding mechanics | Platform; Backup selects method-specific byte/work/skip/type/header limits |
| Internal principal admission | Platform-owned root `Principal`; Backup enforces bounded canonical text/raw bytes |
| External Candid/SDK principals | Platform contract/external exports over qualified upstream 0.1.5, with explicit boundary conversions |
| All 24 snapshot/status/transfer/upload DTOs | Platform management exports over exactly upstream 0.11.0 |
| Native SDK identities and operations | Platform Agent exports, including PEM |
| URL/client configuration | Explicit Reqwest dependency; Backup owns middleware, trust, timeout and retry policy |
| Native simulator client identities | Platform, forwarded by development-only Testkit |
| Simulator assets and managed processes | Testkit; Backup owns original plans, byte evidence and fixture custody |

The exact external principal and management DTO identities are reviewed
compatibility constraints, not general exact-pin policy. Changes require
normalization, identity/hash and codec/golden/bounds review, public compatibility
assessment and relevant native/IC qualification as described in
[AGENTS.md](../AGENTS.md#qualified-dependency-constraints).

Backup retains bounded DTO visitors, strict reserved-marker admission, redacted
domain errors and exact six-byte empty replies. Management calls remain Candid.
Agent retains canonical receiver/target/arguments, signed ingress and mandatory
retention before one dispatch. Response-versus-pending classification, reply
ceilings and pending spending stay local. No automatic polling, retry, root-key
fetch, replacement identity or storage/memory migration is introduced by this
boundary.

## Dependency enforcement

`make dependency-boundaries-check` uses the canonical checker with
[ci/cargo-dependency-boundaries.json](../ci/cargo-dependency-boundaries.json).
It rejects direct upstream declarations by canonical package identity, including
aliases and normal/build/dev/target edges. The independent Core MSRV consumer
separately rejects transitive Agent reachability; both independent consumers
exclude simulator/WAT packages.

The direct gate does not constrain transitive features, inspect generated/import
paths, discover independent workspaces or recognize new package names
automatically. Review those separately when selections change. This repository
has two maintained packages and no independent workspace, production Rust
generator, deployed canister or CDK runtime integration. Consumer fixture
generators emit no old Candid/CDK paths.

Candid and the SDK remain necessary production transitives. Upstream management
DTOs and Agent enable Candid defaults, including printer machinery. The complete
development graph retains PocketIC's management DTO 0.8 separately from the
qualified production 0.11 identity. Simulator clients and CLI dependencies remain
development-only. Production-tree checks and exact public-identity probes are
separate from direct declaration checks; deployment admission belongs to the
application's exact target/features, locked build and artifact identity.

## Adoption history

The [archived adoption record](status/archive-platform-adoption-2026-10-21.md)
preserves exact package versions, archive checksums, source identities, original
adoption receipts and resolved blockers. Those receipts retain their original
graph and host scope. Current qualification is recorded in the handoff; neither
cross-compilation nor owner CI establishes native macOS, installed-IC or complete
application acceptance.
