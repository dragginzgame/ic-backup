# Shared engineering and tooling adoption

## What this adoption means

Shared Tooling **0.31.5**, committed revision
`b2026a118e38de4af1555c527af327cb604eacdf`, owns the reviewed engineering
baseline and common setup, formatting, hook, validation and release mechanisms.
The canonical refresh from `d2a2f39` selects **160 payloads**: three updates and
157 unchanged, with no additions or retirements. It closes release-batch companion
annotations in the release and engineering guides; executable payloads are unchanged.
Earlier 0.31.3 adoption supplies the consumer behavior below. It adopts Bash 3.2 checksum
parsing, missing-scratch diagnostics and the rule requiring output owners to
create parents on fresh runs. Backup's four override-aware validation scripts now
create their selected scratch parent. The 0.31.2 release-editor repair changes
Shared's own fixture; it supplies no additional consumer payload.
Host/Cargo/IC catalogs and Rust **1.99.1** are unchanged. Snapshot payloads are
never locally patched. Delivered **0.59.1** qualifies Host **0.75.0**, Platform
**0.24.0** and Testkit **0.92.0** through the complete Linux release gate and independent
registry-only backup/restore acceptance. [The handoff](status/current.md) records
the exact source, receipts and separate native macOS/Canic scope. Focused adoption
and subsequent release evidence retain their distinct identities.

Backup owns product contracts, qualification selection, the ordered Testkit setup
extension, release metadata adapter and exact resume receipt checks. Owner CI or
successful tool setup does not qualify Backup behavior or authorize delivery.
Backup has no Canic runtime/control-plane dependency and keeps one coordinated
release, root changelog and complete release gate. Independent groups, stage-only
validation candidates and stage-proof reuse are not selected.

The snapshot supplies release batches and release-efficiency guidance. Version
0.31 hard-cuts to `RELEASE_GROUPS`, `RELEASE_PUBLISH_TARGET`, `RELEASE_SETUP_TARGET`,
`RELEASE_SETUP_MODE` and `RELEASE_BATCH_ACTIVE`; old settings refuse even when
empty. Canonical `run-release-batch.pl` and `release-batches.md` replace their old
names. Schema-2 journals require the recorded publication commit; pre-0.31 batches
must finish with their original tooling, with no schema rewrite or HEAD inference.
There are no real pre-0.31 batch journals in this checkout. Earlier reproducible
fixture evidence is preserved. Recovery storage paths/source pins remain unchanged.
Upstream owns generic batch execution/lifecycle regressions. Shared derives the
single-group roster from `RELEASE_GROUP`, and clearing it cannot select live
delivery. Backup retains its coordinated `backup` group. Shared
owns setup dispatch, batch recovery, fast-forward reconciliation and explicit
retirement; Backup retains actual setup, complete validation, original-source
receipts and exact resumed-receipt admission. Fresh candidate setup installs tools
once; recovery checks them offline. New tags use `backup-v…`; historical `v…`
recovery explicitly selects an empty group. The group catalog retains workspace
inheritance and the root ledger, adding the complete member dependency catalog.

Consumer qualification uses actual sources, installed setup, clean locked native
builds, verified archives and local-only Git delivery. It loses a real commit
reply, defers a normal-tree lock edit, and substitutes only registry upload while
checking archive/source identities and uncertain-publication refusal. Completed
replay, dirty reconciliation refusal, preserved deferred work and explicit fixture
retirement pass. This is adapter qualification, not a new complete product test
or live delivery. The 0.31 integration passes for renamed hooks, schema-2 intent
and recorded publication identity on captured Platform 0.23.4/Testkit 0.89.3.
See [releasing](releasing.md)
for authority and receipt custody.

Snapshot export now requires version/entrypoint annotations, while historical
manifest inspection remains available. Checksum verification rejects NUL output
before shell capture and distinguishes backend failure statuses. The consumer
bootstrap checks the actual Make refusal and retains evidence. VERSION/changelog
base alignment and npm lock schema 3 were reviewed; Backup has neither a VERSION
ledger nor maintained npm manifests to migrate.

Backup is full-gate-only, with no qualified stage reuse or isolated implementation
mapping. CI already omits the duplicate root `check` after Clippy; independent
normal-dependency consumers and package archives establish different contracts.
They remain required. No runtime or elapsed-time improvement is claimed.

The existing Candid/CBOR boundary and central package inventory stay active. Backup enables the shared lexical Candid/CBOR check once for
both crates, including tests/examples and the explicitly selected inventory
`target` module. Its 36 external Candid allowances remain live; no CBOR exception
is needed. Signed ingress bytes retained as `.cbor` are external protocol evidence,
not a local codec or owned CBOR record. No persisted byte/hash migration is needed.

The Cargo boundary runs for the root and each materialized independent consumer
before compilation. Generated MSRV API probes also receive their own empty-policy
Candid/CBOR source check; the standalone consumer uses Rust files already selected
under `crates`. There are no maintained browser roots or alternate manifest templates.
The shared inventories own forbidden package names; no local list duplicates them.
These are declaration/literal-source checks, not macro expansion, type resolution,
authentication or external-byte compatibility proof.

Current retention rules keep compact conclusions in Git and raw output under
`target/evidence/`; [the prior retention summary](reports/README.md) owns historical
cleanup conclusions. Product artifacts, journals, release receipts, failed bundles
and tool stores are not cleaned by adoption.

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
The last delivered qualification selected Testkit **0.92.0** for simulator
assets/lifecycle and Build **0.74.4** for metadata editing. Incoming lockfile
updates retain their separate dependency qualification. [The Platform boundary](ic-platform.md) owns dependency compatibility.

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
and [CBOR inventory](../ci/cbor-dependencies.json) with Backup's
[local exclusions](../ci/cargo-dependency-boundaries.json). Independent
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
