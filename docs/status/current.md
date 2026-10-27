# Current handoff — 2026-10-27

## Released source and ownership

Released Backup is **0.44.6**, `7a13aca63c5449ceeb815e10f18a7c56f61eb1f4`.
The [released handoff](https://github.com/dragginzgame/ic-backup/blob/7a13aca63c5449ceeb815e10f18a7c56f61eb1f4/docs/status/current.md)
retains the earlier simplification and qualification evidence. Core owns bounded
records, guarded persistence, reconciliation and original-plan coordination.
Agent owns reserved signed transport. Applications retain authority, membership,
release binding, whole-unit fencing and external obligations.
[Implementation contracts](../implementation-contracts.md) remain authoritative.

## Pending 0.44.7 simplification

All ten items in the third requested batch are implemented. Download admission
compares the plan's original metadata once against the writer. Request admission
reads pending attempt IDs from the validated journal projection without building
reporting views. Mutation, upload and transfer-read policies share borrowed response
identity projection. Fence validation compares original bindings without temporary
obligation records, preserving purpose/allowance/binding rejection order.

Internal artifact transitions carry their required checksum evidence by type.
One canonical lookup supplies the exact artifact for reads and mutation; persisted
state/checksum corruption checks, immutable identity and immediate transitions remain
unchanged. Agent endpoint admission separates written syntax, root-origin shape and
allowed transport while preserving raw-path rejection and the existing trust policy.

Read-only release receipt and validation checks are shared directly, including
selected-commit resume. The mandatory exact receipt check remains in place; no Git
or release effects move into these helpers. The documented `--receipts-only` test
selection exercises isolated consumer receipt/recovery fixtures. Ordinary restore
and data-stage unit fixtures use typed model inputs. Shared test-support hex decoding
rejects partial and malformed bytes; independent wire/hash data remains unchanged.

Public APIs, typed errors, wire/storage formats, digest domains, original spending
and recovery ordering are preserved. Notes select compatible **0.44.7**; package
versions remain **0.44.6**. Dependencies, features and the Shared Tooling **0.26.21**
snapshot are unchanged. The original lock remains byte-identical, SHA-256
`e2a02bae7c470acdb439f0867c5fc816e63c81e095e77e37e1625a8302e1feb5`.
It selects Host/Testkit **0.68.9** and Platform **0.6.19**, with tooling minimums
still **0.68.8**.

## Focused evidence

Strict all-target/all-feature Clippy passes for both packages. Focused Core model,
policy, workflow and persistence tests pass, including combined fence rejection
precedence, invalid persisted checksum states, receipt/reservation rejection,
custody, process-death recovery and strict fixture hex. Thirteen selected public
integration targets, both Agent endpoint tests and all fourteen HTTP boundary tests
pass. Formatting, local documentation links, Perl syntax and focused ShellCheck pass.
Logs are retained under `/tmp/backup-simplify3-*`; starting source remains in
`/tmp/backup-simplify3-20261027-start/`.

Nine isolated receipt/recovery cases pass, retaining traces and evidence at
`target/shared-release-tests.14GPyM/`. They include exact older-commit receipt
selection, missing/conflicting proof and completed resume without repeated effects.
The locked Testkit server passes offline admission. PocketIC **16.1.0** passes the
connected same-ID capture/restore/replay journey, retaining
`/tmp/ic-backup-pocketic-694701-0/`. Both independent normal-consumer journeys pass:
capture/restore/acceptance/terminal replay and death after authenticated load without
redispatch. Graph/build evidence is at
`/tmp/ic-backup-agent-independent-build-696383-0/`; simulator evidence remains at
`/tmp/ic-backup-agent-pocketic-696383-1/` and
`/tmp/ic-backup-agent-pocketic-696383-2/`.

## Limits and delivery

This is focused native Linux, simulator and isolated release-helper evidence, not a
new full CI, macOS, MSRV, installed IC, package-archive/registry-only or release gate.
Deployment binding and application acceptance remain consumer-owned. Agent's prior
transitive crypto-bigint 0.5.5 concern remains unresolved. Changes are unstaged and
uncommitted. No package version, release, tag, publication, deployment, live IC effect
or sibling edit is performed.
