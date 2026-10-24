# Current handoff — 2026-10-24

## Ownership and released source

Released Backup is **0.43.0**, `79bc98b`. Core owns bounded records, guarded
persistence, reconciliation and original-plan coordination. Agent owns reserved
signed transport. Host owns generic IO/process/hash capabilities; Platform owns
external IC identities and SDKs; Testkit owns simulator assets and processes.
Applications own current membership, permissions, release binding, whole-unit
fencing and external obligations. [Implementation contracts](../implementation-contracts.md)
and the [record/API boundary](../extraction-boundary.md) retain safety obligations.
The released incomplete-stage API cut and fixture cleanup remain in place.

## Published IC Tooling adoption

Pending **0.44.0** adopts published Host Artifacts/FS/Process, Testkit and the Build
release editor **0.68.0**, source **8f281177faaeee0adc51edc1031d4a7b268b0ce0**.
All three Host Rust source trees and Build's Rust source match 0.67.0 byte-for-byte.
Testkit changes only its optional Wire adapter/tests; Backup does not enable that
feature. No product codec, storage, decoder, lifecycle or recovery behavior changes.
The minor note reflects exposed Host Rust type identities; consumers exchanging
Host checksum/publication errors must align their dependencies to 0.68.

The lock changes exactly four packages from 0.67.0 to 0.68.0. Platform 0.6.14,
Agent 0.49.3, management DTO 0.11.0, external Principal 0.1.5 and PocketIC 16.1.0
remain selected. Shared Tooling stays 0.26.9; no snapshot payload changes.
Archive checksums/source identities: `/tmp/backup-tooling-068-published.json`.
The canonical installer prepared the new Testkit and Build CLIs. Offline admission
passes and the existing PocketIC asset is reused; old/failed tools remain retained.
Current lock SHA-256: `9e5cf974a16a179c39628e99935d3a199d1838502a70fb48f78f57e1e5efec16`.

## Qualification

Linux strict all-target/all-feature Clippy, 525 Core and three Agent unit tests,
actual Rust 1.91.0 all-target/all-feature compilation, dependency inheritance and
boundary checks, formatting and local documentation links pass. Both independent
MSRV consumers pass, including public Host checksum/publication error identities.
Their normal graphs select only Host 0.68.0, exclude Testkit/PocketIC/WAT/Clap and
Host development features, and keep Core Agent-free. Evidence:
`target/msrv-consumers.f3o4Vt/`.

Both authenticated independent candidate-consumer simulator cases pass: cold
capture/restore/acceptance with terminal replay, and process death after load
without redispatch. Build/graph evidence:
`/tmp/ic-backup-agent-independent-build-2659235-0/`; fixture evidence:
`/tmp/ic-backup-agent-pocketic-2659235-{1,2}/`.
The new release editor passes isolated metadata/retention fixtures at
`target/shared-release-tests.eYnzZX/`. Logs:
`/tmp/backup-tooling-068-{setup,tools-check,unit,clippy,msrv,consumers,boundaries,editor,simulator}.log`.

## Remaining application work and limits

The [Canic guide](../canic-adoption.md) describes the application-owned adapter.
Canic's active restore engine and original journals remain until its replacement
qualifies current authority, whole-unit fencing, Root-paid effect reconciliation
and external obligations. Generic fixture acceptance does not qualify those facts.
Earlier registry-only and package qualification retain their original source scope
in the [released handoff](https://github.com/dragginzgame/ic-backup/blob/79bc98b/docs/status/current.md).
No native macOS, installed Canic/IC, new registry-only acceptance, package-archive
qualification or full local make ci run is claimed for this batch. Workspace
package versions remain 0.43.0. No staging, commit, push, release, publication,
live IC effect or sibling edit is performed.
