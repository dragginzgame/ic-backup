# Shared engineering and tooling adoption

## What this adoption means

Shared Tooling **0.30.4**, committed revision
`2aa41c7394300302fe802b69b31b855bdc89b671`, owns the reviewed engineering
baseline and common setup, formatting, hook, validation and release mechanisms.
The canonical refresh from `0c5eebc` selects **157 payloads**: three updates,
two additions and 152 unchanged. Catalogs, Cargo selections and Rust **1.99.1**
are unchanged. Snapshot payloads are never locally patched.

Backup owns product contracts, qualification selection, the ordered Testkit setup
extension, release metadata adapter and exact resume receipt checks. Owner CI or
successful tool setup does not qualify Backup behavior or authorize delivery.
Backup has no Canic runtime/control-plane dependency and keeps one coordinated
release, root changelog and complete release gate. Optional groups, candidate
sources and stage-proof reuse are not selected.

The refresh adds the shared lexical Candid boundary check and fixes empty-option
LOC reporting for Bash 3.2. Current experimental-retention rules keep compact
conclusions in Git and raw output under `target/evidence/`. The authorized cleanup
retires superseded per-run reports/handoffs; [the retention summary](reports/README.md)
records the retained conclusions and exact historical source. Product provenance,
release receipts, journals, artifacts and recovery evidence retain their owners.
No ignored bundles or tool stores are cleaned by adoption.

## Reviewed sources

[The snapshot manifest](../.shared-tooling.snapshot) binds the exact committed
revision, paths, checksums and executable modes. [The review](shared-tooling-review.json)
records consumer decisions and focused qualification. [AGENTS.md](../AGENTS.md)
is the product overlay. Current selections belong to [Cargo.toml](../Cargo.toml)
and [Cargo.lock](../Cargo.lock); status and limitations belong to
[the handoff](status/current.md).

Refresh with [the canonical exporter](consuming-snapshots.md) at a reviewed full
commit. Compare payload bytes/modes with an independently selected source through
the trusted verifier's paired `--source` and `--revision` arguments. Ordinary
builds retain offline local integrity checking; integrity alone is not provenance.
Backup keeps its stricter regular-file inventory and product filesystem admission.

## Required setup and offline admission

`make install-tools` prepares common host/Cargo tools, fetches locked workspace
dependencies, then prepares the locked Testkit CLI/server and Build release
metadata editor selected in [Makefile](../Makefile). `make tools-check` admits
those prepared selections offline and never installs. Setup retains previous
executable generations and failed bundles; no old selection supplies a fallback.

The common host/Rust catalogs belong to [ci/tool-versions.env](../ci/tool-versions.env).
The compiler floor and exact development/release toolchain both remain 1.99.1.
Testkit **0.86.0** owns simulator assets/lifecycle; Build **0.74.4** owns metadata
editing. [The Platform boundary](ic-platform.md) owns dependency compatibility.

Formatting uses grouped Cargo sorting followed by Rustfmt through the shared
driver. Hooks format selected files, reject partial staging and preserve unrelated
edits; CI/release use non-mutating checks. `make fmt-check
FORMAT_RUST_ARGS=--no-cache` forces every pass. Installer identity changes require
explicit setup and offline admission; store reclamation and compiler-cache
maintenance are separate [owner operations](tool-store-maintenance.md).

## Consumer-owned IC pins

The [optional IC executable catalog](../ci/ic-tools.tsv) is snapshot-owned.
Manual setup/checks require the same explicit selection, for example
`make install-ic-tools IC_TOOLS=wasm-opt` and
`make ic-tools-check IC_TOOLS=wasm-opt`. Aggregate host/Cargo setup, CI and release
preflight select no IC executable bundle or IC PATH.

PocketIC is selected solely by locked Testkit, including its exact CLI through
Shared's Cargo installer. `make install-testkit-server` prepares it;
`make testkit-server-check` prints the offline-admitted absolute path. Both
simulator suites independently check Testkit's library-owned assets before
managed startup. Backup supplies fixture custody, original journals and exact
management ingress; no baseline reset, hidden retry, funding policy or receipt
inference is adopted. Native application acceptance remains separate.

## Consumer qualification selection

The local [Makefile](../Makefile) and [CI workflow](../.github/workflows/ci.yml)
select Backup's checks. Shared's producer matrices stay upstream. Local tooling,
hook and release fixtures qualify actual consumer routing, refusal and evidence
custody. Use [focused checks](development.md) for ordinary work; full repository
and release gates need their own authority. Native Linux, macOS, cross-compilation
and installed IC acceptance are distinct proof.

`make candid-boundary-check` scans both crates, including tests and the standalone
example, once in the sequential CI/release gate. Its
[declaration policy](../ci/candid-boundary.json) uses the shared checker's JSON
input format. All 36 exact allowances serve named external management protocols:
private bounded projections, management argument encoding and adversarial wire
fixtures. Domain/persisted records and owned application payloads receive no
allowance. Prefer existing Platform DTOs; bounded projections retain product
allocation/work limits and explicit conversions.

The checker prunes directories named `target`; Make explicitly selects the
maintained `inventory/target/mod.rs` as well, covering all 324 Rust files. New
source under a pruned directory needs an explicit selection too.

The [shared checker](candid-boundary.md) is lexical, not type resolution or macro
expansion. Review generated implementations and cross-module aliases separately.
Source review found no hidden Candid re-export, application source injection or
path-based module override. The source check does not prove codec
compatibility, caller authority or IC effects.

`make cloc` uses schema-2 source categories and counts each admitted path once,
including shared target roles and inline tests. Use `--json` for hashes/diagnostic
notes and `--subcrates` for member rows. Categories do not measure binary size or
code removal. Tests use `tests/`, `tests.rs` or `*_tests.rs`; fixtures declare
roles in their manifests. Checked target-scoped lint expectations retain the
selective helper-use boundary; unfulfilled expectations remain denied.

## Current guard and recovery integration

The dependency gate combines Shared's [external IC inventory](../ci/external-ic-dependencies.txt)
with Backup's [local exclusions](../ci/cargo-dependency-boundaries.json). Independent
normal consumers separately exclude simulator, runtime and WAT packages. Role
labels and declaration checks do not establish deployment admission.

The shared validation runner retains failure logs/excerpts and observes declared
Cargo inputs. Documentation selection omits Git-reported pending deletions while
still checking new documents and broken links. Formatting/report scratch follows
the owning target directory; explicit temporary-directory overrides remain.
`make tools-report` retains all offline failures; `tools-check` is fail-fast.
Admitted contention never authorizes lock removal or installation fallback.

Normal releases start fresh from committed source. Backup retains original
manifest/lock/notes, its v1 receipt and exact saved-source validation. Use
`release-resume VERSION=X.Y.Z` only for the selected retained attempt. The shared
runner owns Git effects; publication remains a separate command. See
[releasing](releasing.md) for delivery authority and interrupted-effect recovery.

## Adoption history

Earlier adoption and audit text remains at its original Git revision, linked from
[historical retention](reports/README.md). Historical counts, hashes and local
paths do not qualify current code or prove that raw bundles are still available.
Current source/review/status replace per-run documentation copies; protected
product provenance and recovery records remain separate.
