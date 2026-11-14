# Development setup

The virtual workspace has one API-free library under `crates/ic-backup`.
It has no runtime dependencies and publication is disabled.

## Preparation

Use Bash, GNU Make, Git, Perl and a SHA-256 backend. Prepare the exact compiler
and shared tools explicitly:

```bash
rustup toolchain install 1.100.0 --profile minimal --component rustfmt --component clippy
make install-tools
make install-hooks
make fetch-dependencies
```

Validation is locked and offline; it does not install tools. Hook activation is
explicit through `make install-hooks`. Tooling 0.43.0 is adopted from its registry
archive with SHA-256
`060d3507b66eb290ce12bc2572098f7e7cb1e12e9d6697a9b1a5d1552fcb2588`.
The canonical exporter installs nine selected entrypoints, including the shared
pre-commit formatter hook, with derived companions, and owns [.toko-tooling.snapshot](../../.toko-tooling.snapshot).
No producer regression suites or release/publication entrypoints are selected.

## Commands

```bash
make deps-list
make deps-update
toko-cloc -d 0
make check
```

`deps-list` lists local package versions, not the transitive graph or publication
status. `deps-update` explicitly updates Cargo dependencies and can use the
network. To update one dependency, use `DEPENDENCY_PACKAGE=name`; add
`DEPENDENCY_PRECISE=version` for a specific compatible version. The exact tooling
pin still controls its selection. `toko-cloc -d 0` runs the installed PATH reporter; without the launcher,
use `bash vendor/toko-tooling/scripts/dev/toko-cloc -d 0` after host-tool setup.

Focused checks are `make toko-tooling-check`, `make doc-links-check`,
`make fmt-check`, `make clippy`, `make rustdoc` and `make test`.
`make check` / `make ci` aggregate those scaffold checks. `PACKAGE=ic-backup`
selects Rust checks; formatting and link checks remain repository-wide.
There are no product tests yet; a successful empty test run proves no IC behavior.

CI configures Linux x86-64 and macOS ARM64/x86-64, plus formatting and a separate
pinned history secret scan. Local Linux validation does not qualify macOS.

Add ordinary tests beside implemented code. Introduce an independent
`qualification/` workspace only when maintained test-only packages are needed.
Tooling can discover that location; fetch and validation must be extended then.
Future dependency adoption uses the root exact tooling pin, member inheritance,
lockfile and `make toko-tooling-update`; never edit managed payloads manually.
