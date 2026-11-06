# Current handoff — 2026-11-06

## Delivered 0.58.8

Both **ic-backup 0.58.8** and **ic-backup-agent 0.58.8** are published on crates.io.
Release `78a0fa3dbf9e34b96e872ed7811ddf6a058c9e4c` and annotated `v0.58.8` are on
`origin/main`; qualified source is `cbddde14ecb1e7b59671e34ff0c28b9e0a99962c`.
Registry readback confirms non-yanked versions, matching archive checksums and VCS
provenance, and all **330 packaged Rust files** match the release commit byte for
byte. Remote branch/tag identities match; the later handoff commit is documentation only.

The complete Linux `release-verify` gate passes: strict all-target/all-feature
Clippy, **544 tests**, documentation, tooling/release/hook regressions, independent
consumers and packaging. The one explicit online registry-preparation test stays
ignored in the normal gate and passes separately after publication. The registry-only
normal consumer then passes authenticated capture/restore, application acceptance
and effect-free terminal replay against both exact published packages. Native macOS
and Canic acceptance remain separate; hosted CI was not manually dispatched.

Shared **0.30.6** at `45c0295e23d3d23225e31f47c018c30ff0c75581` supplies **158**
canonical payloads. [The adoption review](../shared-tooling-review.json) owns the
pinned source, decisions and focused evidence; the supplier's uncommitted 0.30.7
was excluded. The existing source gate enforces Candid/CBOR ownership across all
330 Rust files, explicitly including inventory `target`; all 36 external Candid
allowances remain live, with no CBOR exception or product-format migration.
Generated consumers receive Cargo boundary admission before compilation; generated
MSRV API probes get their own source check. There are no maintained browser roots
or alternate manifest templates. Checks are lexical, not semantic authentication.

The qualified lockfile selects Platform **0.23.2**, Platform Types **0.18.2**,
Testkit **0.89.0**, its exact CLI and PocketIC **16.1.0**. Rust minimum and pinned
compiler remain **1.99.1**. Types and Testkit archive checksums and published sources
match their VCS commits `5eeb016` and `094d144`. Types changes dependency features
and Principal lint annotations; canonical identity/byte goldens pass. Testkit removes
automatic client logging, unused by Backup, and fifteen transitive packages leave
the lockfile. Explicit server output, HTTP evidence, original journals and async
cleanup retain their owners. No product Rust code or public Backup API changed.

Delivery logs, registry archives/provenance, verification scripts and remote refs
are in `target/evidence/deploy-0588.vRuWtx/`; receipts/publication attempts remain
under `.git/release-state/` and [the release receipt](../release.json). Published
consumer graph/build evidence is in
`target/tmp/ic-backup-agent-independent-build-874235-0/` and authenticated runtime
evidence in `target/tmp/ic-backup-agent-pocketic-874235-1/`.
Earlier focused checks and corrected mechanical failures remain under
`target/evidence/shared-0306.TCumgu/` and
`target/evidence/types-0182-testkit-089.rHqTdd/`; no retained evidence was removed.

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
