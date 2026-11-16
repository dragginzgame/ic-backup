# Current handoff — 2026-11-16

Work: `/home/adam/projects/ic-backup`, branch `main`. **0.102.1 is published** on
[crates.io](https://crates.io/crates/ic-backup/0.102.1), with annotated
[tag v0.102.1](https://github.com/dragginzgame/ic-backup/tree/v0.102.1) at
`1db82ba22ac15c761906af0935e0e918acf40801`. The release commit and tag were pushed
atomically and independently read back from GitHub. Product, both locks and the
qualification requirement select 0.102.1; qualification stays private.
Hosted CI was left asynchronous, without polling, as requested.

The [Canic-readiness code audit](../reports/audits/2026/11/16/canic-readiness/01/report.md)
found one reproducible Backup defect: an admitted request ceiling above 1 MiB
(including the documented 2 MiB restore profile) fails lookup authentication at
the stale 1 MiB bound after reserving the observation. A focused actual-IC
interrupted-allocation probe fails with `CapturePlan(Lookup(Bounds))`; original
journal/evidence remain under `target/evidence/canic-readiness-audit/`.
Fix and qualify this before Canic pins the dependency. No runtime fix, commit or
publication occurred in the audit; the release passes below did not exercise
this cross-profile recovery case. Canic integration can start as coordinated
work on real signing, manager execution/admission/replay and fenced recovery.

Release supplier review found the selected Host 0.79.0, Platform 0.33.7,
Testkit 0.106.0 and Tooling 0.44.4 are the current stable non-yanked registry
versions. No supplier changed; only Backup's entries changed in both locks.
Evidence is under `target/evidence/release-0.102.1/`. The first gate exposed inherited
evidence paths in the nested publication fixture, before any upload. The corrected
fixture owns its path and excludes parent timing records. Its regression and the
complete rerun pass: 232 native tests, five examples, eleven IC cases, strict lint,
docs, tooling admission, package/consumer and publication-caller checks. The full
gate took 175.10 seconds; Cargo verification/upload/availability took 7.90 seconds.
Original failed evidence remains; no upload was retried.

The non-yanked registry archive has SHA-256
`551f6abc1e7a631982c41f7e4ab807ccc6e0b6bf6213f013f0b16a331d2c73aa`, matches the
verified local archive byte-for-byte and names the release commit in its clean
Cargo provenance. A normal consumer resolves `ic-backup = "=0.102.1"` from the
registry, builds offline with its lock, then replays completed capture and restore
after backend shutdown. Its graph excludes Testkit/PocketIC; `registry-consumer/`
retains resolution, build and replay evidence. This does not qualify live HTTPS.

## Capacity and publication scope

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
and one failed dispatch without retry. Automated version/tag orchestration is not
installed; this release used explicit Git delivery and the existing publication
adapter. Future attempts retain the complete gate and independent registry
reconciliation. Successful publication does not authorize another upload.

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

Post-push review of the delivered release and affected Make, package, fixture and
documentation callers found no additional actionable drift, duplicated recovery
policy or unused production abstractions. Product Rust behavior and stored formats
are unchanged by this release preparation. Earlier macOS socket, cancellation and
deadline corrections retain their Linux evidence under `macos-*`.
`twenty-mib-{release,warm}-*` retains the new capacity evidence; the warm fixture is
`qualification/target/fixtures/capture-Capacity-3805726-1794820857897640283/`.
`twenty-mib-archive-replay.json`: that archive's consumer reopens completed 20 MiB
capture and restore offline (21.95/51.62 seconds in its debug build). Earlier
publication-audit/preparation evidence and failed sandbox runs remain historical.

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

Replay remains quadratic. Canic adapters, production HTTPS, actual-IC on macOS
and physical power loss remain unqualified. This is the authorized experimental
direct-controller release, not qualification of those unfinished boundaries.
