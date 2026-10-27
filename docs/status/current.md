# Current handoff — 2026-10-27

## Released source and ownership

Released Backup is **0.44.8**, `15ffb73c001269c07e329d0e3f26044d98ac5d87`.
The [released handoff](https://github.com/dragginzgame/ic-backup/blob/15ffb73c001269c07e329d0e3f26044d98ac5d87/docs/status/current.md)
retains earlier simplification and qualification evidence. Core owns bounded
records, guarded persistence, reconciliation and original-plan coordination.
Agent owns reserved signed transport. Applications retain authority, membership,
release binding, whole-unit fencing and external obligations.
[Implementation contracts](../implementation-contracts.md) remain authoritative.

## Pending 0.45.0 adoption and release-plan cleanup

The reviewed Shared Tooling **0.26.22** snapshot remains at committed source
`5fc904a935d82ad31f0fba14434b71ccb8c10345`, with 124 selected files. The earlier
canonical export and impact review cover five governance payload changes; no
shared setup, formatter or release implementation changed. The current task
preserves that pending adoption and the separate release-plan cleanup.
[The adoption review](../shared-tooling-review.json) records source and qualification.

The workspace selects Platform **0.7.0** and Host/Testkit **0.69.0**. The selected
release editor is also **0.69.0**. Tooling's committed source is
`f268aef5d205e51811e934648afcd13f026635f1`. Published archives match registry
checksums, and every committed src file matches the published source. All consumed
Tooling src files are unchanged from 0.68.12; no sibling checkout or patch supplies
a build dependency. Platform's retired compiler exports are unused.

Compared with the incoming graph, this refresh upgrades only Host Artifacts,
Host FS, Host Process and Testkit from 0.68.12 to 0.69.0 and removes Platform
0.6.21. Normal and development graphs now share one Platform 0.7.0 identity.
Library feature selections remain unchanged. Independent normal consumers exclude
Testkit/PocketIC/WAT, and Core remains Agent-free. External principal, Candid,
management DTO and Agent identities remain unchanged; the separate development
management DTO 0.8.0 remains distinct from production 0.11.0.

Public Host error identities now belong to 0.69. Consumers exchanging
`NamedWriteError` or `DirectoryChecksumError` must align direct Host dependencies.
That concrete Rust type change selects pending **0.45.0** notes; package versions
remain **0.44.8**. Product Rust source, wire/storage formats, hashes, spending and
recovery order are unchanged. Final Cargo.lock SHA-256 is
`5266a7ec6a05d9c050dbbe0047ad6a822813a318708d17f9fb78793ed4c9335e`.
Canonical setup and offline common/Testkit/editor admission pass. Old bundles
remain retained; optional IC executable selection is unchanged.

Release plans reuse Shared's `StableVersion::version_cmp`, removing the private
comparator under [#71 — Reuse Shared stable version ordering in the release-plan helper](https://github.com/dragginzgame/ic-backup/issues/71).
Local canonical syntax, component ceilings, equality and downgrade diagnostics
remain in place, with an explicit module companion for fixture copying. Its prior
five metadata and nine receipt/recovery cases retain evidence under
`target/shared-release-tests.lfUKmF/`, `target/shared-release-tests.22gSc9/` and
`/tmp/backup-issue71-*`. Native macOS and delivery remain required for closure.

## Focused evidence on Tooling 0.69.0

Strict all-target/all-feature Clippy passes for both libraries. All **84** selected
Core codec, identity, inventory, artifact publication, JSON and command-custody
unit tests pass, including independent golden bytes, bounds, typed failures and
process-death recovery. All **3** Agent endpoint/evidence unit tests and **14**
HTTP boundary tests pass. Independent Core and Agent consumers pass on Rust
**1.91.0**, including public Host 0.69 error probes, retaining
`target/msrv-consumers.GTw8qo/`.

All **12** isolated preparation cases pass with release editor 0.69.0, retaining
`target/shared-release-tests.D8SwiX/`. Dependency inheritance/boundaries, snapshot
integrity, formatting and maintained documentation links pass. Detailed logs and
published-source/lock-delta verification remain under `/tmp/backup-tooling69-*`;
normal graph evidence is `/tmp/backup-tooling69-graphs.MFafMm/`.

PocketIC **16.1.0** passes connected same-ID capture/restore/replay, retaining
`/tmp/ic-backup-pocketic-408287-0/`. Both independent normal Agent journeys pass:
capture/restore/acceptance/terminal replay and death after authenticated load
without redispatch. Graph/build evidence remains at
`/tmp/ic-backup-agent-independent-build-451335-0/`; simulator evidence is at
`/tmp/ic-backup-agent-pocketic-451335-1/` and
`/tmp/ic-backup-agent-pocketic-451335-2/`.

## Limits and delivery

This is focused native Linux, simulator, independent normal/MSRV consumers and
isolated editor evidence, not new full CI, native macOS, installed IC, package
archive or registry-only Backup acceptance. Application acceptance remains
consumer-owned. Agent's prior transitive crypto-bigint 0.5.5 concern remains
unresolved. Changes are unstaged and uncommitted. No own package version, release,
tag, publication, deployment, live IC effect or sibling edit is performed.
