# Current handoff — 2026-10-21

## Ownership and integration

Core owns bounded backup records, guarded persistence, reconciliation and workflow
coordination. Agent owns reserved signed transport. Platform owns external IC
identities/codecs/SDKs. IC Tooling owns host custody, release rendering and Testkit.
Applications retain current permissions, consistency, fencing and external-effect
settlement. The [implementation contracts](../implementation-contracts.md) remain
mandatory; this tooling adoption changes no Rust, wire, storage or recovery behavior.

[PR #63](https://github.com/dragginzgame/ic-backup/pull/63) is merged on main as
**`7714c97`**. It supplies the held-journal workflow reader and independent
Core/Agent capture/restore/recovery/terminal-release qualification. The merge tree
matches reviewed head **`1090459`** exactly. Versions remain **0.37.2** and the
combined unreleased notes select **0.37.3**. The 0.37.1/0.37.2 history remains intact;
those releases contained pending notes without this implementation. No new release
or publication occurred. The local main checkout now contains the actual code.

[PR #65](https://github.com/dragginzgame/ic-backup/pull/65) is merged as
**`5f34c63`**, resolving [#64 — Keep dependency fetching in setup](https://github.com/dragginzgame/ic-backup/issues/64).
Ordinary CI consumes the prepared dependency cache; explicit setup retains fetch.

## Shared Tooling adoption

The working tree adopts committed **Shared Tooling 0.26.4**, revision
**`dc1ced7e240e90f0c6daa89fb6248fcb074164eb`**, through the canonical exporter.
All **119** payloads retain exact source checksums and modes. The isolated producer
checkout is clean; sibling draft edits are excluded. The upstream version is
verified from committed VERSION/changelog, not an absent GitHub Release object.
[The review](../shared-tooling-review.json) records selection and evidence.

Ordinary development and authorized merges follow the baseline's
[asynchronous CI cadence](../../DRAGGINZGAME.md#validation-and-evidence).
Focused checks and enforced GitHub requirements still apply. Pending hosted runs
remain visible, not passing native qualification. Release, publication, deployment
and application-specific acceptance keep their separate gates.

Manual IC setup and offline checks now require the same explicit `IC_TOOLS` list,
for example `make install-ic-tools IC_TOOLS=wasm-opt` and
`make ic-tools-check IC_TOOLS=wasm-opt`. Required host/Cargo/Testkit/editor setup,
ordering and offline admission exclude optional IC tools. No downloads, funding,
reset policy or bundle cleanup is added; retained bundles and failures stay intact.
The complete optional catalog is unchanged. Full-mode CI evidence collection keeps
its caller interface; compact collection and reporting use the canonical selection.

## Focused evidence and limits

Evidence: `/tmp/backup-shared-practices-20261021-M8mR3LwA/`. Snapshot integrity,
offline tool admission, formatting, Cargo inheritance, IC dependency boundaries
and repository file/license checks pass. Runner regressions, consumer manifest/lock
checkpoints, transitive simulator/WAT refusal, original fixture retention, full-mode
failure evidence, checksums and archive custody pass. ShellCheck now follows
sourced helpers with `-x`; CI help correctly describes prepared-cache validation.
The refreshed runner preserves primary failures and bounded complete diagnostics;
the checksum verifier refuses NUL-corrupted backend output. Hook formatting and
partial-stage safeguards, isolated release metadata/retention and Testkit routing
fixtures pass, as do the canonical checksum owner's focused GNU/Perl cases.

[Shared Tooling #300](https://github.com/dragginzgame/shared-tooling/issues/300)
tracks clarification of asynchronous merge guidance under an additional agent
runtime approval gate. Runtime rejection remains an external blocker; no bypass
or invented GitHub review requirement is adopted. New DFINITY ownership guidance
requires solutions at the canonical Dragginzgame owner; retained historical
incident links are not dependencies on future DFINITY work.

The current lock selects **Host/Testkit 0.63.2 and Platform 0.6.5**, together with
`crypto-common 0.1.7` and `generic-array 0.14.7`. This differs from the earlier
Platform 0.6.4 qualification. Published Platform 0.6.5 adds feature-gated stable
memory-manager exports; Backup does not enable that feature and needs no Rust
migration. Its Candid, management DTO, principal and Agent boundaries are unchanged.

Selected Testkit CLI and authenticated PocketIC 16.1.0 admission pass, together
with snapshot integrity, formatting and dependency declaration/boundary checks.
Fresh all-target/all-feature build, strict Clippy and Rust 1.91 checks pass for
both packages. Both independent consumer process cases pass, including death after
authenticated load without redispatch. The connected Core same-ID restore and
effect-free replay case also passes. The checked lock remains unchanged.
Logs: `/tmp/backup-pull-platform-065-{check,msrv,consumers,core}.log` and
`/tmp/backup-pull-check-20261021.log`. Earlier evidence remains under
`/tmp/backup-canic-adoptions-20261021-_usix6or/`. The full local suite was not run;
native macOS for this pending graph and installed-IC qualification remain separate.

Exact merged-source [CI](https://github.com/dragginzgame/ic-backup/actions/runs/39595767750)
passed Linux and both native macOS jobs. It covers the committed 0.25.9 snapshot
and Platform 0.6.3 graph, not the current uncommitted adoption or dependency update.

## Canic consumer follow-through

Isolated contributions update Canic’s [tooling PR #604](https://github.com/dragginzgame/canic/pull/604)
and [TLS prerequisite #617](https://github.com/dragginzgame/canic/pull/617) with the confirmed
hook/Storage fixture repairs. [Checkpoint PR #607](https://github.com/dragginzgame/canic/pull/607)
now targets current main, preserving its two-participant proof and the parent’s
retained Root-journal proof. Both exact PocketIC cases and the governed complete
unit case on Rust 1.91 pass. These PRs remain open; hosted qualification/review and
production application acceptance are separate. The normal Canic checkout is untouched.

## Prior qualification and remaining application acceptance

The [pre-merge handoff](https://github.com/dragginzgame/ic-backup/blob/10904596e0d8c263429be7e665d6ee3e59804c46/docs/status/current.md)
retains complete implementation scope and original evidence. In particular,
`/tmp/backup-pr63-conflicts-20261021-9rk6cm0u/` records 64 focused native/PocketIC
cases, strict Clippy, Rust 1.91 and Rustdoc on Platform 0.6.3;
`/tmp/backup-pr63-conflict2-20261021-esb6jar_/` reruns strict Clippy, independent
consumer success/process-death cases and MSRV with 0.37.2 package metadata.
Earlier Tooling and Platform evidence remains in the
[Shared archive](archive-shared-tooling-2026-10-21.md) and
[Platform archive](archive-platform-adoption-2026-10-21.md). Failed fixtures,
original journals and retained evidence are preserved.

Keep [#29 — Complete original-plan execution and stage binding before Canic
runner adoption](https://github.com/dragginzgame/ic-backup/issues/29) open for the
installed authenticated application runner, current membership/control/read
permission, whole-selection fencing, command/byte custody and external-obligation
settlement. Candidate consumer paths and local Applied checkpoints do not establish
those production facts. Canic contributions use isolated worktrees; its normal checkout and pending work
remain untouched. No live IC effects occur here.
