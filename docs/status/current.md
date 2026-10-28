# Current handoff — 2026-10-27

## Delivered source and accepted continuation

Released source is **0.47.0**, `97a0be383097fbeda819614fcea9dc9061122689`.
Its [changelog](../../CHANGELOG.md#0470---2026-10-27) describes the workflow and
settlement error hard cuts. The [released handoff](https://github.com/dragginzgame/ic-backup/blob/97a0be383097fbeda819614fcea9dc9061122689/docs/status/current.md)
retains the preceding simplifications and their original focused evidence.
It was written before release; its pending-version and dirty-state descriptions
are historical. Source delivery is not registry publication or application acceptance.

The accepted continuation reviews the incoming Host/Testkit lock update,
current native issue acceptance and the next application integration step.
Core owns bounded records and original-plan coordination; Agent owns reserved
signed transport. Applications retain membership, authority, release binding,
fencing and external obligations. [Implementation contracts](../implementation-contracts.md)
remain authoritative. No live backup/restore, release, commit or push is selected.

## Pending compatible dependency adoption

The incoming Cargo.lock update selects published Host Artifacts/FS/Process and
Testkit **0.69.2**, replacing 0.69.1. No other package selection changes.
The reviewed upstream release is `1e245a9e8791b04daa0bf971124ae8908d956821`;
the registry Testkit package records that source. Process state and private
exchange modes are consolidated; baseline-pool completion retains fatal discard,
one bounded rebuild and both failure causes. Backup does not use that pool.
No public API, retained format or product policy change is selected.

The [pending changelog](../../CHANGELOG.md#0471) selects **0.47.1** for compatible
dependency adoption and refreshed Canic guidance. Own package versions remain
0.47.0. Workspace dependency declarations remain on the compatible 0.69 line;
Platform stays **0.7.0**, release editor **0.69.0**, Shared Tooling **0.26.23** and
consumer Rust floor **1.91**. No vendored snapshot payload or sibling is edited.
Working lock SHA-256:
`e2bf03f5442ce8fe2a6ed646333793fcf3f88953c99ff369f0d5fc9f5f502193`.

## Current focused evidence

All six Core command-lifetime custody tests pass under the new graph, including
failed spawn, descendant custody after owner death, unsafe sidecars and bounded
finish. Strict Agent library and HTTP-boundary target Clippy passes. Logs:
`/tmp/backup-host0692-custody.log` and
`/tmp/backup-host0692-agent-clippy.log`.
Dependency inheritance/boundary checks, local documentation links and diff checks
also pass; static logs are `/tmp/backup-host0692-{boundaries,doc-links}.log`.

The matching Testkit CLI is explicitly prepared through the shared installer;
its offline check admits the existing authenticated PocketIC 16.1.0 server.
The first setup was denied write access to the shared store; the authorized retry
succeeded. Original setup/check failures remain retained. Logs:
`/tmp/backup-host0692-testkit-{setup-authorized,check-prepared}.log`.
These are native Linux checks, not simulator execution or current native macOS/MSRV
qualification. No full repository gate or new Agent HTTP execution ran here.

The earlier full Clippy failure and its narrow correction remain documented in
the released handoff. Original failure:
`target/validation-failures/20261027T172540Z-3411640-8-clippy.log`.
Passing Core `--tests` Clippy and six capture-settlement cases are retained under
`/tmp/backup-settlement-lint-fix-*`; they predate the new dependency graph.

## Native acceptance and application boundary

Exact delivered-source [CI 40553307413](https://github.com/dragginzgame/ic-backup/actions/runs/40553307413)
remains in progress on Linux, Intel macOS and Apple Silicon. It selects released
0.47.0's 0.69.1 lock, not this dirty 0.69.2 graph. Do not claim a complete native
pass or close [#70](https://github.com/dragginzgame/ic-backup/issues/70) and
[#71](https://github.com/dragginzgame/ic-backup/issues/71) without their applicable
complete matrix. Both fixes and their consumer cases are delivered; inspect the
latest complete results on the next activated maintenance pass without polling.

[Canic adoption](../canic-adoption.md) records the read-only inspection of local,
unpublished Canic `f768a5519edb35512912dd73fce45936155edc6e`, selecting Backup
0.45.1. Its installed Root-dispatch fixture already covers two participants,
partial preparation, Root upgrade and release replay. No Canic test ran here.
Reuse that owner for original-plan and Root-paid-effect integration; preserve
live-create refusal and the active restore engine until replacements qualify.
[#29](https://github.com/dragginzgame/ic-backup/issues/29) retains complete
application, current-permission, external-obligation and terminal acceptance;
the independent registry consumer's earlier proof has its narrower scope.

The unresolved crypto-bigint qualification is now owned by
[Platform #50](https://github.com/dragginzgame/ic-platform/issues/50).
Backup #68 closed as superseded tracking, not as a vulnerability fix or exemption.
Original plans, journals, bytes, references, obligations and failed evidence stay
retained. No spending reset, blind retry, inferred receipt or cleanup is introduced.
