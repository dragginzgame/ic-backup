# IC Backup

A fresh workspace for a standalone host-side Internet Computer backup and recovery
library. Canic is a required integration alongside standalone consumers. Its
adapter consumes the public library contract; application coordination and
control-plane authority remain Canic-owned. The planned v1 stores complete
snapshot bundles on the operator host and provides local backup history. Canic can coordinate
independent operations for several canisters; Blob Storage integration is v2.

This repository is a scaffold. Its only package, `ic-backup`, exposes no product
API and has no runtime dependencies. Backup, restore, CLI and simulator workflows
have not been implemented. Publication is disabled; `0.68.0` is an unpublished
development version following the legacy `0.67.x` line.

The [legacy repository](https://github.com/dragginzgame/ic-backup-legacy) retains
existing implementations, published history and recovery obligations. Its source,
backlog, release automation and retained data do not define the new implementation.

After [tool preparation](docs/development/setup.md):

```bash
make fetch-dependencies
make deps-list
make check
```

See the [library design proposal](docs/design/library-design.md),
[legacy assessment](docs/design/legacy-assessment.md),
[sibling integration review](docs/design/sibling-integration.md),
[proposed operation contract](docs/design/operation-contract.md),
[local storage and inventory](docs/design/local-store.md),
[restart direction](docs/design/restart.md), [local instructions](AGENTS.md)
and [pending notes](CHANGELOG.md). Every package remains non-publishable until
the implementation is mature and publication is explicitly authorized.
