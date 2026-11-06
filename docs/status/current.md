# Current handoff — 2026-11-06

## Delivered 0.58.10 through Shared release batches

Both **ic-backup 0.58.10** and **ic-backup-agent 0.58.10** are published on crates.io.
Release `b1aa039c68621af7c89570c78c1f0e38f601bed2` and annotated
`backup-v0.58.10` are on `origin/main`; qualified source is
`8dd5949bb438e5f69bbb8afc14626b6553281916`. Registry readback confirms non-yanked
versions, matching checksums/VCS provenance and all **330 packaged Rust files**
matching the release commit. Remote refs matched before the documentation handoff.

Shared **0.31.0** at `74313941af2fce4399bec0e6909ca6229673c3f8` supplies 160
canonical payloads and the integrated checksum fixes. Schema-2 batch
**`batch-DM8XPVj6`** captured the source, ran the complete gate, committed/tagged,
atomically pushed and invoked its saved two-crate publication hook. The initial
preflight refused a busy shared tool-store lock before validation/preparation;
after its owner released it, exact batch resume completed on the original source.
The normal checkout was fast-forwarded through Shared's reconciliation command.

The full Linux gate passes strict all-target/all-feature Clippy, **544 tests**,
documentation, tooling/release/hook regressions, independent normal consumers and
archive verification. Post-publication cache preparation and registry-only
authenticated capture/restore, application acceptance and effect-free terminal
replay also pass against the exact published pair. The one online setup case stays
ignored in normal tests. Native macOS, Canic acceptance and hosted CI are separate.

Qualified selections are Platform **0.23.4**, Platform Types **0.18.2**, Testkit
**0.89.3** with its exact CLI, Host/Build **0.74.4** and PocketIC **16.1.0**.
Dependency archive checksums match the lock; Platform `31649c1` has unchanged
production source from 0.23.3, and Testkit `5d9cdbf` changes pool lifecycle behavior
unused by Backup. Minimum/pinned Rust **1.99.1**, product APIs, v1 bytes/hashes and
explicit server/application custody remain unchanged.

Hooks and docs use `RELEASE_GROUPS`, `RELEASE_PUBLISH_TARGET`, `RELEASE_SETUP_TARGET`,
`RELEASE_SETUP_MODE` and `RELEASE_BATCH_ACTIVE`; retired settings refuse even empty.
`run-release-batch.pl`/`release-batches.md` replace their old names. Schema-2 batch
journals require the recorded publication commit, with no HEAD inference. There
are no real pre-0.31 batch journals here; finish any external older batch with its original
tooling rather than rewriting schemas. Recovery storage paths/source pins stay.
Backup now enables the shared frozen workflow for one coordinated `backup` group.
Workspace-version inheritance and the root changelog remain; the root catalog now
includes both members. New tags/plans carry the `backup-` prefix. Historical `v…`
recovery explicitly selects an empty group in its original qualified checkout.
Candidate setup prepares tools before version lookup; recovery checks them offline.
Exact v1 receipts and mandatory resumed-validation checks remain unchanged.
Publication is explicit and requires the exact delivered commit/tag/receipt.

Delivery logs, registry artifacts/provenance, refs and reconciliation evidence are
in `target/evidence/deploy-05810/`. The managed checkout remains
`target/release-frozen/batch-DM8XPVj6/`; its Git store, exact validation/plan and
publication receipts remain under `.git/release-state/frozen/batch-DM8XPVj6/`.
The batch is complete and reconciled, **not retired**. Preserve its source pin,
records, artifacts and failure evidence until explicit safe disposition.
Registry consumer evidence is beneath that checkout's `target/tmp/`, in
`ic-backup-agent-independent-build-2023157-0/` and
`ic-backup-agent-pocketic-2023157-1/`. [The snapshot review](../shared-tooling-review.json)
retains focused adoption evidence in `target/evidence/shared-031/`; earlier
adoption, cache/setup and pending-deletion fixture failures remain retained.

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
