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
make install-tools
make tools-check
make deps
make install-testkit-server
make testkit-server-check
make check
make test
```

Explicit local setup uses the reviewed [host pins](../ci/tool-versions.env)
and [IC tool matrix](../ci/ic-tools.tsv), through the shared
[Make commands](../make/tools.mk). It downloads and verifies jq/yq, ripgrep with
PCRE2, cloc and the five IC executables, then activates each complete set under `.tools/`; previous
sets and failed candidates are retained. Make selects `.tools/host/bin`, `.tools/ic/bin` and the optional `.tools/rust/bin`.
Interactive shells can use:

```bash
export PATH="$PWD/.tools/host/bin:$PWD/.tools/ic/bin:$PWD/.tools/rust/bin:$PATH"
```

See [bootstrap prerequisites](local-setup.md#bootstrap-prerequisites) and
[IC setup](ic-tools.md). Cargo toolchains, cargo-sort, ShellCheck and
macOS GNU Make/flock remain separate product prerequisites. Published Testkit
0.25 supplies the test-only PocketIC client and managed server. The locked Testkit package also selects its CLI and server assets; run
`make install-testkit-server` explicitly before tests. Testkit authenticates the
server through `make testkit-server-check`, which prints its admitted absolute
path offline. Both simulator suites call that check before managed startup.
Original server digests, private logs and fixture state remain retained.
Offline admission precedes [real single-canister qualification](pocketic-qualification.md).
Direct Agent transport is implemented separately; runners remain unimplemented.
Installing tools alone establishes
no backend or application qualification.

The optional shared `make install-rust-tools` and `make rust-tools-check` prepare
and check Cargo-sort, derive-sort and Candid-extractor under `.tools/rust/`.
This library requires only the existing Cargo-sort formatter; the extra set is
not part of aggregate setup or CI. Both commands use the reviewed shared pins.
The pre-commit hook and formatting-adoption checker find prepared checkout-local
tools without an interactive PATH export. Their source/configuration inputs remain
the isolated staged export; missing or wrongly pinned tools still reject.

The shared installer also supports an explicitly selected exact Cargo package,
binary or example, and debug/release profile. See [consumer-selected Cargo
tools](local-setup.md#consumer-selected-cargo-tools) for setup and
offline receipt checks. Backup uses this selected mode for its locked `ic-testkit-server` executable;
formatter prerequisites remain separate.

`make cloc` reports source/test LOC from this root workspace and excludes Cargo's
selected build outputs. Run fleet tooling/LOC reports from Shared Tooling; this
consumer no longer selects those reporters or their regression suite. The shared
`cloc-tooling` entry point explains that optional ownership when invoked here.
Local setup, offline checks and workspace LOC retain their prepared tools and
perform no implicit installation.

`make dependency-pins-check` checks declarations and tracked lockfiles without
downloads. The three qualified exact constraints and their required review are
recorded in [the local overlay](../AGENTS.md#qualified-dependency-constraints)
and the scoped exception file. CI explicitly runs setup before the full gate;
CI and releases include offline `tools-check` and the declaration check. Ordinary
validation never installs missing tools implicitly.

`make check-doc-links` checks supported local Markdown links in all tracked and
non-ignored new Markdown documents. The consumer selects that roster; the shared
Perl checker resolves local files/directories and ignores code examples and remote
URLs. It does not check anchors or network availability. CI and releases include
this offline check. See [the shared helper contract](verification-helpers.md).

`make deps` fetches the committed lockfile dependencies. The validation commands
use this repository's own `target/` directory. Check for an active build before
editing source or lockfiles or starting another compilation.

With the locked cache prepared, `make testkit-server-check` obtains the unique
Testkit package version from Cargo metadata and verifies the selected CLI receipt
through Shared Tooling before invoking Testkit's offline server check. No separate
server pin, cache scan, version equality check or shared-bundle fallback exists.
The normal gate runs it after `deps`; it installs nothing and qualifies no product
permissions. Missing/changed selections refuse before server startup.

Shared 0.2.0 requires explicit `make install-ic-tools` to select the five-tool
bundle. Old six-tool bundles, pins, receipts and failed candidates are retained.
Their offline check refuses under the new selection; no automatic conversion or
cleanup occurs. CI prepares both the shared bundle and Testkit on all three hosts.

`make shell-check` also requires ShellCheck and Perl. It prefers a prepared
`.tools/ci/bin/shellcheck`, then searches `PATH` and `~/.local/bin/shellcheck`.
It reports the selected command; lint failures never trigger a fallback or skip.
Perl must be on PATH. Linux CI selects reviewed ShellCheck 0.11.0 because Ubuntu
24.04's 0.9 checker rejects an intentional expression in the shared pin checker.
Prepare the same pinned Linux x86-64 binary explicitly:

```bash
source ci/tool-versions.env
bash scripts/ci/install-shellcheck.sh \
  --version "$SHARED_TOOLING_SHELLCHECK_VERSION" \
  --sha256 "$SHARED_TOOLING_SHELLCHECK_SHA256_LINUX_X86_64" \
  --install-dir "$PWD/.tools/ci/bin"
```

macOS uses Homebrew ShellCheck. Local installations on PATH or at the user-local
location still work when no checkout-local binary is prepared; validation never
installs a tool implicitly.

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
and shared lints. The members are `crates/ic-backup` and `crates/ic-backup-agent`. It provides the local
artifact and persistence mechanisms described in
[the implemented extraction boundary](extraction-boundary.md). Capture/restore
runners remain unimplemented; [Agent transport](agent-transport.md) owns bounded
async submission. Pure checksum records belong to
model; filesystem operations belong to ops.
`ops::artifacts::checksum_relative_files` also composes a directory checksum from
owned `(PathBuf, ArtifactChecksumRecord)` entries without filesystem IO. It accepts
exact canonical relative UTF-8 names, rejects duplicates and malformed identities
through `DirectoryChecksumError`, and retains the existing path ordering/framing.
Traversal, staging, synchronized publication and IC-tree verification use this same
owner. Consumer descriptor sync, byte custody and publication remain independent.
The focused public check is
`cargo test --offline --locked -p ic-backup --test directory_checksum`.
JSON serializers run once before parent/staging effects; their byte buffer retains
that contract. Host's typed descriptor publisher owns staging/atomic publication,
with private parent policy and acknowledged crash barriers in Backup. In the
0.6.0 release, match `PersistenceError::Publication` for its original typed producer,
cleanup and before/after-publication failures; visible output requires recovery.
Released 0.8.1 selects the Host 0.7.1 publication-error Rust identity. The
released 0.9.0 selected Host 0.8. Pending 0.11.0 selects incoming direct Host 0.9
and changes that exposed Rust identity again, preserving variant shapes and
persisted records. Testkit keeps its published dev-only Host 0.8 graph.
Consumers sharing `NamedWriteError` values or matching through a direct Host
dependency must select that same compatible line. Production direct Host
dependencies still disable default features. Testkit enables artifact archive/Wasm
and Candid-extraction features in the dev graph, and owns dev-only Host process
startup/cleanup. No new production inspection, installation-limit or process
policy is added. The direct process dependency retired with the ICP probe. Local bounded test capture still
cleans only its direct child; process-group signalling
does not replace Backup's inherited lock-descriptor quiescence checks.
Unix artifact and lock operations use Rustix's `io::Error::from` conversion
directly. Native codes and existing typed error projections stay intact; no local
errno wrapper or retry policy is layered over that upstream implementation.
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
freshness, custody and per-call accounting remain integration-owned. The separate
metadata/data codecs grant no read permission; complete transfer remains pending.
See [the typed read contract](contracts/snapshot-read-port.json)
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

`model::ic_snapshot_metadata` encodes an exact metadata-read request for a canonical
target and raw snapshot ID. Its bounded reply preserves required nat64 fields,
ordered globals and unavailable slots, exact floating bits, unique SHA-256 chunk
identities and optional source/timer/hook values. It shares the existing management
payload hash owner without changing the six-method lifecycle/recovery record.
No provider or transfer-completeness, permission or settlement API appears.
See [the metadata contract](contracts/ic-snapshot-metadata.json) and
[source review](ic-snapshot-metadata-source.json). Regenerate independent wire fixtures
and the contract with `perl scripts/dev/generate-snapshot-metadata.pl`; `--check`
verifies exact generated bytes. Focused checks are
`cargo test --offline --locked -p ic-backup --lib model::ic_snapshot_metadata` and
`cargo test --offline --locked -p ic-backup --test ic_snapshot_metadata`.

`model::ic_snapshot_data` borrows the retained metadata reply and encodes a checked
Wasm/heap/stable range or exact known chunk-store hash. It decodes bounded chunks
with exact range lengths or matching actual SHA-256 bytes, including valid known
empty chunks. Request hashing reuses the management payload owner; reply evidence
also binds exact original metadata. It adds no aggregate coverage, provider,
spending owner, receipt or transfer-completeness claim. See
[the generated data contract](contracts/ic-snapshot-data.json) and
[source review](ic-snapshot-data-source.json). Regenerate the metadata fixtures
first, then use `perl scripts/dev/generate-snapshot-data.pl`; both generators accept
`--check`. Focused checks are
`cargo test --offline --locked -p ic-backup --lib model::ic_snapshot_data` and
`cargo test --offline --locked -p ic-backup --test ic_snapshot_data`.

`model::ic_snapshot_coverage::IcSnapshotDataCoverage` admits already decoded replies
incrementally under exact metadata request/raw evidence. Each region advances from
zero without gaps or overlaps; chunks admit once in any order. Three nat64 cursors
and at most 1,024 presence bits retain no data buffers. `complete()` borrows a
read-only view only after every declared size and chunk is covered. Dropping this
ephemeral value loses its coverage; reconstructing it starts empty and never resumes
or resets spending. It has no serialization, durable-transfer attestation, provider
or release permit. The data contract also describes these checks. Focused coverage
checks are `cargo test --offline --locked -p ic-backup --lib model::ic_snapshot_coverage`
and the existing public `ic_snapshot_data` test target above.

`DownloadJournalGuard::stage_ic_snapshot_artifact` creates the distinct private
v1 metadata/region/chunk tree for an original Created entry. Its consuming writer
streams already admitted replies and checks full coverage against actual retained
bytes. Explicit `finish` persists the exact checksum through existing model/journal
owners, then synchronizes and publishes through the canonical artifact publisher.
Errors/drop retain partial bytes; occupied paths reject recreation. Reopen and
`finalize_artifact` recover ChecksumVerified publication without repeating reads.
Generic token/raw-ID mapping, authentic transfer, spending/permission and stable
external custody remain integration-owned. Explicit `verify_ic_snapshot_artifact`
checks one published tree against the full retained original plan, unchanged journal,
original metadata/request, region lengths and bounded chunk hashes. Complete Durable
selection is required, but only the requested target's bytes are verified. No evidence
is rewritten, ordinary resume reads no artifact bytes, and the returned checksum
holds no upload permission or fresh byte custody. See
[the tree contract](contracts/ic-snapshot-artifact.json) and
[source review](ic-snapshot-artifact-source.json). Focused checks are
`cargo test --offline --locked -p ic-backup --lib ops::persistence::download_journal`,
`cargo test --offline --locked -p ic-backup --lib artifact_commit` and
`cargo test --offline --locked -p ic-backup --test ic_snapshot_artifact`.
Unix FIFO fixtures use the host's `mkfifo -m 600` utility and verify the created
file type and permissions. This works with the standard Linux/macOS utility;
Rustix's descriptor-relative FIFO creation API is unavailable on Apple targets.

Public integration journeys reserve their fixture directories through
`crates/ic-backup/tests/support/mod.rs` before creating children. Atomic directory
creation and a process-local sequence replace clock-derived ownership; occupied
files, directories and links are skipped without replacement, with a 128-candidate
ceiling. Each allocated path is printed for failure diagnosis. The helper performs
no automatic cleanup: journeys retain failed evidence and explicitly remove only
their successful owned fixtures. Focused allocation/recovery checks are
`cargo test --offline --locked -p ic-backup --all-features --test fixture_roots --test fence_reconciliation`.

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
minimum supported version is 1.88.0. The selected Host and Agent dependencies
declare that floor; both libraries compile on the actual compiler. Install it
separately for `make check-msrv`, which records Cargo/rustc versions, compiles
all targets/features and checks each public library in an independent consumer
without Testkit feature unification. Those fixtures seed the unchanged workspace
lock, prepare only their own standalone locks offline and refuse any different
selected package/version/source. No dependency downgrade or ignored Rust floor
is used. The retained isolated candidate passed before the support claim changed.
Current native macOS qualification remains separate. Native builds are the supported product lane; the toolkit
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

`DownloadJournalGuard::publish_download_manifest` reuses that guarded fresh byte
verification and immutably publishes the exact existing `DownloadJournalRecord` at
`download-manifest.json`, under its original plan. The v1 schema/1 MiB/1,024-entry
bounds are unchanged; its model-owned binary digest binds exact snapshot metadata,
paths, state and checksums. `read_download_manifest` validates the expected digest,
original plan and unchanged retained download journal under exclusion. Drop active
download guards before replay; replay reads records only, even with absent artifact
trees. Complete backend transfer/consistency and full terminal proof remain separate.
See [the contract and independent goldens](contracts/download-manifest.json) and
[source inspection](download-manifest-source.json). Focused checks are
`cargo test --offline --locked -p ic-backup --lib download_manifest` and
`cargo test --offline --locked -p ic-backup --test execution_settlement`.

`policy::local_restore_source::validate` joins the original restore/source plans,
requirement and exact local manifest without IO. Its selected artifacts borrow the
complete original durable source; no IDs are rebound. Opt into this local source
binding by retaining the existing download-manifest digest as `source_artifacts`;
generic integration artifact digests keep their existing meaning.
`DownloadJournalGuard::read_download_manifest` reads exact retained evidence while
borrowing its held original journal, without reacquiring that journal's lock.
`verify_local_restore_source` additionally admits the retained requirement and both
plans before/after fresh no-follow checks of every source tree, including artifacts
outside a restore subset. The view borrows both layout lifetimes. Record-only replay
never invokes this explicit verification. No references, spending or obligations change;
stable noncooperating bytes and actual capture/transfer/application safety remain
integration-owned. See [the contract](contracts/local-restore-source.json) and
[fresh inspection](local-restore-source.json). Focused checks are
`cargo test --offline --locked -p ic-backup --lib local_restore_source` and
`cargo test --offline --locked -p ic-backup --test local_restore_source`.

`DownloadJournalGuard::stage_local_restore_artifact` binds a private copy to an
original operation and checksum after complete fresh source verification. Existing
destinations reject, and failures/drop preserve unfinished bytes. Explicit
`verify_staged_local_restore_artifact` re-admits retained originals and hashes the
copy without re-reading source trees or repeating a copy. Both views borrow the
original guards; no records, references, spending or fences change. Staging is
not durable publication or upload/load authority. Destination custody and actual
backend/application qualification remain integration-owned. See
[the contract](contracts/local-restore-artifact.json) and
[fresh inspection](local-restore-artifact-source.json). Focused checks are
`cargo test --offline --locked -p ic-backup --lib ops::persistence::download_journal`,
`cargo test --offline --locked -p ic-backup --lib ops::artifacts` and
`cargo test --offline --locked -p ic-backup --test local_restore_source`.

`publish_staged_local_restore_artifact` is the separate explicit durable operation.
It checks retained original metadata and the exact selected checksum, synchronizes
the staged tree, publishes without replacement to `restore-artifact-{sequence}` and
re-admits the originals/canonical bytes before returning the view plus Published or
Recovered outcome. Recovery synchronizes an existing matching canonical tree without
source-tree reads or recopying. It shares staging's operation lock; both/neither paths,
changed bytes, unsafe entries, replaced custody and original drift reject while
retaining evidence. `verify_published_local_restore_artifact` freshly checks canonical
bytes without synchronization or a publication attestation. No journals, attempts,
fences or references change; actual backend/application/dispatch qualification remains
separate. See [fresh publication inspection](local-restore-artifact-publication-source.json).
Focused checks are `cargo test --offline --locked -p ic-backup --lib local_restore_artifact`,
`cargo test --offline --locked -p ic-backup --lib artifact_commit` and
`cargo test --offline --locked -p ic-backup --test local_restore_source`.

`model::restore_safety` retains an immutable original restore/source/artifact/safety
requirement. Persistence requires both original plans under unchanged layout guards
and never replaces the 1 KiB declaration. Ephemeral exact load/start requests and
canonical actual evidence feed pure `policy::restore_safety`; the provider port has
no installed implementation. Applications qualify source/upload completeness,
fresh lifecycle, irreversible-work absence or rewind-independent retained fence/
replay safety, and restored-state acceptance. Views grant no effects or release.
See [the requirement schema](contracts/restore-safety-requirement.schema.json),
[typed port contract](contracts/restore-safety-port.json) and
[source provenance](restore-safety-source.json). Focused checks are
`cargo test --offline --locked -p ic-backup --lib restore_safety` and
`cargo test --offline --locked -p ic-backup --test restore_safety`.

`model::fence_obligation` retains exact original capture/restore requirements,
fence revisions and the explicit application acquisition operation. Fixed
`fence-obligation.json` publication is immutable and bounded to 1 KiB; existing
plan/requirement owners and both restore/source layout guards admit its originals.
`policy::fence_obligation::acquisition_progress` returns the existing exact attempt
journal view. It creates no spending owner, Active/released flag or dispatch permit.
Actual request semantics, acquisition and authenticated reconciliation remain
integration-owned; controlled release requires terminal evidence before an API
can be added. See [the schema](contracts/fence-obligation.schema.json),
[source inspection](fence-obligation-source.json) and
[maintained boundary](extraction-boundary.md#original-application-fence-obligation-retention).
Focused checks are `cargo test --offline --locked -p ic-backup --lib fence_obligation`
and `cargo test --offline --locked -p ic-backup --test fence_obligation`.

`model::fence_reconciliation`, its read-only provider port and pure policy bind an
existing pending acquisition to the exact reserved observation. The journal's
pending-request accessor reads its existing projection without changing v1 records
or accounting. Responses require exact original unit/context and attributed fence
evidence; a single descriptive remote observation never grants spending authority.
Provider failures retain pending spent attempts, obligations and source references;
retained late replies can be admitted without another provider call. See the
[typed contract](contracts/fence-reconciliation-port.json),
[fresh source inspection](fence-reconciliation-source.json) and
[maintained boundary](extraction-boundary.md#reserved-fence-acquisition-reconciliation).
Focused checks are `cargo test --offline --locked -p ic-backup --lib fence_reconciliation`
and `cargo test --offline --locked -p ic-backup --test fence_reconciliation`.

`model::fence_acquisition` admits exact original application update envelopes and
checks their target/digest against the existing reserved acquisition journal.
Opaque arguments are bounded to 1 MiB before copying; methods retain 1..128 visible
ASCII bytes. Application codecs retain/qualify original whole-unit semantics and
actual dispatch authority. The provider contract permits one prior-accounted update
without hidden retries or observations. Pure acknowledgement association writes no
receipt or outcome; errors/acknowledgements leave the original mutation pending.
See the [typed contract and independent goldens](contracts/fence-acquisition-port.json),
[fresh inspection](fence-acquisition-source.json) and
[maintained boundary](extraction-boundary.md#exact-application-fence-acquisition-requests).
Focused checks are `cargo test --offline --locked -p ic-backup --lib fence_acquisition`
and `cargo test --offline --locked -p ic-backup --test fence_acquisition`.

`model::ic_mutation::IcMutationRequest` binds the exact original plan operation,
immutable allowances and existing closed IC payload to an already pending mutation.
The provider contract describes one previously accounted host update. Pure
`policy::ic_mutation::validate_acknowledgement` checks current reservation and actual
claimed context/target/authority/attempt, then uses the existing bounded snapshot or
lifecycle decoder. Raw reply ownership is capped at 1 MiB and Debug is redacted;
matching preserves all pending spending and establishes no receipt or fresh permit.
Lost replies need original reconciliation, and pending recovery observations cannot
be bypassed. Fresh authentication, permissions/prerequisites, capture consistency,
same-release load/start source/upload/application safety and exclusive custody remain
integration-owned. No transport/provider implementation exists. See
[the typed contract](contracts/ic-mutation-port.json),
[fresh source inspection](ic-mutation-source.json) and
[maintained boundary](extraction-boundary.md#exact-originally-reserved-ic-mutation-updates).
Focused checks are `cargo test --offline --locked -p ic-backup --lib ic_mutation`
and `cargo test --offline --locked -p ic-backup --test ic_mutation`.

`model::ic_observation::IcObservationRequest` binds both pending original attempts,
original mutation bytes and exact already reserved status/list payload to the full
plan and immutable authority. The provider contract permits one previously accounted
host replicated observation, with no hidden retries or follow-up calls. Pure
`policy::ic_observation::validate_response` rechecks current reservation, exact payload,
both attempt IDs and actual claimed context/target, then reuses existing status or
snapshot inventory decoders. Raw replies are capped at 1 MiB with redacted Debug.
Matching writes no receipt; lost replies remain pending rather than settled Uncertain.
Status/controller projections and inventory cardinality establish no automatic effect
outcome. Authentication, method-specific current read permission, chronology and
exclusive attribution/dispatch custody remain integration-owned. No implementation
is installed. See [the contract](contracts/ic-observation-port.json),
[source inspection](ic-observation-source.json) and
[boundary](extraction-boundary.md#exact-originally-reserved-ic-recovery-observations).
Focused checks are `cargo test --offline --locked -p ic-backup --lib ic_observation`
and `cargo test --offline --locked -p ic-backup --test ic_observation`.

`model::execution_settlement` retains bounded strict v1 original plan/journal
fingerprints, with independent binary goldens for full chronological history.
Pure admission reuses `execution_progress` and requires the exact original journal
set, every operation Applied and unchanged receipt/reservation histories. Fixed
`execution-settlement.json` has immutable 2 MiB publication/read under retained
original plans and per-journal guards. Drop active journal guards before invoking
these operations. Bulk canonical authority derivation hashes the full plan once;
sequential journal locks bound descriptor use. Replay performs local IO only and
supplies no terminal, fresh verification, command-quiescence or release authority.
See the [schema and goldens](contracts/execution-settlement.schema.json),
[fresh inspection](execution-settlement-source.json) and
[maintained boundary](extraction-boundary.md#original-execution-settlement-checkpoints).
Focused checks are `cargo test --offline --locked -p ic-backup --lib execution_settlement`
and `cargo test --offline --locked -p ic-backup --test execution_settlement`.

## Snapshot upload payloads

`model::ic_snapshot_upload` owns same-target metadata/data arguments and bounded
passive replies. Metadata preserves every available global and absent timer/hook
values; an unavailable global rejects. No replacement is encoded. Data binds exact
original extents or known chunk hashes, actual bytes and a distinct new raw ID.
Keep `binding_digest()` in the new original operation plan; `digest()` hashes wire
bytes only. Each later data intent includes the allocated ID before its own
reservation. A metadata reservation cannot supply data spending.

`DownloadJournalGuard::prepare_ic_snapshot_upload_metadata` freshly verifies the
original published IC tree. `prepare_ic_snapshot_upload_data` verifies before/after
one bounded no-follow source read. These operations change no records or references
and hold no future byte custody. `IcSnapshotUploadAttempt` and pure
`policy::ic_snapshot_upload::validate_acknowledgement` recheck current original
pending evidence, without receipt, retry, remote call or complete-upload proof.
Lost replies remain pending, including lost recovery observations.

Focused checks are `cargo test --offline --locked -p ic-backup --lib upload` and
`cargo test --offline --locked -p ic-backup --test ic_snapshot_upload`. Regenerate
metadata fixtures first, then `perl scripts/dev/generate-snapshot-upload.pl`;
`--check` independently compares all registered wire/hash fixtures and the
[machine contract](contracts/ic-snapshot-upload.json). Native fixtures qualify local
bytes and declarations only; real IC upload and lost-effect reconciliation remain
unimplemented. See [the boundary](extraction-boundary.md#original-source-bound-ic-snapshot-upload).

Exact data-upload recovery and settlement use
`policy::ic_snapshot_upload_data_observation::validate_response` and
`validate_settlement`. Retain original destination metadata/read bytes before
reservation. The latter also matches independently qualified attribution, a fresh
challenge and exact readback/provenance evidence. It never records a receipt or
performs another call. Matching bytes alone do not supply attribution; lost replies
stay pending. The public local upload case also exercises an explicit synthetic
settled-uncertainty receipt through the existing journal owner and exhausted reopen.
This qualifies persistence only. See
[the contract](contracts/ic-snapshot-upload-data-settlement.json).

## Local IC artifact diagnostics

Local IC artifact diagnostics are available through
`DownloadJournalGuard::ic_snapshot_metrics()`. Its duration summaries use nanoseconds;
`prepared_chunk_bytes()` summarizes successfully returned local data payload sizes.
`prepared_chunk_bytes_histogram()` returns the canonical shared
`MeasurementHistogram<4>`, re-exported through the same persistence facade. Its
inclusive byte bounds are zero, 32 KiB, 256 KiB and 1 MiB, with separate overflow.
Counts are disjoint and saturate independently. The histogram owns the summary;
there is no separately updated byte aggregate. Empty chunks and repeated work
remain samples, while rejected calls contribute no bytes. Six duration summaries
retain their existing owners and inclusive timing.
Counts, totals, latest and maximum reuse `ic-metrics::MeasurementSummary`, exposed
as `ic_backup::ops::persistence::MeasurementSummary`. This re-exports the shared type;
no local summary model or aggregation is maintained. Callers can name returned
summaries through this path without adding a direct Metrics dependency. Successful
and rejected calls stay separate. Empty chunks are valid zero samples; repeated
preparation is repeated work. Preparation timings include nested verification and
must not be added to verification timings as exclusive work. Measurements are empty
on reopen and never persist into journals or grant progress/spending authority.
Four bounds add 72 bytes of fixed storage per guard and at most four bound
comparisons per admitted chunk. No allocation, clock or reporting task is added.
This describes storage/work, not a measured speed-up or IC cycle cost.

The workspace catalog declares registry `ic-metrics` with default features disabled;
the Unix member inherits it without feature `ic`. It installs no host IC instruction
counter, network telemetry or transport measurement. Focused sampling tests are
`cargo test --offline --locked -p ic-backup --lib metrics`, followed by the existing
download-journal and public upload journeys when those operations change. See
[diagnostic semantics](extraction-boundary.md#local-ic-snapshot-diagnostics) and
[dependency review](ic-metrics-adoption.json).

## Supported host scope

IC Backup runs on the operator host. The [vendored host matrix](supported-hosts.md)
describes Shared Tooling's own scripts, CI and installers; its macOS and installer
entries do not establish IC Backup qualification.

| Scope | Current position |
| --- | --- |
| Linux x86-64 | Ubuntu 24.04 CI; retained local native evidence is described in the handoff |
| macOS 15 Apple Silicon | Required native library and tooling support; `macos-15` CI configured, native qualification pending |
| macOS 15 Intel | Required native library and tooling support; `macos-15-intel` CI configured, native qualification pending |
| Repository scripts | Bash 3.2 or newer with the dependencies listed below |
| Windows and non-Bash shells | No supported product lane |
| Complete backup/restore | Unimplemented; no host has end-to-end product qualification |

The pinned Rust 1.99.0 development toolchain, Rust 1.88.0 minimum compiler,
rustfmt and Clippy are required on
each native host. Repository tooling requires Git, GNU Make, Bash, Perl with core
JSON::PP and Digest::SHA, ripgrep, flock, ShellCheck, cargo-sort 2.1.4 and either `sha256sum` or
`shasum`. Native process fixtures also require Python 3. The standard Unix utilities
listed below may use their GNU or BSD implementations.

On Ubuntu, install missing tooling with `sudo apt-get install make git perl
python3 ripgrep util-linux`, then prepare ShellCheck with the pinned command above.
On macOS 15, install the Xcode command-line
tools and Homebrew, then run `brew install flock make ripgrep shellcheck`; provide
Python 3 if it is absent. Add Homebrew GNU Make to the current shell with
`export PATH="$(brew --prefix make)/libexec/gnubin:$PATH"`. Install Rust through
rustup, then run `rustup show active-toolchain` and
`rustup toolchain install 1.88.0 --profile minimal` in this checkout. Install the
reviewed manifest formatter after `source ci/tool-versions.env` with
`cargo install cargo-sort --version "$SHARED_TOOLING_CARGO_SORT_VERSION" --locked`,
then run `make install-hooks` once per clone or after updating the hook contract.
Setup is explicit; hooks and validation never install tools. CI installs
the host-specific tooling and uses the same native validation gate on all three
hosts, with additional system Bash 3.2 tooling checks on macOS. These setup paths
and configured jobs are not evidence that the pending native macOS runs passed.

## Commands and build ownership

`make help` lists the command family. `check`, `clippy`, `test`, `doc`,
`check-msrv` and `package` select both libraries explicitly. `fmt` formats the
workspace after `cargo sort --workspace`; `fmt-check` independently checks both
manifest ordering and Rust formatting with the same pinned cargo-sort version.
`shell-check`, `release-check` and `hooks-check`
validate contributor tooling. `shared-tooling-check` verifies the exact local
snapshot; `tooling-check` tests its integrity and CI diagnostics without network
or compilation. [Shared adoption](shared-tooling.md) identifies the reviewed
source and the product overlay. `version` and `release-plan` inspect release
metadata without changing the workspace. `make tasks` reads the adopted shared
maintenance catalog. Its runner is qualified only with a substitute CLI; no live
agent or timer is activated by snapshot adoption.

Release tooling uses the exact vendored common runner.
Patch/minor/major share one workflow and explicit branch/remote inputs. Normal
targets reconcile exact unfinished intent before selecting another increment;
newer committed fixes or a different requested kind then require fresh preflight
and complete validation. `release-resume VERSION=X.Y.Z` selects only the saved
release. Late receipt/validation checks use `RELEASE_COMMIT`, which may precede
HEAD, without rebuilding old proof from newer source. Old standalone
preparation/stage/commit/push commands are removed. Consumer adapters own original
validation evidence and bounded package metadata, and preserve dependency selection,
build artifacts and source backups. `release-check` runs actual Make entry points
with Git/Cargo substitutes plus the common runner's isolated regressions. Read
[the release guide](releasing.md) before an explicitly requested one-shot release
or resume. Ordinary development and PR delivery grant no release authority.
`make -n` remains read-only.

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
License metadata also inherits from the workspace. The member's regular `LICENSE`
copies the maintained root MIT notice exactly, preserving contributor attribution
in the standalone crate archive. Update both copies together; a tracked symlink
would violate the shared index-snapshot formatting contract.

Run checks targeted to changed packages and behavior while developing. Full
validation requires a maintainer request or CI. `make ci`, `make validate` and
`make release-verify` run the full configured gate: snapshot verification,
offline local-tool and pin checks, dependency fetch, tooling, formatting,
native compilation, Clippy, tests, docs,
MSRV and package verification.
CI runs this gate on the Linux and macOS hosts above. The shared runner preserves target order, stops at
the first failure and prints target-labelled diagnostics and a result/timing
summary. Full and highlighted failure logs remain under
`target/validation-failures/`; GitHub Actions also gets a step summary.
The runner requires GNU Make, `awk`, `sed`, `tail`, `tee` and ripgrep or grep.
Snapshot verification requires `sha256sum` or `shasum`. Shared Tooling is never
fetched at validation time. Local package verification permits reviewed dirty
source and builds the packaged crate; it does not publish it.

## Formatting and evidence

Install the exact shared pre-commit formatter with `make install-hooks`. It formats
an isolated export of the selected index, copies back and automatically refreshes
only selected regular files. Partial staging rejects before formatting; unrelated
tracked/untracked edits remain untouched. A failed formatter leaves the working
files and index unchanged. Review concurrent copy/staging failures before retrying.
The installer preserves different, inherited or disabled hook paths and executable
private hooks. `hooks-install` is retired; update setup automation to the standard
command. Ordinary development leaves source uncommitted; explicit commit or PR
delivery follows the [contribution rules](../rules/contributions.md).

Release-helper regressions use isolated Git/Cargo/Make substitutes, with no real
commits, tags, pushes or uploads. Hook regressions use temporary Git indexes and
the actual Make formatting targets, cargo-sort and rustfmt, with no commits.
All consumer fixtures and registered case logs remain under `target/` for inspection.
[Tooling provenance](tooling-provenance.json) records the inspected sibling source
bytes; the MIT notices remain in the root license.

Native tests qualify the extracted local mechanisms, including actual child
process death around publication and lock ownership. They perform no IC effects.
Snapshot and lifecycle behavior will
require PocketIC or a deliberately selected real local IC backend. The original
[Canic source baseline](source-baseline.json) remains retained planning evidence.
[Fresh source provenance](extraction-source.json) records this extraction input
and the public consumers; it is separate from test qualification.

## Metadata-allocation settlement admission

The existing metadata-observation module admits independently qualified allocation
claims against exact original/current inventory evidence and still-pending attempts.
Reuse canonical inventory delta and source/destination identity admission; never
infer outcomes from candidate counts. See [the maintained boundary](extraction-boundary.md#exact-metadata-upload-settlement-claims).
Focused tests select `upload`, `snapshot_inventory_delta` and `ic_observation` with
`cargo test --offline --locked -p ic-backup --lib FILTER`, plus the existing public
`ic_snapshot_upload` target. Native synthetic settled claims qualify only local
admission/accounting/reopen; actual authenticated providers remain integration-owned.

## The 0.4 hard cut

The public local-metrics getters now return the shared Metrics 0.2 type. Import
`ic_backup::ops::persistence::MeasurementSummary` to name those results through this
library. An application that also uses Metrics directly must update its own
`ic-metrics` dependency to the 0.2 line; arithmetic, units and empty/saturated sample
semantics are unchanged. There is no Metrics 0.1 adapter, local arithmetic wrapper
or dual API. Host Artifacts and Host FS 0.3 remain private to the existing adapters.

The maintained v1 records, digests, retained source artifacts, spending reservations,
fence obligations and restore references retain their exact formats and owners.
Do not reset or discard unfinished evidence as part of this dependency hard cut.
Package version changes, release execution and publication remain maintainer-owned.

## The 0.5 stream hashing contract

`ops::artifacts::checksum_reader` delegates raw hashing to `ic-host-artifacts` and
retries `Interrupted` reads internally. Callers must own blocking/timeouts instead
of treating the first interruption as termination. Other reader IO errors retain
their kind and source; impossible byte counts reject as `InvalidData` without a
checksum. This observable change selects a 0.5.0 draft from released 0.4.2; package
versions remain maintainer-owned. The earlier fixture allocation fix shares that
draft.

Raw checksum construction and digest formatting use the shared `Sha256Digest`,
while Backup owns its normalized v1 record and permissive equivalent hex casing.
ASCII case normalization precedes shared digest parsing; malformed-hash errors
retain the original input. Layout lock keys use the same canonical raw digest
formatter. Raw digest-text fields in plans, attempt identity/history, inventories,
requirements, downloads and references call the one internal normalizer directly;
they do not construct checksum records merely to extract a normalized string.
Independent persistence reopen admission uses that same owner and keeps its
original error conversion/order. Layout lock keys retain their exact prior names.
IC artifact verification supplies
the opened file's admitted length to shared bounded hashing, checks the returned
byte count and rejects short/excess input as `FileShape`. Original IO errors and
independent descriptor/path identity checks remain intact. These consolidate
existing contracts without another public checksum API or retained-data reset.
The generic hex helper remains for IC wire byte fields. Bounded copies/collection
and output counting use Host Artifacts; no-follow record reads use Host FS. Unused
archive/compression/Wasm features are disabled and their lockfile entries removed.
Directory digest framing, 0700/0600 staging, synchronization/crash barriers,
no-replace multi-file publication, journals and spending stay local. Existing
records/artifacts need no reset. See [the adoption review](ic-host-tools-adoption.json).
The published 0.3.0 archives' recorded source commit is available locally but was
not retrievable from their declared GitHub repository at inspection; the upstream
[provenance report](https://github.com/dragginzgame/ic-host-tooling/issues/7)
records that separate limitation. Cached archive/source verification does not
establish upstream CI or native qualification.
Focused unit filters are `ops::artifacts`, `model::artifacts` and
`ops::persistence::json`; artifact commit and IC source preparation retain their
existing checks above. Native filesystem qualification is distinct from IC effects.

## Workspace inheritance and version inspection

`make dependency-pins-check` enforces the shared Cargo inheritance contract as well
as existing immutable dependency/action selections. Member package versions and
ordinary/dev/build/target dependencies inherit the owning workspace catalog. Root
packages use that catalog for their own dependency edges too; independently declared
workspace discovery does not authorize a new graph or sibling path.

`make version` reads the current workspace version through prepared Cargo, jq and
yq, offline. Follow [local setup](local-setup.md) first; this command installs nothing
and neither resolves dependencies nor compiles. The release adapter owns selected
Git metadata, numeric release bounds and all record/recovery transformations.
`make tooling-check` includes shared Cargo metadata regressions; `make release-check`
retains focused consumer source/version/recovery checks using isolated fixtures.

CI creates `target/ci-fixtures` before setting `TMPDIR`, then uploads that directory,
retained test directories and validation failure logs on job failure. Hidden fixture
files include private Git index evidence. These test artifacts have seven-day
retention; they do not replace durable operator journals or source references.
Linux and native macOS evidence remain distinct. See
[the current adoption review](shared-tooling-review.json).

Set `VALIDATION_LOG_DIR` to retain complete successful and failed target logs
and a timing table, including nested validation. Each invocation gets its own
directory; the logger never replaces a prior run. For a focused formatting check:

```bash
VALIDATION_LOG_DIR="$PWD/target/validation-runs" bash scripts/ci/run-validation-targets.sh fmt-check
```

Gate arguments must be named Make targets; options and assignments reject before
logging or dispatch. Export ordinary settings through the environment. The logger
preserves Make failure status. This evidence remains distinct from source-bound
release validation receipts.

Shared dependency, validation-runner and release-runner regressions retain their
original fixture inputs on unexpected failure. Successful temporary fixtures are
removed by their owning tests. The consumer tooling check injects failures through
these actual adopted helpers and checks exact status, retained inputs and reported
paths; it runs in the existing Linux/macOS CI selection without adding tool setup.

`fmt` and `fmt-check` share `format-tools-check`, which reads the reviewed pin
from `ci/tool-versions.env`. It requires successful exact cargo-sort and rustfmt
availability probes, offline with rustup automatic installation disabled. It
installs nothing and changes no files. CI setup uses that same pin; workspace
sorting, actual Rust formatting and hook/index custody remain consumer-owned.
The helper's rejection fixtures run in normal tooling checks and explicitly
under macOS system Bash 3.2. Native qualification belongs to the configured
consumer jobs, independently from a Linux pass or upstream source adoption.
