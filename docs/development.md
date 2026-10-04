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
Local download journal identities/transitions and derived resume views belong to
model; its guard borrows layout exclusion and owns bounded durable updates,
byte verification and publication reconciliation. Remote download completeness,
execution journals and runner authority remain integration/contract work.
Local attempt journal identity, immutable budgets and chronological replay belong
to model. Its persistence guard reserves before returning and stops after an
indeterminate write. Qualified receipts, fresh authority, backend call/retry bounds
and remote reconciliation remain caller-owned; the ledger performs no dispatch.
See [the v1 attempt schema](contracts/attempt-journal.schema.json) and
[source provenance](attempt-journal-source.json). Native fixtures kill acknowledged
owners before/after reservation persistence and after the returned reservation;
they qualify local accounting, not IC paid effects.
Canonical inventory identities, bounded forest validation and binary hashing belong
to model. Immutable inventory publication/admission belongs to persistence ops;
exact/direct-child/subtree decisions belong to pure `policy::selection`. Policy
performs no IO, serializes no records and changes no persisted state. It returns
read-only references bound to the full inventory digest. Parent links can point
outside a selected subset while remaining inside the original declared forest.
See [the v1 inventory schema](contracts/inventory.schema.json) and
[inventory provenance](inventory-source.json). Current declaration tests qualify
graph/hash/selection and native local persistence, not authoritative discovery.
Explicit operation dependencies, cycle checks, deterministic topological order and
canonical graph hashing belong to model. Immutable graph persistence belongs to
ops. Pure `policy::effect_order` admits causal declared completion identities and
projects ready/blocked nodes without reading receipts, scheduling or executing.
Qualified receipts and application ordering remain integration-owned. See
[the graph schema](contracts/effect-graph.schema.json) and
[ordering provenance](effect-graph-source.json).
Operation-plan context, exact graph/target/request binding, aggregate assigned
ceilings, canonical hashing and derived original attempt authority belong to model.
Ops owns immutable `operation-plan.json` admission under the exact reviewed digest.
Native public tests derive authority and reopen an already-consumed attempt through
the original plan, without remote calls or allowance replenishment. See
[the plan schema](contracts/operation-plan.schema.json) and
[binding provenance](operation-plan-source.json). Complete backup/restore plan
semantics, typed request codecs, preflight and runner wiring remain unimplemented.
Pure `policy::execution_progress` joins the original plan with one retained
`AttemptJournalRecord` per operation. It validates complete exact authority/limit
binding and causal Applied prerequisites for attempted operations, then derives
graph-ordered local conditions and totals over assigned allowances only. It never
reads files, creates missing journals, schedules work, serializes records or changes
state. The caller owns coherent retained custody and qualified actual receipts;
the projection proves neither freshness nor cross-journal dispatch chronology.
See [progress provenance](execution-progress-source.json) and
[the maintained boundary](extraction-boundary.md).

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
thiserror, pinned `ic_principal` conversion and Unix rustix/command-fds
declarations belong in `[workspace.dependencies]`,
with package-level entries inheriting them. There are no Canic dependencies or
sibling checkout patches.
License metadata also inherits from the workspace. The member's `LICENSE` link
points at the maintained root MIT notice; Cargo includes its exact regular-file
content and contributor attribution in the standalone crate archive.

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
