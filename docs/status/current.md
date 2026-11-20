# Current handoff — 2026-11-20

## Toko Tooling 0.45.23, CI network settings and product version ownership

Both workspaces select 0.45.23. The shared lint audit covered all five manifests and reported zero discrepancies. The dependency-pin and inheritance audit passed. The hygiene pass consolidated the chunked HTTP fixture builder and split long qualification evidence messages for readability.

CI inherits the caller’s Cargo network settings after preparing the locked dependency cache. Testkit independently verifies the installed simulator. The root workspace owns the published product version through `workspace.package.version`; the private qualification workspace keeps its own package identities.

## 0.105.12 published

[ic-backup 0.105.12](https://crates.io/crates/ic-backup/0.105.12) shipped from tag `v0.105.12` at `19e4240`. All 247 native tests, five doctests and 11 PocketIC journeys passed. Registry readback verified the checksum, commit, license and all 181 tracked package files. A consumer built against the published version. Evidence is under `target/evidence/release-0.105.12/`.

## Cargo network policy

Development and publication commands inherit Cargo network settings. In 0.105.12, prepared CI qualification steps selected offline Cargo explicitly, including the child consumer build. Platform tracks the corresponding Makefile change in [issue #35](https://github.com/dragginzgame/ic-platform/issues/35).

## Toko Tooling 0.45.21

Both workspaces select 0.45.21 and inherit the shared Clippy setting for spaced type headers. Rust return expressions are separated from preceding statements, and the retained Root codec and qualification authority adapters use named headers. Backup exports no canister endpoints and has no Wasm-specific guards. The Unix store depends on Unix metadata and directory operations in Host FS. For `wasm32-unknown-unknown`, the library compiles with an empty normal dependency graph and the crate documentation builds with warnings denied. Wasm checks are under `target/evidence/tooling-0.45.19/`; hygiene checks are under `target/evidence/tooling-0.45.21/`.

## 0.105.11 published

[ic-backup 0.105.11](https://crates.io/crates/ic-backup/0.105.11) shipped from tag `v0.105.11` at `4377bf9`. All 247 native tests, five doctests and 11 PocketIC journeys passed. Registry readback matched the checksum, commit, license and all 181 tracked package files. A consumer built against the published version. [Backup #4](https://github.com/dragginzgame/ic-backup/issues/4) is closed. Evidence is under `target/evidence/release-0.105.11/`.

The procedure suite now lives under `operation::tests::procedure_tests`. All 64 operation test cases remain in the inventory; only the five procedure case paths changed. Before and after lists are under `target/evidence/issue-4/`.

## 0.105.10 published with Toko Tooling 0.45.16

[ic-backup 0.105.10](https://crates.io/crates/ic-backup/0.105.10) shipped from tag `v0.105.10` at `f86088a`. All 247 native tests, five doctests and 11 PocketIC journeys passed. Registry readback matched the checksum, commit, license and all 181 tracked package files. A consumer built against the published version. Evidence is under `target/evidence/release-0.105.10/`.

Both workspaces select 0.45.16. The Rust source has no Wasm target guards. The Unix store boundary and its Cargo dependency selectors are required by Host FS directory custody, which uses Unix metadata and file descriptors. Consolidating capture status and HTTP lookup test helpers reduced Rust `#[cfg(test)]` attributes from 92 to 75. Moving recorder fixture methods and legacy framing helpers into tests and removing transfer getters used only by tests brought the count to 66. Consolidating the remaining capture advance helpers under their test modules brought it to 32. Capture advance retains four module guards and one grouped test-only re-export. Local validation evidence is under `target/evidence/tooling-0.45.16/`.

## 0.105.9 published with Toko Tooling 0.45.15

[ic-backup 0.105.9](https://crates.io/crates/ic-backup/0.105.9) shipped from tag `v0.105.9` at `00449ce`. All 247 native tests, five doctests and 11 PocketIC journeys passed. Registry readback matched the checksum, commit, license and all 178 tracked package files. A consumer built against the published version. Evidence is under `target/evidence/release-0.105.9/`.

Both workspaces select 0.45.15. The fixture macros implement traits for existing types and the host library has no Wasm-only canister state. Their packages inherit the tooling marker as an ordinary dev-dependency. Child module declarations precede imports throughout the Rust source. The manager capacity regression checks the precise refusal and remaining capacity, and the setup fixture exercises bare Bearer and JSON payload redaction. Validation logs are under `target/evidence/tooling-0.45.13/`.

## 0.105.8 published with Toko Tooling 0.45.10

[ic-backup 0.105.8](https://crates.io/crates/ic-backup/0.105.8) shipped from tag `v0.105.8` at `5eb580bac6c203e61e8f0af70e9224cb16a9b848`. Registry readback verified the checksum, commit, license and all 178 tracked package files. A consumer built against the registry release. Release evidence is under `target/evidence/release-0.105.8/`.

Both workspace pins and locks select 0.45.10. Backup retains its Unix store boundary for directory-handle custody and file permissions. Testkit download failures use the shared bounded diagnostic reader, and the CI setup regression covers bare and embedded access-key values. The local gate passed both workspace Clippy checks, 247 native tests, five doctests, all 11 PocketIC journeys, archive consumer and publication caller checks. Evidence is under `target/evidence/tooling-0.45.10/`.

## Published Canic integration — selected persistent counter

Canic v0.224.0 qualifies the published host adapter with Backup 0.105.6 and IC Host Sign 0.1.0. The installed direct route passed with a disposable keyring-backed secp256k1 identity. The Root-managed route recovered the original take and load after lost replies and host restart. With Root as sole controller, the selected counter was restored stopped, accepted under a fresh authorization and explicitly opened. The consumer scope is one same-release persistent counter without timers, external work or additional participants. These are installed test results; deployment of a live application with operator credentials is a separate operational step. See [external signer qualification](https://github.com/dragginzgame/canic/issues/5), [Root executor delivery](https://github.com/dragginzgame/canic/issues/6) and [application fencing qualification](https://github.com/dragginzgame/canic/issues/7).

## 0.105.7 published with Toko Tooling 0.45.8

[ic-backup 0.105.7](https://crates.io/crates/ic-backup/0.105.7) shipped from commit `2234509cdef1024414c25fdfff3861812319c257`. The release gate passed 247 native tests, five doctests and all 11 PocketIC journeys. Registry readback verified the checksum, embedded commit, license and all 178 tracked package files. A consumer built against the registry release. Evidence is under `target/evidence/release-0.105.7/`.

Both workspace pins, lockfiles and the 62-file managed snapshot agree on 0.45.8. The hygiene pass grouped Rust imports, separated adjacent functions, and moved test declarations and shared fixtures beside their owning tests. All 70 relocated wire fixture files matched the previous revision byte for byte. The publication caller now accepts unstaged file moves and checks the exact package and version in the timing journal. Local validation evidence is under `target/evidence/tooling-0.45.8/`.

## 0.105.6 published with Toko Tooling 0.45.2

[ic-backup 0.105.6](https://crates.io/crates/ic-backup/0.105.6) and tag `v0.105.6` select release commit `b1bda32b882f44146b03d6b222b1c86428db4d40`. Both workspaces pin Toko Tooling 0.45.2. The canonical exporter retained twelve entrypoints and 62 managed files. Post-push review confirmed that the only changed managed helper is `SetupDiagnostics.pm`. The Backup caller regression exercises assignment, URL fragment and access-key redaction while preserving the original failure status.

All 247 native tests, five doctests, 11 direct PocketIC journeys (106.52 seconds), strict Clippy and archive checks passed. Independent readback matched the registry SHA-256 `fc99dc21176ef5640cda1201a6322148ec09f457667994299d9ce72ae44e62f9`, embedded commit, all 178 tracked package files and the license. A fresh registry-backed consumer built successfully. Evidence is under `target/evidence/release-0.105.6/` and `target/publication/publish.selection.hTeeOT`. Hosted CI runs asynchronously.

## 0.105.5 published

[ic-backup 0.105.5](https://crates.io/crates/ic-backup/0.105.5) is published. The annotated tag `v0.105.5` points to release commit `f5052caf6ec7c073a120d60e3a6aa105ecb92bbc` on `origin/main`. The release gate passed 247 native tests, five doctests and all 11 direct-controller PocketIC journeys (104.14 seconds). The separate Root/child host-restart witness below used the local Canic supplier. Its production deployment remains with Canic.

Independent registry readback matched SHA-256 `4403bbeb216671e964aff1b853237bcdc8065dda4955411d260cc820046f2f3d`, the release commit, all 178 tracked package files and the license. A fresh registry-backed normal consumer built successfully. Post-push review confirmed that Root recovery uses the existing journal, a separately budgeted lookup, and the original execute binding. The normal dependency graph excludes Canic management, Testkit and PocketIC. Release evidence is under `target/evidence/release-0.105.5/`, with single upload attempt `target/publication/publish.selection.sShqZ1`. Hosted CI runs asynchronously.

## Root execute and original-result recovery — local qualification

The public capture and restore loops now support an approved Canic Root through
`BackupConnection::with_root_manager`. Backup derives the typed Platform effect,
inner method, argument length/hash and absolute execute expiry. The Canic-owned
codec prepares the execute and full-binding lookup frames. Backup signs, verifies
and records each outer ingress separately. Reopening with
`BackupStore::with_root_codec` selects the original protocol for offline replay.
The child status checks require Root as sole controller. Direct approvals retain
their existing bytes and direct recovery procedure.

Linux PocketIC exercised the local unpushed Canic Root and child with a real
host signer, Root as sole child controller and the Canic protocol's checked
decoder. A loopback proxy retained then dropped the take and load responses.
After each loss the host process exited. A fresh process reopened the approval,
observed `NeedsAttention`, and recovered the original result with a separately
signed lookup. Wire evidence records one take, one load and two lookups.
Restore performed full readback and returned the stopped child to its original
stable bytes after an independent fixture mutation. Another process verified
completion after the server shut down. Captured payload was 1,690,546 bytes
across seven 256 KiB-or-smaller reads. The 20 MiB profile is direct-only.

The local harness, its dependency lock, HTTP evidence, original journals and
certificates are under `target/evidence/root-host-consumer/`. The final run is
`run-1794996922952207507`. The harness selected the unpublished Canic protocol
read-only outside both tracked workspaces. The installed Root and child Wasm
SHA-256 values are `d7302ff0f798570e0a8cf18192e2b6cd5d694b0a0e1a89e251b1fb773f7f4017`
and `b062e14bc119ee14f194eac36b1819cc68e3d29574815676410075de2957a922`.
The working protocol module SHA-256 was
`7db42d24c980e1313c0f2e31754d98b2ac25ab3528be2f4af1cad5ec3b362d60`
at Canic base `d652c55e4cc455a642f44645bf2e16a6d08baefc`. Qualification used
fixture signing and application permission. The later Canic v0.224.0 consumer qualification above covers the published
supplier, external signer and selected application fence. This Root witness
is local and outside hosted CI. Backup shipped the host boundary in 0.105.5.

Five small test fixtures retain the approved intent, both signed Take/lookup
envelopes, the lookup evidence and the independently decoded inner reply. The
native replay regression authenticates the real certificates without an IC
server and rejects corruption of each evidence input. Final checks passed:
247 native tests, five doctests, 11 direct PocketIC journeys (104.60 s), both
strict Clippy checks, Rustdoc, formatting, local links and diff checks.

## Toko Tooling 0.45.1 adoption

Both workspace pins and locks select Toko Tooling 0.45.1, source
`9497997f8abdb72aea01b32da6d0a092c0ed6ed4`, registry archive SHA-256
`dc38b1d6932310739c21ab8a821c811d4bf43e306f89751faea38638c8bc1373`.
Canonical preview/update retained the pending CI toolchain helper and selected
`SetupDiagnostics.pm`: twelve entrypoints / 62 admitted managed files. No managed
file or receipt was edited by hand. Relative to saved starting locks, only the
Tooling version/checksum changed in each workspace; existing supplier updates
are preserved. Testkit/Process and product source are unchanged by this refresh.

Backup's setup wrapper now delegates command execution, bounded stream capture
and redaction to the shared module. Its own exclusive artifact writer retains
phase/status evidence, the 8 KiB failure excerpt and original failure status;
success stores a status-only receipt. CI artifact selection/retention stays with
Backup. Consumer checks cover early failure, redaction/bounds, empty output,
collector failure on success/failure, successful status-only retention and
redirected evidence refusal. Both real workflow step bodies pass their eight
toolchain-caller cases with rustup substituted, and publication-caller fixtures
pass with registry effects substituted. No compiler installation or live upload
was performed by those fixtures.

Pending 0.105.5 includes this tooling refresh and the preceding CI helper change;
package versions remain 0.105.4. Work is uncommitted/unpublished. Hosted CI was not
awaited, and product/IC suites were not rerun for this setup-only change. Evidence:
`target/evidence/tooling-refresh/`.
Snapshot admission, Perl syntax, Actionlint, formatting, local links and diff
checks pass; both workspace locked metadata checks also pass.

## CI toolchain helper adoption

Both Backup CI jobs now call the shared toolchain-file helper through the existing
setup diagnostics wrapper. The invoking step exposes prepared host `yq` and `jq`
via `.tools/host/bin`; channel/profile/components/targets have one owner in
`rust-toolchain.toml`. Toko Tooling remains 0.45.0. Canonical preview/update retained
the ten existing entrypoints and selected the helper with its declared companion:
eleven entrypoints, 61 admitted managed files. No managed bytes were edited by hand.

Actionlint and snapshot admission pass. Disposable checks execute both actual YAML
step bodies with real helper/JSON/TOML tools and a stubbed rustup: current selection,
changed profile/components/targets, original failure status/diagnostics and invalid
policy refusal all pass (eight cases). No compiler was installed, no product/IC
tests were rerun for CI-only wiring, and hosted CI was not awaited. Evidence is
under `target/evidence/ci-toolchain-helper/`.

Query was inspected read-only at base `51666c45c77e614972e951363d3f0b556999fbe3`
with a dirty workflow: both corresponding steps already use this helper and PATH
through Query's own diagnostics wrapper. No sibling edit or duplicate issue was
needed, and its delivery/hosted status is not established by that inspection.
Pending 0.105.5 records the compatible CI change; package versions remain 0.105.4.
Work is uncommitted/unpublished.

## 0.105.4 published

**[ic-backup 0.105.4](https://crates.io/crates/ic-backup/0.105.4) is published.**
Release commit `0e7358b4c9e5def07fe96a69d493aa76fbf78dbf` and tag `v0.105.4`
are pushed to `origin/main`. Tooling 0.45.0 and private Testkit 0.110.0 /
Process 0.81.0 are delivered. The retained Platform 0.34.15 / bytes 1.12.2 locks
are covered by this release's caller checks. The manager-contract proposal is
delivered as a proposal; Backup #1 / Canic #6 remain open and no manager route
is enabled. Product APIs, stored formats and recovery evidence are unchanged.

The complete local gate ran once: **240 native tests**, five doctests,
**11 actual-IC journeys (104.17 seconds)**, both strict Clippy checks, Rustdoc,
tool/snapshot/format/link admission and archive/isolated normal-consumer checks
pass. Earlier changed-tooling caller qualification remains retained separately.
Hosted CI was not awaited. Single successful upload attempt:
`target/publication/publish.selection.p044da`; release evidence:
`target/evidence/release-0.105.4/`.

Independent registry readback matches SHA-256
`c7682b24dc23d1214ad9fba17595dfefc10bb1260a887d9429558b58e8c2cbe7`,
its embedded release commit and all 168 tracked package files. A fresh
registry-backed normal consumer builds without Testkit/PocketIC.

Post-push review of the adoption and manager proposal found no new production
path, journal or authority input. Canonical export changed only managed baseline
bytes; executable selectors and publication dispatch remain unchanged. The
proposal explicitly separates signer/controller, original outer/inner outcomes
and both-hop spending without adding an unsupported API or qualification claim.
Earlier development evidence follows.

## Toko Tooling 0.45.0 / Testkit 0.110.0 adoption

Both workspaces select exact Toko Tooling 0.45.0. Canonical preview/update and
offline admission pass with the same ten entrypoints / 59 managed files. Only
the baseline document changes; managed executables and tool selectors are
byte-identical. The delivered Backup #3 flow already matches the new consistent
validation/delivery policy. No new release runner, evidence cache or skip flag.

Qualification selects Testkit 0.110.0 and Process 0.81.0. Testkit source is
byte-identical to 0.109.0, but its server report exposes the new Process identity;
the direct Process declaration must align. Its finite-stdin addition is not used
by Backup's null-input server path. PocketIC remains 16.1 and qualification
thiserror remains 2.0.18. Setup records verified archives/source provenance.
Saved-lock comparison shows only Tooling changed in root, and Tooling/Testkit/
Process in qualification; other supplier selections are preserved.

Offline tool/snapshot admission, qualification all-target Clippy, authenticated
existing-server setup/check and the actual publication-caller fixture pass.
All 11 actual-IC journeys pass (105.39 seconds), including 20 MiB, separate-process
recovery and retained server cleanup reports. Both locked metadata graphs admit
one Tooling identity; qualification has one Testkit and one Process identity.
Formatting, documentation links and diff checks pass. Evidence is under
`target/evidence/tooling-testkit-adoption/`. No macOS or live publication claim.

The manager-contract proposal and pre-existing Platform 0.34.15 / bytes 1.12.2
locks remain preserved. Product source/APIs/formats are unchanged. Pending
0.105.4 describes these compatible development/qualification changes; manifests
remain 0.105.3. Work is uncommitted/unpublished, and no hosted CI was awaited.

## Backup #1 manager-contract proposal

The maintainer requested progress on the remaining manager route. A concrete
semantic proposal is now in the sibling integration contract under
`Manager exchange proposal — 2026-11-17`. Read-only review of Canic base
`d652c55e4cc455a642f44645bf2e16a6d08baefc` and identified dirty engine bytes
found no snapshot executor/receipt endpoint. The proposal separates host signer
from manager controller, pins operation/approval/step/arguments and generation,
defines execute versus finite lookup and exact retained inner outcomes, and
records trust, wire ownership, byte/cycle/request accounting and IC acceptance.

Two essential refusal rules: outer rejection is not proof of inner rejection,
and changed generation/approval must not create another slot for the original
operation/step. The proposed initial profile allows one explicit manager lookup
per pending step, with separate original-ingress observation accounting. No
production route/API, format, dependency or effect is added before the actual
Canic protocol and consumer exist. The agreement, endpoint/schema and portable
protocol ownership remain outstanding on Backup #1 / Canic #6; neither issue is
complete. Canic #7 owns the separate fence/reopening journey.
The concrete proposal is posted on
[Backup #1](https://github.com/dragginzgame/ic-backup/issues/1#issuecomment-6961354980)
with its protocol-owner action on
[Canic #6](https://github.com/dragginzgame/canic/issues/6#issuecomment-6961355422).

Focused legacy submission/attempt-journal review and exact source provenance are
recorded with the proposal. Documentation checks pass; no runtime tests were
activated for this design-only work and no IC qualification is claimed. Work is
uncommitted/unpublished. The pre-existing Platform 0.34.15 / bytes 1.12.2 lock
updates are preserved, outside this design work and not newly qualified here.

## 0.105.3 published

**[ic-backup 0.105.3](https://crates.io/crates/ic-backup/0.105.3) is published.**
Release commit `709d8eeb686c62d8181b6a0b00cb51252df6f5e4` and tag `v0.105.3`
are pushed to `origin/main`. Backup #3's one-pass publication flow is delivered;
product APIs, algorithms, stored formats and recovery evidence are unchanged.

The changed-caller fixture passed before the full release gate. That gate ran
once and passed **240 native tests**, five doctests, **11 actual-IC journeys
(105.65 seconds)**, both strict Clippy checks, Rustdoc, tools/format/links and
archive/isolated normal-consumer admission. The caller fixture is no longer nested
in that gate. Hosted CI was not awaited. The single successful upload is
`target/publication/publish.selection.apASRP`; release evidence is under
`target/evidence/release-0.105.3/`.

Independent registry readback matches SHA-256
`6fa09c8c9e54b0b44ba86f65b63aef6e022198b563638625b93161b4f84bc041`,
the embedded release commit and 168 tracked package files. A fresh registry-backed
normal consumer builds without Testkit/PocketIC.

Post-push review of the delivered Make/CI/caller/documentation changes found no
new recovery path, journal, skip flag or proof cache. Upload retains the full
product gate and fresh archive/source/registry admission; caller regressions have
one CI owner. Backup #3 is complete and delivered. Backup #1 / Canic integration
remains the outstanding product work. Earlier development evidence follows.

## Backup #3 local completion

The full publication gate no longer runs for optional Cargo dry-run. `publish-check`
selects a linted archive/registry preflight; `publish` retains complete product and
actual-IC qualification, archive/normal-consumer checks and fresh upload admission.
Check-plus-upload therefore runs one complete gate. The local caller-regression
fixture moves from every release to scaffold CI and explicit changed-tooling
qualification. The publication guide maps leaf commands, guarantees and triggers.

This removes unnecessary work before adding reuse: no proof cache or skip flag is
introduced. Earlier check output cannot authorize skipping upload checks; a full
standalone `publication-verify` followed by `publish` still repeats that explicit
readiness check. Normal delivery calls `publish` directly. Installed Toko 0.44.11
and managed bytes are unchanged; no sibling source edits or supplier blocker.

Real preflight tool admission/product Clippy passes. Disposable actual Make/shared
dispatch under `--jobs=2` passes fixed selection/policy, missing/tampered check
records, dirty/changed source, failed gates, retained failed dispatch and surviving
lock checks. The warm simulated check/upload pair measured 0.115 / 0.117 seconds;
validation cost and Cargo effects are substituted, so no native/IC runtime or
delivery speedup is claimed. Product APIs/algorithms/tests are unchanged; existing
240-native/11-IC release evidence remains applicable, not a fresh execution.
ShellCheck, Actionlint, formatting, links and diff checks pass. Evidence is under
`target/evidence/publication-flow/` and the owning disposable fixture directories.
Pending 0.105.3 records the compatible contributor-workflow change; manifests stay
0.105.2. Work remains uncommitted and unpublished; hosted CI was not awaited.
[Backup #3](https://github.com/dragginzgame/ic-backup/issues/3) is closed after
local implementation and qualification; its comment preserves delivery status.

## 0.105.2 published

**[ic-backup 0.105.2](https://crates.io/crates/ic-backup/0.105.2) is published.**
Release commit `fbfda5f22923a8fe96cf58f199273ac1fbcefaa8` and tag `v0.105.2`
are pushed to `origin/main`. This releases Backup #2's direct test-SDK removal,
independent fixture preservation and private Testkit 0.109.0 adoption. Production
behavior, public APIs, stored formats and original recovery evidence are unchanged.

The complete local gate passes: **240 native tests**, five doctests, **11 actual-IC
journeys (101.97 seconds)**, both strict Clippy checks, Rustdoc, tool/format/link
admission, archive/isolated normal consumer and publication-adapter regressions.
Hosted CI was not awaited. The single successful upload attempt is
`target/publication/publish.selection.2my56T`; release evidence is under
`target/evidence/release-0.105.2/`.

Independent registry readback matches SHA-256
`120ff34287bb1e3987bbe788ea9068aaefb6fc248507a9287248f232c6ae639a`,
its embedded release commit and 168 tracked package files. A fresh registry-backed
normal consumer builds without Testkit/PocketIC. All 68 independent fixtures and
their original provenance are included in the archive.

Post-push review of the delivered changes and directly affected test callers found
no new recovery path, journal, authority input or production SDK dependency.
Backup #2 is complete and delivered; Backup #1 / Canic integration remains.
The following sections preserve preceding development and release evidence.

## Testkit 0.109.0 adoption

The private qualification workspace selects Testkit 0.109.0. Verified archive
provenance and source comparison are recorded in setup. The working lock already
selected 0.108.1; this update changes only Testkit's version/checksum. The new
held-reply clock feature is unselected; server code and dependencies are unchanged.
The pending Backup #2 cleanup is preserved. Product version remains 0.105.1;
pending 0.105.2 notes include both test-only changes. No commit or publication.
Qualification all-target Clippy, prepared-server setup/offline admission and the
actual-IC interrupted-take/observation/offline-replay case pass (1.40 seconds).
Formatting, documentation links and diff checks pass. The preceding 11-journey
101.75-second result is retained, not reported as a fresh full-suite run; selected
server sources are unchanged. Evidence: `target/evidence/testkit-refresh/`.

## Backup #2 local completion

Removed direct Candid/management SDK test declarations and imports. Independent
malformed/schema inputs and expected request bytes now use 68 retained fixtures
with original source/test/schema provenance and length/SHA-256 admission. Normal
policy fixtures use published Platform encoders. Legacy literal goldens remain
unchanged. The transfer contract records focused legacy review and case mapping.
The release-scoped SDK exception is removed from AGENTS.md.

All 42 focused checks pass (status 10, snapshots 8, metadata 12, readback 8,
restore procedure 3 and retained-fixture integrity 1). Both workspace Clippy
checks pass; all 11 actual-IC journeys pass in 101.75 seconds, including 20 MiB
and separate-process recovery. Locked metadata admits both graphs with no direct
consumer SDK declarations; no supplier version changed. Package-file admission
includes all 68 vectors. Formatting, documentation links and diff checks pass.
Evidence is under `target/evidence/sdk-fixture-retirement/`; the first golden
replay exposed lowercase JSON status versus Protocol's Rust variant spelling,
resolved with an explicit fixture-label mapping before the passing recheck.

[Backup #2](https://github.com/dragginzgame/ic-backup/issues/2) is closed after
local implementation and qualification; its comment records the delivery status.
Pending 0.105.2 notes describe this compatible test-only cleanup; package versions
remain 0.105.1. Work is local, uncommitted and unpublished. Backup #1 / Canic
manager execution and fenced recovery remain the integration work.

## 0.105.1 published

**[ic-backup 0.105.1](https://crates.io/crates/ic-backup/0.105.1) is published.**
Commit `dba14cd3dda9c0d1eff32414b57b86b11f69a97c` and tag `v0.105.1` are
pushed to `origin/main`. The complete local gate passes: 239 native tests, five
doctests, 11 actual-IC journeys (103.14 seconds), both strict Clippy checks,
Rustdoc, tool/format/link checks, archive/isolated consumer and publication-adapter
regressions. Hosted CI was not awaited.

Single successful publication: `target/publication/publish.selection.MlBegG`.
Registry readback matches SHA-256
`577733b324a255c92e711cd27ca0f509c85b1baac794030f52e254c48ae7f06e`,
the embedded release commit and 97 tracked package source files. A fresh
registry-backed normal consumer builds without Testkit/PocketIC. Evidence is
under `target/evidence/release-0.105.1/`.

Moved the digest module's sole inline unit-test body to its owning `digest/tests.rs`,
leaving ordinary `#[cfg(test)] mod tests;` wiring. Assertions, test identity,
visibility and production behavior are unchanged. A maintained-source scan found
no other inline test module bodies. The dev-only SDK exception remains unchanged.
Both locks already selected zeroize 1.9.2 at the start of release preparation;
that compatible patch is retained and qualified in the gate. Post-push review
confirmed the test body and assertions were preserved with ordinary module
discovery; no new recovery path, journal or authority input was introduced.
Backup remains parked pending the existing Canic integration work.
The owning digest test, formatting, documentation links and diff checks pass.

## 0.105.0 published

**[ic-backup 0.105.0](https://crates.io/crates/ic-backup/0.105.0) is published.**
Release commit `76f0adfef047e205741c824388f85bc4408156a9` and tag `v0.105.0`
are pushed to `origin/main`. This completes the Platform consumer port with
Platform 0.34.13 / Protocol 0.3.5 / Types 0.21.2 and private Testkit 0.108.0.
The packaged README records the breaking public type/error migration; stored
formats and original evidence are unchanged. Canic remains unqualified.

The complete local publication gate passes: **239 native tests**, five doctests,
**11 actual-IC journeys (104.93 seconds)**, both strict Clippy checks, Rustdoc,
format/link/tool admission, archive/isolated consumer and publication-adapter
regressions. Hosted CI was not awaited. The single successful upload attempt is
`target/publication/publish.selection.WRRZlm`; release evidence is under
`target/evidence/release-0.105.0/`.

Independent registry readback matches SHA-256
`396a9b42064405de02c88ff3ebea3153f66cbe16078a66002517efc628782596`.
The embedded commit matches the tag and all 96 tracked package source files match.
A fresh registry-backed normal consumer builds without Testkit/PocketIC, then
successfully reopens pre-port isolated and 20 MiB capture/restore stores offline
using their original IDs and approvals. No server or live authority is needed.

Post-push review of the delivered migration and directly affected callers found
no new competing recovery path, journal or authority input. The disclosed dev-only
SDK fixture debt remains on Platform #11 under the release-scoped exception in
AGENTS.md; signing, manager execution and fenced recovery remain Canic work.
Backup can remain parked at this released direct-controller scope pending those
concrete consumers.

The sections below retain the preceding development and 0.104.0 release evidence.

**[ic-backup 0.104.0](https://crates.io/crates/ic-backup/0.104.0) is published.**
Release commit and tag `v0.104.0` are pushed to `origin/main` at
`2941ca1324a95c1afdb57e33cef79362c28b0731`.
The minor bump covers public snapshot-data/restore decode errors now carrying
Protocol's codec error type; stored formats and retained evidence are unchanged.
The packaged README documents consumer migration. Full Platform adoption remains
unfinished and is not a claim of this release.

The complete publication gate passes: **237 native tests**, five documentation
examples, **all 11 actual-IC journeys (102.26 seconds)**, strict Clippy in both
workspaces, Rustdoc, tools/format/link checks, archive/isolated normal consumer and
publication-adapter regressions. Actual-IC cases include 20 MiB capture/restore,
original-call recovery and separate-process replay. Hosted CI was not awaited.
The single successful upload attempt is `target/publication/publish.selection.bcLVA8`;
release evidence is under `target/evidence/release-0.104.0/`.
Independent registry download has SHA-256
`233bd3de78ea8aab3040eada7f44a5396ef063aff2c32e30c7d5d80955759394`.
Its embedded source matches the tag, all 96 tracked package source files match,
and a fresh registry-backed consumer builds without Testkit/PocketIC and replays
completed 20 MiB capture and restore offline. The post-push review found no new
competing recovery path, journal or authority input in this release.

## Parking checkpoint — 2026-11-17

The implemented direct-controller scope has completed its Platform port and
Testkit adoption. No additional Backup implementation blocker was identified in
the adoption closeout. Keep this scope stable until the Canic consumer supplies
new requirements; do not add speculative manager adapters or another recovery
path. This is a development stopping point, not full Canic qualification.

The working tree remains uncommitted and unpublished, including the full port,
qualification CI, directory/module moves and Testkit update. Pending 0.105.0 is
the next delivery unit; package manifests still say 0.104.0. On an explicit release
request, follow the [publication workflow](../development/publication.md), apply
the version bump and required release checks, then deliver. Hosted CI remains
asynchronous. Existing focused evidence below is not a completed 0.105.0 release
gate, and no release was attempted during this closeout.

Remaining integration belongs to the existing Canic issues listed below:
bounded signing, authenticated manager execution/original-result lookup,
continuous application/rewind fencing and actual consumer qualification.
Production HTTPS, actual-IC macOS and physical power-loss qualification remain
explicit limits. Quadratic history replay is a known cost, not a newly introduced
blocker or a promise of larger-capacity support beyond the qualified 20 MiB profile.

Closeout corrected stale supplier wording in this handoff and the local agent
overlay. No product source, dependency selection or stored format changed;
the preceding adoption evidence remains applicable. Documentation validation
passes (124 local references), as do snapshot admission and diff whitespace checks.

## Current Platform adoption — 2026-11-17

Native CI now includes qualification-workspace Clippy on all three native hosts,
plus locked Testkit preparation/offline admission and the actual-IC `capture`
target on Linux. The job budget is 45 minutes to cover cold qualification builds.
Setup reuses sanitized diagnostics and failure artifact handling. Hosted execution
is not yet observed; local caller evidence below remains applicable. No publication
gate or requirement to wait for hosted CI was added.
Actionlint, setup failure/collection regressions and documentation links pass;
the unchanged qualification commands reuse the local lint and 11-journey evidence
below. No hosted run was dispatched or awaited.

The full port was qualified with **Platform 0.34.11 / Protocol 0.3.3 / Types 0.21.2**;
the current locked selections are recorded below.
Production management arguments/replies, source-read DTOs and public identities
use the new packages. Certificate authentication uses bounded decode, checked
root keys and explicit original target/time/traversal limits. Neither lock or
manifest retains Platform 0.33 / Types 0.20, including dev dependencies. Private
snapshot/metadata schema walkers and unreachable typed global-range checks are
removed; exact float bits, option semantics and original evidence remain.
[Setup](../development/setup.md#current-protocol-033--platform-03411-adoption)
records verified registry provenance.

Native fixtures now use Platform construction/signing and Protocol response
encoders. Independently authored malformed schemas and request comparisons use
Candid/Management Canister Types only as test dependencies; normal product feature
admission excludes fixture signing/encoding and the SDK management crate.
All 11 actual-IC journeys pass (113.23 seconds), including 20 MiB and independent
process recovery. Native capture, operation, restore and source cases pass with
focused rechecks for migrated fixture differences; exact logs are under
`target/evidence/platform-full-port/`. Both workspace Clippy checks, Rustdoc,
all five doctests, formatting and documentation links pass. Upload-metadata
arguments match independent SDK bytes for NaN payload bits, signed zero and the
full u128 range. Final archive/isolated normal-consumer verification passes at
`target/package-consumer/check.ljvB1t/`. The earlier archive consumer at
`target/package-consumer/check.ggE5dP/` also
reopens completed pre-port isolated and 20 MiB capture/restore stores using only
original ID/approval, without server, signer or live authority. Exact fixture paths
and commands are retained in `prior-replay.tsv` / `prior-replay.log` under the
port evidence directory.

Public type/error changes are recorded for pending **0.105.0** in the changelog
and packaged README. Package versions remain 0.104.0; no commit or publication.
Canic integration remains unqualified; this completes the Platform consumer port.

The private consumer now lives at `qualification/ic-backup-consumer`, matching
its Cargo package name. Workspace membership/path dependency, archive-consumer
script and documentation callers are updated. All four moved files are byte-identical.
Qualification all-target compilation, archive/isolated normal-consumer verification,
formatting and documentation links pass; no actual-IC suite was repeated for this
path-only change. Archive evidence: `target/package-consumer/check.YVjlAr/`.

The sole maintained Rust file/directory collision is removed: restore tests now
use `restore/tests/mod.rs`, with two relative fixture includes adjusted. Original
fixture bytes and module paths are unchanged. All 11 owning restore tests pass
with loopback access; formatting and a recursive source-layout scan pass.

## Testkit adoption — 2026-11-17

Testkit 0.108.0 is selected in the private qualification workspace. The published
archive checksum matches the registry; setup records exact source provenance.
Its selected server implementation is byte-identical to 0.107.4. Only the Testkit
version/checksum changed against the qualification lock saved at adoption start;
PocketIC 16.1, thiserror 2.0.18 and other selected dependencies are retained.
Qualification Clippy and existing-server preparation/offline admission pass. All
11 actual-IC journeys pass (106.93 seconds), including 20 MiB capture/restore,
interrupted transfers and separate-process offline recovery. Formatting and
documentation links pass. Evidence is under `target/evidence/testkit-next/`.
Hosted CI was not awaited; no commit or publication.

## Platform package identity recheck — 2026-11-17

Registry review confirms Platform 0.34.13, Protocol 0.3.5 and Types 0.21.2.
Both locks already contained these versions at this recheck's start. Exact
targeted Cargo updates confirmed availability; unrelated Windows dependency-edge
rewrites were reverted to the saved locks. No source, feature or minimum-version
change was needed. The newer changes affect unselected native signed transport,
Caffeine and general management APIs; Backup's selected source is unchanged.
Both workspace locked metadata and strict all-target Clippy checks pass. Existing
actual-IC evidence is retained, not reported as a fresh run. Setup records archive
provenance and evidence under `target/evidence/platform-park-refresh/`.

Each graph has exactly one registry identity for each of Platform, Protocol and
Types, with no old Platform generation or Backup dependency alias. Their different
version numbers are intentional independent release groups. Backup's direct
normal/build graph excludes Candid/management SDK/Testkit/PocketIC declarations;
Candid remains transitively owned by Platform's transport dependency.

The review found one remaining ownership cleanup: direct dev-only Candid and
management SDK imports used for malformed-wire fixtures and independent argument
comparisons. These are migration debt, not an approved dependency-boundary
exception. This corrects the earlier complete-cleanup implication; retain the
regressions while replacing those direct test uses with retained wire fixtures or
narrow supplier support. The follow-up is recorded on
[Platform #11](https://github.com/dragginzgame/ic-platform/issues/11).
No sibling edits, commit or publication were made.

## Selected suppliers and adoption evidence

The locks currently select Host FS/Artifacts 0.80.3, HTTP 0.80.0, Protocol 0.3.5,
Platform 0.34.15 and Types 0.21.2. Backup owns Reqwest 0.13.5 and its original
TLS/no-retry/redirect/proxy/deadline policy; Host bounds complete response bodies.
Qualification selects Testkit 0.110.0 and Process 0.81.0; both locks select Tokio
1.54.1. Platform 0.34.12 and Tokio were already selected before the Testkit update.
These Host selections
were present in the working locks during the port and are covered by its caller
checks. Fixture authority uses ordinary library imports. Original server logs,
process/cleanup evidence, journals and failed fixtures remain retained.
Setup downloads only explicitly into `.tools/testkit-server/`; old
`.tools/testkit/` assets remain untouched. The 20 MiB case has a finite
30-minute server lifetime. [Setup](../development/setup.md) owns preparation.

The earlier 0.104.0 release-time review qualified its then-selected suppliers and
Toko Tooling 0.44.11. That release used Protocol 0.2.8, whose management codec was
unchanged from the earlier 0.2.7 pass; the new port supersedes that selection.
Earlier evidence remains in `target/evidence/supplier-refresh/` and
`target/evidence/platform-recheck-current/`. Testkit 0.107.3 also resolves with
Protocol 0.2.7 when its optional codec feature is disabled; 0.107.4 was not required
for that compatibility. Their managed-server sources are byte-identical.

Consumer acceptance for [Platform #11](https://github.com/dragginzgame/ic-platform/issues/11)
and [Platform #12](https://github.com/dragginzgame/ic-platform/issues/12) is now
complete for Backup's implemented direct-controller profile. Acceptance updates
are posted on [Platform #11](https://github.com/dragginzgame/ic-platform/issues/11#issuecomment-6945392717)
and [Platform #12](https://github.com/dragginzgame/ic-platform/issues/12#issuecomment-6945393477). No new supplier
blocker was found, and no sibling source was edited. The
[transfer contract](../architecture/capture-transfer.md#full-platform-port--2026-11-17)
records decoder and regression provenance.

Toko Tooling 0.45.1 is installed through its canonical exporter in both workspaces.
Its 62-file snapshot is admitted. CI setup retains bounded sanitized diagnostics
with original failure status and seven-day artifacts; local failure-path tests
pass, while hosted artifact retrieval remains unobserved.
Previous 0.103.0 publication evidence remains in `target/evidence/release-0.103.0/`
and [handoff commit 2239e1b](https://github.com/dragginzgame/ic-backup/blob/2239e1bdf9f503dd65c183445f1e3ef72b4076ec/docs/status/current.md).

## Implemented scope

The experimental direct-controller library implements capture, history/latest,
held sources/payloads, same-ID/network/release restore, original-call recovery and
completed-target succession. Capture/restore leave the application stopped.
One recorder preserves finite spending and exact original evidence; pending calls
never repeat and changed/unknown files refuse intact. Completed replay requires
no live authority or writes. [Capture](../architecture/capture-intent.md),
[transfer](../architecture/capture-transfer.md) and
[restore](../design/local-store.md#immutable-restore-approval-and-original-custody)
own these guarantees. Full-history replay remains quadratic; reduced payload IO
does not establish end-to-end speedup. Measurements belong to the
[restart design](../design/restart.md#size-maintained-surface-and-repeated-io).

## Preservation and remaining integration

Original wires, roots, journals, child outputs and server logs remain under
`qualification/target/fixtures/`, including failed runs. Never discard uncertain
publication attempts or surviving locks; reconcile the registry before retrying.
The original reader remains at `b7c2c6ae754a5ae2452e65b9be537b517614ebd2`,
`preserve/backup-0.101.13-before-replacement`, with evidence under
`.git/recovery/backup-0.101.13-before-replacement/`. The status-only restore reader
remains under `.git/recovery/host-store-1-before-restore-20261116/` (185 source files,
both locks, six artifacts; archive SHA-256
`6658a8963fe3d56cb5fe13ec210c9170f8d6b7b50f0de33583e627287a371b06`).
[Setup](../development/setup.md#host-store1-reader-before-complete-restore) owns details.
Old operations keep their matching readers, IDs, authority and evidence;
this release migrates no records and changes no legacy/sibling data.

Canic work remains [Canic #5 signing](https://github.com/dragginzgame/canic/issues/5),
[Canic #6 manager execution/original-result lookup](https://github.com/dragginzgame/canic/issues/6)
and [Canic #7 fenced recovery](https://github.com/dragginzgame/canic/issues/7), under
[Canic #1](https://github.com/dragginzgame/canic/issues/1). The [sibling review](../design/sibling-integration.md)
owns boundaries: an authenticated manager receipt protocol/surviving ledger,
bounded signer, continuous application/rewind fence and actual consumer qualification.
Canic adapters, production HTTPS, actual-IC on macOS and physical power loss remain
unqualified. Publication does not qualify these unfinished boundaries.
