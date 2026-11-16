# Current handoff — 2026-11-16

Release **[0.103.0](https://crates.io/crates/ic-backup/0.103.0) is published** under
the maintainer's explicit deploy/publish request. Annotated
[tag v0.103.0](https://github.com/dragginzgame/ic-backup/tree/v0.103.0) and `main`
were pushed atomically at `34e268a866aed8d30ffbdb7d09283d768f36fc65` and independently
read back from GitHub. Product, both locks and qualification select 0.103.0;
qualification stays private and supplier selections are unchanged.
Scoped post-push review found no additional issue in payload checks/release callers.
Hosted CI stays asynchronous; no hosted result is claimed.

The complete publication gate passed: 236 native tests, five documentation examples,
all 11 actual-IC journeys (105.82 seconds), both Clippy checks, formatting, rustdoc,
links, isolated archive consumer and publication-caller regression. The single upload
in `target/publication/publish.selection.RkiVSN/` has Cargo/logger/final statuses 0.
Official registry SHA-256 is
`4269639bb9857e5079dcc71d2aaa689537976aa04212bc236eaa7b5a1b24f046`;
the archive has clean release-commit provenance and all 94 tracked source files match.
A fresh normal registry consumer resolves 0.103.0 without Testkit/PocketIC, builds
offline and replays completed 20 MiB capture/restore in separate processes without
a backend. Its graph, build and replay results remain beside the registry readback.
Evidence remains under `target/evidence/release-0.103.0/`; package verification is
in `target/package-consumer/check.g4L1ji/`.

The previous published release is [0.102.1](https://crates.io/crates/ic-backup/0.102.1),
tag `v0.102.1`, commit `1db82ba22ac15c761906af0935e0e918acf40801`.
Its archive, registry consumer and original failed preparation remain under
`target/evidence/release-0.102.1/`. Historical release/adoption details are preserved
in [handoff commit 61e0cac](https://github.com/dragginzgame/ic-backup/blob/61e0cac7de314bcd9f62e88cc975368abf36e677/docs/status/current.md).
That commit delivered the lookup fix and initial replay reductions to `main`;
independent remote readback and scoped post-push review found no additional issue.

## Included changes and evidence

Host FS/Artifacts **0.80.2** are registry-backed in both graphs; HTTP 0.79.0,
Platform 0.33.7, Testkit 0.106.0 and Toko Tooling **0.44.5** remain selected.
Public commitments now use `store::BackupDigest`; filesystem/byte errors expose
Host's new outcomes. Stored SHA-256 bytes, intents, journals and manifests retain
their formats. See the [consumer migration](../../crates/ic-backup/README.md) and
[Host adoption/legacy anchors](../design/local-store.md#host-0801-adoption-blocker--2026-11-16).
The tooling snapshot was installed through its exporter; managed files remain intact.
Adoption evidence and failed probes remain in `target/evidence/host-tooling-adoption/`
and `target/evidence/host-tooling-recheck/`.

The [Canic-readiness audit's lookup defect](../reports/audits/2026/11/16/canic-readiness/01/report.md)
is fixed using the shared 2 MiB request ceiling with original authentication,
expiry and spent-observation checks. Six native lookup tests and actual-IC
interrupted allocation/offline replay at 2 MiB pass. Evidence remains under
`target/evidence/lookup-profile-fix/`; original failed audit journals remain under
`target/evidence/canic-readiness-audit/`. Release 0.103.0 fixes the published
0.102.1 defect.

Replay reuses Host's read digest, authenticates each owned original request once,
and indexes held evidence by canonical name. Read-only materialization removes an
earlier payload comparison while retaining the complete closing tree hash and
custody checks. Creation still admits every existing member before writing any
missing member. No persistent cache, new journal or recovery authority was added.
The latest focused pass passes all 15 custody tests, the mutation-across-HTTP
regression, strict product Clippy and the actual-IC 20 MiB journey (90.10 seconds).
On that retained fixture it saves 80 MiB of capture payload reads and 120 MiB for
restore (13.8% / 8.1% of all store reads); paired timings establish no meaningful
end-to-end speedup. Evidence and journals are unchanged. Provenance, earlier
qualification and measurements belong to the [restart design](../design/restart.md#size-maintained-surface-and-repeated-io)
and [assembly contract](../architecture/capture-transfer.md#local-artifact-assembly),
with raw evidence under `target/evidence/replay-{hash,auth,index,payload}-review/`.
Full-history replay remains quadratic.

## Implemented scope and publication

The experimental direct-controller library implements capture, history/latest,
held sources/payloads, same-ID/network/release restore, original-call recovery and
completed-target succession. Capture/restore leave the application stopped.
One recorder preserves spending and original evidence; pending calls never repeat
and changed/unknown files refuse intact. Completed replay needs no live authority
or writes. [Capture](../architecture/capture-intent.md),
[transfer](../architecture/capture-transfer.md) and
[restore](../design/local-store.md#immutable-restore-approval-and-original-custody) own details.

The [20 MiB qualification](../architecture/capture-transfer.md#20-mib-qualification--2026-11-16)
uses a 22 MiB payload approval, 1 MiB extents, two chunks and 1 GiB evidence.
Actual-IC capture, interrupted transfer, older-source restore, state verification
and offline replay pass on Linux. The fixture uses finite 30-minute backend
idle/hard limits and the optimized profile. Original failed fixtures remain.
[Publication](../development/publication.md) owns the complete gate and selected
single-attempt adapter; no automatic version/tag orchestration is installed.
All four [legacy comparison corrections](../design/legacy-assessment.md#review-corrections--2026-11-16)
are addressed; smaller code does not establish feature parity or speed.

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

Canic work remains [#5 signing](https://github.com/dragginzgame/canic/issues/5),
[#6 manager execution/original-result lookup](https://github.com/dragginzgame/canic/issues/6)
and [#7 fenced recovery](https://github.com/dragginzgame/canic/issues/7), under
[#1](https://github.com/dragginzgame/canic/issues/1). The [sibling review](../design/sibling-integration.md)
owns boundaries: an authenticated manager receipt protocol/surviving ledger,
bounded signer, continuous application/rewind fence and actual consumer qualification.
Canic adapters, production HTTPS, actual-IC on macOS and physical power loss remain
unqualified. Publication does not qualify these unfinished boundaries.
