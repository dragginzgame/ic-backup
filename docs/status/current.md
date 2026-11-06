# Current handoff — 2026-11-06

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

## Delivered 0.58.5

Both **ic-backup 0.58.5** and **ic-backup-agent 0.58.5** are published on crates.io.
Release `210a517c4dd88d7cd922e4e754e530101d183db3` and annotated `v0.58.5` are on
`origin/main`; qualified source is `176467c026babbd397f22e8a375563d3e1eeae7e`.
The standard runner committed, tagged and atomically pushed, then `make publish`
verified and uploaded both packages. Independent registry readback confirms both
versions are non-yanked, archive checksums match, VCS provenance selects the clean
release commit, and **342 packaged source files** match that commit byte for byte.
Remote branch/tag identities match; the later handoff commit changes documentation only.

The [release receipt](../release.json) binds the complete Linux `release-verify`
gate: strict all-target/all-feature Clippy, **542 tests**, independent consumers,
documentation, tooling and packaging. After publication, the registry-only normal
consumer also passes complete authenticated capture/restore, acceptance and
terminal replay against the exact published packages. Its first offline attempt
stopped before runtime because the packages were not cached; fetching the exact
consumer lockfile admitted the successful rerun. Original attempts are retained.

Delivery and registry proof are in `target/evidence/deploy-0585.b88i271n/` and
`.git/release-state/`. Registry consumer graph/build and simulator evidence are in
`target/tmp/ic-backup-agent-independent-build-1984251-0/` and
`target/tmp/ic-backup-agent-pocketic-1984251-1/`. Native macOS, installed IC and
Canic acceptance remain separate; hosted CI was not manually dispatched.

## Released test consolidation and selected dependencies

Shared's count is **38,134 test lines**, down **1,417** from 39,551 at baseline
`b70084f`; production remains **15,187**. Public APIs, product logic, stored formats
and standalone behavior remain. Native suites share workflow providers,
identity-fault assertions, artifact fixtures and immutable IO checks. Eight model
suites share explicit schema-field checks; duplicate-version regressions retain
both the original first-serialized-version case and explicit root duplication.

The independent standalone consumer owns the complete upload/restore journey,
including a known stored Wasm chunk, a cleared live store, changed live state,
complete byte/state verification, acceptance and terminal replay. First and second
lost uploads have separate cold-process checks; successors follow stage order.
Twelve superseded Core cases and the overlapping direct Agent round trip are
retired after transferring distinct regression coverage. All **22 retained Core
simulator-target cases** and **37 Agent example cases** pass the release gate.
Core capture/download, retained-prefix and explicit reconciliation probes remain;
native matrices cover malformed second-upload and start replies. Five direct
Agent probes retain signed ingress, trust rejection and passive upload/readback
proof. [Coverage ownership](../development.md#simulator-coverage-ownership)
distinguishes native validation/order proof from actual IC effects.

Original source snapshots, case mappings, counts and focused qualification remain
in `target/evidence/test-cleanup-0585/`,
`target/evidence/test-convergence-0585/` and
`target/evidence/test-trim-0585-03/`. Earlier mechanical and sandbox failures remain
in their original evidence; authenticated simulator runs used loopback networking.

The qualified lockfile selects Platform **0.23.2**, checksum/source-verified at
`81b3f05f53ed113d02a671d47d13f1a36222e845`. Its additive create-canister DTO/protocol
exports are unused by Backup. Testkit **0.87.0**, authenticated PocketIC **16.1.0**,
Host/Build **0.74.4**, Types **0.18.1**, external Principal **0.1.5**, management
DTO **0.11.0**, Agent **0.49.3**, Candid **0.10.40** and Wasm-only Runtime **0.19.2**
remain selected. Rust minimum and exact development/release compiler are **1.99.1**.
Shared **0.30.4** at `2aa41c7394300302fe802b69b31b855bdc89b671` retains its 157
exact payloads; adoption review is [recorded separately](../shared-tooling-review.json).

Prior 0.58.4 release `c43c196582ccc8249263c6a01dfee8b8ac70c748`, its receipt and
qualification remain in Git and `target/evidence/testkit-deploy-0584.yt3b8nrm/`.
Earlier async Testkit migration and historical report consolidation are recorded
at `b70084f` and in [retained report conclusions](../reports/README.md).
No historical evidence is relabelled or removed.

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
