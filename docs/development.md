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
tooling. It covers setup, focused checks and contribution workflows. Product contracts
belong to the linked implementation guides; this is not an operator guide for
performing a canister backup or restore.

## CI source qualification

CI/release validation dispatches targets sequentially. `make ci` consumes the
selected dependency cache without fetching. Prepare it through `make install-tools`
or the explicit `make deps` target before validation. After prepared tools,
locked dependencies, source checks and formatting, the existing `make clippy`
gate runs before expensive product checks, tooling/release/hook suites and
packaging. A lint failure stops later stages under serial or parallel Make.
Publication and its dry run use the same lint scope before Cargo packaging/upload.

`make ci` declares the root Cargo.toml and Cargo.lock as validation inputs. The
shared runner checks them before and after each target, including same-checkout
nested gates. Changed, missing or unreadable inputs stop later dispatch and retain
original/observed identities. Pause dependency updates, prepare the selected graph
and restart the complete gate; earlier stages cannot qualify a changed graph.
This detects changes at checkpoints rather than isolating files during a target.

Automatic CI runs for main pushes and pull requests. An atomic main/tag release
selects one complete native matrix for that source; the tag does not repeat it.
Distinct main commits retain their own runs. Every selected run keeps Linux,
Intel macOS, Apple Silicon, MSRV, simulator and shell-portability checks.

Ordinary development and authorized merges follow the baseline's
[asynchronous CI cadence](../DRAGGINZGAME.md#validation-and-evidence). Complete
focused checks and review, then merge when authorized and enforced GitHub checks
and reviews permit it. Report pending runs with their exact source identity;
pending native jobs are not passing qualification. Release, publication and
deployment retain their separate acceptance gates.

For an explicitly authorized tag-only or other branch qualification, select its
exact ref through the existing manual workflow:

```bash
gh workflow run ci.yml --repo dragginzgame/ic-backup --ref REF
```

The dispatch-enabled workflow must be on the default branch and at the selected
ref. Verify the resulting run's `headSha` and complete native results before
claiming qualification. A tag-only release may reuse successful main evidence
only for its exact commit. Historical refs retain their own workflow version;
an unavailable dispatch path or missing result is unqualified, not permission to
use another commit's evidence. Tag/version/source guards remain in the release
and publication owners. This guide does not authorize dispatch or release.

## Quick start

From the repository root:

```bash
rustup show active-toolchain
make install-tools
make tools-check
cargo test --offline --locked -p ic-backup --test directory_checksum
```

Explicit local setup uses the snapshot-owned [host and Cargo pins](../ci/tool-versions.env)
through the shared [common Make entrypoint](../make/consumer.mk). It prepares
verified jq/yq, ripgrep with PCRE2, cloc, Cargo-sort and Cargo-sort-derives. The
ordered Backup extension fetches locked dependencies, then prepares the selected
Testkit CLI/server and release editor. Previous sets and failed candidates remain
retained. Make selects `.tools/host/bin` and `.tools/rust/common/bin`.
Interactive shells can use:

```bash
export PATH="$PWD/.tools/host/bin:$PWD/.tools/rust/common/bin:$PATH"
```

See [bootstrap prerequisites](local-setup.md#bootstrap-prerequisites).
Optional IC executables are separate; see [manual setup](#current-shared-tool-selection).
Prepared Cargo toolchains, ShellCheck and macOS GNU
Make/flock remain separate product prerequisites. Published Testkit supplies the
test-only PocketIC client and managed server. Its locked package selects CLI and
server assets; the aggregate prepares them before tests. The narrow
`make install-testkit-server` target remains available after a selected-version
change. Testkit authenticates the
server through `make testkit-server-check`, which prints its admitted absolute
path offline. Both simulator suites call that check before managed startup.
Original server digests, private logs and fixture state remain retained.
Offline admission precedes [real single-canister qualification](pocketic-qualification.md).
Direct Agent transport and generic workflow coordination are implemented.
Integrations supply current authority, application admission and stage execution;
installing tools alone establishes no backend or application qualification.

Common setup also prepares the published IC Build release editor through
`make install-release-editor`; `make release-editor-check` admits its absolute
path offline. The release adapter supplies the bounded candidate-rendering input;
the editor is tooling only and does not enter either library's dependency graph.
Both installer selections explicitly enable `cli`; the library dependency on
Testkit keeps that feature disabled. Repeat `make install-tools` when these
selected package versions or features change; older tool receipts do not substitute.
See [release preparation](releasing.md) for its ownership and recovery boundaries.

The shared `make install-rust-tools` and offline `make rust-tools-check` prepare
and check the common Cargo formatters selected by the reviewed catalog through
verified generations referenced under `.tools/rust/`, activated together at
`.tools/rust/common/bin`. Both commands use the reviewed shared pins. A compiler
change selects new build identities even when those pins stay fixed. Explicitly
run `make install-rust-tools` and `make install-testkit-server` after changing
the development compiler, then check them with `make tools-check`. Prior
installations and failed evidence remain retained; they supply no fallback.
Partial setup keeps the previous complete common set active. Shared Tooling owns
[shared executable storage](local-setup.md#shared-executable-storage). Host/IC
and Cargo bundles reuse a per-user store with exact build identities. Application
flags, wrappers and Cargo configuration do not configure tool builds. Testkit's
selected CLI uses that owner; Testkit still owns local server storage and lifecycle.
The default Backup formatter uses Cargo manifest sorting and rustfmt. Initial
setup uses `make install-tools`, then `make tools-check`. Command paths remain
`.tools/host/bin` and `.tools/rust/common/bin`. Existing local bundles and failed attempts
stay retained; setup activates new references without moving or deleting them.
An explicit absolute `SHARED_TOOLING_STORE` setting must be shared by setup,
checks and evidence collection. CI uses one isolated store per native job and
checks actual host/common/selected Cargo reuse from a second consumer offline.
Avoid independent
store owners, sibling PATHs or manual binary relocation. The aggregate
runs common host and Cargo steps in order, then product targets registered
through `LOCAL_TOOL_INSTALL_TARGETS`/`LOCAL_TOOL_CHECK_TARGETS`. Backup registers
locked fetch, Testkit and release editor setup in that order, then Testkit and
editor admission after common checks. Optional IC targets are outside both aggregates.
Do not attach unordered aggregate prerequisites or call the aggregate from an
extension target.
The pre-commit hook and formatting-adoption checker find prepared checkout-local
tools without an interactive PATH export. Their source/configuration inputs remain
the isolated staged export; missing or wrongly pinned tools still reject.

The shared installer also supports an explicitly selected exact Cargo package,
binary or example, and debug/release profile. See [consumer-selected Cargo
tools](local-setup.md#consumer-selected-cargo-tools) for setup and
offline receipt checks. Backup uses this selected mode for its locked `ic-testkit-server` executable;
the complete common formatter set is also checked by `tools-check`. Native `check`, `check-msrv`, `clippy`
and `test` depend on offline `testkit-server-check`, so parallel Make also refuses
missing or changed selected tools before dependent qualification. Prepare them
explicitly with `make install-testkit-server`; ordinary validation never installs.

`make cloc` reports source/test LOC from this root workspace and excludes Cargo's
selected build outputs. Run fleet tooling/LOC reports from Shared Tooling; this
consumer no longer selects those reporters or their regression suite. The shared
consumer include no longer exposes `cloc-tooling`; run that command at its owner.
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
The Core and Agent fixtures call the locked library’s offline
`pic::server_assets::check` directly against the prepared physical bundle, then
retain a bounded server digest privately before managed startup. They do not
spawn Make, the Cargo installer or the CLI. Their application-specific evidence
and original WAT bytes remain local.
After a locked Testkit version update, repeat `make install-testkit-server`; the
older retained CLI cannot satisfy the updated selection. Admission failures name
the selected version and the explicit setup command.

`make shell-check` also requires ShellCheck and Perl. It prefers a prepared
`.tools/ci/bin/shellcheck`, then searches `PATH` and `~/.local/bin/shellcheck`.
It reports the selected command; lint failures never trigger a fallback or skip.
Perl must be on PATH. Linux CI selects reviewed ShellCheck 0.11.0 because Ubuntu
24.04's 0.9 checker rejects an intentional expression in the shared pin checker.
Prepare the same pinned Linux x86-64 binary explicitly:

```bash
source ci/tool-versions.env
bash scripts/ci/install-ci-tool.sh shellcheck \
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
| Ports | Defines async effect and recovery capabilities supplied by integrations |
| Workflow | Coordinates original plans and journals with mandatory async admission |
| Agent | Implements configured single-update transport with explicit evidence retention |

The library deliberately separates a recorded declaration from current authority
to act. A plan, journal or decoded reply does not by itself permit a network call,
prove application safety or establish that a remote operation succeeded.

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-backup/ic-backup-implementation-boundary.svg" alt="Implementation boundary showing locally tested models, policy and persistence, application-supplied integration contracts, and implemented Agent transport with application-owned runners and authority" width="800">
</p>

## Product contracts and ownership

Read [implementation contracts](implementation-contracts.md) before product changes.
The [record/API boundary](extraction-boundary.md) owns detailed limits, evidence
requirements and recovery semantics. [Agent transport](agent-transport.md) describes
configuration and single-update submission. The [current handoff](status/current.md)
records source-specific qualification and outstanding integration limits;
[release notes](../CHANGELOG.md) own historical API removals and migration actions.

Core owns product records, codecs, guarded persistence and workflow coordination.
Host owns generic filesystem/process/hash capabilities; Platform owns external IC
identities, codecs and SDK boundaries; Testkit owns simulator setup and processes.
Applications supply fresh membership, control/read permissions, consistency and
restore-safety evidence through mandatory async admission. Their pure policy
validators remain in Core. No synchronous observation-provider interface or default
application authority is installed.

Use complete operation plans for inventory/graph persistence and original journals
for progress. The workflows provide snapshot capture, transfer, upload, restore and
reconciliation steps; complete application runners, fencing and external-obligation
settlement remain integration-owned. Local records or decoded replies never grant
fresh authority, automatic retries or reference release.

## Focused product checks and fixtures

Prepare the locked tools and dependencies using the quick start before running
checks. Select the tests that exercise the changed boundary; a native model or
filesystem test does not qualify an IC effect. See the current handoff for actual
results rather than inferring qualification from the commands below.

`make check`, `make clippy`, `make test`, `make check-msrv` and `make package` are
aggregate selections. Ordinary work uses the affected package, target or test
filter below; full repository suites remain release or explicitly requested work.
For dependency adoption affecting both libraries, use
`cargo clippy --offline --locked -p ic-backup -p ic-backup-agent --all-targets --all-features -- -D warnings`
after admitting the locked root and independent consumer graphs. The existing
consumer scripts own generated fixture locks; no maintained independent workspace
or deployed canister artifact is added by this host-library adoption.

| Changed boundary | Focused selection |
| --- | --- |
| Model or policy | `cargo test --offline --locked -p ic-backup --lib FILTER`, using the owning module path |
| Public consumer contract | `cargo test --offline --locked -p ic-backup --test TARGET`, choosing the corresponding file in `crates/ic-backup/tests` |
| Artifact publication and recovery | Public targets `directory_checksum`, `ic_snapshot_artifact`, `download_integrity` and `local_restore_source`; unit filters `ops::artifacts`, `artifact_commit` and `ops::persistence::json` |
| IC management/workflow effects | `cargo test --offline --locked -p ic-backup --test pocketic_snapshot FILTER`; `planned_upload` selects complete-upload and failed-write cases |
| Agent HTTP policy | `cargo test --offline --locked -p ic-backup-agent --test http_boundary` |
| Local release receipt/recovery helpers | `bash scripts/release/test-release.sh --receipts-only`; all Git/release effects remain in isolated substitute fixtures |
| Release manifest/lockfile rendering | `bash scripts/release/test-release.sh --preparation-only`; checks ownership, layout preservation and rejected candidates in isolated fixtures |
| Agent simulator integration | `cargo test --offline --locked -p ic-backup-agent --test pocketic_transport` |

Keep independent wire fixtures independent of the Rust encoder. Regenerate metadata
before data/upload fixtures; pass `--check` to compare existing generated bytes
without writing them:

```bash
perl scripts/dev/generate-snapshot-metadata.pl --check
perl scripts/dev/generate-snapshot-data.pl --check
perl scripts/dev/generate-snapshot-upload.pl --check
```

After intentional regeneration, run the corresponding codec tests and review the
changed contracts/goldens. Relevant targets include `ic_snapshot_metadata`,
`ic_snapshot_data` and `ic_snapshot_upload`; the data tests also cover incremental
coverage. Goldens qualify bytes and bounds, not transport or application authority.

Public integration journeys allocate directories through
`crates/ic-backup/tests/support/mod.rs`, print each path and retain failed evidence.
Only successful fixtures explicitly remove their own directories. Fixture allocation
and recovery checks use the `fixture_roots` and `fence_reconciliation` targets.
Unix FIFO fixtures require the standard `mkfifo -m 600` utility and verify its
resulting type and permissions on Linux/macOS.

Install the workspace MSRV separately before `make check-msrv`. That target checks
all targets/features and each library in an independent consumer without Testkit
feature unification. Fixtures seed the original lock, prepare only their own locks
offline and reject changes to selected package/version/source identities. Native
macOS evidence remains separate from Linux; the supported product runs on a host.

Measure artifact verification/preparation in the caller or performance fixture.
Diagnostics never supply progress, spending, receipts or recovery authority.

## Current shared tool selection

The [snapshot](../.shared-tooling.snapshot) and its [catalog](../ci/ic-tools.tsv)
own optional IC executable versions. Operators explicitly choose executable names:
`make install-ic-tools IC_TOOLS=wasm-opt` prepares that selection, and
`make ic-tools-check IC_TOOLS=wasm-opt` checks it offline. Use a quoted list for
multiple tools, with the same selection for setup and checks. No complete bundle
is selected implicitly. Neither manual target is part of
`install-tools`, `tools-check`, CI or release preflight. Add `.tools/ic/bin` to an
interactive PATH only when deliberately using these manual tools. Existing bundles
and failed evidence remain retained. See [the shared installer](ic-tools.md);
these commands grant no live backup/restore authority. Testkit remains mandatory
for native checks and simulator qualification.

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

The pinned Rust 1.99.1 development toolchain, Rust 1.91.0 minimum compiler,
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
`rustup toolchain install 1.91.0 --profile minimal` in this checkout. Install the
reviewed manifest formatter after `source ci/tool-versions.env` with
`cargo install cargo-sort --version "$SHARED_TOOLING_CARGO_SORT_VERSION" --locked`,
then run `make install-hooks` once per clone or after updating the hook contract.
Setup is explicit; hooks and validation never install tools. CI installs
the host-specific tooling and uses the same native validation gate on all three
hosts. On macOS the full gate selects system Bash 3.2, including tooling and
Testkit GNU Make routing checks. These setup paths and configured jobs are not
evidence that the pending native macOS runs passed.

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

`make check-cargo-tool-versions` is an explicit online inspection of the shared
Cargo tool pins, checkout selection and user-installed Cargo packages against
current non-yanked stable crates.io releases. It retains response evidence and
reports unavailable observations separately. It does not install a tool or
change pins; `make rust-tools-check` remains the offline byte/build admission.
The [Cargo tool freshness task](../tasks/cargo-tool-versions.md) explains its
prerequisites and result codes. The checker does not add `cargo-modules` or any
other package to the selected tool set.

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
Use `make release-plan` for a non-mutating release preview. Make rejects
ignore-errors, dry-run, touch and question modes before recipes run, so skipped
commands or ignored failures cannot appear to validate a release or formatting.

`publish-dry-run` and `publish` delegate to Cargo for the current library version.
They use locked dependencies and crates.io. Cargo checks package cleanliness and
publication eligibility; receipt/tag checks belong to repository release commands.

Native custody regressions require Python 3 on the selected Unix host. They use
real inherited descriptors and acknowledged owner/direct-child/descendant exit
with bounded fixture waits. The production library delegates owned child descriptor
setup and custody to published Host Process and contains no Rust unsafe code.
It does not invoke Python; those commands are local test fixtures. A transport
still needs its own output/deadline/retry and inherited-descriptor qualification.

The Makefile exports this checkout's absolute `target/` directory. Never point
it at Canic's target or add Cargo patches to sibling checkouts. Before editing
source or locks and before compilation, check for an active command using this
repository's build directory. Let that command finish first. Direct Cargo
commands must use the same local directory.

Compilation uses `--offline --locked`. `make deps` fetches the committed lockfile's
dependencies when needed; it does not select new versions. Serde/JSON, SHA-256,
thiserror, Platform principal conversion and Unix rustix/Host Process
declarations belong in `[workspace.dependencies]`,
with package-level entries inheriting them. There are no Canic dependencies or
sibling checkout patches.
License metadata also inherits from the workspace. The member's regular `LICENSE`
copies the maintained root MIT notice exactly, preserving contributor attribution
in the standalone crate archive. Update both copies together; a tracked symlink
would violate the shared index-snapshot formatting contract.

Follow the [shared validation cadence](../DRAGGINZGAME.md): run meaningful focused
checks for affected boundaries before handoff and reuse passing evidence for
unchanged inputs. Run the local full suite for an explicitly authorized release
or explicit request for those checks. Configured CI retains its own gates;
release effects need separate authority. `make ci`, `make validate` and
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
source and builds both current archives together; it does not publish them.
`make package` retains a fresh `package-check.*` fixture under `TMPDIR`, defaulting
to `target/`. CI's retained `target/ci-fixtures` root includes package and Testkit
routing failures in its archive. Cargo creates the
archives; an independent normal consumer then mandatorily builds their exact
unpacked pair. A fixture-only registry override selects the unpacked core, avoiding
the already published same-version core during unreleased API work. Metadata checks
preserve original dependency selections, exclude simulator packages and require
both exact archive paths; archive/source hashes and root manifest/lock remain
unchanged. This adds no root/sibling patch or version transaction. Registry
publication continues to use Cargo's ordinary admission.

## Dependency roles and production graphs

Both workspace members, `ic-backup` and `ic-backup-agent`, are production host
libraries. Their supported targets are Linux x86-64 and macOS on Intel and Apple
Silicon; neither package declares feature switches. The root Cargo catalog uses
`Production dependencies` and `Tests and tooling only` groups. Entries used by
both production and tests remain in production. In particular, direct `sha2` and
`tokio` uses are test-only, but production graphs need SHA-256 through Host/Agent
and Tokio through Agent. `ic-platform` is the production Candid and native Agent
boundary; direct Agent declarations are prohibited. Reqwest
owns the consumer’s HTTP client configuration;
`ic-testkit` and Host Artifacts' `wasm-text` feature are test/tooling selections.
Fixture WAT compilation uses `ic_host_artifacts::wasm_text::compile` with explicit
64 KiB source and output bounds. Only development edges enable this feature;
`wat` remains Host's private transitive backend. Names do not determine roles.
`make dependency-boundaries-check` rejects direct WAT declarations, including
aliases and development/build/target-specific edges, alongside the IC boundaries.

Independent MSRV and unpacked-package consumers reuse `consumer-graph.jq` to
reject `ic-testkit`, `pocket-ic` and `wat` anywhere in their normal graph by resolved
package identity, so renaming an edge does not evade admission. Adding another
test-only catalog entry requires reviewing and updating that explicit policy;
comments are not an automatically interpreted exclusion list. The dependency-pin
check enforces source/version declarations and inheritance, not these roles.
The Core MSRV consumer also imports Host Artifacts and FS directly using the
requirements projected from Core's production metadata. It compiles the checksum
error conversion and publication-error payload through those consumer-selected
identities, catching split Host versions without hard-coded fixture pins.

The independent journey executable uses Clap for its private command input,
with its requirement projected from Agent's development catalog. Its own normal
dependencies include this parser; neither published library's production graph
includes Clap. Fixture graph admission still preserves the selected package
identities and excludes Testkit, PocketIC and WAT.

This adoption inspected locked normal/build graphs for both libraries with default
features across all three supported target triples. None contained the excluded
packages. Cross-target Cargo trees are source/graph inspection, not native macOS
build evidence. This repository publishes libraries and has no application
deployment entrypoint or deployable canister artifact. A consuming application's
build/deployment owner must still enforce its actual target/features and bind the
locked successful build and final artifact digest to the bytes deployed. Passing
formatting, snapshot integrity or these library fixtures is not deployment admission.

## Formatting and evidence

Both `make fmt` and `make fmt-check` select Cargo sort's `--grouped` policy before
Rust formatting. The first role heading decorates the dependency table itself,
so the pinned sorter retains it when repairing the first group's order. Hooks
call the same write command; release preparation calls the same check command.
Committed release checks use Shared's `--manifest-order-grouped` against the exact
selected Git source, preserving independent receipt and resume checks.

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
Snapshot and lifecycle checks require PocketIC or a deliberately selected real local IC backend. The original
[Canic source baseline](source-baseline.json) remains retained planning evidence.
[Fresh source provenance](extraction-source.json) records this extraction input
and the public consumers; it is separate from test qualification.

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
`make tooling-check` exercises this repository's actual CI dispatch, snapshot
adoption, retained evidence and Testkit command routing. Its proof owners are:

| Obligation | Consumer check |
| --- | --- |
| Exact snapshot bytes/modes and tamper refusal | `shared-tooling-check`, `scripts/ci/test-tooling.sh` |
| Input-change stop, later-stage refusal and original failure retention | `scripts/ci/test-tooling.sh`, `make release-check` |
| Local documentation, pins and package graph | `check-doc-links`, `dependency-pins-check`, `check`, `package` |
| Prepared host/IC/Cargo/Testkit tools and formatting | `tools-check`, `testkit-server-check`, `fmt-check`, `hooks-check` |
| Evidence checksum/archive behavior and Testkit CLI routing | `tooling-check`'s three retained focused fixtures |

Shared Tooling's generic implementation regressions remain in its own CI; this
consumer no longer dispatches their entire fixture matrix on every local gate.
`make release-check` retains focused consumer source/version/recovery checks
using isolated fixtures. Native CI remains the host acceptance owner.

CI creates `target/ci-fixtures` before setting `TMPDIR`, then uploads that directory,
retained test directories and validation failure logs on job failure. Hidden fixture
files include private Git index evidence. These test artifacts have seven-day
retention; they do not replace durable operator journals or source references.
The upload contains a local log archive and a separate canonical selection archive
with local Cargo records and admitted shared bundles. Local logs are archived first.
Rejected shared paths preserve diagnostics, safe local selections and the original
collector status; the upload remains eligible with collection explicitly incomplete.
The shared composite action retains the same safe local rejection evidence under
[#121](https://github.com/dragginzgame/shared-tooling/issues/121).
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

`fmt`, `fmt-check` and `format-tools-check` come from the selected Shared Tooling
`make/consumer.mk` entrypoint and its `make/rust-format.mk` owner.
`FORMAT_CARGO` may explicitly name the prepared
Cargo executable; the recipes use portable checkout-local PATH lookup.
`fmt` and `fmt-check` share `format-tools-check`, which reads the reviewed pin
from `ci/tool-versions.env`. It requires successful exact cargo-sort and rustfmt
availability probes, offline with rustup automatic installation disabled. It
installs nothing and changes no files. CI setup uses that same pin; workspace
sorting, actual Rust formatting and hook/index custody remain consumer-owned.
The shared Rust driver reports each pass and its elapsed time. It can reuse a
successful result only for identical observed inputs, tools and policy; prepared
tools are still checked. Use `make fmt-check FORMAT_RUST_ARGS=--no-cache` to run
every pass explicitly. Isolated staged-hook worktrees always run their formatters.
The helper's rejection fixtures run in normal tooling checks and explicitly
under macOS system Bash 3.2. Native qualification belongs to the configured
consumer jobs, independently from a Linux pass or upstream source adoption.
