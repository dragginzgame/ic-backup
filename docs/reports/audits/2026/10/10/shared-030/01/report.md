# Shared Tooling 0.3.0 complete-toolset adoption

Released Backup base: 0.13.0, `bf5138ec99c2f02650b8af618fc30195ef50dada`.
The incoming Testkit 0.30 manifest/lock selection is preserved. Package version
and receipt remain 0.13.0. Select draft 0.14.0 because standard aggregate tool
commands change their required roster and invocation contract.

Adopt committed Shared Tooling `88a73139a0f083344c41a6f6f4b5c3a8aca7dc1d`,
version annotation 0.3.0, through a clean private clone and canonical exporter.
The same 95-file roster is retained. Dirty sibling-reporter work stays upstream.
Common host, IC and Cargo tools are now mandatory; old optional host flags and
redundant Rust aggregate prerequisites retire. Preserve admitted installations,
receipts, server assets and failed/recovery evidence.

Backup registers `deps install-testkit-server` in the ordered local setup list
and `testkit-server-check` in the check list. Aggregates own host → IC → Cargo →
product order even under parallel Make. Release preflight uses setup/check after
original-source admission and before the full gate; saved prepared/committed/
tagged recovery skips setup. Source/manifest/lock/receipt rechecks remain.
CI prepares the declared toolchain, then uses the same aggregate and local PATH,
retiring separate Cargo-sort and Testkit preparation. Ordinary checks stay offline.
No schema, spending, product transport or full runner is added.

Incoming Testkit 0.30.0 points to published
`6ac161b8ed689012bf8b0ce926946f94ea9f407d`. Its build diagnostics retain bounded
prefixes without terminating builds; identity/metadata probes require complete
bounded output. Backup uses existing managed-server APIs, not Host process
re-exports. Direct Host remains 0.11; released Metrics requirement 0.4 stays selected.
No independent dependency reselection or compatibility shim is added.

The complete unchanged `make ci` roster passes on the final graph: snapshot and
full common/product offline admission, declarations/links/fetch, ShellCheck, all
selected tooling/release/hook fixtures, formatting, both-library native check and
strict Clippy, all tests/doctests, warning-denied docs, Rust 1.88 and both packages.
The core has 492 unit tests and 22 actual simulator cases; Agent has eight HTTP
boundary cases and three actual simulator cases. Other public integrations pass.
Both independent MSRV consumers exclude simulator/process/tool dependencies.

Actual setup prepares Cargo-sort 2.1.4, Cargo-sort-derives 0.13.0, Candid-extractor
0.1.6 and the exact Testkit 0.30.0 CLI. Repeated aggregate setup/offline check
reuses them, preserving captured binary/receipt/server bytes and older Testkit
selections; incoming root manifest/lock hashes remain unchanged.

The actual parallel Make routing fixture passes for common setup/check order,
locked fetch before Testkit and stopping after common failure. Release fixtures
prove admission before the gate, setup failure before effects, original retry
reuse and saved recovery without provisioning. Canonical fixtures retain real
installer/receipt ownership. Workflow actionlint passes after grouping PATH writes.
Root help is ASCII sorted. The first release fixture incorrectly intercepted
Make commands that the aggregate dispatches through its original recursive Make;
corrected common installer substitutes exercise that real path. Its failed fixture
and initial workflow-lint evidence stay retained.

[qualification.json](qualification.json) binds exact source hashes, selected graph,
commands, logs, failed attempts and reuse evidence. Documentation-only completion
receives snapshot/link/whitespace checks after this complete gate.
Released 0.13.0 [CI run 38044109719](https://github.com/dragginzgame/ic-backup/actions/runs/38044109719)
has Linux success and both macOS jobs queued at final inspection; it does not
qualify the dirty 0.14.0 candidate. Keep matching native acceptance with
[#36](https://github.com/dragginzgame/ic-backup/issues/36) and
[#32](https://github.com/dragginzgame/ic-backup/issues/32).
Logs and exact source/graph evidence remain under `target/shared-030-review/`.
No root Git write, release/publication, production IC effect, sibling edit or
recovery cleanup is performed. Native hosted acceptance remains separate from
this dirty candidate's local proof.
