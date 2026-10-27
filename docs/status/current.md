# Current handoff — 2026-10-27

## Released source and ownership

Released Backup is **0.45.1**, `5444f6f83c963e21af379089f39c9fa61abf9088`.
The [released handoff](https://github.com/dragginzgame/ic-backup/blob/5444f6f83c963e21af379089f39c9fa61abf9088/docs/status/current.md)
retains the Shared Tooling 0.26.23 adoption, #70 diagnosis and earlier evidence
with their original scope. This batch does not requalify those hosted results.
Core owns bounded records, guarded persistence and original-plan coordination;
Agent owns reserved signed transport. Applications retain authority, membership,
release binding, fencing and external obligations.
[Implementation contracts](../implementation-contracts.md) remain authoritative.

## Pending 0.46.0 simplification batch

All ten reviewed simplifications are implemented. Fence acquisition and
reconciliation now preserve their returned evidence through one `AfterReply`
variant and a typed Journal, Association, Stage or Obligation cause. Every ordered
post-reply check remains; original pending spending, obligations and source
references remain retained. Existing failure fixtures match the nested typed
causes and still assert exact replies, reservations and bounded provider calls.

Complete workflow replay keeps each admitted binding and settlement projection in
one canonical vector. Predecessor lookup uses the catalog's sequence order, without
parallel vectors or another index. Opening/closing reads, complete journal replay,
ancestor evidence and one-lock-at-a-time custody are unchanged.

Agent observation reply bindings derive from the exact reserved request variant,
so callers no longer supply a second independent identity. Transfer payload
accessors return their original request lifetime, allowing Agent to borrow their
bytes directly while keeping metadata/data reply limits separate.

Inventory settlement identities compose the existing operation settlement
identity instead of rebuilding it for comparison. Metadata retains its admitted
Platform Principal; data requests reuse it instead of reparsing canonical text or
retaining another raw principal copy. Membership views borrow the admitted
observation. Progress derives Applied membership from its original journal index,
and plan admission searches its already canonical selection. Original wire/hash
framing, normalization, typed denials, schemas and numeric bounds remain unchanged.

The Testkit command fixture uses Shared's existing dependency-closure copier for
its selected roots. No snapshot-owned tooling or sibling checkout was edited.

The public fence error enum changes require **0.46.0** notes. Consumers match
`AfterReply { source, acknowledgement }` or `AfterReply { source, observation }`
and inspect `FenceAcquisitionReplyError` or `FenceReconciliationReplyError`.
There are no compatibility aliases or retained-data migration requirements.
Package versions remain **0.45.1**; no version or release operation was performed.

## Focused evidence

All **100** selected Core tests pass: model filters `ic_snapshot_data`,
`ic_snapshot_metadata` and `operation_plan`; policy filters `execution_progress`,
`membership`, `ic_observation::capture_settlement`,
`ic_snapshot_upload_observation::settlement` and `ic_snapshot_transfer_read`;
`ops::persistence::execution_workflow`; `workflow::fence_acquisition` (including
reconciliation/settlement); and public targets `fence_acquisition` and
`fence_reconciliation`. These include fixed codec/digest goldens, mismatched
identities, retained-reply failures, cancellation and sparse-stage replay.
The initial model transfer-read filter selected zero tests; its actual policy
owner was then selected and all five registered tests pass.

Agent's `http_boundary` target passes all **14** tests, including the three
observation variants, signed byte retention, zero/one-call bounds, cancellation
and pending spending. The sandbox initially refused local socket listeners;
the approved loopback-capable rerun passes. The `reply_evidence` golden test also
passes. These native substitutes qualify local transport/orchestration, not IC
management effects or authenticated application settlement.

Library Clippy for both packages passes with `-D warnings` after eliding the
unused metadata lifetime. Testkit Make routing/refusal passes using the copied
closure. Formatting, focused shell checks, local Markdown links and diff checks
pass. Logs, including initial failed attempts, remain at
`/tmp/backup-complexity-*`. Native test fixtures retain their original custody
and cleanup rules; no original evidence was manually removed.

Cargo.toml and Cargo.lock are byte-identical to HEAD. Lock SHA-256 is
`6a49a1ab0cc5a506a8780541a4b60125b819e7c3ffb52d96e14c4b2fe495c825`.
Shared Tooling **0.26.23**, Platform **0.7.0**, Host/Testkit/editor **0.69.0** and
the Rust **1.91** floor are unchanged.

## Limits and delivery

No full suite/CI, simulator, native macOS or MSRV run is claimed for this batch.
No live IC effect, complete application acceptance, terminal settlement or
fence/reference-release qualification is added. Earlier Agent crypto-bigint
qualification remains unresolved. Changes are unstaged and uncommitted; no own
version bump, tag, push, publication, deployment or release was performed.
