# Current handoff — 2026-11-05

## Pending upstream adoption

The user requested Shared Tooling and IC dependency adoption, then selected
**0.58.3** delivery of both crates to `origin/main`, annotated `v0.58.3` and
crates.io. Package versions remain **0.58.2** until standard release preparation;
the full release gate and final remote/registry verification own completion.

Production requires Platform **0.23.0** and Host Artifacts/FS/Process **0.74.4**.
Build **0.74.4** is the installed/offline-admitted release metadata editor.
Testkit **0.86.0** remains the latest published selection, with its exact CLI and
PocketIC **16.1.0**. It retains development-only Platform **0.22.3** while both
facades share Types **0.18.1**, external Principal **0.1.5**, management DTO
**0.11.0** and Agent **0.49.3** identities. Incoming Candid/derive **0.10.40**,
Wasm-only Runtime **0.19.2** and mime **0.3.18** lock changes are preserved.
Rust minimum and exact compiler remain **1.99.1**.

Platform's consumed adapters are unchanged; optional protocol/CBOR capabilities
stay disabled. Candid fixes sequence consumption for fixed-size visitors and tuple
subtyping. Host changes only private test ownership; Build's CLI/metadata sources
are unchanged. Runtime IC0 changes only lint annotations; CDK changes are unused.
Published archive checksums, official Rust/original-manifest source comparisons
and diffs belong to `target/ic-deps-adoption.sq2a3wsj/`.

## Current focused qualification

IC adoption proof in that directory binds stable Cargo/Make/compiler inputs and
unchanged product Rust. Strict all-target/all-feature Clippy, **89 tests**, both
independent normal consumers, compiler/dependency/golden/format/link checks and
release metadata preparation pass. The tests include 78 model codec cases, eight
public API cases and three real simulator journeys: Core capture/download, Agent
same-ID restore and independent cold-process restore/terminal replay. Both normal
consumers select only Platform 0.23.0 and exclude Testkit, Runtime and WAT.
Consumer evidence is `target/msrv-consumers.nxQ9wN/`; release preparation evidence
is `target/shared-release-tests.PYeyhV/`. Original fixture evidence remains retained.

Shared Tooling **0.30.1 at `0c5eebc49534e99b09326c49b003a0adfd2348cb`** is
exported canonically: 155 payloads, 28 updates and 127 unchanged. Its independent
source/mode proof, offline snapshot check, actual hooks, release metadata and
preparation, format/link/shell/Perl checks pass. LOC hashes, classifications and
counts match all 324 previous Rust files; only two diagnostic notes change order.
Proof is `target/shared-0301-adoption.3nbqn9wj/` and
[the adoption review](../shared-tooling-review.json).

LOC uses schema 2, workspace names once and `???` for unclassified counts; no
source is unclassified. Formatting/report scratch follows the owning target;
explicit overrides and failed evidence retain their owners. Existing tests/
ownership, checked lint expectations and ordinary Cargo fixture discovery remain.
Agent transport qualification uses `cargo test -p ic-backup-agent --example standalone`.
Candidate projection and group proof migration remain unselected. No service or
schedule is activated. Full workspace/release gates and hosted CI were not rerun
for this batch; Linux proof does not establish native macOS, live IC or installed
Canic acceptance. No release-stage reuse is enabled.

## Delivered baseline

Both crates **0.58.2** are published. Release
`7ab98664c8de98faf5fc36f803ca5581b26be707`, annotated `v0.58.2` and handoff
`07da92a` are in `origin/main`; original source is
`49512f06f40e982227a673d0a44863067a0969c6`. The [receipt](../release.json)
binds the original complete Linux release gate: strict Clippy, 554 tests,
independent workflows and packaging. Registry checksums, clean VCS identities and
341 source files were independently verified against the release commit.
Proof stays in `target/deploy-0582.ft4gpnz7/` and `.git/release-state/`.
Earlier source/recovery/adoption evidence stays at its original owner and revision.

## Recovery and retained contracts

Standalone host-driven backup/restore, mandatory application admission and fenced
same-ID restore safety remain. Current v1 records/hashes and bounded attempt
spending are unchanged; persist intent before effects, reconcile lost replies,
and preserve unfinished journals, artifacts, references and external obligations.

Layouts containing `consistency-requirement.json` or `fence-obligation.json` refuse
before record use/spending, including malformed bytes and symlinks. Distinct-edge
learned continuations refuse without rewriting bindings. Recover those originals
with **0.57.1 and the original qualified integration before upgrading**. Never
remove markers, rebuild plans, reinterpret payloads, re-sign requests or reset
accounting to bypass admission. No migration, evidence cleanup or refund is provided.
See [the migration](../canic-adoption.md#058-api-migration) and
[implementation contracts](../implementation-contracts.md).
