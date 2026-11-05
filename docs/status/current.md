# Current handoff — 2026-11-05

## Pending Shared Tooling adoption and cleanup

The user requested the new Shared Tooling and cleanup. Shared **0.30.4** at
`2aa41c7394300302fe802b69b31b855bdc89b671` is canonically exported: 157 payloads,
three updates, two additions and 152 unchanged. Package versions stay **0.58.3**;
compatible pending notes are **0.58.4**. The batch is uncommitted, with no staging,
delivery, publication, history rewriting or target/tool-store cleanup.

`make candid-boundary-check` now runs once in CI/release before Clippy. All **324
Rust files** are covered, including the explicitly selected inventory target
module, which sits beneath a normally pruned directory name. Its **36 exact
allowances** cover nine bounded management projections, one management encoder
and 26 external wire fixtures. Domain/persisted records and owned application
payloads receive no exemption. Macro expansion/type resolution remain separate.

Cleanup removes **183** superseded audit reports/handoff archives (**3,407,885
bytes**) from the maintained tree. [The compact retention summary](../reports/README.md)
preserves conclusions and links exact original Git sources. The adoption guide is
139 lines; historical provenance pointers now identify their original revision.
Root source/license provenance, product qualification records, contract vectors,
the release receipt, journals/artifacts, failed bundles and prior ignored evidence
are retained. Historical adoption records change only their reference/scope
guidance. No hash/summary claims that absent raw logs still exist.

## Focused qualification

Source/mode verification and offline integrity pass for all 157 payloads. Actual
consumer tooling and hook checks pass, including CI/Make Candid selection,
unreviewed application rejection, pending documentation deletion and new broken
link refusal. Dependency gates, grouped formatting, shell/Perl syntax and lint,
local links, LOC table/JSON and diff checks pass. Required roster reads/writes
fail on IO errors. Current evidence is `target/evidence/shared-0304.x_mz38e6/` and
[the adoption review](../shared-tooling-review.json).

All 324 Rust file hashes, categories and counts match prior qualified source;
LOC is 15,187 library/app, 39,373 tests, 1,186 examples and zero unclassified.
No Rust, dependency, compiler, release-receipt or backup/recovery behavior changes
are selected. Full product/release tests were not rerun for this tooling batch.
Linux tooling execution does not establish native macOS, installed IC or Canic
acceptance. No service, schedule, candidate projection or stage reuse is enabled.

## Delivered baseline

Both crates **0.58.3** are published. Release
`b1c16b05ed601b0ad516abc7581bcf6037903b2e`, annotated `v0.58.3` and handoff
`f241568` are in `origin/main`; original source is
`974fd8cb96aaf4df81f682781aa0a4ce2dfdecaa`. The [receipt](../release.json)
binds the original complete Linux release gate: strict Clippy, 554 tests,
independent workflows, docs, tooling and packaging. Registry checksums, clean VCS
identities and 341 source files were independently verified. Proof stays in
`target/deploy-0583.cl8y8lzv/` and `.git/release-state/`.

Production Platform **0.23.0**, Host Artifacts/FS/Process and Build **0.74.4**,
Testkit **0.86.0**, PocketIC **16.1.0**, Types **0.18.1**, target-only Runtime
**0.19.2** and Candid **0.10.40** remain selected. Testkit retains Platform 0.22.3;
normal consumers exclude it and all simulator/runtime/WAT packages. Rust minimum
and exact compiler remain **1.99.1**. Source/codec and independent consumer proof
remains at `target/ic-deps-adoption.sq2a3wsj/` and its original Git revision.

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
