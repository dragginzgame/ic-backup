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
optional IC executables. Manual `make install-ic-tools` and `make ic-tools-check`
remain available through [the shared installer](ic-tools.md). They are not
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

## Adoption history

The [archived adoption record](status/archive-shared-tooling-2026-10-21.md)
preserves the complete earlier guide, exact revisions, setup transitions and
qualification receipts. It is evidence rather than current command guidance.
