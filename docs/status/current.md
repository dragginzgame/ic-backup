# Current handoff — 2026-11-06

## Pending 0.58.10 frozen delivery

Shared **0.31.0** at `74313941af2fce4399bec0e6909ca6229673c3f8` replaces the
uncommitted 0.30.10 snapshot: 160 canonical payloads, nine updates/two additions/
two retirements/149 unchanged. It includes the integrated checksum fixes.
Hooks and docs use `RELEASE_GROUPS`, `RELEASE_PUBLISH_TARGET`, `RELEASE_SETUP_TARGET`,
`RELEASE_SETUP_MODE` and `RELEASE_BATCH_ACTIVE`; retired settings refuse even empty.
`run-release-batch.pl`/`release-batches.md` replace their old names. Schema-2 batch
journals require the recorded publication commit, with no HEAD inference. There
are no real pre-0.31 batch journals here; finish any older batch with its original
tooling rather than rewriting schemas. Recovery storage paths/source pins stay.
Backup now enables the shared frozen workflow for one coordinated `backup` group.
Workspace-version inheritance and the root changelog remain; the root catalog now
includes both members. New tags/plans carry the `backup-` prefix. Historical `v…`
recovery explicitly selects an empty group in its original qualified checkout.
Candidate setup prepares tools before version lookup; recovery checks them offline.
Exact v1 receipts and mandatory resumed-validation checks remain unchanged.
Publication is explicit and requires the exact delivered commit/tag/receipt.

[The review](../shared-tooling-review.json) records provenance, consumer decisions
and latest qualification in `target/evidence/shared-031/`. Real installed setup,
locked native build, archive-consumer and local Git integration passes in
`target/frozen-delivery-tests.L8NVJc/`: schema 2/recorded commit, deferred lock custody,
exact offline recovery, uncertain upload refusal, completed replay, reconciliation
and explicit retirement. Only upload/lost reply are substituted; required fixture
receipts remain outside its retired batch. The complete historical adapter suite
passes in `target/shared-release-tests.hlWtN8/`; tooling proves cleared-roster,
retired-setting and checksum-NUL refusal. Syntax, formatting, compiler, dependency
and documentation gates pass; annotated provenance matches the supplier commit.
The fixture captures Platform **0.23.4**/Testkit **0.89.3**. Manifest, lockfile and
compiler hashes are unchanged by this refresh. Earlier qualification, setup/cache
failures and the corrected pending-deletion fixture failure remain retained.
No complete new product gate, live upload, macOS or Canic acceptance was run;
the current dependencies still need the complete gate before release.
Rust **1.99.1**, package versions **0.58.9**, product Rust code,
protocols and retained recovery evidence remain unchanged. Changes are uncommitted.
No primary-repository frozen batch, release refs or publication was created.

## Delivered 0.58.9

Both **ic-backup 0.58.9** and **ic-backup-agent 0.58.9** are published on crates.io.
Release `478602d53b8043ec6cdfb92e030269fe82102224` and annotated `v0.58.9` are on
`origin/main`; qualified source is `b9327a33e9459505991bac3146b94f5e0438575c`.
Registry readback confirms non-yanked versions, matching archive checksums and VCS
provenance; all **330 packaged Rust files** match the release commit byte for byte.
Remote branch/tag identities match; the subsequent handoff commit is documentation only.

The complete Linux `release-verify` gate passes: strict all-target/all-feature
Clippy, **544 tests**, documentation, tooling/release/hook regressions, independent
consumers and packaging. Explicit registry cache preparation passes separately;
the registry-only normal consumer then passes authenticated capture/restore,
application acceptance and effect-free terminal replay against both exact published
packages. Native macOS and Canic acceptance remain separate; hosted CI was not
manually dispatched.

Delivered 0.58.9 used Shared **0.30.7**, the normal-checkout release path and
Rust **1.99.1**. Detailed historical decisions remain at `806965f`. Current batch
adoption above retains the full gate without stage reuse or selective mapping;
independent consumer/archive proofs remain distinct. No timing saving is claimed.

The qualified lockfile selects Platform **0.23.3**, Platform Types **0.18.2**,
Testkit **0.89.1**, its exact CLI and PocketIC **16.1.0**. Published archive checksums
match the lockfile; source review binds Platform `e71e8a6` and Testkit `7d7ffbf`.
Platform's production changes concern the unused ledger adapter and dependency
feature declarations; Testkit's library source is unchanged. Product Rust source,
public APIs, records, IC protocols and custody requirements remain unchanged.

Delivery logs, registry archives, byte/provenance verification and remote refs are
in `target/evidence/deploy-0589/`; receipts/publication attempts remain under
`.git/release-state/` and [the release receipt](../release.json). Registry-only
graph/build evidence is in `target/tmp/ic-backup-agent-independent-build-41051-0/`
and authenticated runtime evidence in `target/tmp/ic-backup-agent-pocketic-41051-1/`.
Focused snapshot provenance remains in `target/evidence/shared-0307.qztZdV/`;
full-gate release fixtures remain in `target/shared-release-tests.6XoOya/`.
No journals, artifacts, failed bundles or earlier release evidence were removed.

## Earlier delivery and qualification

Full prior handoffs remain at `ab19b40` (0.58.8), `95d39fc` (0.58.7) and
`8be4f62` (0.58.6/0.58.5). Registry journeys, exact source/ref identities and
compiler qualification remain in Git and their original evidence directories.
Delivery evidence stays under `target/evidence/deploy-0588.vRuWtx/`,
`target/evidence/deploy-0587.yiDOdu/` and `target/evidence/deploy-0585.b88i271n/`;
issue-78 qualification remains under `target/evidence/issue-78/`.
Test consolidation/case mappings and cleanup evidence remain in their original
directories. All 36 external Candid allowances remain live; there is no CBOR
exception or product-format migration. Public planning-error consolidation remains
deferred to an intentional API break; distinct settlement errors retain their owner.
[Coverage ownership](../development.md#simulator-coverage-ownership) identifies
retained IC effects, signed transport probes and native rejection matrices.

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
