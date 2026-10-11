# Async upload integration and Shared Tooling 0.3.8

Released base is 0.16.0, `943ade5afac218d2eeae280111a75f26612ab80f`.
The uncommitted 0.17.0 draft changes the source-bound upload provider and
single-upload/allocation/data-stage coordinators to futures with async fresh
admission and independent qualification. Consumers must await them and migrate
callbacks. Package versions and release receipt remain 0.16.0.

The configured Agent upload adapter rechecks the current guarded original journal,
requires durable exact signed-envelope/request-ID retention, then consumes one
submission through the existing transport. Accepted/lost/cancelled updates remain
pending; no automatic receipt, observation or retry is added. Selected exclusion
spans admission/submission or qualification awaits. Cancellation preserves source
bytes/references, earlier Applied receipts and pending spending and denies reentry.
Retain replies before cancellable qualification. Core runtime/Send requirements,
persisted schemas, bounds and accounting owners remain unchanged. Recovery/fence
provider families and full application/terminal custody remain separate.

Actual Agent simulator metadata/data uploads now use the configured adapter and
public single-upload coordinator; the Testkit driver separately exercises allocation
and complete data-stage coordination. Regression traces cover suspended admission/
submission for both upload kinds, cancelled allocation qualification with durable
request/reply retention and cancelled second data-write qualification after the first
Applied receipt. Source byte checks, journal bytes, references and exact pending
request/reply evidence must survive with no successor or replay dispatch.

Adopt committed Shared Tooling 0.3.8,
`67285b28a98b7c4211ad32de726709d4e87edea4`, verified against remote main and
exported from an isolated clean checkout. The 96-file roster stays unchanged; dirty
sibling changelog work is excluded. New coordinated-archive guidance matches the
existing consumer-owned paired-package check. Registry publication retains ordinary
Cargo admission. Preserve Host 0.12.8, Testkit 0.33.1 and Metrics 0.5.5 unchanged.

All 16 local delivery gates pass on the frozen final inputs. Initial strict Clippy
failures identify immediate
test providers needing ready futures and imports left unused by the migrated test
path; original evidence is retained under `target/continuation-0170/`. The initial
local clone's remote identity is refused before export; the isolated clone is then
bound to the verified canonical source URL. No source owner or sibling is edited.

The first full gate rejects the new HTTP fixture because the `data-source` metadata
vector includes an unavailable global. Preserve that failed evidence and use the
existing `upload-source` SDK vector, whose globals are available; the corrected
focused transport case passes. Integration fixtures discover their shared ready
helper through an ordinary directory module, without new path overrides.

Final qualification includes 510 core unit cases, 29 core simulator journeys,
11 Agent HTTP cases and four Agent simulator journeys. Registered successful case
membership, exact dependency archives/VCS origins, tool identities and retained
log hashes are recorded in [the machine evidence](qualification.json). The independent
locked Rust 1.88 normal consumer compiles all three async upload coordinators with
the configured Agent adapter. Its graph excludes Testkit/PocketIC and Metrics IC
features. The paired current-package archive consumer and all required MSRV checks
also pass. The technical source bytes/modes, incoming manifest/lock and binary
metadata/Wasm fixture inputs are unchanged throughout the passing full gate.

Matching [released 0.16.0 CI](https://github.com/dragginzgame/ic-backup/actions/runs/38071504142)
passes Linux. Apple Silicon passes setup and Bash 3.2 tooling and is running native
validation; Intel macOS remains queued at final inspection. This uncommitted 0.17.0
candidate has no hosted result. Native consumer acceptance remains with the owning
issues, distinct from the local gate. Recovery/fence providers and full application/
terminal/fence/source-reference custody remain incomplete under #25/#29.

Retain initial and corrected logs under `target/continuation-0170/`; the passing
full gate is `full-ci-second/validation.nodUp9`. No root Git write, release, registry
upload, production IC effect, sibling edit or recovery cleanup ran.
