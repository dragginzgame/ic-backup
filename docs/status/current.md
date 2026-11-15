# Current handoff — 2026-11-15

Work lives at `/home/adam/projects/ic-backup`, branch `main`; the former legacy
local path was absent. Frozen legacy remains read-only at `dragginzgame/ic-backup-legacy`.
No original journals, bundles or obligations were imported, migrated or cleaned up.

The 0.101.1 checkpoint was pushed at
[938e5fc](https://github.com/dragginzgame/ic-backup/commit/938e5fc711878ce8776df7c69fbf30d1cf596f10),
followed by remotely verified 0.101.2 opening
[7478c6c](https://github.com/dragginzgame/ic-backup/commit/7478c6c9517e2ba89440e1ad5e3b33c1b15a23b9).
The 0.101.2 checkpoint contains the provider/snapshot batch for the authorized
GitHub-only delivery to `origin/main`. All packages remain `publish = false`;
registry publication, tags and canister deployment are excluded. A breaking
change to shipped public APIs requires the next minor. New unpublished provider
names were consolidated into `DirectCaptureProvider` and `CaptureSigner` before release.

## Implemented scope and safeguards

The [inventory contract](../architecture/local-inventory.md) owns private stores,
bounded completion listings/latest backup per target, manifest-bound payload
verification and held source reads. Integrity does not authenticate capture evidence
or grant restore authority. Original custody and block hashes remain mandatory.

The [capture contract](../architecture/capture-intent.md) owns immutable plans,
durable target claims and bounded chained ledgers. Uncertain writes invalidate the
live handle; complete uncertain frames remain spent. Partial frames, orphan evidence
and missing owners/locks refuse without repair or fresh authority.

`advance_status` reserves signing/submission, retains exact request and certificate
evidence, and checks stopped state, approved module and original controller.
`advance_snapshot` then authenticates a bounded original list, admits capacity and
takes one new snapshot. Each call retains separate original admission/reservations;
provider/application commitments and cumulative evidence/network/signing budgets
must fit the original plan. No replacement, uninstall, cleanup or list-delta receipt.

Prepared continuation reuses exact retained bytes without resigning. Pending
continuation invokes no provider. Settled replay reauthenticates locally without
effects. `inspect_status` and `inspect_snapshot` report original stages, admissions,
request IDs and reserved costs; a settled stage alone is not authentication or
completion. Existing status-only public helpers still refuse snapshot calls.

`DirectCaptureProvider` uses one bounded external sign-only invocation and one
HTTP submission per call, binding route/network/root, signer/guard protocols and
explicit historical timing into original authority. Expiry, deadlines and spent
allowances never renew. Direct Ed25519/secp256k1 are supported; delegated/other
identities refuse. Integrations own bounded signer output/network accounting,
process custody, exclusive snapshot custody and continuous application maintenance.
Backup owns no key, process launcher, runtime or Canic dependency.

The [transfer contract](../architecture/capture-transfer.md) owns pure bounded
metadata/data decoding, schedules and uploaded-state/readback comparison. Download,
original-request reconciliation, completion writing, full bundle-evidence admission,
upload/load/restore and CLI remain unfinished. A captured remote snapshot is not a
complete local backup. Canic execution still refuses pending its executor protocol.

## Dependencies and qualification

Selections are Host FS/Artifacts/HTTP 0.79.0, Platform 0.33.6/Types 0.20.4, Tokio
1.54.1, Tooling 0.43.2 and Rust/Cargo 1.100.0. The incoming product Platform patch
was preserved and qualification aligned; its added Cloud Engine module is outside
selected features. Reqwest remains behind Host HTTP. No sibling patch/shared target.

Private `qualification/` uses Testkit 0.106.0 and native PocketIC 16.1 through its
re-export. Its lock satisfies the client's exact `thiserror` 2.0.18 requirement;
the compatible product declaration admits it while the product lock retains 2.0.21.
Both graphs select matching Platform/Types/Host lines. Revisit when PocketIC's pin
changes. [Setup](../development/setup.md) retains supplier provenance and commands.

All 80 focused capture tests, 10 ledger tests, 16 custody tests and 15 documentation
examples pass. Strict product/private-workspace Clippy, warning-denying Rustdoc, formatting and
local documentation-link checks pass. The actual IC fixture
passes status/list/take, preserves an existing snapshot and installed module, leaves
the canister stopped, and verifies offline cold replay after server shutdown and
guard revocation. It retains exactly three signing/submission reservations. Native
tests cover pending cancellation and prepared recovery; they do not establish that
cancelling a host future cancels an accepted IC effect.

Evidence is under `target/evidence/snapshot-capture/`; earlier direct-status evidence
remains intact. Raw IC output, original journals and failed/successful fixture bundles
remain under `qualification/target/fixtures/`. Testkit owns explicit setup and offline
server admission. Core checks remain separate from `make test-qualification`.
Local Linux acceptance does not qualify external signer processes, Canic fences,
trusted HTTPS sessions, hosted macOS, power loss or full backup/restore. The full
product suite was not selected; affected persistence/provider checks were selected.

## Next work

Implement bounded original-request reconciliation and snapshot download, then verify
and complete a local bundle. Never grant another take because its reply was lost.
Canic owns membership, concurrency, routing, release identity, maintenance, external
settlement and reopening. V1 remains one already-stopped canister, local bundles and
same-release restore to the same existing ID, left stopped. Blob storage remains v2.
[Canic #1](https://github.com/dragginzgame/canic/issues/1) covers signer/application
adapters and executor/fenced recovery; no new supplier gap was found. Source edits
remain Backup-only; file supplier issues rather than editing siblings.
