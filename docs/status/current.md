# Current handoff — 2026-11-03

## Selected delivery: 0.56.1

The user requested deployment of the complete pending compatible batch to
`dragginzgame/ic-backup` main with annotated `v0.56.1`, followed by crates.io
publication of **ic-backup** and **ic-backup-agent**. This includes the shared
fixture/standalone cleanup and published Platform 0.20.0/Testkit 0.82.0 adoption.
The standard direct release runner owns complete validation, patch metadata and
receipt preparation, commit/tag and atomic push; publication follows separately.
No further dependency update is selected during release qualification.

Evidence is `/tmp/backup-deploy-0561/`: incoming source archive/diff, selected
source and release plan, release/publication logs and final registry observations.
At admission, remote main matched the delivered 0.56.0 baseline; the selected tag
and both registry versions were absent. This preparation note establishes no
completion: the generated [receipt](../release.json) and completed Git/registry
observations establish actual delivery.

## Current dependency adoption: Platform 0.20.0 and Testkit 0.82.0

The pending 0.56.1 batch now requires published Platform **0.20.0** and Testkit
**0.82.0**. Verified archives bind Platform to release source
`cc8214bf6e2a3dcbe1649c1cd4e71d8a44925bdf` and Testkit to
`e644afe09f0fa9ef248db6f98b3efa461de9f890`. All 121 Platform facade source files
and 30 Testkit source files are byte-identical to their previous releases.
Platform changes only its development client requirement to **0.20.0**; canonical
Types **0.18.0**, Runtime **0.19.0** and external Principal, management DTO and
Agent identities are unchanged. Testkit changes only its disabled optional Wire
requirement to 0.16. Host and Shared Tooling remain selected as before.

Testkit still selects Platform/client **0.19**. Those stay development-only;
fixtures use its bounded `start_managed_server` and construct the independently
released Platform 0.20 client at that URL. The async fixture uses a blocking worker
for synchronous server startup. No client identity crosses the boundary, and
explicit field order keeps client teardown ahead of the owned server. Original
server output, admitted binary digest, topology and request/startup bounds remain.
No client-specific snapshot helper, retry, reset or funding policy is adopted.

Evidence is `target/platform-testkit-adoption-0561/`: incoming pending changes,
original manifests/lock, registry metadata, verified old/new archives and source
comparisons, exact lock delta, CLI/server setup and focused qualification.
Qualification on pinned Rust **1.99.1** passes strict Core/Agent library, simulator
target and standalone-example Clippy; **164** model identity/hash/codec/bounds
regressions; all **34** Core and **28** Agent simulator cases; exact Testkit
CLI/server setup and offline admission; dependency inheritance/boundaries;
formatting and maintained links. Registered selections and final input hashes
are retained in `qualification.json`.

Independent normal Core and Agent consumers pass without development-feature
unification (`target/msrv-consumers.Q5Z5LJ/`). Their production graphs select
Platform **0.20.0** and Types **0.18.0** without simulator/runtime packages; Core
remains Agent-free. No public type identity or persisted-data migration is needed
for the consumed facade exports. Earlier logs below retain their original source
and dependency selections. Native macOS and live Canic acceptance remain separate;
no full release gate, release or publication is selected.

## Latest fixture cleanup for pending 0.56.1

Four capture/download entry paths share the same accounted stop/status setup.
Focused download and workflow-runner fixtures share private artifact-directory,
reference and journal preparation with their exact original tokens and observed
snapshot metadata. Each caller still selects its unfinished restore location and
retains its own fault, byte/reference and cold-replay assertions. Reference
retention precedes journal creation. The restore readback oracle compares the same
certified data, global bits, region sizes, chunk store, timer and hook fields in
one explicit Candid projection; timestamp/source differences remain outside it.

Strict Clippy and all **34** registered Core PocketIC cases pass on pinned Rust
**1.99.1** with Testkit **0.81.0**. Evidence is `target/fixture-cleanup-0561/`,
including original inputs, retained first lint failure, final lint/simulator logs
and source-bound case membership. Formatting and maintained links pass. Agent and
production source/dependencies are unchanged from the qualification below; no
public API or Canic integration migration is introduced. Work remains local and
uncommitted, without a new release gate or delivery.

## Earlier fixture-step cleanup and standalone source custody

Core's focused capture/download cases and original workflow driver now use one
real capture, metadata and data-transfer fixture implementation. Each case retains
its original fault, exact spending, byte/reference preservation and cold-resume
assertions. Captured raw oracle bytes remain separate from qualified returned
evidence. Metadata/data qualification durably retains exact arguments and replies
before explicit Applied receipts; source byte oracles remain independently decoded.

The standalone example reuses its existing private source helper under explicitly
held layout guards for upload, restore and terminal acceptance. Terminal admission
borrows the original source guard/journal; it does not reacquire a held source lock
or cache admission across driver calls. Reference-set and release replay assertions
remain in the independent consumer and guarded release owner's qualification,
rather than the example's release wrapper. Fresh acceptance and all primitive
source, manifest, safety and terminal checks remain in their original order.

This cleanup preserves public APIs, package versions and v1 records. Canic's
integration tests remain with Canic; this batch requires no consumer API migration.
Pending compatible notes are under [0.56.1](../../CHANGELOG.md).

Focused Linux qualification on pinned Rust **1.99.1** passes strict Clippy for
the Core PocketIC target, Agent PocketIC target and standalone example, all **34**
Core simulator cases and all **16** selected independent-consumer cases. These
include normal capture/restore, stage-by-stage cold continuation, signed/reply
retention failures, partial output, lost replies, changed acceptance and unrelated
unfinished restore references. Evidence is `target/cleanup-flows-0560/`, including
the original source archive, compiler, lint and simulator logs. No complete release
gate or new delivery was selected for this development batch. These logs precede
the separate Testkit adoption below.

## Earlier Testkit 0.81.0 adoption

The workspace now requires published Testkit **0.81.0**. Its verified registry
archive matches checksum
`2019970678905d4fb434778447c70a4a675b33afd5e56ff983d2a9a32f6633a8`
and release source `4b023d5e3b2cdac0424f17d37b2bacc3f186c44a`.
Testkit now shares Backup's Platform/client **0.19.0** and canonical Types
**0.18.0**. The older Platform/client 0.18, Types 0.17 and Strum 0.26 packages
leave the development lock; target-specific Runtime advances to **0.19.0**.
Production dependencies, external management/Principal/Agent identities, Host
**0.74.2**, Shared Tooling and the authenticated **16.1.0** server stay selected.

The synchronous and async fixtures use Testkit's published `build_instance` and
`build_instance_async`, retaining the existing topology, request/startup bounds,
original output files and explicit client-before-server drop order. The temporary
server-only URL bridge and blocking startup worker are removed. Exact CLI setup
and offline admission use the newly locked Testkit selection; earlier tool and
fixture evidence remains retained. No retry, reset, funding policy or permissive
application admission is adopted.

Adoption evidence is `target/testkit-adoption-0561/`: incoming cleanup diff and
manifest/lock, old/new verified registry archives, source review, selected CLI and
server setup/admission, dependency delta and focused qualification. Qualification
passes on pinned Rust **1.99.1**: strict Clippy for both simulator targets and the
standalone example, all **34** Core and **28** Agent cases (including the **16**
independent-consumer cases), exact Testkit CLI/server setup and offline admission,
dependency inheritance/boundaries, formatting and maintained links. The two normal
production trees retain Platform 0.19/Types 0.18 without Testkit, PocketIC or runtime;
Core remains Agent-free. The registered cases and final input hashes are recorded
in `qualification.json`. The earlier cleanup logs do not qualify this changed graph.
No full release gate, native macOS or live Canic acceptance is claimed.

## Delivered 0.56.0

Release **0.56.0**, commit `b6be0a15157f058d4babd11bcbd4aa836515f3a0`, was
atomically pushed to `main` with annotated `v0.56.0`; both libraries were published
to crates.io. The [receipt](../release.json), complete Linux release gate and exact
registry archive checksums/source identities are retained in
`/tmp/backup-deploy-0560.xagghlwi/`, including `release.log`, `publication.log`,
`registry-verification.json` and `delivery-result.json`. Those proofs qualify the
delivered source, not the subsequent development batch.

## Delivered Platform 0.19 adoption

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

## Delivered 0.56.0 source batch

The delivered source includes all reviewed cleanup and standalone work. The runner
prepared both package versions from **0.55.3** to **0.56.0**;
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
