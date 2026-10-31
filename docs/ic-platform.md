# Platform dependency boundary

Published `ic-platform` owns Backup's external IC types, Candid codecs and Agent
SDK boundary. [Cargo.toml](../Cargo.toml) and [Cargo.lock](../Cargo.lock) own the
selected requirement and resolved version. [The current handoff](status/current.md)
records source verification, qualification and limits. No sibling checkout or
local Cargo patch supplies dependencies.

## Ownership and features

The workspace disables defaults and selects `candid` and
`management-canister-types`. These features include `principal-external`, which
supplies the owned `principal` capability and explicit external byte conversions.
Platform separates public-key self-authentication into its own optional feature;
Backup does not use that owned helper or enable it. Only Agent adds
`agent`; an independent Core consumer remains Agent-free. Parser, executor, ic0,
certification, ledger, service generation and CDK runtime features are unused.
Production and test imports use Platform;
no local compatibility re-exports, direct upstream declarations or pin exceptions
remain. Application protocols owned by IC Wire keep their existing representation.

The selected feature families retain upstream Candid, management DTO, Agent and
external Principal Rust identities. Platform's separately owned ICRC packaging is
not selected by Backup. Production selects Platform 0.13; Testkit 0.72 retains
Platform 0.11 as a development dependency. Native external Principal, management
DTO and PocketIC exports retain their upstream identities across both selections.
No Platform-owned runtime handles cross this boundary.
Independent normal consumers still qualify Core and Agent without development
features supplying their production capabilities.

Platform embeds its IC0/executor. The Cargo singleton guard belongs to its
optional executor/runtime features.
Backup enables neither feature, so its host consumers can coexist with an
unrelated registry executor. Consumers enabling Platform's runtime must still
select one runtime owner. Native management exports retain the qualified
upstream 0.11.0 identities. Platform's Wasm-only owned DTO changes do not turn
these host libraries into canister libraries.

Platform supplies prepared calls under `runtime`; Backup uses its existing host
Agent transport with that runtime disabled. Platform 0.11 removes
its optional Wire schemas; Wire owns that integration through `platform-types`.
Backup needs neither those schemas nor Testkit's optional Wire call API, so no
direct Wire dependency, new protocol or runtime is added.

Platform 0.13 removes its gzip facade; generic artifact codecs belong directly to
Host Artifacts. Backup already consumes Host directly and needs no codec migration.
The consumed native IC implementation and external type suppliers are unchanged.
Platform's owned Principal has the new dependency identity internally, while
Backup's public principal, DTO and SDK boundaries retain the qualified external types.

| Surface | Owner and maintained constraint |
| --- | --- |
| Candid contracts and bounded decoding mechanics | Platform; Backup selects method-specific byte/work/skip/type/header limits |
| Internal principal admission | Platform-owned root `Principal`; Backup enforces bounded canonical text/raw bytes |
| External Candid/SDK principals | Platform contract/external exports over qualified upstream 0.1.5, with explicit boundary conversions |
| All 24 snapshot/status/transfer/upload DTOs | Platform management exports over exactly upstream 0.11.0 |
| Native SDK identities and operations | Platform Agent exports, including PEM |
| URL/client configuration | Explicit Reqwest dependency; Backup owns middleware, trust, timeout and retry policy |
| Native simulator client identities | Platform's development-only `pocket-ic` feature, imported directly |
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
qualified production 0.11 identity. Simulator clients remain development-only.
CLI features are selected only for separately installed tools; library dependencies
do not enable them. Production-tree checks and exact public-identity probes are
separate from direct declaration checks; deployment admission belongs to the
application's exact target/features, locked build and artifact identity.

Independent normal consumers exclude Testkit, PocketIC and WAT. Backup's public
Host error types belong to the 0.72 line; consumers exchanging
those values must align direct Host dependencies. Their fields, source chains,
recovery behavior and serialized data retain their existing contracts.

Backup selects neither the parser feature nor removed Candid frontend compiler
exports. Management DTO 0.11.0, external principal
0.1.5, Agent identities and the consumed codecs remain unchanged. The local
bounded status, list, metadata and data projections serve the management-canister
protocol; test-only malformed shapes qualify rejection of that same external
wire format. They are separate from Backup's domain and persisted records.

## Adoption history

The [archived adoption record](status/archive-platform-adoption-2026-10-21.md)
preserves exact package versions, archive checksums, source identities, original
adoption receipts and resolved blockers. Those receipts retain their original
graph and host scope. Current qualification is recorded in the handoff; neither
cross-compilation nor owner CI establishes native macOS, installed-IC or complete
application acceptance.
