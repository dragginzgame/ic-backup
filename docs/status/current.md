# Current handoff — 2026-11-07

## Pending Testkit and Shared Tooling adoption

Testkit **0.90.0** from `c03161b9f1395c343756fda6b8721e957798c3df` is selected
through the root catalog/registry lock. Both managed builders pass explicit
60-second request budgets; 120-second startup bounds, original custody and
same-ID restore semantics remain. Other incoming package selections are preserved;
three unused packages and obsolete dependency edges leave the graph.

Shared **0.31.3** at `d2a2f395bc06b22cb3f18afc7d2978a150726360` exports 160
payloads: three updates, 157 unchanged. Provenance, actual Make checksum-NUL
refusal, dependency admission, fresh-parent LOC, Testkit routing and CI-evidence
callers pass. Four local override-aware scripts create their scratch parents.
The exact Testkit CLI/server is prepared and admitted offline. Strict Clippy
passes for Core's PocketIC target and Agent's standalone example on Rust 1.99.1.
Five installed acceptance cases pass: three Core capture/load-recovery cases and
two Agent cases covering pending authority and independent normal-consumer
capture/restore/acceptance/terminal replay. Evidence is in
`target/evidence/adopt-testkit090-shared0313/`, including exact registry archive
checksum/VCS provenance. [#79](https://github.com/dragginzgame/ic-backup/issues/79)
meets its migration and focused Linux acceptance; leave it open until these
uncommitted changes land. Native macOS and full release qualification remain
separate. No delivery or issue-state change has occurred.

## Pending cleanup

Tests share settlement faults, workflow declarations, native replay setup,
completed upload sources and positive single-operation plans. Original-preserving
filesystem replacements remain distinct from in-place corruption; decoding,
coverage, spent-attempt and recovery assertions remain independent.

The standalone example delegates private evidence writes to Host FS's retained
writer, preserving partial entries and native I/O errors. Test support reuses it.
Endpoint admission owns its rejection matrix with one public-constructor case;
the copied Host Process fragmented-capture regression is retired while required
Content-Length and retained HTTP diagnostics remain.

Restore-target, fence/context and controller-set admission lives with its concrete
owner. Three private helper modules and their generic callbacks/intermediate
errors are removed, preserving validation order and public errors. Shared principal
parsing and distinct JSON/Candid visitors remain.

Guides link canonical coordination/settlement contracts and separate current
Platform, Agent and simulator behavior from exact-revision history. Simulator
library admission is distinct from the CLI/tool check. Retired fence recovery and
original evidence remain. The release fixture map separates frozen delivery from
historical adapters and retains its original review/hash.

Focused Linux checks use Rust **1.99.1**:

| Batch | Passing checks | Evidence beneath `target/evidence/` |
| --- | --- | --- |
| Core tests/contracts | 29 selected tests; strict Clippy for Core library and metadata/data targets | `cleanup-20261107/` |
| Artifact fixtures | 13 upload/source/restore-source tests | `artifact-cleanup-20261107/` |
| Agent retention/admission | 18 endpoint, retention, HTTP and independent normal-consumer journey tests; strict Clippy for Agent library and standalone example | `agent-cleanup-20261107/` |
| Restore/controller localization | 30 controller, restore, lifecycle codec and retained-evidence tests; strict all-feature Core library Clippy | `restore-localization-20261107/` |
| Public plan fixtures/simulator guide | Both operation-plan and restore-safety integration tests; strict all-feature Clippy for those targets | `plan-fixture-cleanup-20261107/` |

Formatting, edited Markdown links, JSON and diff checks pass. The initial Agent
HTTP check's sandbox loopback denial is retained; its local-network retry passes.
The independent consumer exercised actual managed PocketIC, normal dependencies,
capture/restore, application acceptance and effect-free terminal replay.

Cleanup preserved its incoming lockfile byte-for-byte and its focused receipts
used Platform **0.23.5** and Testkit **0.89.4**. The adoption above has its own
evidence. Core/Agent APIs
and v1 bytes/hashes are unchanged; the example writer is the maintained behavior
change. Native macOS, current Canic acceptance and full release qualification
remain separate. Cleanup has caused no live IC or publication effect.

## Delivered 0.58.10 through Shared release batches

Both **ic-backup 0.58.10** and **ic-backup-agent 0.58.10** are published. Release
`b1aa039c68621af7c89570c78c1f0e38f601bed2` and annotated `backup-v0.58.10` were
verified on `origin/main`; qualified source is
`8dd5949bb438e5f69bbb8afc14626b6553281916`. Registry readback verified checksums,
VCS provenance and all **330 packaged Rust files** against the release commit.

Shared **0.31.0** at `74313941af2fce4399bec0e6909ca6229673c3f8` supplies 160
canonical payloads. Schema-2 batch **`batch-DM8XPVj6`** completed the full gate,
commit/tag, atomic push, publication and normal-checkout reconciliation. Its first
busy tool-store refusal and exact resume evidence remain retained. The batch is
**complete and reconciled, not retired**.

The delivered Linux gate passed strict Clippy, **544 tests**, docs/tooling/release/
hook regressions, independent consumers and archive checks. Registry-only
capture/restore, acceptance and terminal replay also passed. Qualified selections
were Platform **0.23.4**, Types **0.18.2**, Testkit **0.89.3** with its exact CLI,
Host/Build **0.74.4**, PocketIC **16.1.0** and Rust **1.99.1**. Native macOS,
Canic acceptance and hosted CI were separate.

Preserve `target/evidence/deploy-05810/`, the managed checkout
`target/release-frozen/batch-DM8XPVj6/` and durable records beneath
`.git/release-state/frozen/batch-DM8XPVj6/`, including its source pin, exact
validation/plan/publication receipts and failed evidence. Registry consumer roots
beneath the managed checkout's `target/tmp/` are
`ic-backup-agent-independent-build-2023157-0/` and
`ic-backup-agent-pocketic-2023157-1/`. Snapshot review evidence remains in
`target/evidence/shared-031/` and [its reviewed manifest](../shared-tooling-review.json).

The [complete prior handoff](https://github.com/dragginzgame/ic-backup/blob/b1c1c33f1e9d80c2c22a605ed4b39383b281ede7/docs/status/current.md)
retains 0.58.9 and earlier release/source/compiler identities, evidence paths and
release-adoption details. Exact v1 receipts and mandatory resumed-validation
checks remain binding. Historical `v…` recovery uses the original qualified
checkout and explicit empty group. No journals, artifacts or failed bundles were
removed. Public planning-error consolidation remains deferred to an API break;
[coverage ownership](../development.md#simulator-coverage-ownership) remains local.

## Recovery and retained contracts

Preserve standalone host-driven same-release/same-ID backup/restore, mandatory
application admission and fenced restore safety. Persist intent before effects,
reconcile lost replies and preserve original bounded spending, unfinished
journals, artifacts, references and external obligations. Terminal replay is
effect-free.

Layouts containing `consistency-requirement.json` or `fence-obligation.json` refuse
before record use/spending, including malformed bytes and symlinks. Distinct-edge
learned continuations refuse without rewriting bindings. Recover those originals
with **0.57.1 and the original qualified integration before upgrading**. Never
remove markers, rebuild plans, reinterpret payloads, re-sign or reset accounting
to bypass admission. No migration, evidence cleanup or refund is provided. See
[the migration](../canic-adoption.md#058-api-migration) and
[implementation contracts](../implementation-contracts.md).
