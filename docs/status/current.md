# Current handoff — 2026-10-30

## Complexity cleanup after 0.50.3

Baseline HEAD is `5996b364f77780bc62b9ced315ac21a3c9be8f96` (Release 0.50.3).
The user authorized all ten further cleanup recommendations. The batch is complete
with focused checks; **0.50.4** draft notes cover it. Package versions remain
**0.50.3**. Source and notes are unstaged/uncommitted. The incoming dirty Cargo.lock
is preserved byte-for-byte; no dependency version, snapshot payload, sibling,
Git/release state, issue, publication or live IC state was changed by this batch.

Seven record readers share bounded input/model decoding followed by canonical
pretty-JSON size admission, under their original locks and before identity checks.
Raw readers with separate contracts remain separate. Complete context comparison
has one model owner retaining network/caller/release mismatch order. Single-stage
checks use slice patterns, and Applied counts project existing admitted progress
rows without another journal-map scan. Original coverage and causality remain.

Capture and allocation settlement views retain one closed outcome with an index
only for explicit Applied attribution. Their public outcome/descriptor accessors
and exact candidate/destination checks remain unchanged. Signed-update evidence
stores only the original request, envelope and ingress identity; the sole submission
owner retains its fixed transport, target and reply ceiling locally. Mandatory
retention, one submission, reply bounds and indeterminate failure behavior remain.

Source reads distinguish regions and whole stored chunks with explicit bounds.
The ephemeral batch scan groups its original extent cursor and bindings; no IO,
checksum, source admission or spending check is cached or removed. Cold fixture
reconstruction separates exact chunk and allocation replay while keeping the source
layout and journal held. Named retained allocation fields also feed the unchanged
terminal checks. Public APIs/errors, record/wire bytes and recovery obligations
remain intact; this adds no product runner or authority provider.

## Fixed validation inputs

The incoming lock selects Platform **0.8.3**. Host/Testkit **0.70.3** are already
selected in the baseline. Candid **0.10.39**, Agent SDK **0.49.3**, external Principal
**0.1.5**, management DTO **0.11.0**, PocketIC **16.1.0** and Rust floor **1.91.0**
remain selected. No dependency declarations were edited.
`/tmp/backup-simplify053-incoming.lock` retains the exact incoming lock;
`/tmp/backup-simplify053-graph.sha256` binds the unchanged validation inputs:
manifest `c484f12826fc45ba511748c0c44735458608cab6319086b6d90acff1637c7ff8`
and lock `4c76293791b8a45f12c58e97618998af2b1822a662724b8098249a42661754d3`.

## Focused qualification

Strict Clippy passes for Core's library/tests and Agent's library plus affected
HTTP/simulator targets with warnings denied. All **214 selected Core tests**, **four
Agent unit tests** and **16 HTTP boundary cases** pass. Coverage includes canonical
record limits and reopen, complete journal/ancestor admission, mismatch precedence,
explicit settlement outcomes, source-byte bounds and original pending/Applied
preservation. Existing exact linear source-read and maximum-region/chunk cases pass;
no throughput or timing claim is made.

All **six connected Core PocketIC journeys** pass: complete same-ID round trip,
cold continuation at allocation/load/start, and lost allocation/load/start/second
upload refusal. All **nine selected Agent PocketIC cases** pass: the seven
independent normal-consumer journeys, complete Agent transfer/upload/restore and
wrong trusted root. The independent build admits the selected normal graph without
Testkit/PocketIC/WAT. Original ingress, replies, source bytes and terminal replay
remain retained; process death after authenticated load never grants redispatch.

Logs are `/tmp/backup-simplify053-*`: `-core-clippy.log`,
`-agent-clippy-recheck.log`, `-core.log`, `-agent-native.log`, `-core-pocketic.log`,
`-agent-pocketic.log`, `-source-checks.log` and `-server-check.log`.
The initial ambiguous SDK target conversion remains in `-agent-clippy.log`;
its explicit Platform external-principal conversion passes the subsequent check.
Formatting, local documentation links, Cargo inheritance/boundaries, regular-file/
license admission and all 126 Shared payloads pass. Offline selected server admission
passes without setup; fixtures separately admit authenticated assets at startup.
No full CI/full local gate, new MSRV/macOS matrix, registry-only acceptance,
installed Canic migration or live application qualification was run.

Core originals are `/tmp/ic-backup-pocketic-3427141-*`, including cold resume `-8`
and complete round trip `-10`. Independent graph/build evidence is
`/tmp/ic-backup-agent-independent-build-3555589-0`. Agent originals are
`/tmp/ic-backup-agent-pocketic-3555589-*`, including complete cold restore `-9`,
load death `-11` and operator failures `-13`, `-15`, `-17`. Preserve every original
plan, journal, request/reply, manifest, artifact, server identity and process log.

## Retained obligations and historical evidence

[Implementation contracts](../implementation-contracts.md),
[Platform ownership](../ic-platform.md), the
[operator map](../canic-adoption.md#reusable-operator-composition) and
[Canic removal map](../canic-adoption.md#canic-engine-removal-map) remain canonical.
Original spending, unsettled effects, outside-snapshot obligations and referenced
artifacts retain their existing owners. Local completion grants no application
fence/reference release or cleanup authority. Shared **0.27.7**, revision
`11d519c604b1727a2bf6272557a97e770a63b29f`, retains its reviewed 126 payloads.

The [previous committed handoff](https://github.com/dragginzgame/ic-backup/blob/5996b364f77780bc62b9ced315ac21a3c9be8f96/docs/status/current.md)
preserves earlier source/graph fingerprints, qualification scope and exact evidence
locations. Retain `/tmp/backup-simplify052-*`, `/tmp/backup-simplify051-*`,
`/tmp/backup-simplify050-*`, all earlier `/tmp/backup-*` evidence,
`/tmp/ic-backup-agent-independent-build-*`, `/tmp/ic-backup-agent-pocketic-*`,
original Canic/release/consumer journals, `target/msrv-consumers.*`,
`target/shared-release-tests.*`, old/failed tool bundles and the 20261027/20261028
logs in `target/validation-failures/`. Historical passes retain their original
source/graph/host scope. This summary removes no artifacts and authorizes no disposal.
