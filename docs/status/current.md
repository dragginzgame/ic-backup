# Current handoff — 2026-10-27

## Released source and ownership

Released Backup is **0.44.7**, `304e9236a5a8a6952881bcbdf64c90fbb67f9f2b`.
The [released handoff](https://github.com/dragginzgame/ic-backup/blob/304e9236a5a8a6952881bcbdf64c90fbb67f9f2b/docs/status/current.md)
retains the earlier simplification and qualification evidence. Core owns bounded
records, guarded persistence, reconciliation and original-plan coordination.
Agent owns reserved signed transport. Applications retain authority, membership,
release binding, whole-unit fencing and external obligations.
[Implementation contracts](../implementation-contracts.md) remain authoritative.

## Pending 0.44.8 simplification

All ten items in the fourth requested batch are implemented. The plan derives
operation bindings directly for evidence requests and shares that derivation with
attempt authorities. Observed target admission uses a canonical inventory lookup
after normalizing each input once. Metadata owns module/heap/stable size ordering
and original ordered chunk lookup for data validation, planning, coverage and members.

Topological sorting resolves dependencies against the already sorted node table.
Selection expansion builds one local child index, preserving exact/direct/descendant
semantics and canonical output. Complete journal scans move freshly reopened records
out of checked guards; the selected borrowed current journal is still cloned.
Sequential locks, closing root checks and fail-closed write behavior remain intact.
Ancestor validation retains binding/settlement identities without unused learned
fields; learned evidence remains admitted at its owning edge.

Lifecycle, metadata and data-read requests share bounded management argument encoding;
upload keeps its separate allowance. Fence digest framing writes each scope's fields
directly without a temporary hash vector. Existing golden bytes are unchanged.
Release preparation reuses the parsed root manifest within its rendering phase,
without caching across writes or selected commits. The documented
`--preparation-only` selection runs existing rendering/rejection fixtures; exact
selected-commit receipt and mandatory resume checks remain unchanged.

Public APIs, typed admission errors, wire/storage formats, digest domains, original
spending and recovery ordering are preserved. Notes select compatible **0.44.8**;
package versions remain **0.44.7**. Dependencies, features and the Shared Tooling
**0.26.21** snapshot are unchanged. The original lock remains byte-identical, SHA-256
`422d9fc763e2ee8adc443b84a7b9521854a9efbc34ef9b36efda4751ed1f940e`.
It selects Host/Testkit **0.68.9** and Platform **0.6.19**, with tooling minimums
still **0.68.8**.

## Focused evidence

Core `cargo clippy --offline --locked -p ic-backup --lib --tests -- -D warnings`
passes. All **141** selected unit tests pass across the changed model/policy owners,
journal/progress/settlement/workflow persistence and IC artifact custody. Eight
selected public integration targets pass (**10** tests): plan, inventory/selection,
request/metadata/data codecs, artifact, workflow and fence obligation. Existing
independent wire/hash goldens, graph bounds, selection overlap, ancestor drift,
accounting, custody and process-recovery checks remain effective.

All **12** isolated preparation fixtures pass, retaining traces and evidence at
`target/shared-release-tests.SorWRb/`. All **9** exact receipt/recovery fixtures pass
at `target/shared-release-tests.UFUUnr/`, including older selected commits and
completed resume without repeated effects. Perl/Bash syntax, focused ShellCheck,
formatting and local documentation links pass. Logs are retained under
`/tmp/backup-simplify4-*`.

The locked Testkit server passes offline admission. PocketIC **16.1.0** passes the
connected same-ID capture/restore/replay journey, retaining
`/tmp/ic-backup-pocketic-1853598-0/`. Its first sandboxed attempt failed before server
startup because loopback binding was denied; raw output remains at
`/tmp/ic-backup-pocketic-295-0/`. The permitted local-server rerun passes.

## Limits and delivery

This is focused native Linux, simulator and isolated release-helper evidence, not a
new full CI, macOS, MSRV, independent Agent consumer, installed IC or registry gate.
Deployment binding and application acceptance remain consumer-owned. Agent's prior
transitive crypto-bigint 0.5.5 concern remains unresolved. Changes are unstaged and
uncommitted. No package version, release, tag, publication, deployment, live IC effect
or sibling edit is performed.
