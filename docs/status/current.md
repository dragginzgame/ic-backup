# Current handoff — 2026-10-27

## Released source and ownership

Released Backup is **0.44.4**, `063318da739d42833d0998fe40d0aeddd68ef65e`.
The [released handoff](https://github.com/dragginzgame/ic-backup/blob/063318da739d42833d0998fe40d0aeddd68ef65e/docs/status/current.md)
retains the earlier adoption evidence. Core owns bounded records, guarded
persistence, reconciliation and original-plan coordination. Agent owns reserved
signed transport. Applications retain authority, membership, release binding,
whole-unit fencing and external obligations. [Implementation contracts](../implementation-contracts.md)
remain authoritative.

## Pending 0.44.5 simplification

All ten reviewed complexity reductions are implemented. Platform supplies bounded
Candid sequence decoding and strict reserved-marker admission. Backup retains its
product bounds, aggregate quotas and persisted-record visitor. Snapshot extent
classification and learned-predecessor checks have one local owner; stage-binding
validation borrows retained data instead of constructing a temporary record.

Qualification, exact Applied receipt admission, domain work and journal recording
are explicit in each affected workflow. Shared journal publication invalidates
cached state before writing and only restores usability after success. Artifact
verification and upload reads share descriptor custody through closing namespace
and original-plan checks. Agent reservation admission now returns its wire
projection in one match. Ordinary integration journeys reuse typed plan fixtures;
malformed JSON and independent wire/hash goldens remain explicit.

Public APIs, typed errors, v1 records, digest domains, wire bytes, bounded authority
and effect/recovery ordering are preserved. Notes select pending **0.44.5**;
package versions remain **0.44.4**.

The incoming lock selects Host/Testkit **0.68.9** and Platform **0.6.19**; manifest
minimums remain tooling **0.68.8** and Platform **0.6.19**. Shared Tooling remains
the reviewed **0.26.21** snapshot. This batch does not update dependencies: the
incoming lock remains byte-identical, SHA-256
`a72b14d4ff05e69f8e98a006ea8094ab743f73ddc6de90b44925112b568adbbc`.

## Focused evidence

Formatting, documentation links and strict all-target/all-feature Clippy pass for
both packages. Focused Core model, workflow, attempt/download journal and execution
workflow tests pass, including receipt rejection, cancellation, indeterminate
publication, custody and process-death cases. All thirteen affected integration
targets and fourteen Agent HTTP boundary tests pass. Logs are retained under
`/tmp/backup-simplify-*`; starting source is retained in
`/tmp/backup-simplify-20261027-start/`.

Independent normal consumers pass on Rust **1.91.0**, with evidence at
`target/msrv-consumers.0C7Esd/`. The locked Testkit CLI is explicitly prepared and
offline-admitted; PocketIC **16.1.0** is reused through its authenticated asset
owner. Core's original-workflow capture/restore/replay case passes, retaining
`/tmp/ic-backup-pocketic-1406912-0/`. Both independent Agent consumer journeys pass:
cold capture/restore/acceptance/terminal replay and death after authenticated load
without redispatch. Build evidence is at
`/tmp/ic-backup-agent-independent-build-1424340-0/`; simulator evidence remains at
`/tmp/ic-backup-agent-pocketic-1424340-1/` and
`/tmp/ic-backup-agent-pocketic-1424340-2/`.

## Limits and delivery

This is focused native Linux and simulator evidence, not a new full CI, macOS,
installed IC, package-archive/registry-only or release qualification. Deployment
binding and application acceptance remain consumer-owned. Agent's previously
recorded transitive crypto-bigint 0.5.5 concern remains unresolved. Changes are
unstaged and uncommitted. No package version, release, tag, publication, deployment,
live IC effect or sibling edit is performed.
