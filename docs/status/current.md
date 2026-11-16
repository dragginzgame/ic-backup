# Current handoff — 2026-11-16

Work: `/home/adam/projects/ic-backup`, branch `main`. **0.102.1 is published** on
[crates.io](https://crates.io/crates/ic-backup/0.102.1), with annotated
[tag v0.102.1](https://github.com/dragginzgame/ic-backup/tree/v0.102.1) at
`1db82ba22ac15c761906af0935e0e918acf40801`. The release commit and tag were pushed
atomically and independently read back from GitHub. Product, both locks and the
qualification requirement select 0.102.1; qualification stays private.
Hosted CI was left asynchronous, without polling, as requested.

Toko Tooling 0.44.5 is selected in both manifests/locks and installed through the
canonical exporter. Its verified archive is
`aef8ac5d84866cf4ceb5b9d91ce9d458708069fc0c585345f07054186d267199`, source
`081753852ae398661fadfd0fe3b3fc631f0a96e6`. The same 59 managed payloads are
byte-identical; only the receipt's version/origin changed. Executable tool pins
are unchanged. Product version remains 0.102.1. Tooling evidence: `target/evidence/host-tooling-adoption/`.
Exporter/tooling admission, standard dependency declarations and tracked locks,
documentation links (122 references in 19 documents), and `git diff --check` pass.
The optional `--cargo-inheritance` probe rejects the existing explicit product
version; this update leaves release-version ownership unchanged. Its failed log
is retained separately from the passing standard declaration check.

Host FS/Artifacts **0.80.2** are now registry-backed in both graphs, from verified
source `828c2692523a7a906188c111c9b58f30d0f52468`. HTTP 0.79.0 and Toko 0.44.5
remain current. The published fix for [IC Tooling #2](https://github.com/dragginzgame/ic-tooling/issues/2)
unblocks exact original-inode manifest recovery. Backup uses explicit Host
stage/seal/commit outcomes and retained files; original authority, spending,
canonical SHA-256 and stored formats are unchanged. Public commitments now use
`store::BackupDigest`; Host/error type changes require the pending **0.103.0**
compatibility note. The maintainer authorized committing and pushing this adoption
to `main`; version bump and crates.io publication remain pending.
Direct SHA-256 0.11.1 owns hashing; Artifacts owns bounded complete byte streams.
HTTP's Artifacts 0.79 and Testkit's private FS/Process 0.79 remain isolated;
qualification no longer imports Host just to construct Backup commitments.

Both full locked metadata graphs, dependency boundaries, and affected product/
qualification Clippy pass. All 100 selected native cases pass across the retained
runs: inventory/payload/source, digest/golden encodings, recorder framing/custody,
fixed capture policy, runtime recovery and restore sources. The original sandbox
run passed 69 and refused local sockets/process setup; the FIFO test was adapted
to Host's typed `Kind` refusal. The affected 34-case rerun passes, including
original-inode publication and retained partial/changed-manifest refusal.
The actual-IC multi-extent capture/restore/succession/offline-replay case and the
isolated normal-dependency consumer's fresh-process continuation/replay pass.
The final manifest closing-read check retains its exact readback before sealing;
its three affected native publication regressions pass. Five public examples,
strict rustdoc, formatting, tooling admission and documentation links pass.
Evidence, failed runs and original fixture roots remain under
`target/evidence/host-tooling-recheck/` and `qualification/target/fixtures/`.
See [the migration and legacy anchors](../design/local-store.md#host-0801-adoption-blocker--2026-11-16).
No full release gate, hosted macOS, CI polling or physical power-loss proof is claimed.

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

The published 0.102.1 baseline selected Host 0.79.0, Platform 0.33.7, Testkit 0.106.0
and Tooling 0.44.4. Its complete release gate passed 232 native tests, five examples,
eleven IC cases, lint/docs and package/consumer/publication-caller checks. This
historical evidence does not cover the later cross-profile lookup defect above.
The registry archive SHA-256 is
`551f6abc1e7a631982c41f7e4ab807ccc6e0b6bf6213f013f0b16a331d2c73aa`;
archive bytes and clean release provenance match independently. Its normal
registry consumer built offline without Testkit/PocketIC and replayed completed
capture/restore after backend shutdown. Evidence, including original failed
preparation, remains under `target/evidence/release-0.102.1/`.

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
uses the verified Toko 0.44.5 single-package adapter, installed through its exporter
(59 managed files), with local package/consumer and caller checks. Caller tests
cover private/unknown/duplicate selection, validation/source drift, lock retention
and one failed dispatch without retry. Automated version/tag orchestration is not
installed; this release used explicit Git delivery and the existing publication
adapter. Future attempts retain the complete gate and independent registry
reconciliation. Successful publication does not authorize another upload.

## Legacy review corrections

All four [comparison findings](../design/legacy-assessment.md#review-corrections--2026-11-16)
are addressed; that assessment owns historical LOC comparisons, provenance and
scope exclusions. Smaller code does not establish feature parity or speed.

## Implemented Backup scope

The direct-controller library implements capture, history/latest, held source/payload,
same-ID/network/release restore, original-call recovery and completed-target succession.
Capture/restore leave the application stopped. One recorder preserves spending and
original evidence; pending calls never repeat and unknown/changed files refuse intact.
Completed replay needs no live authority or writes. [Capture](../architecture/capture-intent.md),
[transfer](../architecture/capture-transfer.md) and
[restore](../design/local-store.md#immutable-restore-approval-and-original-custody) own details.

## Acceptance and preservation

The 0.102.1 post-push review found no additional actionable drift in its release
callers. Earlier macOS socket/cancellation/deadline corrections retain their Linux
evidence under `macos-*`.
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
