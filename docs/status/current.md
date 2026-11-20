# Current handoff — 2026-11-20

## 0.105.13 published with Toko Tooling 0.45.23

[ic-backup 0.105.13](https://crates.io/crates/ic-backup/0.105.13) shipped from tag `v0.105.13` at `6ae096f`. All 247 native tests, five doctests and 11 PocketIC journeys passed. Registry readback verified the checksum, embedded commit, license and all 181 tracked package files. An independent registry consumer built successfully. [Backup #5](https://github.com/dragginzgame/ic-backup/issues/5) and [Backup #6](https://github.com/dragginzgame/ic-backup/issues/6) are closed. Evidence is under `target/evidence/release-0.105.13/`.

The published source selects 0.45.23. The shared lint audit covered all five manifests and reported zero discrepancies. The dependency-pin and inheritance audit passed. The hygiene pass consolidated the chunked HTTP fixture builder and split long qualification evidence messages for readability.

CI inherits the caller’s Cargo network settings after preparing the locked dependency cache. Testkit independently verifies the installed simulator. The root workspace owns the published product version through `workspace.package.version`; the private qualification workspace keeps its own package identities.

## Local tooling and current suppliers

Both workspaces select Toko Tooling 0.46.0. The registry exporter verified all
62 managed files. The Rust type-header and placement pass covers the product
and private qualification packages. The next changelog entry is 0.105.14,
a compatible tooling and documentation patch. All 247 native tests, five
doctests and 11 Linux PocketIC journeys passed. Both workspaces passed strict
Clippy and formatting. The shared lint audit found zero discrepancies across
five manifests. Logs are under `target/evidence/tooling-0.46.0/`.

The working locks select Platform 0.34.19, Types 0.21.4, Protocol 0.3.7,
Host FS/Artifacts 0.80.7 and HTTP 0.80.2. Backup owns its Reqwest 0.13.5
client. The qualification workspace uses Testkit 0.110.3.
[Setup](../development/setup.md) owns tool preparation and dependency evidence.
The [transfer contract](../architecture/capture-transfer.md#full-platform-port--2026-11-17)
records Platform decoder and regression provenance.

## Implemented scope

The experimental direct-controller library implements capture, history/latest,
held sources/payloads, same-ID/network/release restore, original-call recovery and
completed-target succession. Capture/restore leave the application stopped.
One recorder preserves finite spending and exact original evidence; pending calls
never repeat and changed/unknown files refuse intact. Completed replay requires
no live authority or writes. [Capture](../architecture/capture-intent.md),
[transfer](../architecture/capture-transfer.md) and
[restore](../design/local-store.md#immutable-restore-approval-and-original-custody)
own these guarantees. Full-history replay remains quadratic; reduced payload IO
does not establish end-to-end speedup. Measurements belong to the
[restart design](../design/restart.md#size-maintained-surface-and-repeated-io).

## Published Canic integration — selected persistent counter

Canic v0.224.0 qualifies the published host adapter with Backup 0.105.6 and IC Host Sign 0.1.0. The installed direct route passed with a disposable keyring-backed secp256k1 identity. The Root-managed route recovered the original take and load after lost replies and host restart. With Root as sole controller, the selected counter was restored stopped, accepted under a fresh authorization and explicitly opened. The consumer scope is one same-release persistent counter without timers, external work or additional participants. These are installed test results; deployment of a live application with operator credentials is a separate operational step. See [external signer qualification](https://github.com/dragginzgame/canic/issues/5), [Root executor delivery](https://github.com/dragginzgame/canic/issues/6) and [application fencing qualification](https://github.com/dragginzgame/canic/issues/7).

The earlier local Root/child restart witness retained the original take and
load replies, both lookup certificates and journals under
`target/evidence/root-host-consumer/run-1794996922952207507/`.
The 20 MiB profile is qualified on the direct route.
See the [Root witness and supplier hashes at release](https://github.com/dragginzgame/ic-backup/blob/6ae096f/docs/status/current.md#root-execute-and-original-result-recovery--local-qualification).

## Preservation and remaining integration

Original wires, roots, journals, child outputs and server logs remain under
`qualification/target/fixtures/`, including failed runs. Never discard uncertain
publication attempts or surviving locks; reconcile the registry before retrying.
The original reader remains at `b7c2c6ae754a5ae2452e65b9be537b517614ebd2`,
`preserve/backup-0.101.13-before-replacement`, with evidence under
`.git/recovery/backup-0.101.13-before-replacement/`. The status-only restore reader
remains under `.git/recovery/host-store-1-before-restore-20261116/` (185 source files,
both locks, six artifacts; archive SHA-256
`6658a8963fe3d56cb5fe13ec210c9170f8d6b7b50f0de33583e627287a371b06`).
[Setup](../development/setup.md#host-store1-reader-before-complete-restore) owns details.
Old operations keep their matching readers, IDs, authority and evidence;
this release migrates no records and changes no legacy/sibling data.

Production HTTPS, actual-IC qualification on macOS and physical power loss
remain unqualified. The [sibling review](../design/sibling-integration.md)
owns the boundary for other applications and external obligations.
