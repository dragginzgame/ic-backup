# Platform dependency boundary

Published `ic-platform` owns Backup's external IC identities, Candid codecs and
Agent SDK exports. [Cargo.toml](../Cargo.toml) and [Cargo.lock](../Cargo.lock) own
the selected requirements and resolved versions. [The handoff](status/current.md)
records source verification, qualification and limits; historical comparisons
remain with their exact sources below.

## Ownership and features

Core disables Platform defaults and selects `candid` and
`management-canister-types`. Only Agent adds `agent`; an independent Core consumer
remains Agent-free. Backup leaves parser, executor/runtime, certification, ledger,
service generation and CDK features unselected. It has no direct Types, IC SDK or
Canic runtime dependency and no sibling Cargo patch.

| Capability | Maintained owner and boundary |
| --- | --- |
| Internal principal admission | Platform root `Principal`, supplied by canonical Platform Types; Backup owns bounded text admission and explicit conversions |
| External Candid/SDK principals | Platform contract/external exports over qualified upstream 0.1.5 |
| Management DTOs | Platform management exports over qualified upstream 0.11.0 |
| Candid encode/decode mechanics | Platform, with Backup's exact method-specific projections, bounds and errors |
| Native SDK identities and operations | Platform Agent exports, including PEM |
| HTTP client configuration | Explicit Reqwest dependency; Backup owns middleware, trust, deadlines and retry policy |
| Simulator clients and DTOs | Development-only `ic_testkit::pocket_ic`; no Platform PocketIC feature or separate client package |
| Simulator assets/readiness and managed processes | Testkit; callers retain original plans, private state, byte evidence, request budgets and explicit client/server lifetime |
| Filesystem, artifact and process primitives | Host FS, Host Artifacts and Host Process; Backup owns product custody and recovery transitions |

The external principal and management DTO suppliers are reviewed compatibility
constraints. Changes require normalization, identity/hash and codec/golden/bounds
review, public compatibility assessment and relevant native/IC qualification under
[AGENTS.md](../AGENTS.md#qualified-dependency-constraints). Backup imports internal
principal identity only through Platform, with explicit conversions to external
Candid/SDK identities. Consumers exchanging public Host error values align direct
Host dependencies with [the workspace selection](../Cargo.toml).

Backup retains bounded wire projections, strict reserved-marker admission, exact
result framing and redacted domain errors. Persisted JSON sequence admission and
Candid sequence admission retain their distinct overflow/work rules. The
[wire contracts](implementation-contracts.md#management-wire-and-response-admission)
own method-specific bounds; [the transport guide](agent-transport.md) owns exact
receiver/target/arguments, signed-ingress retention and one submission. These
boundaries confer no automatic retry, polling, root-key discovery or new identity.

The [simulator guide](pocketic-qualification.md#current-simulator-dependency-boundary)
owns caller qualification. Fixtures supply explicit finite request budgets and
startup deadlines, hold managed ownership through setup, retain raw output and
qualify deletion failures. Client-before-server teardown, original journals and
application safety stay local. Testkit selection does not adopt reset, reusable
baseline, funding or hidden-retry policy.

## Dependency enforcement

`make dependency-boundaries-check` uses the canonical checker with
[ci/cargo-dependency-boundaries.json](../ci/cargo-dependency-boundaries.json).
It rejects forbidden direct upstream declarations by canonical package identity,
including aliases and normal/build/dev/target edges. Independent normal consumers
separately exclude Testkit, PocketIC and WAT; the Core consumer also rejects
transitive Agent reachability. Installed tool CLI features stay outside library
features. [Development](development.md#focused-product-checks) owns focused checks;
[the release guide](releasing.md) owns full qualification and delivery.

The direct declaration gate does not constrain transitive features, inspect
Rust imports or generated source, discover independent workspaces or recognize
new forbidden package identities automatically. Review those boundaries when
changing selections. Candid and the SDK remain necessary production transitives;
source/import checks, normal dependency graphs and exact public-identity probes
prove different obligations. Application deployment admission still requires its
actual target, features, locked build and artifact identity.

## Adoption history

The [complete pre-cleanup boundary and version comparisons](https://github.com/dragginzgame/ic-backup/blob/b1c1c33f1e9d80c2c22a605ed4b39383b281ede7/docs/ic-platform.md)
retain the Platform, Types, Testkit and Host adoption chronology, archive/source
comparisons, license/provenance observations and historical qualification limits.
The [earlier adoption record](https://github.com/dragginzgame/ic-backup/blob/f241568f3714c0fd9283fd505ff34007c6bcf3e8/docs/status/archive-platform-adoption-2026-10-21.md)
retains original package versions, checksums, source identities, receipts and
resolved blockers. Both remain evidence for their original graph and host scope.

Current qualification belongs to the handoff. Native macOS, installed-IC and Canic
application acceptance retain their separate owners; source similarity and owner
CI do not establish those results. [The retention summary](reports/README.md)
explains historical Git custody. Original receipts, journals, byte evidence and
failed bundles are preserved.
