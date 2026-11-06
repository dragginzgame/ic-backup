# Current handoff — 2026-11-06

## Pending 0.58.7 cleanup

The follow-up audit over `8be4f62` has eight implemented simplifications: shared
native download setup/accounting assertions, one standalone phase/stage catalog,
checksum-bearing artifact progress, paired download requests/plan, resolved fixture
command/fault configuration, shared checkpoint finishing and parent-synced native
reply retention, and Host file-identity comparison. Public paths/types, error causes,
JSON fields/order, digest framing, attempts and effect/replay boundaries remain.
No tests were removed or product modes added. Package versions remain 0.58.6;
deployment to `origin/main` and crates.io is now authorized.

Two audit items intentionally stay: settlement already shares operation identity;
its remaining status/readback checks have distinct errors, so another comparison
abstraction adds machinery without a simpler owner. Public planning-error merging
remains deferred to an intentional API break. All ten audit items are accounted for.

Evidence in `target/evidence/cleanup-0587/` passes **176 distinct selected tests**:
80 download/model/persistence cases, 41 execution-workflow cases, ten metadata and
four restore cases, 22 Core simulator cases, 17 independent-consumer cases and two
stage-binding cases. The explicit online registry setup remains ignored. Exact
four-state JSON bytes and existing independent digest goldens pass; strict Core
library/tests and Agent example/tests Clippy pass. Earlier mechanical compile
failure and warning output remain retained. Full CI/release, other-host and Canic
qualification were not rerun during focused cleanup.

Follow-up artifact verification now shares guarded plan/journal admission without
moving domain policy or changing opening/closing check order. Restore references
use the same phase catalog as stage paths. A later concurrent dependency edit
selected flate2 **1.1.11**; it is preserved. All **176 selected tests**, both strict
Clippy selections, formatting and documentation checks pass again against this
current lockfile. Before/after SHA-256 confirms the lockfile stayed unchanged.
Evidence is in `target/evidence/cleanup-0587-followup/`, with the independent
consumer graph/build at `target/tmp/ic-backup-agent-independent-build-308335-0/`.
This refresh supersedes the earlier focused selection for current-lock readiness;
the authorized deployment now requires the full release gate; other-host and
Canic qualification remain separate.

## Delivered 0.58.6

Both **ic-backup 0.58.6** and **ic-backup-agent 0.58.6** are published on crates.io.
Release `ca37252a55dee1f535ab9656a0ba273a7e7131d4` and annotated `v0.58.6` are on
`origin/main`; qualified source is `9806ae163e830267da527c5abeb5735f0ea2f396`.
Registry readback confirms non-yanked versions, matching archive checksums and
VCS provenance, and all **330 packaged Rust files** match the release commit.
The later handoff commit changes documentation only.

The complete Linux `release-verify` gate passes: strict all-target/all-feature
Clippy, **544 tests**, documentation, tooling/release/hook regressions, independent
consumers and packaging. One online registry-preparation test is deliberately
ignored in the normal gate; it passes explicitly after publication. The registry-only
normal consumer then passes authenticated capture/restore, application acceptance
and effect-free terminal replay against the published packages. Native macOS and
Canic acceptance remain separate; hosted CI was not manually dispatched.

[Issue #78](https://github.com/dragginzgame/ic-backup/issues/78) is closed. Its
management encoder allowance includes the return type: installed Shared `2aa41c7`
accepted the old prefix, but committed checker `be377103` treats the argument's
`impl` as an inherent implementation and requires the complete header. Both
checkers pass all 330 selected files and reject extra owned Candid declarations,
including in the explicitly selected inventory `target` file. Rust wire code and
the Shared 0.30.4 snapshot remain unchanged.

Testkit **0.88.0** replaces 0.87.0; its checksum and all 51 published source files
match `0bc2cc4`. Its removed APIs require no Backup migration and remove ten unused
lockfile packages. Platform remains **0.23.2**, PocketIC **16.1.0**, and the finished
lockfile includes `cc` **1.11.1**. The earlier release attempt stopped after detecting
a concurrent lock edit; no release effects occurred. The successful retry reran the
entire gate against the finished inputs. Original attempts remain retained.

The cleanup shares cancellation/provider and HTTP retention test helpers, separates
upload fixture ownership, extracts standalone capture assembly, and consolidates
restore preparation within held source scopes. Fresh checks after application
callbacks and across effects remain. Stage bindings enforce the existing 512 KiB
bound with exact-sequence and malformed/oversized regressions. Registry cache setup
is explicit; ordinary consumer tests remain offline. Canonical contracts own repeated
workflow documentation. Public planning-error consolidation remains deferred.

Evidence is in `target/evidence/issue-78/`, including release/publication logs,
registry archives, provenance and remote-ref verification. Release receipts and
publication attempts remain under `.git/release-state/`. Registry consumer evidence
is in `target/tmp/ic-backup-agent-independent-build-468527-0/` and
`target/tmp/ic-backup-agent-pocketic-468527-1/`. Earlier focused cleanup evidence is
in `target/evidence/cleanup-0586/` and `target/evidence/cleanup-0586-followup/`.
Shared counts at this cleanup are 38,160 test, 15,187 production and 1,211 example
lines: net +26 test/+25 example against 0.58.5, with no production LOC change.

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
