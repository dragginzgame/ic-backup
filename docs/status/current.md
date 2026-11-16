# Current handoff — 2026-11-16

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

## Selected suppliers and remaining migration

Product uses Host FS/Artifacts 0.80.2, HTTP 0.80.0, Protocol 0.2.8 and Platform
0.33.7. Backup owns Reqwest 0.13.5 and its original TLS/no-retry/redirect/proxy/
deadline policy; Host bounds complete response bodies. Obsolete Host 0.79 edges
are removed. Protocol owns data/allocation/exact-empty decoding, removing 61 net
production schema-decoder lines while preserving policy and resource bounds.

Qualification uses Platform 0.34.9 for actual external signing/proxy admission
and Testkit 0.107.4 for prepared-server/instance ownership. Explicit old Platform
0.33.7 dev selection remains necessary at the product source-read DTO/Principal
boundary. Fixture authority uses ordinary library imports. Original server logs,
process/cleanup evidence, journals and failed fixtures remain retained.
Setup downloads only explicitly into `.tools/testkit-server/`; old
`.tools/testkit/` assets remain untouched. The 20 MiB case has a finite
30-minute server lifetime. [Setup](../development/setup.md) owns preparation.

Release-time registry review found these Host/Testkit/new Platform versions and
Toko Tooling 0.44.11 current. Protocol 0.2.8 leaves the management codec unchanged
from the earlier 0.2.7 pass, and is now qualified in the release gate.
Earlier evidence remains in `target/evidence/supplier-refresh/` and
`target/evidence/platform-recheck-current/`. Testkit 0.107.3 also resolves with
Protocol 0.2.7 when its optional codec feature is disabled; 0.107.4 was not required
for that compatibility. Their managed-server sources are byte-identical.

Remaining [Platform #11](https://github.com/dragginzgame/ic-platform/issues/11)
and [Platform #12](https://github.com/dragginzgame/ic-platform/issues/12) work:
port product status/list/take/metadata, request DTOs, certificate verification and
retained-envelope authentication, then remove the old dependency and requalify.
The response-fixture construction gap is documented in
[Platform #11](https://github.com/dragginzgame/ic-platform/issues/11#issuecomment-6931401980):
Protocol 0.2.8 lacks public bounded constructors for dynamic recovery replies.
Evidence is in `target/evidence/platform-final-port/`. Preserve rejection coverage;
do not copy the generic legacy codec or retain an old crate solely as a hidden
fixture encoder. No sibling source was edited. The baseline permits this qualified
partial release while the remaining upgrade is blocked; neither issue is closed.
The [transfer contract](../architecture/capture-transfer.md#partial-protocol-adoption--2026-11-16)
owns decoder and legacy-regression rationale.

Toko Tooling 0.44.11 is installed through its canonical exporter in both workspaces.
Its 59-file snapshot is admitted. CI setup retains bounded sanitized diagnostics
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
