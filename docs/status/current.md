# Current handoff — 2026-10-19

The current local release is **0.33.0**, commit
`d7921ae` (`v0.33.0`); package versions remain 0.33.0.

## Platform 0.2.1 adoption

Published **Platform 0.2.1** is selected with minimum requirement `0.2.1` and
empty defaults. The incoming lock already selected it, Host 0.30.0 and Testkit
0.55.2; those selections and the lock bytes are preserved. The archive and all
43 unpacked files match the registry checksum; its source revision and checksum
are recorded in the [Platform contract](../ic-platform.md). No sibling source,
Cargo patch or new adapter supplies the dependency.

Core retains `candid` and `management-canister-types`; only Agent adds `agent`.
The used Candid, bounded decoder, Agent and management module sources are unchanged
from 0.1.18. Principal 0.1.5, management DTO 0.11.0, Candid 0.10.38 and SDK 0.49.2
identities remain. Platform 0.2's CDK/runtime changes are outside this host graph.
Mandatory Candid and SDK transitives remain; no direct upstream IC declaration,
parser or CDK-family package is introduced. Testkit still selects development-only
Platform 0.1.18 and Host 0.29.0; PocketIC retains its independent management 0.8 DTOs.

Focused Linux checks pass: 212 IC-related unit cases, twelve public codec cases,
three Agent unit cases, fourteen HTTP boundary cases and all twelve actual Agent
PocketIC cases. Strict Core-library and Agent all-target Clippy, actual Rust 1.91
library/independent-consumer checks and native/Wasm identity probes for all 24
management exports and Principal pass. All six normal/build target trees preserve
Core/Agent isolation without parser, simulator, WAT or CDK reachability. Formatting,
dependency boundaries/inheritance, local links and snapshot integrity pass.

The workspace all-target check still fails at the unchanged Core simulator import:
neither published Platform 0.1.18 nor 0.2.1 exports `RawEffectivePrincipal`.
A standalone 0.2.1 consumer reproduces E0432, reported on
[Platform #27](https://github.com/dragginzgame/ic-platform/issues/27#issuecomment-6272016874).
Publish the original export and adopt it through Testkit before qualifying Core's
exact-routing simulator journey; no routing fallback or copied DTO is added.
Native macOS, installed-IC and full local CI are not qualified by these checks.

The pending notes are **0.34.0** because the incoming Host 0.30 selection changes
the public publication-error identity. Package versions remain 0.33.0. Original
task-entry source, registry evidence, standalone probes and per-command logs remain
at `/tmp/backup-platform-adopt-29kiauql/`; independent consumers remain at
`target/msrv-consumers.amei8S/`. Unrelated pending work and staging are preserved.
No commit, push, release, publication, deployment or artifact cleanup ran.

## Earlier local renderer and fixture consolidation

Both local version-edit commands delegate literal rendering to the reviewed Shared
Tooling renderer. Backup supplies parsed member ownership and retains its numeric
bounds, original lock transformation, receipts, source admission and rollback.
Comments, aliases and dependency tables follow one renderer; requirement operators
and precision are preserved. Failed or empty helper output refuses before either
candidate is installed.

The two single-stage simulator cases now use the existing complete workflow driver
and capture provider. Exact bind/execute counts, pending mutation/exhausted allowance,
unchanged original journals and no-redispatch checks remain. Distinct reconciliation
cases are retained. No product API, stored record or spending contract changes.
These compatible changes are included in the pending **0.34.0** notes alongside
the incoming Host update, without a package-version edit.

All 78 local release-adapter cases pass, including comments, tables, aliases,
Unicode paths and failed/empty renderer output. The actual Make/receipt/rollback
and private-index checks use isolated substitutes; no real Git/release effects run.
Their original cases/logs remain at `target/shared-release-tests.7SVWdG/`.

The incoming Testkit 0.55.2 catalog/lock update is preserved. Its forwarded Platform
0.1.18 surface omits `RawEffectivePrincipal`, so current-graph fixture compilation
stops at the unchanged management-routing import. The published export blocker is
[Platform #27 — Complete PocketIC exports used by Testkit consumers in Backup and Canic](https://github.com/dragginzgame/ic-platform/issues/27).
No direct PocketIC exception, copied DTO, routing fallback or sibling edit is added.
The exact locked Testkit CLI/server has been explicitly prepared and admitted.

An isolated copy of the current fixture source with the released commit's exact
Cargo inputs (Testkit 0.54.1, Platform 0.1.18, Host 0.29.0) passes strict target
Clippy, actual Rust 1.91 compilation and all eleven affected PocketIC cases. Its
physical server bundle matches the admitted original SHA-256. The first isolated
attempt correctly refused a linked tool directory before any IC call; that failure
and its fixtures remain retained. This source qualification does not approve the
pending 0.55.2 graph. Native macOS, installed-IC and full local CI remain separate.

Task-entry source/inputs, isolated source/lock, command logs and failed attempts are
retained at `/tmp/backup-local-followups-xf8fjzza/`. Current formatting, dependency
boundaries/inheritance, local links, snapshot integrity and whitespace checks pass.
Original Cargo inputs and staging are unchanged. No commit, push, release,
publication, deployment or artifact cleanup ran.

## Earlier Host 0.29 and WAT adoption

The incoming catalog and lock already selected published Host Artifacts, FS and
Process **0.29.0**. Their registry checksums and cached source/archive identities
are verified. Preserve the incoming Platform 0.1.18 and Testkit 0.54.1 selections;
Testkit's separate Host 0.27.6 dependencies remain development-only.

Both fixture compilers now call Host Artifacts' `wasm_text::compile` with explicit
64 KiB source/output bounds. Only development edges enable `wasm-text`; the direct
WAT catalog/member declarations are removed. WAT remains Host's private transitive
backend. All three fixture modules retain their exact earlier binary hashes.
The dependency-boundary policy rejects direct WAT, including aliases and
normal/build/development/target-specific declarations. Independent consumers retain
the existing transitive WAT/simulator exclusion.

Host FS 0.29 changes the public Rust identity carried by
`PersistenceError::Publication`; consumers using its Host error type must align
their direct FS dependency to 0.29. Pending notes therefore move from 0.32.1 to
**0.33.0**, incorporating the earlier compatible work. Package versions remain
0.32.0; no staging, commit, release, publication or deployment ran.

Focused Linux checks pass: 538 library cases, five local artifact/recovery cases,
14 HTTP boundary cases and all 50 Core/Agent PocketIC cases. Strict all-target/
all-feature Clippy, actual Rust 1.91 all-target checks and both independent MSRV
consumers pass, including the canonical public Host error identity. All six normal/
build trees for Linux and both macOS targets exclude WAT, simulator, parser and CDK;
Core remains Agent-free. Formatting, dependency ownership/inheritance, local links,
snapshot integrity and diff whitespace checks pass. Cross-target graph inspection
does not qualify native macOS or installed-IC behavior. No full local suite ran.

Inputs, task-entry before-images, registry/archive checks, fixture-byte comparison
and per-command logs are retained at `/tmp/backup-host-wat-22wfe_qj/`; independent
consumer evidence is at `target/msrv-consumers.X2YEvB/`. Manifests and lock remain
unchanged during qualification. Earlier receipts below retain their original scope;
this batch supersedes their draft compatibility classification, not their evidence.

## Earlier connected capture/restore fixture

The existing PocketIC workflow driver now connects frozen capture and restore
allocations through download/manifest publication, allocation, data upload, same-ID
load, full stopped-state verification and start. Exact original request/reply bytes
precede qualified receipts. The fixture changes live state after capture and requires
the restored module/memory/global/certified/chunk evidence before start.

Six registered cases cover successful completion, cold reconstruction before
allocation/load/start, and lost allocation/second-write/load/start replies. Both
catalogues are frozen before capture; source custody remains outside restore-stage
ancestry. The attempted single-catalogue composition contended with its own held
source layout during ancestor admission; its failure log remains retained. No
library lock, public API, stored schema, dependency or spending contract was changed.

All eleven affected original-workflow PocketIC cases and strict target Clippy pass.
The fixture also compiles with actual Rust 1.91; formatting, local documentation
links, shared snapshot integrity and diff whitespace checks pass. These are focused
Linux checks, not a full local suite. Evidence and task-entry
before-images are at `/tmp/backup-round-trip-70gl75h1/`; failed attempts/fixtures
remain retained. Earlier pending work, dependency selections and the index are
preserved. This extends the same compatible 0.32.1 notes, with no package-version
change or release action. Application/Canic authority, production terminal/reference
release, native macOS and installed-IC qualification remain outside this fixture.

## Earlier local simplification

The pending **0.32.1** notes cover compatible implementation and test cleanup;
no package-version change is made. Compound allocation/data uploads qualify and
record receipts under the same original guard and bounded decoded reply used for
dispatch. Checkpoints remain outside that guard. Public interfaces, wire formats,
persisted records, spending and cancellation/replay rules remain unchanged.

The contract guide groups maintained obligations by capability and removes obsolete
restore-copy publication and adoption announcements. Dependency selections stay in
manifests/lockfiles, migration history in the changelog, and qualification receipts
here and in the archives. Numeric safety and custody contracts remain mandatory.

PocketIC first-read and retained-prefix loss cases use the production download
coordinator. The redundant manual success/lost-read entrypoints and their unused
failure plumbing are removed; distinct capture/allocation/data/lifecycle successful
reconciliation scenarios remain. Planned upload/restore retains source metadata,
full-byte verification, immutable manifest/journal and reference checks.

The incoming Host 0.28.1 lock update is preserved and outside this cleanup.
The current locked graph also selects Platform 0.1.16 and Testkit 0.54.1, with
Testkit's separate development-only Host 0.27.6 graph. Validation below applies
only to its stated source and dependency selection. Earlier receipts do not
qualify this pending work or the incoming dependency update.

Focused Linux validation passes on this preserved locked graph: 16 upload/allocation
and seven data-upload unit cases, all 32 Core PocketIC cases, strict all-target/
all-feature Clippy for both packages, actual Rust 1.91 production-library checks
for both packages, and warning-denied Rustdoc. Formatting, local links, snapshot
integrity and diff whitespace checks pass. Original dependency-input identities
and the Git index remain unchanged. Original inputs, source before-images, fixture
assertion mapping and per-command logs are retained at
`/tmp/backup-complexity-ejkvtr4a/`.

PocketIC requires loopback networking; the sandbox-only attempt failed at listener
startup and is retained separately. The first network-enabled pass exposed an
incorrect new fixture reference assertion: restore legitimately retains an
additional source reference. The final pass checks survival of all prior references
and preserves exact post-admission reference equality inside the restore fixture;
all cases pass. Failed fixture directories and logs remain retained.

These focused results do not replace native macOS, installed-IC or complete
application acceptance. No full local CI, package-version edit, staging, commit,
push, release, publication, deployment or artifact cleanup ran.

## Earlier scheduling and contract cleanup

Fresh and retained workflow entry points keep their original admission paths and
share one post-admission scheduler. Existing execution, checkpoint, spending and
replay owners are unchanged. The retained-prefix regression now checks that an
absent successor uses the retained edge commitment even when the driver's current
projection differs. Public APIs, persisted records and sibling consumers do not
change.

All eight owning workflow regressions and five actual PocketIC workflow cases
pass on the preserved Platform 0.1.15 graph. Strict Core library/test Clippy,
actual Rust 1.91 Core compilation, formatting, local links and snapshot integrity
checks pass. Original inputs, before-images and per-command evidence remain at
`/tmp/backup-local-simplification-jr888nke/`. These are focused Linux checks; no
full local suite, native macOS or installed-IC qualification is claimed. No own
version change, staging, commit, push, release, publication or deployment ran.

The implementation contracts now describe the current Host publication error and
guarded reference release. Superseded migration directions are removed from active
guidance; the unchanged historical changelog retains those transitions.

## Earlier native qualification

Earlier native-qualified baseline: `ic-backup` and `ic-backup-agent` **0.31.2**, commit
`db9c5d99c00657b1fda6a766a256599bc195f235` (`v0.31.2`). The tag resolves to the
same source as the [successful native CI matrix](https://github.com/dragginzgame/ic-backup/actions/runs/39130374520).
Release metadata retains authority. Earlier receipts remain in the
[October 18 archive](archive-2026-10-18.md),
[October 17 archive](archive-2026-10-17.md) and
[ownership review](../reports/audits/2026/10/18/architectural-sediment/03/report.md).

## Current boundary

Core owns backup records, bounds, guarded persistence and coordination. Agent owns
one reserved update's signing, retention and transport. Platform owns Candid,
Principal, management DTO and SDK dependencies. Core selects `candid` and
`management-canister-types`; only Agent adds `agent`. The
[Platform contract](../ic-platform.md) retains the original 0.1.8 adoption evidence.

Host owns hashing, descriptor IO and process custody. Testkit owns simulator assets
and managed processes. Backup retains fixture evidence, bounded DTO visitors,
Reqwest client policy and original journals. Applications own membership, routing,
release identity, consistency and external-effect settlement. A standalone CLI and
complete application runner remain outside the implemented scope.

The 0.31.2 receipt selects Testkit 0.54.0, Host 0.27.4 and Platform 0.1.10. Shared
Tooling 0.24.0 remains at `cab12cd3dfa4fdb34846b25397f0d72881bb9ae0` with 114
exact payloads. Development Rust is 1.99.1; consumer MSRV remains 1.91.0.

## Delivered qualification

The exact 0.31.2 CI source passes the complete configured gate on Linux x86-64,
Intel macOS and Apple Silicon. Every job passes explicit tool preparation and
offline reuse, including application flag/wrapper isolation, the admitted common
formatter directory and the locked Testkit CLI's physical identity. Native logs
include 34 Core and 12 Agent PocketIC cases, actual Rust 1.91 independent Core/Agent
consumers and compilation of both current package archives together. The earlier
[0.31.1 failure](https://github.com/dragginzgame/ic-backup/actions/runs/39123427210)
remains historical evidence; its stale tool comparison is corrected in 0.31.2.

These receipts satisfy the containing native-matrix requirements for
[#55 — Testkit library server admission](https://github.com/dragginzgame/ic-backup/issues/55),
[#58 — Rust 1.99.1 qualification](https://github.com/dragginzgame/ic-backup/issues/58)
and [#60 — Admitted Rust tool comparison](https://github.com/dragginzgame/ic-backup/issues/60).
The advisory [#59 — README compiler floor](https://github.com/dragginzgame/ic-backup/issues/59)
correction is delivered too. CI does not qualify installed-IC or complete
application acceptance. Application transport authority and full runner/terminal
proof remain [#25 — Reserved Agent transport qualification](https://github.com/dragginzgame/ic-backup/issues/25)
and [#29 — Original-plan runner adoption](https://github.com/dragginzgame/ic-backup/issues/29).

## Earlier Platform 0.1.14 qualification

The 0.31.3 release selects Platform 0.1.14 and Testkit 0.54.1; the Platform
requirement remains `0.1`. Its lock also changes crypto-common/generic-array
selections. The 0.31.2 native receipt selects Platform 0.1.10 and Testkit 0.54.0 and
retains its original scope. The incoming Platform 0.1.15 lock update is separate
from these earlier receipts.

The Platform 0.1.14 graph passed focused Linux all-target/all-feature compilation, strict
Clippy, actual Rust 1.91 compilation and both independent public consumers. All 78
IC codec/model, three Agent unit, 14 HTTP, 34 Core PocketIC and 12 Agent PocketIC
cases pass. The exact locked Testkit CLI and authenticated server are prepared and
admitted through their canonical owners. Boundary/inheritance checks pass; Core
remains Agent-free and both production graphs exclude parser, CDK and simulator
packages. Platform's published archive matches its locked checksum and the used
Candid/Agent/management module sources are unchanged from 0.1.10 and 0.1.13.

Original inputs and hashes, per-command logs/results, production trees, archive
provenance and source comparisons are retained under
`/tmp/backup-dependency-qualification-wf8emamv/`; independent Rust 1.91 consumers
retain `target/msrv-consumers.3bBhRh/`. Both actual package archives compile together
on the original normal graph; archive/unpacked checksums and source identities
pass, with evidence at `target/package-check.tCYU4z/`. The focused build/test sequence
verifies unchanged manifest/lock identities. These are focused Linux results, not a full local suite,
fresh native macOS evidence, installed-IC or complete application qualification.

## Remaining acceptance and evidence

The remaining application acceptance requires a concrete integration contract for
current membership/control/read authority, whole-selection fencing and external
obligations. The restore-safety and terminal-evidence ports intentionally install
no default provider. Their test implementations qualify fixture behavior only.
The next implementation must supply these application-owned inputs before wiring
the complete runner; local all-Applied checkpoints cannot substitute for them.

The linked Canic adoption issues remain open. Its reviewed default-branch source
at `53f1f22d2e23a16c5566b8590f3b0a2a643b6d84` already implements the
[terminal-evidence adapter](https://github.com/dragginzgame/canic/blob/53f1f22d2e23a16c5566b8590f3b0a2a643b6d84/crates/canic-backup/src/restore/persistence/retention/mod.rs#L94)
and calls guarded reference release. Its
[live-create entry point](https://github.com/dragginzgame/canic/blob/53f1f22d2e23a16c5566b8590f3b0a2a643b6d84/crates/canic-cli/src/backup/create/mod.rs#L38)
still refuses execution. This is source inspection, not fresh consumer execution.
The remaining work is application authority/fencing integration and complete
retained-plan capture-to-restore qualification with interruption, reconciliation
and effect-free terminal replay. It does not require a second terminal API or
Canic control-plane dependencies in Backup.

The isolated [Canic checkpoint follow-up #607](https://github.com/dragginzgame/canic/pull/607)
adds an actual two-participant capture/restore case; both it and the original
single-participant case pass through the governed PocketIC runner. It targets
[adoption PR #592](https://github.com/dragginzgame/canic/pull/592), whose hosted
acceptance remains pending. These fixtures retain Root records and sender authority
in the driver; they do not complete production Root orchestration or application
acceptance and do not enable live creation.

Exact 0.31.2 hosted job metadata and all three raw native logs are retained at
`/tmp/backup-issue-reconcile-0wqan5nq/`. Earlier focused Linux receipts remain at
`/tmp/backup-issue-pass-mlttf_tl/`, `/tmp/backup-native-followup-wafn4qmr/`,
`target/msrv-consumers.dGrOin/`, `target/ci-fixtures/native-reuse.y_641iwl/` and
`target/package-check.CN2Dxh/`. Those receipts retain their original source and
dependency selections. The earlier 0.1.8 adoption has separate provenance in the
Platform contract.

This qualification preserves the incoming dependency update and changes no library
behavior, public API or persisted schema. No new local full-suite run, live IC
operation, release, publication, deployment or artifact cleanup is part of it.
