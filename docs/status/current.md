# Current handoff — 2026-11-06

## Delivered 0.58.7

Both **ic-backup 0.58.7** and **ic-backup-agent 0.58.7** are published on crates.io.
Release `4774dc7343272b0ae8e55f5ab998c9a855384b54` and annotated `v0.58.7` are on
`origin/main`; qualified source is `a3d520c2cfb768b4d10f2b4a51058ca6f9a00349`.
Registry readback confirms non-yanked versions, matching archive checksums and VCS
provenance, and all **330 packaged Rust files** match the release commit byte for
byte. Remote branch/tag identities match; the later handoff commit is documentation only.

The complete Linux `release-verify` gate passes: strict all-target/all-feature
Clippy, **544 tests**, documentation, tooling/release/hook regressions, independent
consumers and packaging. The one explicit online registry-preparation test is
ignored in the normal gate and passes separately after publication. The registry-only
normal consumer then passes authenticated capture/restore, application acceptance
and effect-free terminal replay against both exact published packages. Native macOS
and Canic acceptance remain separate; hosted CI was not manually dispatched.

Artifact progress now keeps checksums in verified/durable states; nonempty download
requests carry their plan. Exact four-state v1 JSON bytes and independent digest
goldens pass. Native tests share download setup/accounting and parent-synced reply
retention. The standalone phase/stage catalog also derives restore-reference paths;
fixture commands resolve action/fault interpretation at admission. Checkpoint
finishing and original-record verification admission share existing owners. Opening/
closing checks, callback revalidation, typed errors, original spending and effect-free
replay remain. No fault cases or public interfaces were removed.

Settlement retains its existing shared operation identity and distinct domain errors;
a further comparison abstraction added no simpler owner. Public planning-error
consolidation remains deferred to an intentional API break.

The qualified lockfile includes flate2 **1.1.11**, Testkit **0.88.0**, Platform
**0.23.2** and PocketIC **16.1.0**; the minimum and pinned compiler remain **1.99.1**.
The later flate2 edit was requalified through all 176 focused selections and then
the complete release gate. No dependency edits occurred during this release.

Delivery logs, registry archives/provenance, verification scripts and remote-ref
proof are in `target/evidence/deploy-0587.yiDOdu/`; receipts/publication attempts
remain under `.git/release-state/` and [the release receipt](../release.json).
Registry-only consumer evidence is in
`target/tmp/ic-backup-agent-independent-build-2516776-0/` and
`target/tmp/ic-backup-agent-pocketic-2516776-1/`. Earlier focused qualification and
mechanical failures remain in `target/evidence/cleanup-0587/` and
`target/evidence/cleanup-0587-followup/`; no retained evidence was removed.

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
Shared 0.30.4 at `2aa41c7394300302fe802b69b31b855bdc89b671` still owns 157 exact
payloads; [the adoption review](../shared-tooling-review.json) remains unchanged.

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
