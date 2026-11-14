# Development setup

The virtual workspace has one unpublished library under `crates/ic-backup`.
Local inventory, payload checks and capture intent use published Host filesystem/artifact primitives and Platform
identity and management DTOs, with Serde/JSON and typed errors. No IC transport is selected yet.

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
`make check` / `make ci` aggregate those checks. `PACKAGE=ic-backup` selects Rust
checks; formatting and link checks remain repository-wide. For ordinary inventory
changes, select the owning tests and lint/docs checks:

```bash
cargo test --locked --offline -p ic-backup --lib store::
cargo clippy --locked --offline -p ic-backup --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --locked --offline -p ic-backup --no-deps
cargo test --locked --offline -p ic-backup --doc
```

Check for active commands using this workspace's target before compilation or
source/lock edits. Test fixtures remain under `target/local-store-tests/`, including
failed cases. The FIFO replacement regression uses the native Unix `mkfifo` utility (Linux/macOS).
Select capture intent tests with `cargo test --locked --offline -p ic-backup
store::tests::capture`; select framed-ledger tests with the same command and
`store::capture::ledger::tests` as its filter. Its acknowledged child-process death case verifies original
intent and target-claim retention, not attempt spending or filesystem power loss.
Use `store::capture::transfer` for planning and bounded raw reply tests;
`store::capture::transfer::metadata` selects the metadata decoder alone.
Use `store::capture::status` for decoded status preconditions and exact management
caller membership; `store::capture::status::wire` selects bounded raw status
projection and rejection tests. These exercise Platform DTOs/codecs but do not qualify IC effects. Native filesystem tests
establish no IC capture/load behavior.

CI configures Linux x86-64 and macOS ARM64/x86-64, plus formatting and a separate
pinned history secret scan. Local Linux validation does not qualify macOS.

Add ordinary tests beside implemented code. Introduce an independent
`qualification/` workspace only when maintained test-only packages are needed.
Tooling can discover that location; fetch and validation must be extended then.
Future dependency adoption uses the root exact tooling pin, member inheritance,
lockfile and `make toko-tooling-update`; never edit managed payloads manually.
