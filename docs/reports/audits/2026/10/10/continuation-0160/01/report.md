# Async transfer integration and native jobserver repair

Released base is 0.15.0, `eae844847e109004eb9427bfd02bd66e16f9e6df`.
The uncommitted 0.16.0 draft changes the public transfer-read provider and
single-read/metadata/download coordinators to futures with async admission and
independent qualification. Consumers must await them and migrate callbacks.
Package versions and release receipt remain 0.15.0.

`AgentSnapshotTransferReadProvider` reuses canonical reserved-update admission,
checks the currently guarded original journal and requires durable exact signed
envelope/request-ID retention before one submission. Retention refusal sends
nothing; accepted/lost replies stay pending. Existing response/decoder and receipt
owners remain separate. Selected journal exclusion spans admission/submission or
qualification awaits. Cancellation releases exclusion while preserving spending,
earlier Applied download receipts and partial Created files; reentry cannot repeat
a pending update. Qualifiers retain each reply before cancellable work. No runtime,
`Send` bound, compatibility facade, persisted schema or accounting owner is added.
Upload/recovery/fence provider families remain synchronous; full application and
terminal/fence/reference custody remain independent work under #25/#29.

Regression traces cover cancellation during admission/provider suspension for
both metadata/data, retained exact metadata replies during suspended qualification,
and cancellation of the second download qualification after one applied append.
The latter checks the exact retained Wasm prefix, first receipt, pending second
reservation, occupied staging and refused reentry. Actual HTTP cases verify durable
signed retention before zero/one request and unchanged pending journals for refusal,
accepted and lost responses. Real Agent simulator metadata/data reads use the
configured adapter and public single-read coordinator throughout complete transfer;
the Testkit fixture separately exercises the async metadata/download loop.

Released [0.15.0 CI](https://github.com/dragginzgame/ic-backup/actions/runs/38066879810)
has Linux success, Apple Silicon failure and Intel macOS running at final inspection.
The retained Apple Silicon artifact now includes the original Testkit routing logs:
the local mock compiler rejects GNU Make's supported FIFO authentication before
any build. [#37](https://github.com/dragginzgame/ic-backup/issues/37) owns the defect.
The exact released source reproduces on actual GNU Make 4.4.1/Bash 3.2.57 in a
separate export. The working-tree fix accepts numeric descriptors and actual FIFO
paths, refusing missing authentication and nonexistent/non-FIFO paths. Both default
FIFO and forced pipe fixtures pass with those real tools; the complete consumer
tooling/evidence-retention fixture also passes under Make 4.4.1/Bash 3.2.57. This is
Linux portability evidence, not a matching native macOS result. Vendored Shared
Tooling remains unchanged at committed 0.3.7, revision
`34e5ad7aac3599306c9572bb547f2239d09df1a3`, with all 96 files verified.

Preserve incoming Metrics 0.5.5 and released Testkit 0.33.1. Metrics Rust source
matches 0.5.4; Testkit changes upstream tests only. An incoming lock update selected
Host 0.12.8 after initial focused checks; all four Host Rust trees match released
0.12.7. Retain the incoming selection and qualify the final graph afresh. Archive
hashes match the exact lock and published VCS identities. No sibling patch exists.

All 16 full `make ci` gates pass on the final graph, including 507 core unit cases,
29 core simulator journeys, ten Agent HTTP cases, four Agent simulator journeys,
strict Clippy/docs, Rust 1.88 independent consumers and both current package archives.
A separate locked Rust 1.88 normal consumer compiles all three new coordinators
with the configured Agent adapter, excludes Testkit/PocketIC and keeps Metrics
features empty. Technical source bytes/modes and the lock remain unchanged throughout
qualification. Final documentation links and snapshot checks pass separately.

Evidence is retained under `target/continuation-0151/`; initial mechanical migration,
callback/lint and fixture-helper failures remain separate from passing final proof.
The original failed Make fixture and actual compiled tools remain under `/tmp`.
[Machine evidence](qualification.json) records successful case membership, exact
technical inputs, graph/archive/tool identities and logs. The candidate has no
hosted result. No root Git write, release, registry upload, production IC effect,
sibling edit or recovery cleanup ran. Issue updates retain unresolved native and
full application acceptance rather than closing them from isolated local success.
