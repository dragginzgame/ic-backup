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
Production external IC imports use Platform;
no local compatibility re-exports, direct upstream declarations or pin exceptions
remain. Application protocols owned by IC Wire keep their existing representation.

The selected feature families retain upstream Candid, management DTO, Agent and
external Principal Rust identities. Platform's separately owned ICRC packaging is
not selected by Backup. Production selects Platform 0.22.1, which re-exports the
canonical owned Principal from Platform Types 0.18.1.
Backup keeps its Platform imports; no direct Types dependency or runtime handle
is introduced. Native external Principal, management DTO and Agent SDK exports
retain their qualified upstream identities. Development-only PocketIC clients and
native simulator DTOs now belong to `ic_testkit::pocket_ic`. Backup enables no
Platform `pocket-ic` feature and selects no separate `ic-platform-pocket-ic` package.
Production management DTOs still use Platform's exact 0.11.0 identities; the
maintained Testkit client selects that same external DTO identity. Simulator types
remain private to tests and never enter a normal consumer.
Platform 0.21 removes the unused PocketIC facade; the prior Testkit migration
already supplies its replacement. Verified published archives retain identical
Candid, native management, Principal and Agent adapter source and dependencies.
Testkit 0.86.0 and production share Platform 0.22.1. The older 0.21 facade leaves
the development lockfile.
Independent normal consumers still qualify Core and Agent without development
features supplying their production capabilities.

Platform 0.22 restores optional cryptography and certification implementations to
registry dependencies. Backup selects neither feature family. Its Candid,
management and principal modules are unchanged; Agent exports its certificate and
tree through the same upstream SDK identities instead of direct certification
re-exports. Management DTO 0.11.0, external Principal 0.1.5, Types 0.18.1 and Agent
0.49.3 remain selected. No local codec, record or transport migration is needed.

Platform exports IC0/executor through its owned runtime package. The Cargo
singleton guard belongs to its optional executor/runtime features.
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
The consumed native codecs and external type suppliers are unchanged. Platform
0.14 moved owned Principal/Account definitions into its canonical Types supplier;
0.15 aligned that supplier's generation. Platform 0.15.3 keeps the facade paths
while Types 0.15.1 owns Principal representation, normalization, Serde and explicit
external conversions. Backup's public principal, DTO and SDK boundaries retain
the qualified external types.

Platform 0.16 moves optional certification/cryptography implementations into owned
modules. Backup selects none of those features. The Agent feature retains its
SDK exports and explicitly names upstream external certificate/hash-tree types;
Backup needs no conversion or new feature. Consumed Candid, native management,
Principal and Agent identities remain unchanged.

Platform 0.17 moves its native PocketIC client into a maintained package while
preserving the facade imports. It also selects Types 0.16; the changed optional
self-authentication implementation is not enabled by Backup. The consumed Candid,
management, external Principal and Agent facade source is unchanged. Internal
Principal normalization, hashes and wire contracts retain their existing behavior;
Backup exposes no owned Principal API requiring a consumer migration.

Platform 0.18 removes its unused stable-structures facade. Backup enables none of
that feature, and the consumed Candid, management DTO, external Principal and
Agent adapter sources remain unchanged. Types 0.17 supplies the production owned
Principal in that earlier graph.

Platform 0.19 selects Types 0.18 and the fallible PocketIC client 0.19. Published
archive review found the consumed Candid, native management, external Principal
and Agent facade sources unchanged; Types' Principal implementation is also
unchanged. Types' Account subaccount byte encoding changed, but Backup imports or
serializes no Account/Subaccount and needs no record migration for that cut.
Principal normalization, record hashes and method-specific wire bounds remain
qualified through Backup's own checks.

The workspace requires and resolves Host Artifacts/FS/Process 0.74.3. Consumers
sharing public Host error types must align their direct selections to 0.74.
Verified 0.74.3 archives preserve Artifacts source exactly; FS/Process changes
relocate unit tests and private fixture helpers, with runtime behavior unchanged.
The release metadata editor selects published Build 0.74.2; its CLI and consumed
release-metadata owner are byte-identical to 0.74.0. Build 0.74.3 is not selected
because it was not published when this adoption was reviewed. Testkit's root
minimum agreed with its existing locked 0.84.1 selection in that adoption. This update supplies
no new runtime mechanism replacing Backup's remaining safety machinery.
Platform 0.22 is the production facade selection. Backup already uses Testkit's
client, so removal of Platform's simulator feature requires no further migration.
Backup retains its exact management ingress, method-specific response bounds and
original accounting rather than adopting a second snapshot workflow.
Testkit 0.84 embeds the maintained client previously supplied by Platform's
0.20 client package. Current Testkit 0.86.0 and Backup's production dependencies
share Platform 0.22.1. Canonical Types 0.18 and the
same external DTO identities
remain selected. Testkit 0.84.1's Rust
source matches reviewed 0.84.0. Review against the earlier client found namespace,
lint and formatting changes in production source; its client failure fixture also
sets blocking socket mode explicitly for macOS. Its Apache-2.0 copyrights and
provenance remain included in the published Testkit archive, whose declared license
is MIT AND Apache-2.0. No client implementation is copied into Backup.
The older Platform/client packages and unused `zlib-rs` leave the development lock.
Testkit uses the Rust backend for gzip; no compressed-byte identity is inferred
from source similarity. Other package versions/checksums remain selected. Canonical
Types 0.18 and target-specific Runtime 0.19.1 remain; optional Wire/Metrics stay
disabled. Neither production graph gains simulator or runtime features.
Runtime 0.19.1 is selected through Testkit's Wasm-only `ic0` dependency. Its
published archive matches the registry checksum and source commit `74a5071`;
the selected IC0 source is byte-identical to 0.19.0. Changes relocate tests in
unselected CDK modules. All three supported normal host target graphs exclude
Runtime for both Backup libraries. No direct Runtime declaration or executor
feature is needed.
Both fixtures use Testkit's `build_instance` / `build_instance_async` with their
original topology, state directory, request bound and retained raw output files.
Its existing 120-second startup deadline covers readiness and instance creation.
Keep the managed instance owner through fixture setup. Then `into_parts` transfers
the client and exact owned server into the fixture's
explicit client-before-server field order. The temporary URL bridge and local
blocking startup worker are removed; Testkit owns asynchronous readiness and
cancellation cleanup. No reset, retry or funding policy is adopted.
Testkit 0.85 requires an explicit client endpoint and removes its implicit server
launcher/downloader. Backup's existing managed builders supply that endpoint and
retain server custody. Direct client flate2/semver dependencies are removed; gzip
assets still use Host Artifacts. PocketIC 16.1.0, qualified external DTOs, request
bounds, original journals and application admission remain unchanged.
Testkit 0.85.1 preserves all 56 Rust source files from 0.85.0 while selecting
Platform 0.22. Types 0.18.1 explicitly enables external Principal Serde; Backup
already receives that capability through Candid. Its canonical Principal code
and normalization/hash bytes are unchanged; only the external-adapter test gate
and regression fixtures change.
Testkit 0.86 updates its optional Wire dependency to 0.17. Backup leaves that
feature disabled; no Wire package enters its lockfile. All 56 published Rust files
match 0.85.1, and the consumed dependency identities and features remain unchanged.
The incoming locked Platform 0.22.1 patch changes only unselected BLS/signature
modules. Its other 63 Rust files match 0.22.0, including the consumed codecs,
native management, Principal and Agent adapters. No consumer migration is needed.
Platform operations return checked
results; expected canister rejections are extracted from `ClientError::Canister`,
preserving the original structured reply rather than accepting transport failure.
Testkit retains typed startup failures instead of panic-string adapters. Uncertain
calls are never replayed. Authenticated server selection remains Testkit-owned
PocketIC 16.1.0; optional Wire and Metrics remain outside Backup's graph.

| Surface | Owner and maintained constraint |
| --- | --- |
| Candid contracts and bounded decoding mechanics | Platform; Backup selects method-specific byte/work/skip/type/header limits |
| Internal principal admission | Platform root `Principal`, re-exported from canonical Types; Backup enforces bounded canonical text/raw bytes |
| External Candid/SDK principals | Platform contract/external exports over qualified upstream 0.1.5, with explicit boundary conversions |
| All 24 snapshot/status/transfer/upload DTOs | Platform management exports over exactly upstream 0.11.0 |
| Native SDK identities and operations | Platform Agent exports, including PEM |
| URL/client configuration | Explicit Reqwest dependency; Backup owns middleware, trust, timeout and retry policy |
| Native simulator client identities | Testkit’s development-only `pocket_ic` module, imported directly |
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
DTOs and Agent enable Candid defaults, including printer machinery. The Testkit
development client shares the qualified management DTO 0.11 identity;
the older 0.8 package is removed. Simulator clients remain development-only.
CLI features are selected only for separately installed tools; library dependencies
do not enable them. Production-tree checks and exact public-identity probes are
separate from direct declaration checks; deployment admission belongs to the
application's exact target/features, locked build and artifact identity.

Independent normal consumers exclude Testkit, PocketIC and WAT. Backup's public
Host error types belong to the 0.74 line; consumers exchanging
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
