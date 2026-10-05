<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-backup/ic-backup-readme-header.svg" alt="IC Backup — Verified backups and safe recovery for Internet Computer apps" width="100%">
</p>

<!-- helper-navigation:start -->
<p align="center">
  <a href="https://github.com/dragginzgame/canic"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/canic.svg" width="18" height="18" alt=""> <strong>canic</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/icydb"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/icydb.svg" width="18" height="18" alt=""> <strong>icydb</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-timers"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-timers.svg" width="18" height="18" alt=""> <strong>ic-timers</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-memory"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-memory.svg" width="18" height="18" alt=""> <strong>ic-memory</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-query"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-query.svg" width="18" height="18" alt=""> <strong>ic-query</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-backup"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-backup.svg" width="18" height="18" alt=""> <strong>ic-backup</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-blob-storage"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-blob-storage.svg" width="18" height="18" alt=""> <strong>ic-blob-storage</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-testkit"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-testkit.svg" width="18" height="18" alt=""> <strong>ic-testkit</strong></a>
</p>
<!-- helper-navigation:end -->


# Development

This guide is for contributors working on the Rust library and its repository
tooling. It describes the current local implementation; it is not an operator
guide for performing a canister backup or restore.

## Quick start

From the repository root:

```bash
make deps
make check
make test
```

`make deps` fetches the committed lockfile dependencies. The validation commands
use this repository's own `target/` directory. Check for an active build before
editing source or lockfiles or starting another compilation.

`make shell-check` also requires ShellCheck and Perl. It uses ShellCheck from
`PATH` first, then an executable `~/.local/bin/shellcheck` if PATH lookup fails.
It reports the selected command and fails if neither location provides the tool;
lint failures never trigger a fallback or skip. Perl must be on PATH. CI installs
ShellCheck explicitly; locally, install it with the host package manager or in
the user-local location. An existing user-local installation works with plain
`make shell-check` in non-login shells without changing the terminal's PATH.

## Architecture at a glance

| Area | Responsibility |
| --- | --- |
| Model | Owns records, identities, bounds and valid state transitions |
| Policy | Validates and derives decisions without filesystem or network access |
| Operations | Performs approved local persistence and filesystem work |
| Ports | Defines observations and capabilities supplied by an IC or application integration |

The library deliberately separates a recorded declaration from current authority
to act. A plan, journal or decoded reply does not by itself permit a network call,
prove application safety or establish that a remote operation succeeded.

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-backup/ic-backup-implementation-boundary.svg" alt="Implementation boundary showing locally tested models, policy and persistence, application-supplied integration contracts, and the planned IC transport, runners and CLI" width="800">
</p>

## Detailed component ownership

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
semantics, transfer/response codecs, fresh preflight and runner wiring remain
unimplemented.
Pure `policy::execution_progress` joins the original plan with one retained
`AttemptJournalRecord` per operation. It validates complete exact authority/limit
binding and causal Applied prerequisites for attempted operations, then derives
graph-ordered local conditions and totals over assigned allowances only. It never
reads files, creates missing journals, schedules work, serializes records or changes
state. The caller owns coherent retained custody and qualified actual receipts;
the projection proves neither freshness nor cross-journal dispatch chronology.
See [progress provenance](execution-progress-source.json) and
[the maintained boundary](extraction-boundary.md).

`model::ic_request` is the IC-specific host-ingress codec boundary. It owns closed
method/target/raw-snapshot declarations, pinned upstream Candid argument encoding,
exact wire digests and mutation/observation payload checks. Generic plan/journal
owners remain independent of the codec. SDK DTOs and Candid are registry dependencies,
not Canic runtime imports. Native tests compare independent wire vectors, decode
official argument types and reopen byte-bound local journals; they perform no IC
effects. Callers retaining these declarations use the existing bounded JSON ops
with the codec's 8 KiB bound under their own custody; no runner request layout or
dispatch persistence API is introduced. See
[the wire schema](contracts/ic-request.schema.json) and
[codec provenance](ic-request-source.json). Transfer/response codecs, authenticated
transport, fresh preflight and backend effects remain unimplemented.

`ports::membership::MembershipProvider` is a fallible integration contract, with no
installed provider. `model::membership` derives ephemeral immutable requests from
the original plan and operation, plus an integration-owned challenge, explicit
before/after boundary and 0–1,024 descriptive remote-call ceiling.
`policy::membership::validate` checks exact request/context/full inventory and
reported calls, then returns a read-only view. It performs no IO or record
serialization and changes no plan/journal. Requests/results/views have no Serde
admission or persisted fresh-authority flags. Freshness, challenge uniqueness,
actual permissions, prior per-call spending and revision/fence semantics remain
integration-owned. Before/after equality proves no uninterrupted membership.
The descriptive call ceiling is separate from finite journal allowances and
cannot admit or replenish paid probes. See
[the typed port contract](contracts/membership-port.json) and
[membership provenance](membership-source.json). Native public tests exercise
provider binding and local spent-journal recovery; they certify no live membership.

`model::control_authority` binds original intent/operation and exact IC mutation
bytes into ephemeral controller-observation requests. It owns canonical actual
targets and complete unique controller sets bounded to 10. The fallible
`ports::control_authority::ControlAuthorityProvider` has no installed provider.
Pure `policy::control_authority::validate` checks actual context/target/call count
and requires the original caller itself in current controllers. Status/read access
or a controlling Root does not substitute. No Serde/Proven flag, plan mutation or
dispatch permit is exposed. Actual authentication/freshness, prior per-call spending,
snapshot origin permissions and full restore/lifecycle/application safety remain
integration-owned. See [the typed control contract](contracts/control-authority-port.json)
and [control provenance](control-authority-source.json).

`model::snapshot_read` binds original mutation intent and independently declared
exact snapshot-list bytes to an ephemeral challenge. Its fallible
`ports::snapshot_read::SnapshotReadProvider` has no installed implementation.
Pure `policy::snapshot_read::validate` checks actual context/target/call reporting
and known controller, public or exact allowed-viewer access. Canonical viewers
are unique and bounded to 10; unknown controllers cannot establish controller access.
Public/viewer evidence grants no mutation control. No Serde/default authority,
fresh spending or lost-observation settlement is introduced. Provider authentication,
freshness, custody and per-call accounting remain integration-owned; metadata/data
codecs remain pending. See [the typed read contract](contracts/snapshot-read-port.json)
and [fresh read provenance](snapshot-read-source.json).

`model::consistency` now owns a strict v1 original-plan-bound guarantee declaration,
retained at fixed `consistency-requirement.json` under layout exclusion with 1 KiB
input/output bounds and no-replace publication. Exact expected requirement/plan
reads reject downgraded or rebound declarations. Ephemeral current requests bind
original operation, challenge, capture boundary and retained fence/original revision.
The fallible `ports::consistency::ConsistencyProvider` observes existing obligations;
no provider or acquisition/release operation is installed. Pure admission checks
actual full inventory/exact selected stopped/drained targets and original guarantee;
coordinated evidence requires exact Active fence and original membership revision.
Opaque evidence authenticity, current observations, continuous whole-unit fencing,
retained custody and prior per-call accounting remain integration-owned. No request/
result/view can be admitted from JSON as fresh authority. Native fixtures qualify
local declarations, persistence and spent-journal/obligation recovery only. See
[the requirement schema](contracts/consistency-requirement.schema.json),
[typed current port](contracts/consistency-port.json) and
[fresh consistency provenance](consistency-source.json).

The snapshot reply codec in `model::ic_snapshot_reply` decodes the existing capture
singleton and snapshot inventory methods using required upstream fields. It has
bounded input, sequences and decoder work, canonical unique raw IDs and exact
request/raw-reply evidence hashing. Neither result nor metadata grants settlement,
authority or transfer completion. [The reply contract](contracts/ic-snapshot-reply.json)
and [fresh provenance](ic-snapshot-reply-source.json) define the maintained boundary.
Focused checks are `cargo test --offline --locked -p ic-backup --lib model::ic_`
and `cargo test --offline --locked -p ic-backup --test ic_snapshot_reply --test ic_request`.

`policy::snapshot_inventory_delta::compare` borrows the existing capture declaration
and bounded inventory replies. It rejects method/target mismatch, lost baseline IDs
and changed baseline metadata, then exposes canonical new descriptors without IO,
serialization, journal mutation or capture attribution. The original request/reply
owners supply exact evidence; no new record or hash encoder is added. See
[the comparison contract](contracts/snapshot-inventory-delta.json) and
[fresh Canic provenance](snapshot-inventory-delta-source.json). Focused checks are
`cargo test --offline --locked -p ic-backup --lib policy::snapshot_inventory_delta`
and `cargo test --offline --locked -p ic-backup --test snapshot_inventory_delta`.
These native fixtures qualify local comparison and retention, not remote settlement.

`model::ic_lifecycle_reply` admits the existing stop/start/load canonical empty
tuple and required status/settings/controllers projection. Status uses the upstream
SDK enum and existing bounded `ControllerSet` owner. Unprojected fields are skipped
under finite work limits and remain unqualified; exact raw evidence hashes retain
them. Required fields, duplicate controllers, excess counts/work/types, extra
arguments and trailing bytes reject. No transport, receipt or fresh provider evidence
is installed. See [the reply contract](contracts/ic-lifecycle-reply.json) and
[fresh source provenance](ic-lifecycle-reply-source.json). Focused checks are
`cargo test --offline --locked -p ic-backup --lib model::ic_lifecycle_reply` and
`cargo test --offline --locked -p ic-backup --test ic_lifecycle_reply`.

Rust 1.99.0 is pinned in `rust-toolchain.toml`, with rustfmt and Clippy. The
minimum supported version is 1.91.0. Install that toolchain separately for
`make check-msrv`. Native builds are the supported product lane; the toolkit
runs on an operator host.

`policy::download_integrity::validate` checks retained plan/journal declarations
without IO. `DownloadJournalGuard::verify_durable_artifacts` separately re-admits
the persisted original plan and unchanged journal, then verifies every published
directory checksum. Neither result is a terminal proof or backend transfer permit;
integrations maintain stable byte custody during sequential checks. See
[the integrity contract](contracts/download-integrity.json) and
[fresh source provenance](download-integrity-source.json). Focused checks are
`cargo test --offline --locked -p ic-backup --lib download_integrity`,
`cargo test --offline --locked -p ic-backup --lib ops::persistence::download_journal::integrity`
and `cargo test --offline --locked -p ic-backup --test download_integrity`.

## Supported host scope

IC Backup runs on the operator host. The [vendored host matrix](supported-hosts.md)
describes Shared Tooling's own scripts, CI and installers; its macOS and installer
entries do not establish IC Backup qualification.

| Scope | Current position |
| --- | --- |
| Native library development | Pinned Rust toolchains and repository-local build directory; native evidence is retained in the handoff |
| Repository scripts | Bash 3.2 or newer with the dependencies listed below |
| IC Backup CI | Linux only; the configured workflow uses `ubuntu-latest` |
| Windows and non-Bash shells | No supported product lane |
| Complete backup/restore | Unimplemented; no host has end-to-end product qualification |

## Commands and build ownership

`make help` lists the command family. `check`, `clippy`, `test`, `doc`,
`check-msrv` and `package` select `ic-backup` explicitly. `fmt` formats the
workspace; `fmt-check` checks it. `shell-check`, `release-check` and `hooks-check`
validate contributor tooling. `shared-tooling-check` verifies the exact local
snapshot; `tooling-check` tests its integrity and CI diagnostics without network
or compilation. [Shared adoption](shared-tooling.md) identifies the reviewed
source and the product overlay. `version` and `release-plan` inspect release
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
`make release-verify` run the full configured gate: snapshot verification,
dependency fetch, tooling, formatting, native compilation, Clippy, tests, docs,
MSRV and package verification.
CI runs this gate on Linux. The shared runner preserves target order, stops at
the first failure and prints target-labelled diagnostics and a result/timing
summary. Full and highlighted failure logs remain under
`target/validation-failures/`; GitHub Actions also gets a step summary.
The runner requires GNU Make, `awk`, `sed`, `tail`, `tee` and ripgrep or grep.
Snapshot verification requires `sha256sum` or `shasum`. Shared Tooling is never
fetched at validation time. Local package verification permits reviewed dirty
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
