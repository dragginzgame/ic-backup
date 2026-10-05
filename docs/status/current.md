<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-backup/ic-backup-readme-header.svg" alt="IC Backup — Verified backups and safe recovery for Internet Computer apps" width="100%">
</p>

<!-- helper-navigation:start -->
<p align="center">
  <a href="https://github.com/dragginzgame/canic"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/canic.svg" width="18" height="18" alt=""> <strong>canic</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/icydb"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/icydb.svg" width="18" height="18" alt=""> <strong>icydb</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-timers"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-timers.svg" width="18" height="18" alt=""> <strong>ic-timers</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-memory"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-memory.svg" width="18" height="18" alt=""> <strong>ic-memory</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-query"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-query.svg" width="18" height="18" alt=""> <strong>ic-query</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-backup"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-backup.svg" width="18" height="18" alt=""> <strong>ic-backup</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-blob-storage"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-blob-storage.svg" width="18" height="18" alt=""> <strong>ic-blob-storage</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-testkit"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-testkit.svg" width="18" height="18" alt=""> <strong>ic-testkit</strong></a>
</p>
<!-- helper-navigation:end -->


# Current handoff — 2026-10-05

The maintainer's released repository baseline is `0.2.0` at `3e76590`. The
workspace manifest remains `0.2.0`; the maintainer selected one undated `0.2.1`
changelog draft for the current work.
The implementation and selected changelog were committed by the maintainer at
`8646589`; the release-helper correction below remains uncommitted.
The retained release receipt identifies validated source
`65c9532d85de0d74f9ed19b9237f1dbbcc961a61` and its `release-verify` gate. The
maintainer reports pushing the release; registry publication was not observed.
This continuation performs no real version transaction or release command.

## Explicit release selection and provisional changelog labels

Preparation now lets the explicit release command relabel the single current
future draft, preserving its exact notes and history. The reported `release-minor`
attempt selected `0.3.0` from package `0.2.0` but was blocked by the provisional
`0.2.1` heading. A future draft label now yields to the explicit target; duplicate,
competing, empty, misplaced or dated drafts and existing target sections still
reject. Imported undated history cannot become the new top draft. The final draft
label is resolved against the original manifest before version mutation, under
the existing rollback that restores the original label and all release bytes.
Preparation prints its exact current/target versions before validation.

`make release-check` passed against isolated Git/Cargo/gate substitutes, including
patch/minor/major retargeting, exact preserved notes/history, receipt checks and
gate/update/metadata failure rollback. Evidence is
`target/changelog-retarget-release-check.log`. These tests perform no real commits,
tags, pushes, uploads or cleanup of consumer build/evidence artifacts.
`make shell-check` also passed, including Perl syntax; evidence is
`target/changelog-retarget-shell-check.log`. Read-only admission accepts both
the current `0.2.1` target and explicit `0.3.0` target without changing files.
The manifest remains `0.2.0`, the current changelog stays undated `0.2.1`, and the
existing release receipt is unchanged. This is a consumer-owned release-helper
fix; vendored Shared Tooling bytes and sibling repositories are unchanged.

## At a glance

| Question | Current answer |
| --- | --- |
| Can it perform a complete backup or restore? | No. The transport, runners and CLI remain unimplemented |
| What works today? | Local artifacts, bounded records, journals, plans, selected IC codecs and pure integration checks |
| What has been qualified? | Native local filesystem, record, policy and process behavior within the evidence described below |
| What remains integration-owned? | Live membership, authority, application consistency, authenticated calls and restored-state acceptance |
| What is the next product boundary? | Fence lifecycle and uncertain-effect reconciliation contracts, then runners, transport and PocketIC/real-IC qualification |

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-backup/ic-backup-product-readiness.svg" alt="Product readiness stages showing the implemented local safety foundation followed by application adapters, authenticated IC transport and runners, and PocketIC and live qualification" width="800">
</p>

The current batch adds immutable original application fence obligations and exact
acquisition-journal recovery, following the restore/source/safety declaration batch.
Candid 0.10.37 and management SDK 0.11.0 remain
locked unchanged. The manifest, lockfile, release receipt, earlier provenance and
vendored Shared Tooling bytes are unchanged. Only targeted Linux checks ran for
this continuation; native macOS and actual IC/application qualification remain
unestablished by these checks.

## Original application fence obligations and acquisition recovery

The [maintained boundary](../extraction-boundary.md#original-application-fence-obligation-retention)
now retains exact full-plan/selected-unit capture or restore fence obligations,
original requirements/revisions and an explicit application acquisition operation.
Fixed `fence-obligation.json` has strict v1 1 KiB immutable publication/read with
exact original retained requirements and plans, including both restore/source
layout guards. Weaker lanes, zero acquisition allowance, missing/changed originals,
rebound identities, unsafe/excessive bytes and contention reject. See
[the schema and independent binary goldens](../contracts/fence-obligation.schema.json).

The attempt journal remains the sole spending/reconciliation owner. Pure
`acquisition_progress` joins its exact original operation/context/request/limits
and returns its existing view. Missing evidence cannot mean zero consumption,
lost observations stay pending, Uncertain leaves the mutation unresolved, and
no outcome refunds attempts. Pending, Applied, NotApplied and Uncertain outcomes
retain the original obligation and unfinished source references. An Applied
projection proves no current Active custody or release permission.

Ten focused unit tests and one public native filesystem recovery journey passed.
All-target/all-feature warning-denied Clippy, rustdoc and Rust 1.91.0 compilation
passed. Logs are `target/fence-obligation-unit.log`, `public-final.log`,
`clippy-final.log`, `docs-final.log` and `msrv-final.log`, each with the
`fence-obligation-` prefix. No test or
lint attempt failed in this batch. The public journey preserves exact obligation/
journal bytes and unfinished source references through exhausted lost replies,
every passive settled outcome and drop/reopen. These qualify local contracts only.
Formatting/whitespace checks, all twelve Shared Tooling snapshot files, schema
generator replay, exact inspected source/consumer references and 156 local
Markdown targets also passed; evidence is `target/fence-obligation-format.log`,
`diff-check.log`, `snapshot-check.log` and `evidence-check.log`, each with the same
prefix. Manifest, lockfile, release receipt and vendored baseline remain unchanged.

[Fresh Canic inspection](../fence-obligation-source.json) identifies the same
dirty source HEAD `a2108801d7b2d5802f3e864556dc8781b3b382df`, three exact source
files and their actual text consumers. It confirms retained-reference/terminal
command-custody responsibilities and unreconciled load behavior, supplying no
generic application-fence acquisition or release proof. No source code was copied;
the new contract refines the local product design and reuses existing owners.
Exact read-only inputs, schema/provenance generators and consumer-reference
evidence remain under `target/fence-obligation-*`; earlier provenance is unchanged.

No coordinator implementation, authenticated acquisition reconciliation, fresh
Active proof, dispatch, terminal evidence or controlled fence/reference release
is implemented. Original opaque request semantics, identity and whole-selection
fence custody remain integration-qualified. The next contract work covers actual
coordinator requests and qualified reconciliation, then terminal evidence before
release admission. No full gate, release/version transaction, upload, commit or
live effect ran. This batch was delivered uncommitted under the undated 0.2.1
changelog draft at package 0.2.0; the maintainer subsequently committed it at
`8646589`.

## Original restore/source safety batch

The [maintained boundary](../extraction-boundary.md#original-restore-safety-requirements-and-current-loadstart-checks)
now has immutable same-network/release selected-source declarations and a fresh
`RestoreSafetyProvider` contract for original exact load/start bytes. Required
safety lanes have no generic default. Load needs all selected targets stopped;
start needs its target stopped, every selected restored state accepted for the
exact source and fenced execution qualified. A still-stopping member rejects.
Original source artifacts and fence/membership/external-obligation revisions
cannot silently rebind. Both retained plans and 1 KiB no-replace requirement IO
are checked under unchanged layout exclusion. See
[the schema and independent goldens](../contracts/restore-safety-requirement.schema.json)
and [typed contract](../contracts/restore-safety-port.json).

Sixteen focused unit tests and one public recovery journey passed, with
warning-denied all-target/all-feature package Clippy/rustdoc and Rust 1.91.0
all-target/all-feature compilation. Logs are `target/restore-safety-unit.log`,
`public.log`, `clippy.log`, `docs.log` and `msrv.log`, each with the same
`restore-safety-` prefix. Cases cover strict schema/binary hashes, canonical target
bounds, exact source/context/payload/lane/revision/lifecycle/acceptance denials,
immutable both-layout persistence and unsafe/oversized/rebound files. The public
native fixture preserves exact journal/requirement/outside-source obligation
bytes and unfinished restore references through exhausted pending attempts,
stale/inactive/rebound results, every typed provider failure and drop/reopen.

[Fresh Canic inspection](../restore-safety-source.json) records six working-tree
files and 334 exact text consumer references under HEAD
`a2108801d7b2d5802f3e864556dc8781b3b382df`, with dirty source explicitly recorded.
Fixed-ID/stopped-load checks are adapted; mapping/Root/argv, module-hash-only
verification and status reconciliation are not imported as safety or load receipts.
The source supplies no generic outside-snapshot external-work proof; that contract
is designed from the local product requirements, not claimed as copied qualification.
Exact inspected source bytes remain at `target/restore-safety-source.Z5hWZi/`;
reference inventory and independent contract generation/check evidence remain
under `target/restore-safety-*`. Earlier provenance and licenses are unchanged.

Initial Clippy attempts flagged function length; responsibilities were split and
successful checks rerun. Diagnostics remain at
`target/restore-safety-clippy-first-attempt.log`, `second-attempt.log` and
`third-attempt.log`, each with the same prefix. No full CI/release gate, package
upload, commit or live IC effect ran. The next release version was undecided
at that batch's completion; the current selection is recorded above.
Applications still qualify
authentic complete source/upload association, fresh lifecycle/release/drain,
irreversible-work absence or continuous rewind-independent fence/replay safety,
restored acceptance and prior per-call accounting. No provider, fence acquisition/
release, lost-load settlement, dispatch or terminal/source-reference release is
implemented. These are local contracts, not actual management/application safety.

The entries below retain earlier development evidence. Their draft/version/gate
statements describe original execution before the maintainer's 0.2.0 release.

## Shared Tooling best-practice review and refresh

The current common baseline and all twelve declared tooling files now come from
clean Shared Tooling commit `41e1fd0ba41460bd2127cbf98ac8a4b2b2020d3e`, inspected
read only on 2026-10-05. The reviewed refresh adds `DRAGGINZGAME.md` to the same
checksum/mode manifest as its linked guides and helpers; its source remote is
deliberately recorded with the source checkout's canonical HTTPS spelling.
The upstream exporter read exact committed Git objects, and the completed
snapshot verified offline. No sibling source or Git history was changed.
Previous files and manifest remain at `target/shared-tooling-review.4NA5Aq/previous/`
and `target/shared-tooling-review.4NA5Aq/previous.snapshot`; the export receipt is
`target/shared-tooling-review.4NA5Aq/refresh.log`.

The review found a Linux-only CI configuration and nonportable consumer fixture
helpers. CI now selects Ubuntu 24.04 and macOS 15 on Apple Silicon and Intel, with
host-specific dependencies and GNU Make setup. The macOS jobs explicitly run
tooling with system Bash 3.2 before the common native gate. Consumer release/hook
tests use `shasum` when `sha256sum` is absent and Perl for portable file edits;
the release fixture uses Bash 3.2 subshell identity and positional arguments instead
of BASHPID and empty-array expansion. The upstream verifier includes its own
Bash 3.2 empty-array fix. The local host matrix declares required macOS support
while preserving the absence of native macOS qualification for this revision.

`make shared-tooling-check tooling-check release-check hooks-check shell-check`
passed, including snapshot rejection, retained failure artifacts, release rollback/
retry and hook index preservation. Actionlint passed for the three-host workflow.
The first four focused targets also passed under an isolated PATH with no
`sha256sum`, exercising the real `shasum` path. Logs are
`target/shared-tooling-review-checks.log`, `target/shared-tooling-review-workflow.log`
and `target/shared-tooling-portable-checks.A6zwiZ/checks.log`. Ordinary whitespace
checks passed. These are Linux tooling checks, not native macOS or IC evidence.
No broad CI/release gate or new Rust compilation ran for this refresh.

The public GitHub description review is retained at
`target/shared-tooling-description-review.json`. It still describes extraction
design and contributor instructions rather than the implemented Rust foundations;
[the adoption guide](../shared-tooling.md) contains concrete replacement wording.
Authenticated GitHub CLI access is unavailable, so no issue was filed or metadata
changed. Native macOS CI results and the description correction remain maintainer
actions. No local feedback tracker was created. Product version, earlier SDK
upgrade work, historical provenance and retained failure/recovery evidence remain
preserved; no commit, release transaction, package upload or live IC effect ran.

## Management SDK dependency upgrade

The exact `ic-management-canister-types` pin is now `0.11.0`, upgraded from `0.8.0`.
The official registry index identifies 0.11.0 as the newest non-yanked entry observed
on 2026-10-05. Its cached archive checksum matches that entry and the new lock entry;
inspected source, manifest, changelog and revision metadata also match the archive.
Only this package's version/checksum changed in Cargo.lock; Candid remains 0.10.37.
See [current dependency inspection](../management-types-upgrade.json); the original
0.8.0 source provenance remains unchanged historical evidence.

The six request methods and used snapshot shapes retain their wire fields. Every
original request/reply golden remains byte-for-byte unchanged and passes production
admission. The full SDK status fixture now includes maximum u128 incoming-call
cycles and a distinct allowed status viewer; both new settings remain bounded
skipped work, separate from the required status/controllers and fresh permissions.
No production codec, product v1 record or request/reply hash encoding changed.
The released 0.1.8 API uses SDK DTOs internally and does not expose the lifecycle
status enum; that public API was introduced after the released baseline. The
maintainer selected an undated 0.2.0 draft for this batch. This is a release choice,
not a claim that the SDK upgrade breaks the published API or a version transaction.

All 36 focused cases passed: 26 IC request/reply unit cases, six inventory comparison
cases and four public request/reply/comparison recovery journeys. Warning-denied
all-target/all-feature package Clippy and rustdoc, plus Rust 1.91.0 all-target/
all-feature package compilation passed. Logs are `target/management-types-upgrade-`
`codecs.log`, `comparison.log`, `public.log`, `clippy.log`, `docs.log` and
`msrv-check.log` with that same prefix. The initial MSRV attempt could not run
because the toolchain was absent; its log remains at
`target/management-types-upgrade-msrv.log`. The required toolchain was installed
before the successful retry; installation evidence is
`target/management-types-upgrade-msrv-install.log`. Source/archive comparison files
remain at `target/management-types-source.IDXDP5/`, and the exact registry response
at `target/management-types-upgrade-registry.jsonl`.

These checks qualify native codecs, pure comparison and retained local recovery.
No full CI/release gate, package upload, commit or IC effect ran. The product manifest
remains 0.1.8; the selected undated changelog draft is 0.2.0.

## Retained local ShellCheck lookup correction

The reported `shell-check` failure came from a missing PATH entry, not a script
diagnostic. ShellCheck 0.11.0 already exists at `/home/adam/.local/bin/shellcheck`.
The host's login profile includes that directory, but the inherited non-login
validation environment did not. `make shell-check` now selects the PATH command
first, then an executable `~/.local/bin/shellcheck` only if PATH lookup fails.
It reports the selected command and rejects a missing tool; an available tool's
failure stops validation without another lookup or lint bypass.

Plain Make and the actual shared runner's selected `shell-check` target passed
ShellCheck, Bash syntax and Perl syntax with PATH restricted to `/usr/bin:/bin`.
A fixture PATH command exited 53; its marker and failed Make status verified PATH
precedence and failure propagation. Logs are `target/shellcheck-user-local-validation.log`
and `target/shellcheck-lookup.HTluDY/`. Earlier validation with an explicitly extended
PATH remains at `target/shellcheck-path-validation.log`. Both retained failure logs
remain under `target/validation-failures/` with names
`20261005T125303Z-600509-2-shell-check.log` and
`20261005T125321Z-601273-2-shell-check.log`.

The development quick start and selected changelog describe current lookup behavior.
No dependency download or host-profile edit was needed. The consumer-owned Makefile
changed; vendored scripts and snapshot identity remain intact. No Rust compilation,
full CI/release gate or release transaction ran for this lookup correction.

## Retained Shared Tooling document restoration

The documentation sweep in `c16b909` changed all seven manifest-declared Markdown
files: it added local banners/navigation, removed trailing blank lines and added
consumer prose/tables. The snapshot verifier correctly rejected the first drifted
file, `docs/principles/README.md`. All seven documents now match the existing pinned
Shared Tooling revision `956236a3848c2cfae6ae05f5c77e9c37b01b3366` again. Restoration
used read-only Git objects, verified the complete staged snapshot before copying
and changed neither the manifest nor verifier. No sibling checkout was modified.

Useful local principle navigation, host scope and tooling-snapshot terminology
now live in the consumer-owned documentation index, development guide and adoption
guide. AGENTS.md explicitly excludes manifest paths from branding/documentation
sweeps. `make shared-tooling-check` and `make tooling-check` passed, including exact
bytes/modes, drift/missing/symlink rejection, runner and evidence-retention cases.
Local link and changelog checks passed. Ordinary `git diff --check` reports only
the five restored upstream EOF blank lines; that diagnostic is retained. Scoped
diff checks passed with those checksum-bound EOF lines permitted for the principle
guides, while all other changed files used the ordinary whitespace check.
Original files, pinned restoration bytes and passing logs remain under
`target/shared-tooling-repair.g3TJaJ/`. The reported failure log remains at
`target/validation-failures/20261005T124750Z-588243-0-shared-tooling-check.log`.
No Rust compilation, full CI/release gate or release transaction ran for this
documentation/snapshot correction.

## Focused 0.1.9 draft review

Review of the four selected implementation batches at repository HEAD
`6356c0ff00fd950b9e9f106c335ec7450b56f33c` found no blocking correctness issue in
their maintained local scope. Production owners, bounded wire admission, exact
request/raw-evidence hashing, baseline preservation and original-plan/durable-byte
checks agree with the machine contracts and changelog. Decoded state, candidate
cardinality and local checks remain separate from fresh authority, effect settlement,
transfer completeness and terminal/reference release. No production change was needed.

Fresh offline/locked checks passed: 26 IC request/reply unit cases, six inventory
comparison cases, five integrity-policy cases, seven guarded integrity cases and
five public request/reply/comparison/integrity recovery journeys. The public cases
retain exact evidence, pending exhausted attempts and unfinished restore references.
Logs are `target/0.1.9-review-codecs.log`, `target/0.1.9-review-comparison.log`,
`target/0.1.9-review-integrity-policy.log`, `target/0.1.9-review-integrity-ops.log`
and `target/0.1.9-review-public.log`. All 28 retained Canic source hashes and 273
exact consumer references still match the inspected working-tree bytes; evidence
is `target/0.1.9-review-provenance.log`. The existing Candid/management SDK source
hashes also match retained provenance. Successful tests cleaned only their owned
fixtures; earlier failed/build/recovery evidence remains retained.

This is focused draft review, not the full CI/release gate or IC backend qualification.
This earlier review used the undated `0.1.9` draft; the current draft selection is
recorded above. The package manifest remains `0.1.8`. The maintainer owns source commits and release execution under
[the release guide](../releasing.md). No version transaction, tag, push, upload or
live IC effect ran. Restore-safety/fence and lost-effect contracts, providers,
transport and runners remain unfinished as described below.

## Retained implementation batches

### Fresh local download integrity batch

`policy::download_integrity::validate` borrows the original full plan and download
journal, requiring exact intent, canonical selected coverage and Durable checksums.
Views retain journal-owned snapshot identity and metadata without a new schema,
serialized verified flag or hash encoder. The original pre-capture plan cannot
independently establish the later captured snapshot ID.

`DownloadJournalGuard::verify_durable_artifacts` explicitly admits the persisted
original plan and unchanged held journal before and after no-follow directory
checksum verification. Existing owners retain 1 MiB record IO, 1,024 selected
entries and 256-byte snapshot tokens. Ordinary reopen/resume still reads retained
progress without a fresh check. New typed failures write no evidence, transition
no journal, replenish no attempt and release no reference. Filesystem checks are
sequential observations rather than an atomic tree/set snapshot; integrations own
stable byte custody, authentic capture association, complete backend transfer and
terminal/release admission. No provider, runner, transport or live effect is added.

Eight Canic source sections and 47 exact consumer references were freshly inspected
under the retained dirty source HEAD. Explicit durable-byte verification and exact
coverage are adapted; the old manifest, duplicate artifact metadata, serializable
completion flag, receipt inference, CLI/prune/restore effects are not imported.
See [the contract](../contracts/download-integrity.json) and
[fresh source/consumer provenance](../download-integrity-source.json).

Five pure-policy tests, seven guarded filesystem tests and one public recovery
journey passed. Coverage includes changed original intent/selection, every incomplete
state, maximum target/token limits, missing or replaced plans/journals, changed
published bytes, absent/file/symlink substitutions and unusable custody. Public
recovery preserves exact journal bytes, exhausted pending reservations and unfinished
restore references after successful checks and checksum failure. Native evidence
qualifies local mechanisms, not IC effects or a complete independently usable product.

Warning-denied all-target/all-feature package Clippy and rustdoc, Rust 1.91.0
all-target/all-feature package compilation, formatting/diff, changelog and source/
reference/contract/link checks passed. Logs are `target/download-integrity-policy.log`,
`target/download-integrity-ops.log`, `target/download-integrity-public.log`,
`target/download-integrity-clippy.log`, `target/download-integrity-docs.log` and
`target/download-integrity-msrv.log`. The initial unit compilation rejected a test
cleanup helper moving a borrowed layout; its log is retained at
`target/download-integrity-policy-initial-failure.log`. Explicit ordered fixture
drops fixed it. Successful tests cleaned only owned runtime fixtures; all earlier
failure/recovery artifacts remain retained. No full suite, package/release gate,
version transaction or publication ran.

### Lifecycle reply batch

`model::ic_lifecycle_reply` admits the existing status/stop/start/load methods.
Mutation acknowledgements require the canonical six-byte empty Candid tuple.
Status requires exactly one status/settings/controllers projection, reusing the
upstream state enum and existing `ControllerSet` canonical owner. Explicit empty
controllers remain empty; missing fields and duplicates reject. Bounded visitors
retain at most 10 principals without allocating from untrusted declared lengths.
Raw input is capped at 1 MiB; status work at 2 MiB, skipped work at 64 KiB and
type-table entries at 64. Extra arguments, trailing data and malformed shapes reject.

Unprojected fields are skipped, not retained or qualified. This may admit records
lacking fields outside the projection and is not full SDK status validation.
Read-only kinds retain exact raw checksums plus a separate v1 request/reply digest;
ignored fields and original controller ordering stay in evidence identity. There
is no serialized fresh result, provider, dispatch, receipt or journal transition.
Transport association/authentication, current timing/permissions, continuous fences,
stopped/drained evidence and safe same-release load settlement remain integration-owned.
Neither Stopped/controller values nor acknowledgements settle a pending attempt or
authorize restart, release, retry or new spending. See
[the contract](../contracts/ic-lifecycle-reply.json) and
[fresh provenance](../ic-lifecycle-reply-source.json).

Eight Canic files/sections and 49 exact consumer references were inspected read-only
under the same dirty source HEAD retained below. Typed lifecycle states, required
status/controller projection and pending-observation rejection are adapted. Agent/
CLI calls, optional status defaults, Root/Fleet routing, command-success or
status-equality completion receipts and automatic reconciliation are not copied.
Existing SDK 0.8.0 and Candid 0.10.37 source/registry identities are recorded; the
current primary management interface was also checked read-only.

Ten focused unit cases and one public recovery case passed. Every registered
hand-assembled Candid/hash fixture enters production admission; a separate complete
SDK status exercises bounded skipped fields. Native cases cover required fields,
unknown states, empty/maximum/duplicate controllers, raw/work/type/count limits,
truncations/tuple/trailing rejection and exact target/method/raw load-ID hashes.
The public journey durably retains local load/status wire fixtures and reopens
exhausted pending original reservations. Decoding, wrong-shape/malformed rejection
and changed load association preserve exact journal/evidence bytes and allowances.
It issues no receipts and performs no remote observations or IC effects.

Warning-denied all-target/all-feature Clippy and rustdoc, Rust 1.91.0 all-target/
all-feature package compilation, formatting/diff, changelog admission and fresh
source/reference/dependency/golden/JSON/link checks passed. Logs are
`target/ic-lifecycle-reply-unit.log`, `target/ic-lifecycle-reply-integration.log`,
`target/ic-lifecycle-reply-clippy.log`, `target/ic-lifecycle-reply-docs.log` and
`target/ic-lifecycle-reply-msrv.log`. An initial test enum lacked the Serde derive
needed for its rename attribute; its compiler log remains at
`target/ic-lifecycle-reply-unit-initial-failure.log`. The initial hex-helper lint
failure remains at `target/ic-lifecycle-reply-clippy-initial-failure.log`.
Corrected cases passed. Successful runtime fixtures cleaned only their owned
temporaries; prior failed/build/recovery evidence remains retained. No full suite,
package verification, full CI/release gate or publication ran.

### Snapshot inventory comparison batch

`policy::snapshot_inventory_delta::compare` borrows the exact existing capture and
two inventory reply owners. It requires the closed capture/list methods and exact
canonical target, then linearly compares canonical raw IDs. Every baseline ID must
remain with unchanged timestamp/size; loss and metadata drift reject with typed
errors. The read-only view exposes zero/single/multiple canonical candidates and
borrowed original request/reply evidence. Existing 1,024-entry/256-ID-byte bounds
apply, with no duplicate wire/hash owner or new persisted schema.

Cardinality does not attribute or settle a capture. Another controller could create
one candidate; no candidate does not prove failure. Integrations own original
pre-effect baseline custody, authenticated association, actual observation chronology
and exclusive attribution. No provider, receipt, retry, restart, cleanup, spending
reset or upload reconciliation is introduced. See
[the machine contract](../contracts/snapshot-inventory-delta.json) and
[fresh provenance](../snapshot-inventory-delta-source.json).

Six fresh Canic source sections and 75 exact consumer references were inspected
read-only under the same dirty HEAD recorded below. Baseline preservation and set
difference are adapted; singleton-to-completed-receipt inference and restart/journal
cleanup are not copied. Related upload recovery stays unchanged in Canic and outside
this capture projection. Earlier source provenance remains exact and retained.
The maintained request schema's descriptive Candid selection now matches the
actual `0.10` declaration/0.10.37 lock; historical .35 evidence is unchanged.

Six focused policy cases and one public persistence/recovery case passed. They cover
all candidate cardinalities, exact byte-prefix/raw-order identity, method/target
mismatches, baseline loss at every merge position, timestamp/size drift and maximum
combined bounds. Public recovery retains original baseline/observation bytes and
reopens original exhausted pending mutation/observation reservations. Singleton,
multiple and zero projections, baseline rejection and a denied fresh mutation
preserve exact journal/evidence bytes and original allowances; no receipt is issued.

Warning-denied all-target/all-feature Clippy and rustdoc, Rust 1.91.0 all-target/
all-feature package compilation, formatting/diff checks and fresh source/reference/
JSON/document-link checks passed. Logs are `target/snapshot-inventory-delta-unit.log`,
`target/snapshot-inventory-delta-integration.log`,
`target/snapshot-inventory-delta-clippy.log`, `target/snapshot-inventory-delta-msrv.log`
and `target/snapshot-inventory-delta-docs.log`. Initial test-style lint failures and
an untyped empty-slice assertion compilation failure remain in
`target/snapshot-inventory-delta-clippy-initial-failure.log` and
`target/snapshot-inventory-delta-clippy-empty-assertion-failure.log`; corrected cases
passed. Successful runtime fixtures cleaned only their owned temporary directories.
No full suite/package/full CI/release gate or actual IC effect ran. Transport,
metadata/data transfers, safe lost-effect settlement and runners remain unimplemented.

### Snapshot capture/inventory reply batch

`model::ic_snapshot_reply` decodes exactly one capture descriptor or inventory
vector tied to the caller's borrowed immutable request. Required upstream fields
retain exact 1–256 raw ID bytes and full nat64 timestamp/size. Raw input is bounded
to 1 MiB; decoder work to 2 MiB, skipped work to zero and type-table entries to 16.
Bounded sequence visitors retain at most 1,024 descriptors without allocating from
untrusted lengths. Unknown/missing/wrong fields, malformed data, extra arguments,
trailing bytes and duplicate IDs reject with typed errors and no raw diagnostics.

Inventory views sort exact raw IDs; payload hashes preserve raw ordering and a
separate v1 digest binds the existing request digest plus raw-reply checksum.
The reply contains no target/network/caller/challenge, so the association is
declared, not authenticated. Descriptor metadata is neither transfer completeness
nor a unique lost-capture receipt. No Serde record, reservation, settlement or
dispatch mechanism is introduced; pending attempts and spent limits remain intact.
See [the machine contract](../contracts/ic-snapshot-reply.json) and
[fresh provenance](../ic-snapshot-reply-source.json).

Six Canic source files and 102 consumer references were inspected read-only.
The source still has dirty working-tree material under HEAD
`3978e02d28fea9022c7a6e84a7ec6d6e4a0d4af3`; exact hashes identify inspected bytes.
Raw ID admission, duplicate checks and bounded Candid inventory parsing are adapted.
ICP token readers, optional metadata, agent calls, inventory-delta settlement and
completed execution receipts are not copied. Existing SDK 0.8.0 and actual selected
Candid 0.10.37 source are separately recorded; no dependency or lock update ran.

Native evidence includes every independently assembled DIDL/hash fixture decoded
by production and official DTOs, maximum combined entry/ID bounds, full nat64
values, wire/target hash sensitivity, type/length/truncation/tuple/field rejection
and duplicate identity. A public journey reopens original intent plus exhausted
pending mutation and observation reservations, decodes retained exact bytes,
rejects wrong association and preserves exact journal/evidence bytes and budgets.
It makes no remote observations or receipts. The existing request codec regressions
are also rerun against the current lockfile selection.

Targeted unit/integration, warning-denied Clippy/rustdoc, Rust 1.91.0 library
compilation and formatting/diff checks passed. Logs are
`target/ic-snapshot-reply-unit.log`, `target/ic-snapshot-reply-integration.log`,
`target/ic-snapshot-reply-clippy.log`, `target/ic-snapshot-reply-msrv.log` and
`target/ic-snapshot-reply-docs.log`. An initial integration assertion expected
MutationPending while a separate observation was pending; the corrected typed
ObservationPending assertion passed. The initial integration/lint failure logs
remain under `target/ic-snapshot-reply-*-initial-failure.log`, and its interrupted
fixture remains at `/tmp/ic-backup-public-reply-410-1791184054778669690/`.
No full native test suite, package verification, full CI or live IC check ran.

The following tooling and consistency sections retain prior qualification evidence;
they do not claim fresh broad validation for this batch. Live providers, complete
transfer/remaining response codecs, fencing and safe restore settlement still
precede runners. Independently usable backup/restore remains unimplemented.

### Retained nested release-check correction

The outer validation runner exports `VALIDATION_REPOSITORY_ROOT` for its children.
The dependency-bootstrap fixture inherited the real repository root and dispatched
its simulated Make target there, where the fixture's exact-path substitute rejected
it before Cargo ran. Direct release checks had passed without this parent context.

The consumer-owned release adapter now binds repository, failure-log and GitHub
summary paths to its own fixture. Added success/failure cases seed a different
parent runner context, verify the actual Make gate and require parent summary/log
state to remain unchanged. The tooling adapter similarly owns the upstream runner
test's GitHub summary, keeping simulated failures out of the real parent summary.
Vendored tools, snapshot identity, production release requirements and Rust are
unchanged.

Direct `make release-check` passed. The actual shared runner then passed the
selected `release-check`, `tooling-check` and `shell-check` targets with an explicit
parent failure-log directory and GitHub summary. Only the three real passing
targets appear in that summary; no fixture failure logs escaped into the parent.
Evidence remains at `target/nested-release-check.FPFCKV/`. This was targeted tooling
validation, not full CI or release preparation. Changelog admission and diff checks
also passed.

The original `target/release-tests.QebrYj/` and retained validation logs remain.
A pre-fix reproduction through the shared runner failed identically; its output
is `target/release-check-reproduction.log`, with logs under
`target/release-check-reproduction/` and fixture `target/release-tests.y6HiMu/`.
Earlier failed evidence remains retained too. Successful test fixtures cleaned
only their own temporaries.

### Retained initial shared engineering and tooling batch

The initial [shared adoption](../shared-tooling.md) recorded upstream committed
revision `956236a3848c2cfae6ae05f5c77e9c37b01b3366` and separately hashed dirty
upstream rules. Those were the reviewed working-tree rules retained locally at
that time. The current committed snapshot is described above. AGENTS.md retains
its backup-specific overlay within the mandatory shared baseline. Shared and
local rules agree on GitHub-only feedback tracking and explicit broad-validation
authority. CI does not inherit a mutable sibling checkout.

The local snapshot manifest binds exact committed principles, consumption/host
guides, checksum/snapshot verifiers, validation runner and runner regressions.
Make verifies it before fetching dependencies, then preserves the prior sequential
fail-fast gate with target-labelled failures, timing/result and GitHub summaries,
and full/highlighted failure logs under `target/validation-failures/`. The consumer
adapter checks exact bytes/modes, missing files, symlinks, duplicate/escaping manifest
paths and artifact retention. Vendored files stay unchanged.

Release preparation now labels/dates one top-level unnumbered or selected numbered
draft. It rejects ambiguous/empty/misplaced drafts, preserves imported undated
history and keeps the exact receipt/tag requirements. Registry publication still
delegates directly to Cargo. No package version or backup contract changed.

Targeted checks passed: `make tooling-check`, `make release-check` and
`make shell-check`. The real Make dependency gate was exercised with substituted
Cargo and selected targets, not a full native build or network fetch. Read-only
release planning, snapshot identity/mode checks and diff checks also passed.
An initial new test used a fixed date instead of the generated receipt date;
the corrected test passed. Its failed fixture and trace remain at
`target/release-tests.7mc795/`. Successful isolated test temporaries were removed
by their owners; existing build/package/recovery evidence remains retained.

The consistency qualification below is retained evidence from the prior extraction
batch, not a fresh Rust run for this tooling change. The maintainer's current Candid
dependency declaration is preserved. No broad CI/release gate or native compilation
was requested; product and host qualification boundaries remain as described below.

### Current consistency batch

`model::consistency::ConsistencyRequirementRecord` retains strict v1 `version`,
canonical full original `plan_intent` and explicit `per_canister` or
`application_coordinated` requested guarantee. Missing/unknown fields, other versions
and obsolete flag names reject. A domain-separated binary hash binds original plan
intent and guarantee, not current consistency. Full original intent includes context,
inventory, selection, graph, requests and original allowances.

`create_consistency_requirement` requires the original plan already retained under
layout exclusion, then durably creates fixed `consistency-requirement.json` under
journal exclusion without replacement. Reads require exact original expected
requirement digest and original retained plan. Raw input/canonical output have a
1 KiB bound. Lost local creation replies reconcile through exact retained reads;
missing/corrupt/changed evidence never silently downgrades or recreates original
requirements. No fence or allowance is retained or created by this declaration.

Ephemeral `ConsistencyRequest` binds original requirement/operation, caller-owned
challenge, BeforeCapture/AfterCapture boundary and a 0–1,024 descriptive remote-call
ceiling. Per-canister requests require no expected fence; coordinated requests require
an exact `ApplicationFenceBinding` with retained identity AND original membership
revision recovered by the integration's durable obligation owner. Both fields bind
the request hash. Constructor equality proves neither durable fence retention nor
current Active custody; capture wire/effect admission remains separate.

Model-admitted current observations retain exact request, actually observed canonical
context/full inventory, a nonempty canonical sorted unique 1–1,024 set of inventory-backed
targets with actual Running/Stopping/Stopped states and required opaque stopped/drained
evidence, actual revision, explicit evidence lane, opaque observation evidence and actual
call reporting. Coordinated evidence includes actual Active/Inactive fence state,
identity/revision and required whole-unit write/membership/timer/external-work fencing
and drained-work evidence digests. New request/parameter/result/target/fence/view types
have no Serde/default or persisted authority admission. Only the requirement is persisted.

`ports::consistency::ConsistencyProvider` has a fallible typed signature without an
installed/default implementation. It observes existing obligations; no acquisition/
release API is installed. Pure `policy::consistency::validate` requires exact current
request/context/full inventory/exact selected set/call reporting, every target Stopped
and the original guarantee. Coordinated evidence must have the exact retained Active
fence; both actual and fence revisions equal the original retained revision. This
prevents a changed observation and fence revision from silently rebinding an obligation.
No accepted/expiry/Proven flag or parent/component inference supplies a guarantee.

Integrations qualify actual authenticated state, stop/drain, fresh unique challenge
timing, continuously retained whole-selection fence custody, opaque evidence meaning
and prior per-call accounting. Matching before/after values or sequential stops alone
prove no continuity/distributed checkpoint. Unknown custody fails; known inactive
fences deny. Unavailable/Unsupported reject before effects; Indeterminate retains
spent allowance/evidence and obligations, then stops without retry/reset/release.
Timeout, process death, failure and dropping model values do not release obligations.
The descriptive ceiling grants no paid call/reservation/preflight allowance. Matching
views grant no signing, dispatch, restart, release, capture completion or restore/payment
settlement. Terminal replay invokes no provider. See
[the requirement schema](../contracts/consistency-requirement.schema.json),
[typed current port](../contracts/consistency-port.json) and
[maintained boundary](../extraction-boundary.md).

Fresh native cases cover strict records and independent requirement/request binary
hashes, original plan/allowance sensitivity, challenge/boundary/fence/revision binding,
inactive/non-stopped denial, every member of a multi-target selection, full inventory
drift including an unselected parent, exact selected/lane mismatches, canonical target
aliases/duplicates and 1,024-target admission. Persistence cases cover no downgrade/
replacement, original-plan presence, lock contention, raw bounds, malformed/rebound
records, symlinks and replaced layouts. A public local provider fixture reopens original
requirement and spent pending journals, rejects stale/wrong-fence results and preserves
exact original authority, journal bytes, requirement and integration-owned obligation
fixture bytes across all typed failures/drop. These prove local contracts and recovery,
not actual management/application behavior or continuous fence custody.

Targeted checks passed: 153 unit tests and twelve public integration tests (165 total),
warning-denied Clippy/rustdoc, Rust 1.91.0 all-target/all-feature compilation, formatting,
diff checks and standalone Cargo package verification. The changed unselected-parent
regression also passed in the 13-case focused consistency suite. The archive contains
110 exact current Rust source/test files, the exact IC wire JSON fixture, current README
and exact regular MIT license; verified current archive/source remain under
`target/package/`. Previous snapshot-read archive/source remain retained under
`target/consistency-package-evidence.k2rfq3zw/`, along with earlier snapshot-read logs.
Current logs are `target/consistency-focused-tests.log`, `target/consistency-tests.log`,
`target/consistency-clippy.log`, `target/consistency-msrv.log`,
`target/consistency-docs.log` and `target/consistency-package.log`.
Schema examples/negative cases, independent binary digests, exact fresh source references,
prior Rust/provenance preservation and archive contents passed consistency checks.
No broad CI/release gate ran.

[Fresh consistency provenance](../consistency-source.json) records six inspected Canic
files/sections and 180 exact consumer references. Canic HEAD was
`3978e02d28fea9022c7a6e84a7ec6d6e4a0d4af3` with dirty working-tree source. Exact
hashes identify inspected bytes, not application qualification. Quiescence choices,
request/receipt binding and negative cases are adapted; CrashConsistent/RootCoordinated
names, accepted/Proven flags, expiry and parent-derived application units are not copied.
All source consumers remain unchanged. Earlier tracked provenance and source/lock/license
bytes are preserved, including the prior [snapshot-read provenance](../snapshot-read-source.json).

## Retained boundaries and remaining scope

Snapshot-read contracts retain independent exact list payload/challenge binding, actual
context/target/snapshot visibility and pure controller/public/exact-viewer paths. The
1,024 descriptive call ceiling grants no spending; viewers/controllers are bounded to
10 canonical unique principals. Unknown controllers cannot establish controller access;
independent public/viewer evidence needs no controller projection. Permission evidence
cannot settle a lost observation or replenish original authority. Native snapshot-read
qualification is retained; no metadata/data codec or live provider exists.

Earlier artifact/staging/durable publication, bounded JSON, layout/journal exclusion,
restore reference retention, owned command custody, local download/attempt journals,
canonical inventory/selection, explicit graphs, immutable operation plans and pure
retained-journal progress remain implemented. Their exact bounds/contracts are in
[the maintained boundary](../extraction-boundary.md) and AGENTS.md. Pending paid replies
stay pending, missing journals never mean zero consumption, and allowances never refund/
replenish. Applied prerequisites cannot prove cross-journal actual dispatch chronology
or full terminal/reference-release admission. No prune/reference release exists.

The six-method exact IC host-ingress codec and separate membership/control ports remain.
They bind current evidence to original intent/challenge with pure checks but provide no
live authentication, controller custody, continuity/fence or dispatch permission. Load
origin control, same-ID/same-release safety and external-obligation disposition remain
separate requirements. Every earlier [source provenance](../extraction-boundary.md)
remains retained. This batch adds no dependencies, lock changes, Canic imports, sibling
patches, unsafe code or shared target. Rust remains 2024, development 1.99.0 and MSRV
1.91.0. Maintainers own releases; see [development](../development.md) and
[releasing](../releasing.md).

Full B1/B2 and independently usable backup/restore remain unestablished. Application
fence acquisition/release/uncertain-effect recovery and actual same-release restore
safety still precede runners. Real membership/control/read/consistency/restore-safety
providers, transfer/
response codecs, selected backend snapshot/lifecycle qualification, bounded authenticated
calls and lost create/upload/load reconciliation remain necessary. Prior per-call
observation spending, actual cross-journal chronology, complete execution/restore journals/
manifests, terminal reference release, prune, transport and CLI remain proposed. Follow
[the design](../extraction-design.md) for sequencing. Canic adoption and live effects
need separate instructions; the no-commit rule remains.
