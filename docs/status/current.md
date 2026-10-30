# Current handoff — 2026-10-29

## Further complexity refactor after 0.50.1

Baseline HEAD is `794d65b92e3dd3c2d19c893bd3102114e8f397f9` (Release 0.50.1).
The user authorized all ten further source-review simplifications. Pending
**0.50.2** notes cover this compatible batch; package versions remain **0.50.1**.
All changes are unstaged/uncommitted. The incoming dirty Cargo.lock is preserved
byte-for-byte; this batch changes no dependency, snapshot payload, sibling,
Git/release state, issue, publication or live IC state.

`ExecutionWorkflowRecord::allocation` is now a public read-only original-plan
accessor. The independent normal consumer constructs its singleton child plans
through existing typed model constructors, without serializing or editing workflow
JSON. The catalog still supplies only purposes/ceilings; original stage binding,
complete journals, learned input qualification and fresh authority remain required.
The new API is local candidate source, not an API in the published baseline.

Mutation, transfer-read, upload and application requests share a neutral pending
mutation binding, preserving original payload admission and public error order.
Upload data uses the existing region classifier for wire offsets; readback reuses
metadata region-size projection. Sessions retain immutable member declarations
while freshly admitting the closed tree, file identities/metadata and each exact
payload around every write. No file observation or content digest is cached as
new authority. Existing source read-count measurements remain unchanged.

Download artifact admission checks bounds, sorts once and detects adjacent duplicate
principals without another set. Internal progress/settlement paths consume borrowed
journal iterators; public request APIs and complete coverage/causality checks remain
unchanged. Persisted nullable fields share strict presence decoding; absent fields
still reject. Journal/command lock errors share only unsafe-entry classification.
Capture/fence and checkpoint paths share returned-reply ownership on error, retaining
all original guards and leaving decoded transfer borrowing with its existing owner.
Public errors, record/wire bytes, spending and recovery semantics are unchanged.

## Focused qualification

Strict Core/Agent Clippy passes for all targets/features with warnings denied.
All **180 selected Core cases** pass across request identities, upload codecs,
nullable records, inventory, progress/settlement, workflow reply retention,
artifact/session behavior and actual journal/descendant custody. Existing maximum
chunk/region and linear byte-read checks pass. All **16 Agent HTTP boundary cases**
pass with exact request bytes, bounded replies, retained uncertainty and single
submission behavior. No new throughput or timing claim is made.

All **seven independent normal-consumer PocketIC cases** pass: complete capture,
transfer, manifest, source-bound upload, same-ID load/start and terminal replay;
process death after authenticated load; operator ingress/reply-retention and partial
transfer failures; and four opaque application/recovery cases. The independent
build/normal-graph evidence is `/tmp/ic-backup-agent-independent-build-1755641-0`.
Original fixtures are `/tmp/ic-backup-agent-pocketic-1755641-*`, including complete
journey `-9`, load process death `-11` and operator failures `-13`, `-15`, `-17`.
Preserve original plans, journals, envelopes, replies, byte trees, manifests and
raw process/server output. The production consumer excludes Testkit/PocketIC/WAT.

Logs are `/tmp/backup-simplify051-*`: `-clippy.log`, `-core.log`, `-http.log`,
`-independent.log`, `-rustdoc.log` and `-source-checks.log`. The missing initial
0.70.2 CLI admission is retained in `-server-check.log`; explicit selected setup
passes through verified cache reuse in `-server-setup.log`, preserving the PocketIC
16.1.0 assets and older selections. Simulator startup separately admits assets
offline through the locked library.

The graph stayed fixed throughout validation. `/tmp/backup-simplify051-graph.sha256`
binds manifest `d8645d6e456d8f8dd1f006f43aae16c64f496237941857c8c3f7781e92dd94dc`
and lock `2038a6b872c1b82ca7410d449dda6a5b6311d820209aff91ad1298cae6ba779c`.
Selected Platform **0.8.2**, Host/Testkit **0.70.2**, Candid **0.10.39**, Agent SDK
**0.49.3**, external Principal **0.1.5**, management DTO **0.11.0**, PocketIC
**16.1.0** and Rust floor **1.91.0** remain unchanged. Incoming UTF dependencies
remain unchanged too. No full local gate, new MSRV/native macOS matrix,
registry-only product acceptance, installed Canic migration or live application
qualification was run.

## Earlier complexity evidence

The preceding ten-refactor batch is included in the baseline source. Preserve
`/tmp/backup-simplify050-*`, independent build
`/tmp/ic-backup-agent-independent-build-2604704-0` and original fixtures
`/tmp/ic-backup-agent-pocketic-2604704-*`. Its 147 Core, 11 coordinator, 16 HTTP and
seven independent simulator cases retain their original source and graph scope.
Its corrected fixture caller compile error remains retained. These older results
are not new qualification of this batch or its incoming dependency updates.

## Earlier tooling and integration evidence

The earlier 0.70.0 tooling and Platform 0.8.1 adoption receipts retain their
original source/graph scope. Current selections are recorded above. Shared **0.27.7**, revision
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
