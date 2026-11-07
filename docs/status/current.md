# Current handoff — 2026-11-07

## Delivered 0.58.11

Both **ic-backup 0.58.11** and **ic-backup-agent 0.58.11** are published on
crates.io. Release `e6471464406e97d77b1c4b610dd2a5c22991f27c` and annotated
`backup-v0.58.11` were verified on `origin/main`; qualified source is
`16527d4b5cd97f5f699b80f125ca5ecd1290b1c1`. Registry readback confirms non-yanked
versions, matching archive checksums and Git provenance. All **330 packaged Rust
files** match the release commit: 285 Core and 45 Agent files.

The full Linux gate passes strict all-target/all-feature Clippy, **544 tests**,
documentation, tooling/release/hook regressions, independent normal consumers and
archive checks. The one online cache-preparation case remains ignored in the normal
gate. Explicit post-publication preparation and the registry-only normal consumer
then pass authenticated capture/restore, application acceptance and effect-free
terminal replay against the exact published pair. Native macOS, current Canic
acceptance and hosted CI remain separate from this local qualification.

Qualified selections are Platform **0.23.5**, Types **0.18.2**, Testkit **0.90.0**
with its exact CLI, Host/Build **0.74.4**, Agent SDK **0.49.3**, PocketIC **16.1.0**
and minimum/pinned Rust **1.99.1**. Shared **0.31.3** at
`d2a2f395bc06b22cb3f18afc7d2978a150726360` supplies 160 canonical payloads.
Testkit's archive/VCS provenance binds `c03161b9f1395c343756fda6b8721e957798c3df`.
Both managed builders supply explicit 60-second request budgets, retaining separate
120-second startup bounds and original ownership/recovery. [#79](https://github.com/dragginzgame/ic-backup/issues/79)
is **closed as completed** after publication and independent registry acceptance.
No live IC or provider operations occurred.

## Change scope and evidence

Admission now lives with concrete restore/controller owners; public errors,
validation order and v1 bytes/hashes remain unchanged. The standalone example uses
Host FS's retained writer, preserving partial entries, create-only custody and
native errors. Tests share positive plans, settlement faults, completed sources and
replay setup while retaining distinct corruption, coverage and recovery checks.
Guides separate current contracts from exact-revision history and preserve 0.57.1
recovery. Local scratch owners create missing parents; tooling fixtures isolate
enclosing release/publication settings. [The changelog](../../CHANGELOG.md) records
the delivered outcomes; [the tooling review](../shared-tooling-review.json) retains
focused adoption and later delivery as separate proofs.

Delivery, registry metadata/archives, source comparisons, refs, reconciliation and
post-publication logs are under `target/evidence/deploy-05811/`. Registry-only
graph/build and runtime roots beneath the managed checkout's `target/tmp/` are
`ic-backup-agent-independent-build-2256280-0/` and
`ic-backup-agent-pocketic-2256280-1/`; explicit setup is
`ic-backup-agent-registry-prepare-2252982-0/`.

Batch **`batch-_jjlBLUV`** is complete and reconciled, **not retired**. Preserve
`target/release-frozen/batch-_jjlBLUV/`, its source pin and durable Git/plan/
validation/publication records under `.git/release-state/frozen/batch-_jjlBLUV/`.
Normal source was fast-forwarded through Shared's reconciliation command; release
metadata and dependency selections agree with the delivered commit.

Earlier failed batches remain intact: `batch-DWdrLMkj` caught four missing
semicolons in artifact unit tests; `batch-7uUMU2LC` caught publication-context leakage
into tooling fixtures. Both failed before version preparation or publication.
Their logs and captured sources remain retained; the corrected source passed the
complete gate. No real recovery state or failed bundle was cleaned up.

The [prior handoff](https://github.com/dragginzgame/ic-backup/blob/16527d4b5cd97f5f699b80f125ca5ecd1290b1c1/docs/status/current.md)
retains the cleanup/adoption receipts and earlier delivery identities/locations,
including the completed, unretired 0.58.10 batch. Public planning-error consolidation
remains deferred to an intentional API break. [Coverage ownership](../development.md#simulator-coverage-ownership)
stays local; producer passes do not qualify Canic or application safety.

## Recovery and retained contracts

Preserve standalone host-driven same-release/same-ID backup/restore, mandatory
application admission and fenced restore safety. Persist intent before effects,
reconcile lost replies and preserve bounded spending, unfinished journals,
artifacts, references and external obligations. Terminal replay is effect-free.

Layouts containing `consistency-requirement.json` or `fence-obligation.json` refuse
before record use/spending, including malformed bytes and symlinks. Distinct-edge
learned continuations refuse without rewriting bindings. Recover originals with
**0.57.1 and the original qualified integration before upgrading**. Never remove
markers, rebuild plans, reinterpret payloads, re-sign or reset accounting to bypass
admission. No migration, evidence cleanup or refund is provided. See
[the migration](../canic-adoption.md#058-api-migration) and
[implementation contracts](../implementation-contracts.md).
