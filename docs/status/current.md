# Current handoff — 2026-11-03

## Selected delivery: 0.57.0

The user requested deployment of all four API consolidation cuts to
`dragginzgame/ic-backup` main with annotated `v0.57.0`, followed by crates.io
publication of **ic-backup** and **ic-backup-agent**. The standard direct runner
owns complete qualification and release metadata, commit/tag and atomic push;
registry publication follows separately. No dependency update is selected.

Delivery evidence is `/tmp/backup-deploy-0570.tndhcsav/`, including the incoming
source archive, reviewed roster, destination/registry observations, release plan
and release/publication logs. This selection is not completion: the generated
[receipt](../release.json) and final Git/registry observations establish delivery.
Breaking notes are under [0.57.0](../../CHANGELOG.md).

## API consolidation

- Capture and metadata-allocation recovery share passive
  `model::inventory_settlement::{IcInventoryAttribution, IcInventorySettlement}`.
  Their method-specific validators, views and errors remain separate, including
  allocation's source-ID exclusion. The duplicate policy conversion is removed.
- `create_execution_workflow` publishes original intent once;
  `advance_execution_workflow` is the sole coordinator for fresh and retained work.
  Callers supply original evidence and a positive stage limit, handling `Paused`
  and `Complete` explicitly. The two unbounded convenience entry points are removed.
- Fresh arbitrary-directory `record_downloaded` attestation is removed. Fresh IC
  artifacts use the complete writer. Existing v1 `Downloaded` records retain
  checksum verification and publication recovery; states and their wire names remain.
- Public passive read/upload workflow wrappers are removed. Complete metadata,
  download, allocation and data-upload coordinators share the original private
  dispatch engines. Transport qualification uses explicit reserved-request/provider
  boundaries; private test helpers retain the engines' pending-response fault cases.

Production Rust is reduced by **279 lines**, including the new shared claim module.
Original v1 schemas, digest domains, bounds, spending, byte/reference custody,
uncertain-effect reconciliation and standalone backup/restore remain unchanged.
Native fixtures use the complete IC byte writer or explicitly seeded historical
records; they qualify filesystem/process behavior, not management effects.

## Qualification and consumer action

Evidence is `target/api-consolidation-0570/`: original tracked source archive and
lock, caller inventory, retained first failures, focused logs and final input/case
identities. Pinned/minimum Rust **1.99.1** is admitted. Focused qualification passes:

- Strict Core/Agent library, test-target and standalone-example Clippy.
- **117** workflow, **6** capture-settlement, **7** allocation-settlement and **53**
  download-journal unit cases; **14** affected native integration cases.
- All **34** Core and **28** Agent simulator cases, including the Agent suite's
  **16** registered independent-consumer cases. Original partial output, lost or
  malformed replies, cold continuation and reference retention remain covered.
- Separate normal Core and Agent consumers on Rust **1.99.1**, without development
  feature unification (`target/msrv-consumers.kHxTOA/`).
- Formatting, maintained links, dependency inheritance and dependency boundaries.

Canic's checkpoint unit tests and Root application-checkpoint simulator still call
retired workflow entry points. Keep those tests and migrate their calls using the
[workflow API migration](../canic-adoption.md#workflow-api-migration). That checkout
was inspected read-only and is unchanged; installed Canic acceptance is outstanding.
These focused checks qualify the development batch. The selected release runner
performs the separate complete local gate; native macOS and live IC acceptance
remain separate.

## Delivered baseline and unchanged dependencies

Release **0.56.1**, commit `8880639418bed4a741123f7401ef206dc47bb6b0`, was
atomically pushed to main with annotated `v0.56.1`; both libraries were published
to crates.io and their archive checksums/VCS identities verified. Complete local
Linux release qualification and delivery/publication evidence remain in
`/tmp/backup-deploy-0561/`, including `delivery-result.json`, `release.log` and
`publication.log`. The [receipt](../release.json) records that delivered source;
it does not qualify this subsequent local API change.

The delivered baseline package versions are **0.56.1**; the selected release
advances both to **0.57.0**. The incoming Cargo.lock matches development admission. Platform
**0.20.0**, canonical Types **0.18.0**, Testkit **0.82.0**, Host **0.74.2** and
Shared Tooling **0.27.28** at `03b5d9a47beac9767ced96008a9e7b450b61a82a` remain
selected. Both simulator fixtures retain Testkit's owned managed server and the
Platform 0.20 client at its URL, with original readiness/cleanup/output bounds,
explicit client-before-server drop order and authenticated server **16.1.0**.
Testkit's older Platform/client and target-specific Runtime remain development-only.
Production graphs remain simulator/runtime-free; Core remains Agent-free.

Earlier qualification remains retained in `target/platform-testkit-adoption-0561/`,
`target/testkit-adoption-0561/`, `target/fixture-cleanup-0561/`,
`target/cleanup-flows-0560/`, `/tmp/backup-deploy-0560.xagghlwi/` and
`/tmp/backup-deploy-0553.ZSoQWjKL/`. Preserve original/failed bundles, compiler
admission evidence, unfinished journals, referenced/partial artifacts and source
references. No live backup/restore or product/store cleanup is selected.
