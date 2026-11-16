# Current handoff — 2026-11-16

Local adoption selects **Host HTTP 0.80.0** in both graphs and **Testkit 0.107.3**
in the preceding private qualification pass. Backup directly owns Reqwest 0.13.5 with the original
TLS/no-retry/redirect/proxy/deadline policy; Host only bounds complete response
bodies. No public operation or stored format changed. Obsolete Host 0.79 graph
edges are removed; FS/Artifacts/Process now select 0.80.2 where used.

Testkit's removed CLI/pic wrapper is replaced by its prepared-server and explicit
server/instance owners. The private qualification example handles setup/admission;
`make install-testkit-server` downloads only during explicit setup. Assets use
`.tools/testkit-server/`; old assets remain untouched. Fixtures retain bounded
server output and original process/cleanup reports, including during unwinding.
The 20 MiB journey retains its finite 30-minute server lifetime. Permission refusal
of the old group-writable archive was resolved using a private copy; failed
preparation logs and the original archive remain intact.

Fresh qualification passes: 236 native tests, five documentation examples, strict
Clippy in both workspaces and all 11 actual-IC journeys (107.11 seconds), including
20 MiB capture/restore and separate-process recovery. The archive/isolated normal
consumer, explicit setup reuse, snapshot/tool checks, both workspace format checks
and 121 documentation links pass. Provenance,
commands and preparation ownership are in [setup](../development/setup.md);
legacy/recovery rationale is in the [capture contract](../architecture/capture-intent.md#host-http-080--testkit-0107-adoption--2026-11-16).
Logs/archives are under `target/evidence/supplier-refresh/`. These are prior-pass results; the release candidate qualification is recorded below.

**Partial Platform adoption now runs in the actual callers.** Product decoding
of snapshot-data, restore allocation and exact empty replies uses Protocol 0.2.7;
61 net lines of private production schema decoding were removed. Original bounds,
geometry/hash/identity policy and evidence formats remain intact. Decode errors
now expose Protocol's error type. Nine existing data regressions and the added
allocation/empty-result boundaries pass; the full native suite passes 237 tests
and five documentation examples. Both strict Clippy checks pass.

The actual-IC signer and proxy use Platform 0.34.9, including offline bounded
encoding before Backup retains/submits the exact request. The shared private
fixture authority uses ordinary library imports; its source-path override is gone.
All 11 actual-IC journeys pass in 105.69 seconds, including 20 MiB, original
observation/reply recovery and separate-process replay. The isolated archive
consumer, Rustdoc, formatting, snapshot/tool admission and 122 doc links pass. Logs are in `target/evidence/platform-recheck-current/`.
Qualification now selects Testkit 0.107.4, whose managed-server source is unchanged.
A resolution probe confirmed 0.107.3 remains compatible with its codec feature
disabled; the patch is not required by Protocol 0.2.7.

This remains partial acceptance for [Platform #11](https://github.com/dragginzgame/ic-platform/issues/11)
and [Platform #12](https://github.com/dragginzgame/ic-platform/issues/12).
Product Platform 0.33.7 still owns status/list/take/metadata, request DTOs,
certificate verification and retained-envelope authentication. Qualification's
explicit `ic-platform-legacy` dev dependency is needed at the current product
source-read DTO/Principal boundary. Next: port those remaining public types and
callers, remove that old-cohort dependency, then repeat affected qualification.
A remaining response-fixture construction gap is now reported in Platform #11:
Protocol 0.2.8 supplies request encoders/reply decoders but no public bounded
constructor for the status/snapshot/metadata/data/allocation replies used by native
recovery fixtures. A registry compile probe confirms the missing methods; private
codec internals cannot replace that public boundary. Earlier static golden probes
did not cover these dynamic fixture callers. Evidence and checksums are under
`target/evidence/platform-final-port/`. Preserve all native rejection coverage;
do not retain the old crate solely as a hidden fixture codec or copy its generic
codec into Backup. No sibling source was edited.

The maintainer's subsequent deploy request selects **0.104.0** for the qualified
partial adoption, without claiming completion of Platform #11/#12. The minor bump
covers public Protocol decode-error types; stored formats remain unchanged.
The baseline permits release of a qualified existing dependency selection while
an optional upgrade is blocked. Product Platform 0.33.7 and the qualification
DTO bridge remain necessary for the unmigrated callers above.
Release-time registry review confirms Host FS/Artifacts/Process 0.80.2, HTTP
0.80.0, Testkit 0.107.4, Platform 0.34.9 and Toko Tooling 0.44.11 are current.
Both current locks select Protocol 0.2.8; its selected management codec is unchanged
from the prior 0.2.7 pass. The release gate will qualify 0.2.8 with the matching
updated manifest requirement. Full Platform migration remains deferred to #11/#12.
The complete 0.104.0 local gate now passes: 237 native tests, five documentation
examples, all 11 actual-IC journeys, strict Clippy in both workspaces, Rustdoc,
tools/format/link checks, archive/isolated consumer and publication-adapter tests.
Logs and registry observations: target/evidence/release-0.104.0/. Publication
will repeat the mandatory gate on committed source; no upload is yet claimed.
The [transfer contract](../architecture/capture-transfer.md#partial-protocol-adoption--2026-11-16)
and [setup](../development/setup.md#current-partial-platform-migration) own details.

**Toko Tooling 0.44.11** remains installed through its canonical exporter in both
workspaces. Its 59-file snapshot changed policy only. CI setup diagnostics retain
bounded sanitized output and original status, with seven-day failure artifacts.
Local failure-path and workflow checks pass; hosted upload remains unobserved.
Those changes are included in the selected release candidate.

Release **[0.103.0](https://crates.io/crates/ic-backup/0.103.0) remains the published
release**, tagged/pushed at `34e268a866aed8d30ffbdb7d09283d768f36fc65`.
Its completed publication gate, archive checksum and independent registry replay
are preserved in [handoff commit 2239e1b](https://github.com/dragginzgame/ic-backup/blob/2239e1bdf9f503dd65c183445f1e3ef72b4076ec/docs/status/current.md)
and `target/evidence/release-0.103.0/`. This adoption is not a new release.

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
