# Current handoff — 2026-11-16

Work: `/home/adam/projects/ic-backup`, branch `main`. The maintainer authorized
deployment of the experimental direct-controller library as 0.102.1, tag
`v0.102.1`, to `origin/main` and crates.io. Previous pushed source is
`e8f2ad5a2691020c1feac49768981a94e07d5e4b` (0.102.0, unpublished).
Product, both locks and the qualification requirement now select 0.102.1.
Qualification stays private. The registry lacked 0.102.1 at release preflight;
delivery/publication results must be verified before claiming completion.
Older operations retain their preserved readers and authority; no migration.
Hosted CI remains asynchronous at the user's request.

Release supplier review found the selected Host 0.79.0, Platform 0.33.7,
Testkit 0.106.0 and Tooling 0.44.4 are the current stable non-yanked registry
versions. No supplier changed; only Backup's entries changed in both locks.
Evidence is under `target/evidence/release-0.102.1/`.

## Publication preparation

The [full comparison/readiness audit](../reports/audits/2026/11/16/publication-readiness/01/report.md)
records the original findings. The 20 MiB journey now passes actual-IC capture,
interrupted-transfer recovery, full older-source restore, state verification and
offline replay. Two optimized runs take 89.20/88.63 seconds. The warm command
reports 730,672 KiB maximum RSS including fixture/backend accounting. Its finite
shape needs a 22 MiB payload approval, 1 MiB extents, two chunks and 1 GiB evidence;
see [capacity and measured costs](../architecture/capture-transfer.md#20-mib-qualification--2026-11-16).
Debug replay was slow enough to expire the backend's default idle timeout. The
fixture now has finite 30-minute idle/hard limits and uses the optimized profile.
Failed fixtures remain retained; production limits/behavior are unchanged.

The [packaged guide](../../crates/ic-backup/README.md) covers setup, scope and
original-store migration/recovery. [Publication](../development/publication.md)
uses the verified Toko 0.44.4 single-package adapter, installed through its exporter
(59 managed files), with local package/consumer and caller checks. Caller tests
cover private/unknown/duplicate selection, validation/source drift, lock retention
and one failed dispatch without retry. Actual Cargo registry dry-run passes in a
disposable eligibility-enabled copy; that was not an upload. Product eligibility
is now enabled under deployment authority. Automated version/tag orchestration
is not installed; this release uses explicit Git delivery and the existing
publication adapter. Its complete gate and registry readback remain mandatory;
no prior log silently bypasses that gate.

## Legacy review corrections

All four [comparison findings](../design/legacy-assessment.md#review-corrections--2026-11-16)
are addressed: bounded original certificates in redacted errors, independent
observation-reply import, full 1 MiB envelopes and a normal-dependency process
consumer. Original spending/custody and separate observation timing remain intact.
Audited size before this preparation: 8,098 product / 196 consumer / 12,583 test
Rust LOC; app/lib 46.99% smaller than frozen legacy, total 53.48%, crate-only
production 40.36%. Narrower scope establishes neither parity nor speed.

## Implemented Backup scope

The direct-controller single-canister library implements capture, bounded history/
latest, authenticated original source/payload, same-ID/network/release restore,
original-call recovery, cold continuation and completed-target succession.
Capture/restore leave the application stopped. One `BackupConnection`, signer,
action guard and private sealed recorder serve both fixed procedures.
The [operation contract](../design/operation-contract.md) owns approvals, cold reopen and ordering.
Complete metadata/byte checks precede load. Pending calls never repeat; spending
survives interruption. Explicit observation retains separate original timing.
Dependent effects require live permission. Completed replay uses original evidence
without clock, guard, signing, HTTP or writes. Unknown tails/orphans/changed files
refuse intact; uncertain writes invalidate the handle. No reset/refund/start occurs.
[Capture](../architecture/capture-intent.md), [transfer](../architecture/capture-transfer.md)
and [restore](../design/local-store.md#immutable-restore-approval-and-original-custody) own behavior.

## Acceptance and preservation

`target/evidence/macos-fixture-*`: 232 native tests, strict Clippy for both
workspaces, formatting/links and actual-IC capture/restore pass locally on Linux.
`macos-cancellation-*` and `macos-deadline-*`: all 20 HTTP cases and focused checks pass.
[Current CI](https://github.com/dragginzgame/ic-backup/actions/runs/43836041869)
passes ARM/scaffold; Linux/Intel remain pending at the last observation.
Post-push review of all three commits and their affected callers found no new
actionable drift, duplicate policy, recovery paths or unused abstractions.
`twenty-mib-{release,warm}-*` retains the new capacity evidence; the warm fixture is
`qualification/target/fixtures/capture-Capacity-3805726-1794820857897640283/`.
`publication-adapter-final.log` and `publication-real-online-dry-run.log` retain
caller and actual Cargo preflight evidence. `publication-audit-*` is historical.
`publication-other-ic-cases.log`: all ten remaining IC cases pass (120.15 seconds).
`publication-package-final.log`: current archive and isolated normal consumer pass.
`twenty-mib-archive-replay.json`: that archive's consumer reopens completed 20 MiB
capture and restore offline (21.95/51.62 seconds in its debug build). The first
sandbox subprocess-control error is retained separately; the admitted rerun passes.
Strict qualification Clippy, ShellCheck, formatting, links and snapshot admission
pass. The aggregate publication gate is configured, not claimed as executed.

Original wires, roots, journals, child outputs/graph and server logs remain under
`qualification/target/fixtures/`, including failed runs. Native cases preserve
original spending, late-deadline evidence, uncertain writes and changed custody.

The original reader remains at `b7c2c6ae754a5ae2452e65b9be537b517614ebd2`,
`preserve/backup-0.101.13-before-replacement`, with hashed source/build evidence
under `.git/recovery/backup-0.101.13-before-replacement/`. The newer capture plus
status-only restore reader remains under
`.git/recovery/host-store-1-before-restore-20261116/`: 185 source files, both locks,
six artifacts; archive SHA-256
`6658a8963fe3d56cb5fe13ec210c9170f8d6b7b50f0de33583e627287a371b06`.
[Setup](../development/setup.md#host-store1-reader-before-complete-restore) owns details.
Old operations retain their matching reader/IDs/authority/evidence. This correction
changes no stored encoding, migrates no records and touches no legacy/sibling data.

## External integration and limits

Canic issues remain [#5 signing](https://github.com/dragginzgame/canic/issues/5),
[#6 manager execution/original-result lookup](https://github.com/dragginzgame/canic/issues/6),
[#7 fenced recovery](https://github.com/dragginzgame/canic/issues/7), linked from
[#1](https://github.com/dragginzgame/canic/issues/1). The [sibling review](../design/sibling-integration.md)
retains exact ownership. Full integration needs the actual manager's authenticated
receipt protocol/surviving ledger, bounded signer, continuous application/rewind
fence and real consumer qualification. No substitute route is invented locally.

Replay remains quadratic. Canic adapters, production HTTPS, actual-IC on macOS,
physical power loss and a freshly published registry consumer remain unqualified.
The current deployment request authorizes Git delivery and registry publication
of this limited scope; it does not qualify the unfinished integration boundaries.
