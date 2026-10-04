# Development

The root `Cargo.toml` owns the workspace, package metadata, dependency versions
and shared lints. The sole member is `crates/ic-backup`. It provides the local
artifact and persistence mechanisms described in
[the implemented extraction boundary](extraction-boundary.md). Capture/restore
runners and transport remain unimplemented. Pure checksum records belong to
model; filesystem operations belong to ops.
Restore dependency records and immutable retention transitions belong to model;
layout exclusion, canonical journal-parent resolution and durable publication
belong to persistence ops. No reference release or prune API is exposed.

Rust 1.99.0 is pinned in `rust-toolchain.toml`, with rustfmt and Clippy. The
minimum supported version is 1.91.0. Install that toolchain separately for
`make check-msrv`. Native builds are the supported product lane; the toolkit
runs on an operator host.

## Commands and build ownership

`make help` lists the command family. `check`, `clippy`, `test`, `doc`,
`check-msrv` and `package` select `ic-backup` explicitly. `fmt` formats the
workspace; `fmt-check` checks it. `shell-check`, `release-check` and `hooks-check`
validate contributor tooling. `version` and `release-plan` inspect release
metadata without changing the workspace.

`publish-dry-run` and `publish` delegate to Cargo for the current library version.
They use locked dependencies and crates.io. Cargo checks package cleanliness and
publication eligibility; receipt/tag checks belong to repository release commands.

Native custody regressions require Python 3 on the selected Unix host. They use
real inherited descriptors and acknowledged owner/direct-child/descendant exit
with bounded fixture waits. The production library uses the pinned `command-fds`
dependency for owned child descriptor setup and contains no Rust unsafe code.
It does not invoke Python; those commands are local test fixtures. A transport
still needs its own output/deadline/retry and inherited-descriptor qualification.

The Makefile exports this checkout's absolute `target/` directory. Never point
it at Canic's target or add Cargo patches to sibling checkouts. Before editing
source or locks and before compilation, check for an active command using this
repository's build directory. Let that command finish first. Direct Cargo
commands must use the same local directory.

Compilation uses `--offline --locked`. `make deps` fetches the committed lockfile's
dependencies when needed; it does not select new versions. Serde/JSON, SHA-256,
thiserror and Unix rustix declarations belong in `[workspace.dependencies]`,
with package-level entries inheriting them. There are no Canic dependencies or
sibling checkout patches.

Run checks targeted to changed packages and behavior while developing. Full
validation requires a maintainer request or CI. `make ci`, `make validate` and
`make release-verify` run the full configured gate: dependency fetch, tooling,
formatting, native compilation, Clippy, tests, docs, MSRV and package verification.
CI runs this gate on Linux. Local package verification permits reviewed dirty
source and builds the packaged crate; it does not publish it.

## Formatting and evidence

Install the tracked pre-commit formatter once per clone with `make hooks-install`.
It refuses unstaged Rust source/configuration, formats staged source and requires
review/restaging if formatting changes files. It never stages automatically.
The installer preserves an existing different hooks path. Agents still leave
source uncommitted for the maintainer.

Release-helper regressions use isolated Git/Cargo/Make substitutes, with no real
commits, tags, pushes or uploads. Hook regressions use temporary Git indexes and
rustfmt, with no commits. Failed fixtures remain under `target/` for inspection.
[Tooling provenance](tooling-provenance.json) records the inspected sibling source
bytes; the MIT notices remain in the root license.

Native tests qualify the extracted local mechanisms, including actual child
process death around publication and lock ownership. They perform no IC effects.
Snapshot and lifecycle behavior will
require PocketIC or a deliberately selected real local IC backend. The original
[Canic source baseline](source-baseline.json) remains retained planning evidence.
[Fresh source provenance](extraction-source.json) records this extraction input
and the public consumers; it is separate from test qualification.
