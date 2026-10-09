# 0.11.6 shared execution safeguards and Host review

Released base: `6b5714aed1b3f69636b037fafebdee03b627a418` (0.11.5).
Select the compatible undated 0.11.6 draft for maintained command safeguards and
qualification of the incoming Host patch selection. Package/receipt remain 0.11.5;
ordinary continuation performs no Git write or release.

## Shared ownership

Canonically refresh the snapshot from a clean isolated committed Shared Tooling
0.2.8 checkout at `b2646cde9abbc8861857a4379c683a0c19eba43e`, independently
verified against GitHub main. Select its declared `make/execution.mk` companion:
94 exact files, with unchanged common baseline and tool pins. The sibling's dirty
0.3.0 proposal is excluded. The include admits Make before recipes can be skipped
or failures ignored, resolves its existing behavioral probe beside the selected
include, and uses the actual Make executable when recursive `MAKE` carries args.

Preserve the default help goal, prepared formatter admission and specialized
post-run `release-resume` receipt/tag/saved-validation check. Export the companion
and existing probe in local hook, release and Testkit-routing fixtures. The old
release fixture expected dry-run success; replace that expectation with refusal
and unchanged source/effect evidence for ignore-errors, dry-run, touch and question
modes. Document `make release-plan` as the non-mutating preview. No installer is
added to ordinary checks and no hook is activated. This extends the shared
formatting ownership delivered by 0.11.5 and [#34](https://github.com/dragginzgame/ic-backup/issues/34).

## Published Host selection

The incoming lock selects all four Host crates at 0.9.7. Crates.io confirms them
as latest stable, unyanked releases at inspection; archive checksums match both
registry and lock. Published Artifact/FS/Process sources are unchanged from 0.9.5.
Only Host Tools' `src/response/mod.rs` changes: consolidate empty text-hex admission
into its existing digit pass, preserving JSON empty-byte acceptance and typed
limits/errors. Backup has no response-envelope caller to migrate; Agent consumes
exact raw IC replies. No safe extra code offload or local Rust-symbol removal
follows. Preserve Backup's descriptor confinement, private publication, exact
records/journals and separate spending/custody owners.

Host Process/Tools remain Testkit dev dependencies; both independent normal
consumers exclude the simulator. Testkit remains 0.27.2 and its already prepared
exact CLI passes offline admission with PocketIC 16.1.0. Its published installer
lock is independently owned; root Host patch selection does not invalidate its
unchanged CLI receipt. Keep Metrics 0.3.4 and Agent 0.49.2. Preserve incoming
manifest/lock bytes without reselection, sibling patches or tool reinstallation.

## Focused qualification and limitations

[qualification.json](qualification.json) binds selected sources, inputs, archive
comparisons and retained logs under `target/continuation-0116/`. Shared owner
formatting, release-command and hook suites pass on the exact isolated source.
Eight actual consumer Make admission cases pass, including alternate-root and
recursive-argument cases. Actual hook/index/partial-stage and formatter cases,
Testkit routing, corrected consumer release fixtures and substitute-runner tests
pass. Retain the prior dry-run expectation failure separately.

All 80 focused artifact, JSON-publication, workflow and metrics unit cases pass;
five core and three Agent actual simulator cases pass. Warning-denied core tests
and Agent library Clippy pass, as do both independent Rust 1.88 normal consumers.
Selected formatter, snapshot/pin, Testkit server, ShellCheck and local-link checks
pass. No full workspace/CI/release gate was run and no native macOS result is claimed.

Released 0.11.5 has one exact-source main CI run:
[37958355952](https://github.com/dragginzgame/ic-backup/actions/runs/37958355952).
At job inspection Linux was in progress and both macOS jobs queued. Those results
cannot qualify this dirty candidate. Keep #32/#33 and #34 native acceptance open.
Released #29 data streaming remains separate from complete metadata/capture/upload/
restore orchestration, async Agent integration and application/terminal proof.
No open competing PR was found. No root Git write, release, production IC effect,
sibling edit, recovery cleanup or Rust function/method/type removal occurred.
