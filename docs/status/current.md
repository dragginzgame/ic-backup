# Current handoff — 2026-11-06

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

Shared **0.30.7** at `22a853e26c05ec323f06c57fa316c08b808f252b` replaces 0.30.6:
160 canonical payloads, six updates/two additions/152 unchanged. GitHub main
independently selected that commit; [the review](../shared-tooling-review.json)
records decisions and provenance. AGENTS references the selected baseline;
host/Cargo/IC catalogs and minimum/pinned Rust **1.99.1** remain unchanged.

Backup retains its qualified normal-checkout, full-gate-only release path, with
no stage reuse or selective implementation mapping. Nonempty `RELEASE_FROZEN_GROUPS`
refuses through release/resume/plan/publication entrypoints before effects, including
inherited active selections. No frozen delivery checkout was created or qualified.
CI already omits the duplicate root check after Clippy; independent consumer and
archive checks own distinct contracts and remain. No elapsed-time saving is claimed.

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

## Delivered 0.58.8

Release `78a0fa3dbf9e34b96e872ed7811ddf6a058c9e4c` published both 0.58.8 crates from
source `cbddde14ecb1e7b59671e34ff0c28b9e0a99962c`, with 544 full-gate tests and
registry-only authenticated backup/restore qualification. It adopted Shared 0.30.6,
Platform Types 0.18.2 and Testkit 0.89.0, enforcing Candid/CBOR source boundaries in
root and generated consumers. All 36 external Candid allowances remain live; there
is no CBOR exception or format migration. Explicit server output, HTTP evidence,
original journals and async cleanup retain their owners. Detailed qualification is
retained at `ab19b40`, with delivery evidence in `target/evidence/deploy-0588.vRuWtx/`.
Earlier focused checks remain under `target/evidence/shared-0306.TCumgu/` and
`target/evidence/types-0182-testkit-089.rHqTdd/`.

## Delivered 0.58.7

Release `4774dc7343272b0ae8e55f5ab998c9a855384b54` published both 0.58.7 crates from
source `a3d520c2cfb768b4d10f2b4a51058ca6f9a00349`, with 544 full-gate tests and
registry-only authenticated backup/restore qualification. It consolidated artifact
progress, paired download plans, standalone catalogs, fixture faults, checkpoint
finishing and artifact-verification admission while preserving v1 bytes and recovery.
Evidence remains in `target/evidence/deploy-0587.yiDOdu/`,
`target/evidence/cleanup-0587/` and `target/evidence/cleanup-0587-followup/`; the
full handoff is retained at `95d39fc`. Public planning-error consolidation remains
deferred to an intentional API break; distinct settlement errors keep their owner.

## Delivered 0.58.6

Release `ca37252a55dee1f535ab9656a0ba273a7e7131d4` published both 0.58.6 crates
from qualified source `9806ae163e830267da527c5abeb5735f0ea2f396`, with 544 full-gate
tests and registry-only authenticated backup/restore qualification. It adopted
Testkit 0.88.0 and closed [#78](https://github.com/dragginzgame/ic-backup/issues/78).
Its declaration allowance passes both installed Shared `2aa41c7` and committed
checker `be377103`; extra owned Candid declarations still reject, including under
the explicitly selected inventory `target` file. Evidence remains in
`target/evidence/issue-78/`; the detailed handoff is retained at `8be4f62`.

## Prior delivery and qualification

0.58.5 release `210a517c4dd88d7cd922e4e754e530101d183db3` published both crates from
source `176467c026babbd397f22e8a375563d3e1eeae7e`, with 542 full-gate tests and a
published-consumer capture/restore journey. Evidence remains in
`target/evidence/deploy-0585.b88i271n/` and `.git/release-state/`.
The test consolidation, case mappings and independent qualification remain in
`target/evidence/test-cleanup-0585/`, `target/evidence/test-convergence-0585/` and
`target/evidence/test-trim-0585-03/`; the detailed handoff is retained at `8be4f62`.
[Coverage ownership](../development.md#simulator-coverage-ownership) identifies
retained IC effects, signed transport probes and native rejection matrices.
Earlier release/compiler qualification remains in Git and its original evidence.
Historical Shared 0.30.4 at `2aa41c7394300302fe802b69b31b855bdc89b671` supplied
157 payloads; its adoption review remains in the prior Git handoff.

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
