# Current handoff — 2026-10-24

## Ownership and source

Released Backup is **0.42.0**, `c300c05`. Core owns bounded records, guarded
persistence, reconciliation and original-plan coordination. Agent owns reserved
signed transport. Host owns generic IO/process/hash capabilities; Platform owns
external IC identities and SDKs; Testkit owns simulator assets and processes.
Applications own current membership, permissions, release binding, whole-unit
fencing and external obligations. [Implementation contracts](../implementation-contracts.md)
and the [record/API boundary](../extraction-boundary.md) retain safety obligations.

## Pending cleanup

Pending 0.43.0 removes three provider-interface test fakes that principally tested
their own configured errors. Existing model binding/golden tests and production
workflow failure/reopen tests retain meaningful coverage. Agent fixture writers
share create-only private file writes while preserving file/parent synchronization
order and partial evidence. The independent compiled consumer remains self-contained.
The public incomplete-stage constructor is removed; integrations use `prepare`
for complete fresh allocation and `open`/`resume` for retained evidence. Internal
partial-preparation tests still exercise private allocation and crash barriers.
This API cut requires a minor release; package versions remain 0.42.0. Wire,
storage, runtime and dependencies are unchanged.

Canic's inspected working catalog selects Backup 0.42.0. Its serial restore journal
has replaced the removed effect_order call with graph-based local completion
checks. This removes the old adoption blocker, not the need to integrate original
stage/attempt accounting or qualify application authority and Root-paid effects.
The active engine and original journals remain supported until that replacement
is qualified. See [Canic adoption](../canic-adoption.md).

Historical extraction disposition and Testkit 0.25 qualification now link to frozen
released evidence instead of competing with current contracts. The
[0.42.0 handoff](https://github.com/dragginzgame/ic-backup/blob/c300c05/docs/status/current.md)
retains earlier Linux, MSRV, package and simulator results at their original source
and graph scope; they are not fresh validation of the incoming lockfile edit.

## Validation and remaining work

Focused Linux validation passes: 525 Core unit tests, strict all-target/all-feature
Clippy, five actual PocketIC fence-reconciliation cases, and the application-update
case covering empty replies and retention refusal. Formatting, documentation links
and whitespace checks pass. Logs: `/tmp/backup-cleanup-core.log`,
`/tmp/backup-cleanup-clippy-fixed.log`, `/tmp/backup-cleanup-fence.log`,
`/tmp/backup-cleanup-reply.log` and `/tmp/backup-cleanup-docs.log`.
An initial helper name collision was corrected before the passing Clippy run.
The incoming Cargo.lock update to Platform 0.6.14 is preserved byte-for-byte;
these focused results use that selection, not the earlier 0.6.13 graph. No package version, staging, commit, push, publication, live IC effect
or sibling edit is performed. No full local make ci or native macOS qualification
is claimed. Production Canic acceptance and the Agent crypto advisory remain
separate work; historical passing fixtures do not resolve them.

The stage-constructor cut additionally passes 38 workflow/preparation/recovery unit
cases, the public execution-workflow journey, and the actual PocketIC lost-download
reply case using public `prepare` for occupied-stage refusal. Strict all-target
Clippy, formatting and documentation links pass. Logs are
`/tmp/backup-stage-cut-{tests,public,pic,clippy,docs}.log`. No Canic constructor
caller was found; its existing restore engine remains untouched.
