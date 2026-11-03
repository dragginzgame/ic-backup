# Current handoff — 2026-11-03

## Selected delivery: 0.56.0

The user requested deployment of the complete reviewed batch to
`dragginzgame/ic-backup` main and annotated `v0.56.0`, followed by crates.io
publication of both **ic-backup** and **ic-backup-agent**. The minor increment
covers the public API removals and matches the pending notes. The standard direct
release runner owns full validation, version/receipt preparation, commit/tag and
atomic push. Publication is a separate recorded command after Git delivery.

This handoff records the selected inputs before release preparation. The generated
[release receipt](../release.json) and completed command/registry observations
establish actual delivery, rather than this preparation note. Evidence is
`/tmp/backup-deploy-0560.xagghlwi/`: original tree/index state, source/plan records,
release/publication logs and final registry provenance. The selected registry
versions and remote tag were absent at admission; main matched the local 0.55.3
release. No dependency refresh is selected during release qualification.

## Current batch: Platform 0.19 adoption

The user requested the new published Platform. Root now requires **ic-platform
0.19.0**, resolving canonical Types **0.18.0** and development PocketIC client
**0.19.0**. Old/new registry archives match their published checksums and exact
release VCS commits; consumed Candid, native management, external Principal and
Agent facade source is unchanged. Types changes Account's present-subaccount
Serde encoding; Backup has no Account/Subaccount code or records, and its consumed
Principal implementation is unchanged. External Principal **0.1.5**, management
DTO **0.11.0** and Agent **0.49.3** remain the qualified identities.

Published Testkit **0.80.2** still requires Platform/client **0.18**, Types
**0.17** and target-specific Runtime **0.18.1**. These remain development-only;
both production graphs select only Platform 0.19/Types 0.18. Core stays Agent-free.
The two fixtures use Testkit's existing `start_managed_server` to own bounded
startup, raw output retention and cleanup, then construct Platform 0.19's fallible
client at its returned URL. The async fixture starts the server on a blocking
worker. Client setup/lifecycle failures are checked explicitly, and expected fence
rejection requires `ClientError::Canister` with the exact original structured
rejection. No client/owned-Principal identity crosses the Testkit server boundary.
No local patch, fallback, retry loop, second server catalog or unpublished API is
introduced. See [the dependency boundary](../ic-platform.md).

Evidence is `/tmp/backup-platform-adopt._p6yeq2b/`: incoming tree archive/diff and
lock, live registry metadata, archive/checksum/VCS and source comparisons, exact
lock delta, compiler/server admission and qualification logs. All pre-existing
package versions/checksums remain; the lock only adds the new Platform, Types,
PocketIC client and required Strum 0.28 pair. Host remains **0.74.2**, Testkit
**0.80.2** and the authenticated server **16.1.0**. Shared Tooling remains
**0.27.28**, `03b5d9a47beac9767ced96008a9e7b450b61a82a`.

Focused Linux qualification passes strict Core/Agent/standalone-example Clippy,
**164** model identity/hash/codec/bounds regressions, **34** synchronous snapshot
cases and **28** authenticated async cases, including all 16 independent-consumer
cases. Separate normal Core and Agent consumers pass on admitted/pinned Rust
**1.99.1**, without dev-feature unification or simulator dependencies; Core excludes
Agent. Their evidence is `target/msrv-consumers.00xL6M/`. Source and lock remained
fixed throughout. Formatting, maintained links and dependency boundary/inheritance
checks pass. Full release gates, native macOS, live IC and installed Canic
acceptance remain separate. These focused development checks do not replace the
selected complete release gate.

## Selected 0.56.0 source batch

The selected source includes all reviewed cleanup and standalone work. The runner
prepares both package versions from **0.55.3** to **0.56.0**;
[the 0.56.0 notes](../../CHANGELOG.md) document the breaking API
cuts and migrations. No journal, persisted schema, hash domain or attempt/recovery
authority is changed by this Platform adoption.

The [standalone Rust example](../../crates/ic-backup-agent/examples/standalone/README.md)
is the single maintained complete assembly for original catalogs, typed
inspection, bounded cold continuation, capture/download/manifest, source-bound
upload, same-ID load, independently accepted start and guarded reference release.
Its signer/trust inputs and application authority, retention/qualification,
restore-safety lane, acceptance and terminal callbacks are explicit. Fixed keys,
application assumptions and faults stay in the fixture adapter. Its deliberately
bounded single-canister profile is a reusable Rust assembly, not an installed CLI
or generic application permission provider. Consumer qualification uses its exact
source and includes a different supplied signer, uncertain replies, cold reentry,
changed acceptance and unrelated unfinished source references.

The duplicate complete Core driver and optional membership/control/read preflight
helpers and local-source join/view are retired. Canonical controller decoding,
fresh primitive safety/manifest/artifact checks and their owner fault matrices
remain. Fence, command-quiescence and recovery mechanisms retain their current
contracts. Integrations still own actual membership, quiescence, external
obligations, fresh application evidence and stable byte custody; restore cannot
rewind payments. Scope rationale and implementation belong to
[the contracts](../implementation-contracts.md) and [the boundary](../extraction-boundary.md).

Earlier source/graph qualification remains historical evidence:

- `/tmp/backup-supported-workflow.7y1dnbiq/`: supported assembly, original lock,
  corrected terminal lock lifetime, 16 registered consumer cases and failed logs.
- `/tmp/backup-local-source-cut.3mxzczip/`: local-source retirement and consolidated
  integrity recovery proof with unchanged attempts, fences and references.
- `/tmp/backup-surface-cut.0ETukGjf/`: initial API/journey consolidation and its
  original 513 Core unit, 34 Core simulator and 15 independent-consumer results.

## Delivered baseline and custody

Release **0.55.3**, commit `c4f88f893a748e0c08ff2c51b87dbcb7f1f880d2`, was pushed
and both registry archives verified. Its completed release/publication evidence
remains `/tmp/backup-deploy-0553.ZSoQWjKL/`, including `release.log`,
`publication.log`, `registry-verification.json` and `delivery-result.json`.
Prior 0.55.2 proof remains `/tmp/backup-deploy-0552.0OuUpHJW/`. Those receipts
qualify those delivered sources, not the selected 0.56.0 batch.

Preserve unfinished journals, referenced/partial artifacts, source references,
original/failed qualification evidence and tool generations. No product/store
cleanup or live backup/restore effects are authorized by this development work.
[The operator map](../canic-adoption.md#reusable-operator-composition) retains the
integration responsibilities and scope.
