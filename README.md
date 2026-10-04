# ic-backup

Independent, host-side backup and same-release recovery for Internet Computer
canisters. The intended product provides verified snapshot bundles, reviewed
execution plans, durable interruption recovery and safe local retention.

This repository contains an independent Rust workspace with one library package
at `crates/ic-backup`, contributor tooling and an extraction design. The library
currently establishes the package boundary; backup/restore APIs and the CLI
have not been extracted. No backup or restore capability is available yet.
The public repository is
[dragginzgame/ic-backup](https://github.com/dragginzgame/ic-backup).

Development uses Rust 1.99.0, edition 2024 and a minimum supported Rust version
of 1.91.0, following the sibling library conventions. The workspace owns package
metadata, dependency declarations and lints. Builds stay in this repository's
`target/`. The initial unreleased version is `0.1.0`; registry publication is
disabled.

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

The next implementation work is the source inventory and current v1 contract
freeze described in the design. Do not start by deleting Canic's recovery code
or copying its entire CLI and deployment framework.

MIT licensed, with Canic and sibling tooling contributor attribution retained.
