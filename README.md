# ic-backup

Independent, host-side backup and same-release recovery for Internet Computer
canisters. The intended product provides verified snapshot bundles, reviewed
execution plans, durable interruption recovery and safe local retention.

This repository contains an independent Rust workspace with one library package
at `crates/ic-backup`, contributor tooling and an extraction design. The library
provides local artifact checksums, secure staging, durable verified-directory
publication, bounded JSON persistence and journal locking extracted from Canic.
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
