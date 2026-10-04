# ic-backup

Independent, host-side backup and same-release recovery for Internet Computer
canisters. The intended product provides verified snapshot bundles, reviewed
execution plans, durable interruption recovery and safe local retention.

This repository contains an independent Rust workspace with one library package
at `crates/ic-backup`, contributor tooling and an extraction design. The library
provides local artifact checksums, secure staging, durable verified-directory
publication, bounded JSON persistence, journal/layout locking, durable restore
dependencies, inherited command custody and local download journals extracted
from Canic. Journal transitions retain exact snapshot identity and reconcile
verified artifact publication after interruption. References
remain retained until terminal completion and subprocess-custody contracts
support their release.
Local attempt journals bind exact declared operation identity and immutable
mutation/observation limits. They durably consume allowances and preserve
unresolved outcomes across interruption; integrations still supply fresh
authority, qualified receipts and remote-effect reconciliation.
Canonical physical inventories retain exact declared targets and parent forests.
Pure selection supports exact principals, direct children and descendants;
inventory hashes bind declarations while authoritative discovery stays integration-owned.
Explicit operation dependency graphs provide deterministic planning order and pure
readiness checks. Integrations qualify application ordering and actual completion;
graph declarations and ready views grant no authority to execute effects.
Immutable operation plans bind those declarations to exact targets/request digests
and original per-operation and aggregate attempt ceilings. Their full canonical
digest supplies journal intent; fresh authority and backend qualification remain
integration-owned.
Pure execution progress joins the complete original plan with exact retained attempt
journals. It checks causal Applied evidence and reports pending/exhausted allowances;
missing journals never become fresh zero-consumption declarations.
The IC request codec provides closed typed status/inventory, stop/start, capture
and load declarations with exact Candid bytes and request digests. It checks payload
binding for original mutations and separately reserved reconciliation observations.
A separate membership port binds provider observations to original intent, exact
context and inventory, with pure validation and no default provider. Integrations
still qualify freshness, permissions and application consistency.
The control port checks direct caller-controller evidence for exact mutation
payloads, with pure admission and no implicit delegation or dispatch permission.
The snapshot-read port separately checks controller, public and exact allowed-viewer
access for independently bound snapshot-list requests. Read evidence grants no
mutation control or paid-call authority; actual providers remain integration-owned.
Consistency requirements retain the original requested guarantee. A separate port
checks current stopped/drained targets and exact active application fence evidence,
without acquiring or releasing fences. Application qualification remains with integrations.
Capture/restore runners, an IC transport and the CLI have not been extracted.
It cannot yet perform a canister backup or restore.
The public repository is
[dragginzgame/ic-backup](https://github.com/dragginzgame/ic-backup).

Development uses Rust 1.99.0, edition 2024 and a minimum supported Rust version
of 1.91.0, following the sibling library conventions. The workspace owns package
metadata, dependency declarations and lints. Builds stay in this repository's
`target/`. Package metadata permits crates.io publication; `make publish`
delegates to Cargo for the current library version.

```bash
make hooks-install
make check
make test
```

`make help` lists development and release commands. CI runs the native library
and repository-tooling gate. PocketIC qualification will accompany platform
effects when they are implemented.

- [Comprehensive extraction and implementation design](docs/extraction-design.md)
- [Agent and contributor instructions](AGENTS.md)
- [Current handoff](docs/status/current.md)
- [Source baseline and provenance](docs/source-baseline.json)
- [Implemented extraction boundary](docs/extraction-boundary.md)
- [Fresh extraction source provenance](docs/extraction-source.json)
- [Development commands](docs/development.md)
- [Release workflow](docs/releasing.md)
- [Repository tooling provenance](docs/tooling-provenance.json)

The starting implementation is Canic's `canic-backup` crate. Generic snapshot,
artifact and recovery behavior will move here. Canic will own its Fleet discovery,
authorization and application-consistency adapter. Other applications can supply
their own adapters without depending on Canic.

The proposed package boundary is `ic-backup` for the library, `ic-backup-icp` for
the ICP CLI transport and `ic-backup-cli` for the standalone `ic-backup` command.
These are design names; package availability and naming must be checked before
publication. Start with one library and add the other packages when their
implementation batches require them.

The local machinery has fresh native regression evidence. The remaining B1
work freezes generic authority, journal and runner contracts before their
extraction. Canic adoption and real IC qualification remain separate work.

MIT licensed, with Canic and sibling tooling contributor attribution retained.
