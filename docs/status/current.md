# Current handoff — 2026-10-22

## Ownership and current source

Core owns bounded backup records, guarded persistence, reconciliation and workflow
coordination. Agent owns reserved signed transport. Platform owns external IC
identities/codecs/SDKs; Testkit owns simulator admission and managed server custody.
Applications retain current permissions, consistency, fencing and external-effect
settlement. [Implementation contracts](../implementation-contracts.md) own current
obligations, with exact records/APIs at [the extraction boundary](../extraction-boundary.md).
The [design](../extraction-design.md) retains rationale and future-product proposals.

Task source is **`21b0a94`**, released **0.37.3**, plus the incoming dependency lock.
The new **0.37.4** notes describe compatible fixture/documentation cleanup and
qualification of the incoming lock; workspace/package versions remain 0.37.3.
No commit, release or publication occurs.
Production Rust, public APIs, wire/storage formats and recovery semantics are unchanged.
The incoming Host/Testkit **0.63.3**, Platform **0.6.7** and Tokio **1.54.1** selections
are preserved; this task neither selects nor upgrades their versions.

## Focused recovery fixture cleanup

The seven real lost-reply cases stop at their own recovery boundary. Stop/start and
capture cases avoid a subsequent full backup/restore. Upload-source setup reuses
the existing production capture/metadata/download workflow driver; data recovery
qualifies one original write/readback without uploading the remaining snapshot.
Load recovery uses the original native snapshot, deliberately changes live state,
then independently verifies complete stopped restored state before settling load.
Lost load plus lost status retains both pending reservations, with no verification
capture or restart. Original spending, exact journal bytes, source artifacts and
references remain checked. Connected workflow cases retain full uploaded-destination,
same-ID round trips, cold continuation and terminal replay qualification.

The older hand-written full download/upload/restore orchestration and obsolete
capture success/return plumbing are removed.
Independent extent/readback oracles remain intentionally separate from production
planning. Readback requires expected bytes for every declared extent before another
management call; a truncated paired iterator cannot skip final-byte verification. Core fixtures retain the admitted server digest and original Cargo.lock;
focused result records use Testkit's server version and the copied lock's digest.
No independent client/server version catalog remains in those results.

Current contracts and record details replace duplicate implemented descriptions in
the design. Its former implementation ledger remains at an immutable source link;
unique future-product requirements, rationale and completion criteria stay available.

## Validation and retained evidence

Fresh Linux qualification covers the incoming locked graph on Rust **1.99.1**:
**642 tests pass**, with no failures, ignored cases or filters. This includes all
**536 Core unit tests**, the complete **39-case Core PocketIC target**, Agent's
**14 HTTP-boundary cases** and **14 simulator cases**. The latter include signed
same-ID capture/transfer/restore, independent cold consumer acceptance/terminal
replay, and consumer death after authenticated load without redispatch. These
are library/fixture acceptance, not Canic application qualification.

Native all-target/all-feature checks, strict Clippy and warning-denied documentation
pass. Rust **1.91.0** passes for both libraries' complete targets and independent
Core/Agent consumers. Both actual package archives compile together through the
independent consumer. Its graph and both MSRV consumer graphs exclude Testkit,
PocketIC and WAT; Core's independent graph also excludes Agent. Platform's enabled
features remain `candid`, `management-canister-types` and `principal`, plus `agent`
only for the Agent consumer. Cached archives for all six changed packages match
their locked registry checksums. Host and Platform have no Rust-source changes
against the previous selections; Testkit's Rust changes are test-only.

Evidence: `/tmp/backup-dependency-qualification-20261022-insx2e1_/`, including
per-target logs, package/MSRV consumer identities, source fingerprints, dependency
review and fresh original-lock recovery receipts. The initial aggregate stopped
affected targets at missing exact Testkit CLI admission. Explicit offline
`make install-testkit-server` prepared that selection and readmitted the existing
PocketIC **16.1.0** bundle; tool admission and all blocked targets then passed.
The native test run permits managed localhost servers. No tracked input changed
during validation; manifests, incoming lock, Rust sources and real index remain
preserved through the documentation-only qualification follow-through.

The earlier cleanup receipts remain at `/tmp/backup-complexity-20261022-to5fclwy/`
and `/tmp/backup-complexity-followup-20261022-d4pm7vd6/`, including the regression
that first demonstrated truncated readback expectations passing the old oracle.
Formatting, local documentation links, dependency inheritance/IC boundaries and
Shared Tooling snapshot integrity pass. The full local make ci suite, native macOS,
hosted CI and installed-IC/application acceptance are not run or rechecked here.

## Prior evidence and remaining acceptance

Shared Tooling remains committed **0.26.4**, revision
**`dc1ced7e240e90f0c6daa89fb6248fcb074164eb`**, with 119 exact payloads.
The [review](../shared-tooling-review.json) and
[previous handoff](https://github.com/dragginzgame/ic-backup/blob/21b0a94a1fa8241d26ed1b933e1e353fbc71d2ca/docs/status/current.md)
retain earlier graph qualifications, actual #63 implementation delivery, prepared
cache CI change and isolated Canic follow-through. Their receipts cover their
original sources/graphs; this task does not recheck hosted CI or Canic PR status.
[The Shared archive](archive-shared-tooling-2026-10-21.md) and
[Platform archive](archive-platform-adoption-2026-10-21.md) retain older evidence.

[#29 — Complete original-plan execution and stage binding before Canic runner
adoption](https://github.com/dragginzgame/ic-backup/issues/29) owns installed
authenticated application-runner acceptance: current membership/control/read
permission, whole-selection fencing, command/byte custody and external-obligation
settlement. Fixture checkpoints and local terminal replay do not establish those
production facts. No live IC effect or sibling source change occurs here.
