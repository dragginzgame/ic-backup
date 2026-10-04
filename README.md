# ic-backup

Independent, host-side backup and same-release recovery for Internet Computer
canisters. The intended product provides verified snapshot bundles, reviewed
execution plans, durable interruption recovery and safe local retention.

This repository currently contains a design and contributor instructions only.
It has no Rust implementation, runnable CLI or published packages. The public
repository is [dragginzgame/ic-backup](https://github.com/dragginzgame/ic-backup).
No backup or restore capability is claimed by this bootstrap.

- [Comprehensive extraction and implementation design](docs/extraction-design.md)
- [Agent and contributor instructions](AGENTS.md)
- [Current handoff](docs/status/current.md)
- [Source baseline and provenance](docs/source-baseline.json)

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

MIT licensed. Preserve the Canic contributor attribution when extracting code.
