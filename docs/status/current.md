# Current handoff — 2026-10-29

## Complexity refactor after 0.50.0

Baseline HEAD is `da08504d7ed8f22966fcf1eaa8e5fb8bc13eeaa1` (Release 0.50.0).
The user authorized all ten source-review simplifications. This batch changes
private implementation and independent-consumer fixtures; public signatures,
errors, record/wire bytes, original spending and application authority remain intact.
Pending **0.50.1** notes cover the batch; package versions remain **0.50.0**.
Changes are unstaged/uncommitted. No dependency, snapshot payload, sibling,
Git/release, publication, issue or live IC effect was performed.

Opening stage inspection reads/adopts each binding once, retaining its closing
expected-digest and ancestor checks around journal IO. Fence acquisition,
observation and settlement share original obligation/authority/journal/payload
admission. Their separate reservations, fresh admission and qualification remain
explicit. Pending-observation identity moves to attempt accounting, including
fence reconciliation, with existing public error projections and ordering.

Applied-receipt matching and recording share one owner; domain preparation stays
between matching and recording. Download appends exact bytes before recording;
restore re-admits source/requirements before and after the receipt. Checkpoints
still follow journal-guard release. Transfer loops share pure untouched/all-Applied
progress predicates without changing original plan coverage or execution order.

Upload completion explicitly distinguishes a completed data stage from an empty
upload's exact allocation receipt and operation. Artifact members explicitly
carry fixed metadata, bounded metadata reply, region or chunk obligations, retaining
names, bounds, checksums and original read ordering. Agent management wire
projection shares its existing payload owner and keeps per-lane reply ceilings.

The independent consumer decodes its journey configuration once into typed fields.
Named stage declarations own unchanged IDs 0..6, allowances, workflow grouping
and retained paths; planning and dispatch use those names. The isolated build
emits both new fixture modules. Opaque application protocols, separate fixture
authority and original failure modes remain unchanged.

## Focused qualification

Strict Core/Agent Clippy passes for all targets/features with warnings denied.
All **147 selected Core model/policy/workflow/artifact cases**, **11 original
workflow coordinator cases** and **16 Agent HTTP boundary cases** pass. These
include exact error and byte identity, changed originals, pending/partial evidence,
empty uploads, maximum metadata/chunks, bounded streaming, source substitution,
receipt mismatch, cancellation and effect-free replay. The existing linear source
read-count checks also pass; no new performance or throughput claim is made.

All **seven independent normal-consumer PocketIC cases** pass: the complete
capture/transfer/manifest/upload/same-ID load/start/terminal journey, death after
an authenticated load, operator ingress/reply-retention and partial-transfer
failures, plus four opaque application/recovery cases. Build/normal-graph evidence
is `/tmp/ic-backup-agent-independent-build-2604704-0`. Original fixtures are
`/tmp/ic-backup-agent-pocketic-2604704-*`, including complete journey `-9`, load
process death `-11` and operator failures `-13`, `-15`, `-17`. Preserve exact plans,
journals, envelopes, replies, source bytes, manifests and raw server/process output.

Logs are `/tmp/backup-simplify050-*`; the first Clippy log retains a corrected
fixture caller compile error. Final lint is `-clippy-final.log`, native checks
`-core.log`, `-operator.log`, `-http.log`, and simulator checks `-independent.log`.
Source-consistency results are retained in `-source-checks.log`. The root graph
stayed fixed: `/tmp/backup-simplify050-graph.sha256` binds manifest
`730cd25645b766b5329ab6468c348207250ccb4fec9fce5020c3441489f96a3b`
and lock `1a11ec8969f779d4c1bd2ce38d9efc7d8b07b0d468e4895d0334464b9194481c`.
No full local gate, new MSRV/native macOS matrix, registry-only product acceptance,
installed Canic migration or live application qualification was run.

## Tooling baseline and retained evidence

Host Artifacts/FS/Process, Testkit and the release editor remain **0.70.0**;
Platform remains **0.8.1**, management DTO **0.11.0**, external Principal **0.1.5**,
Agent SDK **0.49.3**, Candid **0.10.39**, PocketIC **16.1.0** and Rust floor
**1.91.0**. Shared **0.27.7**, revision
`11d519c604b1727a2bf6272557a97e770a63b29f`, retains all 126 exact payloads.
[The review](../shared-tooling-review.json) owns unchanged tooling provenance;
[implementation contracts](../implementation-contracts.md) and
[Platform ownership](../ic-platform.md) remain mandatory. Supplier crypto/TLS
qualification stays with [Platform #50](https://github.com/dragginzgame/ic-platform/issues/50).

Earlier tooling adoption evidence remains `/tmp/backup-tooling070-*`,
`target/msrv-consumers.hYmod9`, `target/shared-release-tests.JfUFrx`,
`/tmp/ic-backup-agent-independent-build-536636-0` and
`/tmp/ic-backup-agent-pocketic-536636-1`. Its initial admitted-installer wait timeout,
subsequent successful setup and all older tool bundles remain retained. Its source,
graph and native-host scope are historical, not new qualification of this refactor.

## Earlier integration and retained evidence

The preceding Platform 0.8.1 batch passed 81 IC model/application-ingress tests,
16 HTTP tests, strict Clippy, independent MSRV consumers and a full PocketIC
round trip on its then-current Host/Testkit 0.69.14 graph. Preserve
`/tmp/backup-platform081-*`, `/tmp/ic-backup-agent-independent-build-3940488-0`,
`/tmp/ic-backup-agent-pocketic-3940488-1` and `target/msrv-consumers.g00fEa`.
The Platform archive SHA-256 remains
`b28e5352b4a6fa35217dd5df181a5555197936477570f034a53ffd7dda221c65`, with published
source `10c2610511d6a5525816185c999b25fd159c1d89`.

Delivered operator composition uses existing plan/journal/coordinator owners;
no second journal, scheduler or authority provider exists. The
[operator map](../canic-adoption.md#reusable-operator-composition) and
[Canic removal map](../canic-adoption.md#canic-engine-removal-map) retain integration
ownership. Earlier read-only Canic inspection used
`b9ed7f298bdaf8c9cceac142edf7e56b15d33208`, fingerprinted in
`/tmp/backup75-canic-source.json`. Fleet/Root policy, Root-paid effects, application
hooks and terminal acceptance remain Canic ports; multi-unit/fencing and
external-obligation acceptance remains under Canic #513/#550/#394.

Preserve `/tmp/backup75-*`, original build
`/tmp/ic-backup-agent-independent-build-2456269-0`, fixtures
`/tmp/ic-backup-agent-pocketic-2456269-*`, corrected death-case build
`/tmp/ic-backup-agent-independent-build-2635797-0` and fixture
`/tmp/ic-backup-agent-pocketic-2635797-1`. Preserve `/tmp/backup74-*`,
`/tmp/backup-049-guidance-*`, `/tmp/backup-simplify-*`, earlier independent/PocketIC
fixtures, Canic/release/consumer journals, `/tmp/backup-reference-release-*`,
`/tmp/backup-tooling696-*`, `/tmp/backup-platform07*`, `/tmp/backup-72*`,
`/tmp/backup-shared27*` and the 20261027/20261028 failure logs in
`target/validation-failures/`. Successful tests and closed issues authorize no cleanup.
