# Current handoff — 2026-11-06

## Pending 0.58.6 deployment

Source commit `e7854cc` contains the cleanup, Testkit update and #78 fix. The first
release attempt stopped after the release-helper suite passed: concurrent lockfile
edits changed `cc` 1.11.0 to 1.11.1. No version preparation, release tag, push or
publication occurred. Dependency edits are now confirmed finished; retain `cc`
1.11.1 and restart the complete gate. Original logs: `target/evidence/issue-78/release-0586.log`.

[Issue #78](https://github.com/dragginzgame/ic-backup/issues/78) is fixed locally:
the management encoder allowance includes its return type. Installed Shared
`2aa41c7` accepted the old prefix, but committed checker `be377103` treats the
argument's `impl` as an inherent implementation and requires the complete header.
The new prefix is the shortest supported by both; snapshot and Rust source stay
unchanged. Both checkers pass all 330 selected files and reject extra owned Candid
declarations in the encoder file and explicitly selected inventory `target` file.
Focused lexical evidence is in `target/evidence/issue-78/`; no Rust build was needed.

The audit cleanup is implemented over `cb605a8`. Dependencies now select Testkit **0.88.0** for the requested 0.58.6 deployment. Platform remains
**0.23.2** and PocketIC **16.1.0**. Testkit archive checksum and all published
source files match `0bc2cc4`; exact proof and successful setup are retained in
`target/evidence/issue-78/`. Its removed APIs require no Backup source migration.
Snapshot workflow cancellation shares one
exclusion/byte-retention helper and suspended provider. HTTP provider tests share
scenario/setup/postcondition owners. Upload tests separate source setup, planning,
execution and restore admission while preserving all existing Rust cases. The
source-inspected registry adds only one explicitly ignored online preparation case.

The standalone dispatcher delegates capture settlement and shares stage-binding
reads with exact sequence and 512 KiB raw-byte admission. Restore preparation uses
one held source scope for destination, acceptance and load planning. It rechecks
bytes after the application acceptance callback; effects and terminal acceptance
retain their independent admission. Adjacent verification duplicates are removed;
no runtime speed-up is claimed. Registry preparation uses an isolated consumer
lockfile and reports nested graph diagnostics; normal tests stay offline. Repeated
workflow prose links to the canonical implementation contracts.

Focused qualification in `target/evidence/cleanup-0586/` passes: 55 snapshot workflow
cases, nine upload-plan cases, 15 HTTP cases, two authenticated cold-process consumer
journeys, explicit registry preparation, strict Core library/tests and Agent
example/tests Clippy, local tooling integration, snapshot integrity and documentation
links. The original mechanical compile failures remain retained. Full release,
other-host and Canic qualification were not rerun for this pending batch.

Follow-up evidence in `target/evidence/cleanup-0586-followup/` passes two binding
boundary tests, five cancellation tests, all 12 selected normal-consumer scenarios,
strict Agent example/tests Clippy, formatting and documentation links. The consumer
selection includes restore, cold reentry, terminal replay, lost allocation/upload/
load/start replies and application/operator failures. Earlier mechanical failures
remain retained. Cancellation callers no longer repeat their helper's assertion.

Current Shared counts are **38,160 test**, **15,187 production** and **1,211 example**
lines. Against 0.58.5, the setup and boundary regressions give a net increase of 26
test and 25 example lines; this batch improves ownership and admission rather than
providing a net LOC cut. File splitting is organization, not code removal.
Broader workflow rejection matrices remain at
their typed boundaries; the descriptor-dependent upload tests retain their local
fixture ownership. Backup's own consumer-graph and distinct fixture-retention
proofs stay. Public planning-error consolidation remains deferred to an intentional
API compatibility change. Deployment is authorized: commit this batch, run the complete release gate, push
`origin/main` and `v0.58.6`, then publish both crates before closing #78.

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
