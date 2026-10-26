# Shared engineering and tooling adoption

## What this adoption means

Shared Tooling owns the reviewed engineering baseline and common setup,
formatting, hook, validation and release mechanisms. Backup owns its product
contracts, qualification selection, ordered Testkit extension, release metadata
adapter and exact recovery receipt checks. Owner CI and successful setup do not
qualify Backup behavior or authorize a release.

## Reviewed sources

[The snapshot manifest](../.shared-tooling.snapshot) binds the committed revision,
selected paths, checksums and executable modes. [The review](shared-tooling-review.json)
records provenance and consumer qualification. [AGENTS.md](../AGENTS.md) is the
local overlay. Selected dependency versions belong to [Cargo.toml](../Cargo.toml)
and [Cargo.lock](../Cargo.lock); current evidence and material limits belong to
[the handoff](status/current.md).

Refresh deliberately using [the canonical procedure](consuming-snapshots.md).
Inspect the complete exported change and qualify affected consumer boundaries.
Never patch snapshot payloads in place or use a moving sibling checkout. Every
listed path is excluded from local branding and documentation rewrites; consumer
guidance belongs in unlisted local documents.

## Required setup and offline admission

`make install-tools` prepares common host and Cargo tools, fetches locked
workspace dependencies, then prepares the locked Testkit CLI/server and the
release editor selected by `RELEASE_EDITOR_VERSION` in [Makefile](../Makefile).
`make tools-check` checks the prepared selections offline and never installs.
Required executable paths are `.tools/host/bin` and `.tools/rust/common/bin`;
Testkit and the editor use admitted absolute paths. Previous bundles and failed
evidence remain retained and supply no fallback.

Common catalogs live in [ci/tool-versions.env](../ci/tool-versions.env). Testkit
owns its authenticated PocketIC selection and process lifecycle; the unique
locked Testkit package also selects its CLI through the shared Cargo installer.
Both simulator suites check the library-owned server assets offline before
startup. See [development](development.md#quick-start) for prerequisites and
[releasing](releasing.md) for editor preparation and recovery authority.

## Consumer-owned IC pins

The snapshot-owned [IC catalog](../ci/ic-tools.tsv) owns selections for the five
optional IC executable versions. Manual setup and offline checks require the same
explicit `IC_TOOLS` selection, for example `make install-ic-tools IC_TOOLS=wasm-opt`
and `make ic-tools-check IC_TOOLS=wasm-opt`, through
[the shared installer](ic-tools.md). A quoted list selects multiple executables;
no full bundle is selected by default. These manual targets are not
prerequisites of aggregate setup, CI or release preflight, and their directory is
not added to the default PATH. An operator using these executables must select
and admit them explicitly; installation supplies no live-effect authority.

PocketIC is selected solely by locked Testkit, not by that optional catalog.
Backup maintains no second server catalog, version equality rule, reset/funding
policy or fallback to an old shared bundle. Required Testkit admission and fixture
custody remain independent of optional executable setup.

## Consumer qualification selection

The local [Makefile](../Makefile), [CI workflow](../.github/workflows/ci.yml) and
consumer fixtures select Backup's checks; Shared Tooling's owner suites do not
become consumer gates. Focused validation covers affected routing, ordering,
refusal and evidence custody, with early strict Clippy before expensive product
checks. Reuse passing evidence only for unchanged source and graph scope.

Native Linux, native macOS, cross-compilation and installed-IC acceptance are
distinct evidence. The direct dependency gate and independent package consumers
also have distinct scopes; see [the Platform boundary](ic-platform.md).

The reviewed Shared Tooling 0.26.20 snapshot includes the
[asynchronous CI cadence](../DRAGGINZGAME.md#validation-and-evidence) for ordinary
development and authorized merges. Focused checks, review and enforced GitHub
requirements still apply. Report exact pending run/source identities without
polling the full matrix or claiming its native-host qualification. Release,
publication, deployment and application acceptance remain separate.

The shared validation runner owns failure summaries, bounded excerpts and atomic
latest-log publication. Relative retained-log paths resolve from the caller,
including leading-dash paths, independently of inherited `CDPATH`.
Backup qualifies its actual callers and evidence custody;
it keeps no alternate summary implementation. ShellCheck follows sourced helpers
with `-x`. The checksum verifier rejects NUL-corrupted tool output before shell
capture. Optional producer download helpers are not consumer dependencies.

The adopted baseline includes
[#300 — Clarify runtime approval boundaries](https://github.com/dragginzgame/shared-tooling/issues/300):
enforced GitHub requirements and runtime approval restrictions remain distinct.
Optional hosted CI supplies no additional approval requirement, and runtime
rejection must be reported without switching execution paths to evade it.
The snapshot also includes the shared [CI/release integrity audit](../audits/ci-release-integrity.md).
Its availability adds no recurring audit or new product gate.

Explicit Cargo tool setup now waits up to 60 seconds for an admitted owner of the
selected tool slot. `SHARED_TOOLING_SETUP_WAIT_SECONDS` selects a canonical 0..300
second budget; zero refuses immediately. Setup preserves the owner and failed
evidence, then rechecks compiler/build identity, locked selection, source checksum
and receipt before reuse. Offline `--check`/`--preflight` admission remains
immediate and never repairs selections. The wait tolerates record withdrawal only
for the same previously admitted physical lock; missing initial records and
replacement claims still refuse.

Source-cache initialization now uses admitted idempotent directory creation,
without a global installation lock. Cargo owns download serialization; old
source-preparation slots and previous generations remain retained. This changes
the build-policy digest. When a refresh changes that digest, run `make install-tools`
once to refresh the common Cargo tools, locked Testkit CLI/server and release editor. See
[the canonical setup guidance](local-setup.md#standard-cargo-tool-build-environment).

## Current guard and recovery integration

The direct dependency target composes Shared's reviewed
[external IC inventory](../ci/external-ic-dependencies.txt) through `--external-ic`
with Backup's [additional exclusions](../ci/cargo-dependency-boundaries.json).
The common guard owns Candid, Principal, Agent, CDK, simulator and other known IC
package declarations; the local overlay retains three additional retired IC
packages and direct WAT exclusion. Cargo metadata preserves package identity
through aliases and includes normal/build/dev, optional and target-specific
entries. The actual Make target has focused composition/refusal coverage.

These declaration checks are separate from resolved normal-graph simulator/WAT
exclusions in the independent consumer fixtures. Backup is a host library and
ships no deployed canister entrypoint binding target/features, a locked build and
an artifact digest. Neither role labels nor the declaration guard supply that
missing deployment admission. Application consumers retain that responsibility.

`make tools-report` runs all existing offline tool checks and retains every
failure; `tools-check` remains fail-fast. Cargo setup reports phases on stderr
while stdout remains path-only. Offline Cargo admission remains immediate by
default; explicit `--check --wait-seconds N` allows bounded read-only waiting.
Status 75 means admitted contention, including timeout, and never authorizes
installation or lock cleanup. Backup preserves that status and distinguishes it
from a missing Testkit selection; the release editor adapter already propagates
its admission failure without setup fallback.

Normal release commands start fresh from current clean, committed metadata without
reading historical plans. The selected candidate's stale plan and staging roster
are superseded; other plans, Backup's metadata backups and validation logs remain
retained. Backup archives an earlier validation sidecar before recording fresh
proof. Use `release-resume VERSION=X.Y.Z` for exact saved-attempt recovery, including
the mandatory original receipt check. Consumer fixtures qualify fresh attempts,
failed-gate retention and explicit recovery through the actual Make/adapters.

The compact baseline delegates detailed contracts to task-specific rules and
makes line/symbol accounting opt-in. The npm SDK boundary remains optional;
Backup's Rust-only graph needs no npm tooling. Snapshot adoption supplies neither
application deployment admission nor release/publication authority.

## Adoption history

The [archived adoption record](status/archive-shared-tooling-2026-10-21.md)
preserves the complete earlier guide, exact revisions, setup transitions and
qualification receipts. It is evidence rather than current command guidance.
