# Current handoff — 2026-10-27

## Released source and ownership

Released Backup is **0.44.5**, `b9ea4702e780f256420107013aad512b6f4b3409`.
The [released handoff](https://github.com/dragginzgame/ic-backup/blob/b9ea4702e780f256420107013aad512b6f4b3409/docs/status/current.md)
retains the prior simplification, MSRV and simulator evidence. Core owns bounded
records, guarded persistence, reconciliation and original-plan coordination.
Agent owns reserved signed transport. Applications retain authority, membership,
release binding, whole-unit fencing and external obligations.
[Implementation contracts](../implementation-contracts.md) remain authoritative.

## Pending 0.44.6 simplification

The second requested batch is complete. Original authority matching belongs to the
plan owner, and progress supplies its admitted journal index to exact-history
settlement. Complete coverage, causal prerequisites and chronological fingerprints
remain mandatory. Restore-reference updates use their existing canonical ordering;
consistency and restore safety reuse ordered observation checks. Raw snapshot-ID
admission has one scalar owner, retaining distinct JSON/Candid visitors and errors.

Artifact member declarations carry region/chunk identity instead of a payload flag
and filename inference. Upload preparation carries the checked metadata allocation,
original stage and bounded extents across source IO, then binds the exact ordered
byte digests. Individual and batch source reads share bounded byte admission.
Original allowance preflight, tree verification, closing custody checks and empty
region/known-empty-chunk behavior remain unchanged.

The proposed stored snapshot references were narrowed: an observation view owns its
decoded inventory, so a stored reference would be self-referential. Capture/upload
settlement keeps its safe admitted index and borrows descriptors through one shared
lookup. Independent-consumer setup now has named source, manifest, graph and build
phases, preserving stderr logs and additionally retaining each command's stdout.

Public signatures, typed errors, wire/storage formats, digest domains, original
spending and recovery ordering are preserved. Notes select compatible **0.44.6**;
package versions remain **0.44.5**. There are no dependency, feature or snapshot
changes. The original lock remains byte-identical, SHA-256
`4e32add45a8e544d4ac20c854045436d2af68b8683f943dc4c1dbe909f7736c4`.
It selects Host/Testkit **0.68.9** and Platform **0.6.19**; tooling minimums remain
**0.68.8**, and Shared Tooling remains the reviewed **0.26.21** snapshot.

## Focused evidence

Formatting, documentation links, diff checks and strict all-target/all-feature
Clippy pass for both packages. Fourteen focused Core module selections pass,
including authority/history mismatches, allocation overflow/preflight, exact byte
bindings, custody, observation rejection and reference conflicts. Eight affected
public integration targets pass. Logs are retained under `/tmp/backup-simplify2-*`;
starting source is retained in `/tmp/backup-simplify2-20261027-start/`.

The locked Testkit server passes offline admission. PocketIC **16.1.0** verifies the
complete original allocation/data-upload journey, retaining
`/tmp/ic-backup-pocketic-3451071-0/`. Both independently built normal-consumer
journeys pass: cold capture/restore/acceptance/terminal replay and death after
an authenticated load without redispatch. Graph/build evidence is at
`/tmp/ic-backup-agent-independent-build-3480426-0/`; simulator evidence remains at
`/tmp/ic-backup-agent-pocketic-3480426-1/` and
`/tmp/ic-backup-agent-pocketic-3480426-2/`. The independent build preserves the
original root lock and retains command stdout/stderr in its own directory.

## Limits and delivery

This batch has focused native Linux and simulator evidence, not a new full CI,
macOS, MSRV, installed IC, package-archive/registry-only or release qualification.
Deployment binding and application acceptance remain consumer-owned. Agent's
previously recorded transitive crypto-bigint 0.5.5 concern remains unresolved.
Changes are unstaged and uncommitted. No package version, release, tag, publication,
deployment, live IC effect or sibling edit is performed.
