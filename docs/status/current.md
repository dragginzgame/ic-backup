# Current handoff — 2026-10-29

## Delivered source and local application-ingress work

Delivered **0.48.3** is `49d4cdc50069aa19d783af5ffc6f5c0b557a7ae6`.
It includes the reviewed ten-part simplification batch and Shared Tooling adoption.
The working tree implements [#74](https://github.com/dragginzgame/ic-backup/issues/74),
the Backup-owned opaque application transport gap found during Canic integration.
GitHub remains the issue tracker; #74 stays open pending delivery.

Core now admits immutable opaque application payloads and distinct original
mutation/recovery-observation requests. Digests bind canonical receiver and effective
target, fixed update mode, exact method/arguments and finite reply ceiling. Mutation
and recovery target admission is available before spending. Current original journal
authority, pending attempts and separately reserved observation bytes are rechecked.
No persisted schema, old request commitment or pending layout is reinterpreted.

Agent's application provider reuses the existing single-submit owner. Both signed
ingress/ID retention before dispatch and bounded raw reply/ID retention before
cancellable qualification are mandatory synchronous callbacks. Original kind/attempt
witnesses must reject prior or uncertain dispatch. There is no polling, retry,
envelope import, automatic receipt, application codec or fence-only interpretation.
Fresh authority, custody, Root-paid effects and application acceptance stay external.

`ReservedUpdate` gains two public variants. Pending **0.49.0** changelog notes record
the exhaustive-match compatibility change; package versions remain **0.48.3**.
All local changes are unstaged/uncommitted. No release, publication, issue mutation,
sibling source edit or live operation was performed. User-owned Git/release authority
is unchanged. See the [transport guide](../agent-transport.md#opaque-application-ingress)
and [Canic adoption map](../canic-adoption.md#pending-application-transport-adapter).

## Focused qualification

The final payload/custody example passes **3 Core model tests**, **16 HTTP tests**
and **4 independent normal-consumer PocketIC journeys**. Cases cover exact framing,
bounds, changed original fields/context/signer, mandatory retention, accepted/lost/
cancelled responses, process death after durable reply, separately budgeted recovery,
reply refusal/ceiling and completed replay without new signing/submission. An earlier
run also passed both existing independent capture/restore/death journeys, all six
together. Public Core application-ingress tests (3) and Agent reply redaction (1) pass.
Strict affected Core/Agent Clippy passes for all targets/features. Rust **1.91.0**
checks the Agent library and Core dependency on the unchanged locked graph.
Warning-denied public Rustdoc, formatting, links, dependency boundaries/inheritance,
repository inventory and shared snapshot integrity checks pass.

Logs are `/tmp/backup74-*`. Final independent graph/build evidence is
`/tmp/ic-backup-agent-independent-build-2666934-0`; original application fixtures
are `/tmp/ic-backup-agent-pocketic-2666934-{1,3,5,7}`. The earlier six-case build is
`/tmp/ic-backup-agent-independent-build-1992332-0`, with its original fixtures under
`/tmp/ic-backup-agent-pocketic-1992332-*`. Preserve original declarations, envelopes,
reply bytes, request IDs, plans, journals, binaries and raw server/process output.

These are candidate-source normal-dependency checks, not registry-only or unpacked
archive acceptance for the new API. No full local gate, new native macOS matrix,
production Canic Wire/Root-paid qualification or live IC operation ran. Delivered
CI for an earlier source does not qualify this working tree.

## Dependency graph and tooling

Dependency updates remain paused. Manifest and lock fingerprints are unchanged;
`/tmp/backup74-graph.sha256` records both. The current lock SHA-256 is
`b56435056afc722ac5476ae25bec18fa47ab84693d8622f04a7f5f2c03399e03`.
Locked Host/Testkit **0.69.11**, Platform **0.7.5**, AWS-LC **1.19.0**/sys **0.47.0**
remain selected. Declared Host/Testkit minimums and release editor are **0.69.9**,
Platform's minimum **0.7.3**. Management DTO **0.11.0**, external Principal **0.1.5**,
Agent SDK **0.49.3** and Candid **0.10.39** stay unchanged. Exact Testkit CLI/server
offline admission passes with prepared PocketIC **16.1.0**. This work supplies no
supplier crypto/TLS acceptance; retain [Platform #50](https://github.com/dragginzgame/ic-platform/issues/50).

Shared **0.27.6**, revision `5a72359b5be19d5221334cf2c9b3204621d3f3f6`, remains
exporter-owned. [The review](../shared-tooling-review.json) retains its adoption
and older graph qualification. No snapshot-owned payload was edited. Backup retains
fresh managed servers, raw management ingress and independent fixture custody.

## Integration ownership and historical evidence

[#72](https://github.com/dragginzgame/ic-backup/issues/72) and
[#73](https://github.com/dragginzgame/ic-backup/issues/73) were closed using delivered
0.48.2 native evidence; [CI run 40698974873](https://github.com/dragginzgame/ic-backup/actions/runs/40698974873)
passed Linux, Apple Silicon and Intel macOS for that source. #29 was closed as
redundant coordination, transferring unresolved application acceptance to Canic.

[Canic #513](https://github.com/dragginzgame/canic/issues/513) owns engine/Root-paid
composition, complete transfer, exact custody and terminal/reference authority;
[#550](https://github.com/dragginzgame/canic/issues/550) owns application freeze,
external obligations, post-load acceptance and safe release;
[#394](https://github.com/dragginzgame/canic/issues/394) owns fresh live admission.
At the read-only review, [Canic PR #702](https://github.com/dragginzgame/canic/pull/702)
was open at `78a57d5b1e83a357999bfd4afbe88a06b0809c64`, isolated in
`/tmp/canic-backup-adoption.lzm8Ut`, using older Backup APIs. Its native/synthetic
cases are not production Root-paid or whole-application qualification.

Preserve `/tmp/backup-simplify-*` and its original graph/build/fixture evidence,
including `/tmp/ic-backup-agent-independent-build-886517-0` and
`/tmp/ic-backup-pocketic-822447-*`. Keep the 20261028 test and 20261027 Clippy
failure logs in `target/validation-failures/`, `/tmp/backup-reference-release-*`,
`/tmp/backup-tooling696-*`, `/tmp/backup-platform07*`, `/tmp/backup-72*`,
`/tmp/backup-shared27*` and original consumer/release/Canic journals. Closed issues
and successful tests authorize no cleanup.
