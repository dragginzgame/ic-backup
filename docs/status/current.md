# Current handoff — 2026-10-24

## Ownership and source

Core owns bounded backup records, guarded persistence, reconciliation and workflow
coordination. Agent owns reserved signed transport. Platform owns external IC
identities/codecs/SDKs; Testkit owns simulator assets and managed processes.
Applications retain membership, permissions, consistency, fencing and external
obligations. Current contracts belong to [implementation contracts](../implementation-contracts.md)
and [the record/API boundary](../extraction-boundary.md).

The API cleanup started from clean released **0.40.1**, `73b145f`, leaving package
versions, manifest/lock selections and Shared Tooling **0.26.9** payloads unchanged.
Pending **0.41.0** notes describe the authorized breaking Rust API cleanup; they
are not a version change or release. No staging, commit, push, publication,
deployment, live IC effect or sibling edit occurs here.

## Completed API cleanup

- Metadata/download/allocation/data settlement errors no longer expose unproduced
  request/association variants or their generated conversions. Existing single-call
  Read/Upload failures still own those errors and retained replies.
- Complete operation plans own inventory/graph persistence; original journals own
  execution progress. Standalone file publishers/readers, caller-declared completed
  sequence readiness and their obsolete tests are removed. Standalone file-byte
  constants/schema annotations are removed; embedded model bounds and the complete
  plan's 1 MiB input/output limit remain. Existing files are not deleted or migrated.
- Mandatory async workflow admission remains the integration mechanism. Five unused
  synchronous observation traits/error enums are removed. Passive requests/results,
  pure validators and all application safety obligations remain. Public fixtures
  retain stale/revoked evidence, original spending, bytes and obligation assertions;
  they no longer test injected pass-through provider errors.

Consumers must update imports and error matches as described in the
[changelog](../../CHANGELOG.md). This is a source-level hard cut, with no new wire
format, persisted schema generation, compatibility reader or lifecycle behavior.
Extraction provenance retains original source identities and historical destinations,
with an explicit historical-scope note; current guidance describes the surviving APIs.

## API cleanup qualification

These results precede the incoming dependency update described below.

Linux all-target/all-feature compilation and strict Clippy pass. Warning-denied
Rustdoc, shared formatting and documentation links pass. All **528 core unit tests**
and **7 public integration tests** pass, including inventory selection through the
original plan, execution progress and all five application-evidence journeys.
The unit suite includes the affected compound snapshot workflows, bounded models,
immutable plan persistence, journal-derived prerequisites and retained-reply failures.
Rust **1.91.0** all-target/all-feature MSRV compilation also passes.

Logs are `/tmp/backup-cleanup-{check,lib,integration,clippy,rustdoc,msrv,docs}.log`.
No simulator execution, native macOS, installed IC, independent registry-consumer,
Canic or full local `make ci` qualification was run for this API-only cut.
[The released handoff](https://github.com/dragginzgame/ic-backup/blob/73b145f/docs/status/current.md)
retains prior dependency-adoption evidence at its original source/graph scope.

[Canic adoption preparation](../canic-adoption.md) remains application-owned;
this cleanup supplies no production authority/fence integration and grants no live
backup attempt before Blob storage is complete.

## Documentation follow-up

The development guide now owns contributor setup, focused checks and fixture
instructions. Detailed product behavior links to the implementation contracts,
record/API boundary and Agent guide instead of repeating an extraction ledger.
The crate overview describes current async admission, workflow coordination and
application-owned obligations. Historical source/provenance evidence is retained.
No runtime code, symbol, schema or dependency change belongs to this follow-up.

An incoming lockfile update selects Host Artifacts/FS/Process and Testkit **0.66.3**;
it is preserved byte-for-byte. Earlier API-cleanup test results are not new
qualification of that update. Formatting, documentation links, all three independent
golden-generator `--check` commands and warning-denied Rustdoc pass on the current
checkout. No runtime tests, MSRV, simulator or native macOS checks were rerun for
this documentation-only follow-up. Logs: `/tmp/backup-doc-cleanup-checks.log` and
`/tmp/backup-doc-cleanup-rustdoc.log`.
