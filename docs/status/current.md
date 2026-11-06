# Current handoff — 2026-11-06

## Pending 0.58.8 tooling and dependency adoption

Shared **0.30.6** at `45c0295e23d3d23225e31f47c018c30ff0c75581` replaces 0.30.4:
158 canonical payloads, eleven updates/one addition/146 unchanged. GitHub main
independently selected that commit; the supplier's uncommitted 0.30.7 draft stays
excluded. [The adoption review](../shared-tooling-review.json) owns exact decisions.
AGENTS now references the selected baseline; catalogs and Rust **1.99.1** stay.

The existing source gate now enforces Candid/CBOR ownership across all 330 Rust
files, explicitly including inventory `target`. All 36 external Candid allowances
stay live; no CBOR exception or product-format migration is needed. Materialized
independent Cargo consumers receive the shared dependency gate before compilation;
generated MSRV probes also receive their own empty-policy source check. Standalone
consumer Rust is already covered under `crates`; no browser roots/templates exist.

Focused evidence in `target/evidence/shared-0306.TCumgu/` passes snapshot provenance,
source/dependency/inheritance gates, compiler admission, tooling/hook regressions,
shell/Perl syntax, prepared-tools admission, formatting, both independent library
checks and one complete authenticated standalone consumer journey. Generated
consumer evidence: `target/msrv-consumers.q6KxAu/` and
`target/tmp/ic-backup-agent-independent-build-2026458-0/`. The original fixture
metadata collision failure is retained; corrected checks pass. Full CI/release,
other-host and Canic qualification remain separate; checks are lexical, not semantic.
At tooling-adoption time, the incoming Types **0.18.2** edit was preserved separately;
those evidence hashes bind that earlier selection. The current graph is qualified below.
Deployment is authorized: commit this batch, run the full release gate, push
`origin/main` and `v0.58.8`, then publish both crates. Package versions remain
0.58.7 until the standard runner prepares release metadata.

Platform Types **0.18.2** is now explicitly qualified through Platform **0.23.2**;
no direct Types dependency is added. Testkit advances to **0.89.0**, with its exact
CLI prepared and PocketIC **16.1.0** admitted offline. Both archive checksums and
all published source files match their VCS commits: Types `5eeb016` (12 files) and
Testkit `094d144` (51 files). Types changes normalized dependency feature settings
and Principal lint annotations only; canonical identities/bytes remain qualified.
Testkit removes automatic client logging, unused by Backup; explicit server output,
HTTP evidence, original journals and async cleanup retain their owners. Fifteen
unused transitive packages leave the lockfile; no logging replacement is added.

Evidence in `target/evidence/types-0182-testkit-089.rHqTdd/` passes **188 selected
tests** (144 model, 22 Core simulator, 22 Agent simulator/consumer), strict both-crate
all-target/all-feature Clippy, independent Core/Agent consumer builds, compiler and
dependency/source admission, exact CLI/server checks, formatting and documentation.
The explicit registry-cache test stays ignored. Independent consumer build evidence
is in `target/tmp/ic-backup-agent-independent-build-2674089-0/`; model tests cover
Principal normalization, external conversions, exact Candid and hash goldens.
Input hashes confirm no concurrent manifest/lock/toolchain edits during validation.
The initial byte-identical Types assertion found annotation-only drift; source and
manifest diffs retain the correction. Full release, other-host and Canic qualification
were not rerun. No product Rust code, format or public Backup API change is required.

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
