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

The reviewed Shared Tooling 0.26.7 snapshot includes the
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
immediate; setup waiting alone does not invalidate existing tool binaries. See
[the canonical setup guidance](local-setup.md#standard-cargo-tool-build-environment).

## Adoption history

The [archived adoption record](status/archive-shared-tooling-2026-10-21.md)
preserves the complete earlier guide, exact revisions, setup transitions and
qualification receipts. It is evidence rather than current command guidance.
