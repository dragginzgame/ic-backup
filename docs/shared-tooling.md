<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-backup/ic-backup-readme-header.svg" alt="IC Backup — Verified backups and safe recovery for Internet Computer apps" width="100%">
</p>

<!-- helper-navigation:start -->
<p align="center">
  <a href="https://github.com/dragginzgame/canic"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/canic.svg" width="18" height="18" alt=""> <strong>canic</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/icydb"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/icydb.svg" width="18" height="18" alt=""> <strong>icydb</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-timers"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-timers.svg" width="18" height="18" alt=""> <strong>ic-timers</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-memory"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-memory.svg" width="18" height="18" alt=""> <strong>ic-memory</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-query"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-query.svg" width="18" height="18" alt=""> <strong>ic-query</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-backup"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-backup.svg" width="18" height="18" alt=""> <strong>ic-backup</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-blob-storage"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-blob-storage.svg" width="18" height="18" alt=""> <strong>ic-blob-storage</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-testkit"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-testkit.svg" width="18" height="18" alt=""> <strong>ic-testkit</strong></a>
</p>
<!-- helper-navigation:end -->


# Shared engineering and tooling adoption

The maintainer requested adoption of Shared Tooling as the top-level Dragginzgame
engineering rules on 2026-10-04. [AGENTS.md](../AGENTS.md) identifies the local
product overlay; [DRAGGINZGAME.md](../DRAGGINZGAME.md) is the reviewed, byte-for-byte
local copy of the upstream engineering rules. Common validation rules apply here;
Shared Tooling's own `AGENTS.md` script-suite commands apply only upstream.

## What this adoption means

Here, a tooling snapshot means a reviewed local copy of Shared Tooling files,
distinct from an Internet Computer canister snapshot or application backup.

| Layer | Responsibility |
| --- | --- |
| Shared Tooling source | Supplies reviewed repository-neutral rules and helper files |
| Vendored snapshot | Pins exact local bytes and executable modes so CI needs no sibling checkout or network access |
| `DRAGGINZGAME.md` | Preserves the reviewed common engineering baseline used by this repository |
| `AGENTS.md` | Adds IC Backup's product-specific architecture, safety and authority rules |
| Local adapters | Connect shared helpers to this repository without editing vendored files |

Upstream changes do not alter this repository automatically. Adoption requires a
reviewed refresh, a normal consumer diff and the relevant local validation.

## Reviewed sources

The current 48-file snapshot selects the committed 0.1.7 batch at
[`47cd2ccaf0e8b428f06e6db0262df76cfc1581de`](https://github.com/dragginzgame/shared-tooling/tree/47cd2ccaf0e8b428f06e6db0262df76cfc1581de),
matching remote main and a clean read-only sibling checkout. Upstream's canonical
`VERSION` still reads 0.1.6 and its 0.1.7 notes are undated; this is a committed
source identity, not a new finalized release claim. The exact Git-object exporter
refreshes all declared files; no vendored implementation is locally patched.
[The adoption review](shared-tooling-review.json) binds the files and evidence.

Portable checksum generation now shares one verifier owner with installer
receipts and snapshot export, validates backend output and handles unusual file
names through stdin. IC receipts reject names their line format cannot represent,
and retain failed candidates when traversal or hashing fails. Executable versions,
pins, receipt formats and activation policy retain their existing owners.

The additive `make check-doc-links` uses the exact shared Perl parser with a local
tracked/non-ignored Markdown roster. CI/release gates and native macOS tooling
checks include local target admission and the selected helper regressions. It
checks local file/directory existence, not anchors or remote URLs. Fresh Linux
parser, digest, installer and snapshot/runner fixtures pass; native macOS and
complete configured CI remain separate qualification. No full gate ran locally.

The complete engineering baseline is unchanged. Other new helpers remain optional:
there is no registry-observation or RustSec preparation flow here to replace.
Consumer receipt and original-input checks remain local. The release-command
checker is not installed; this consumer retains its extra completed-resume receipt
check and existing actual-Make fixtures rather than replacing those obligations
with a runner-only fixture. No release/version transaction, publication, Git
commit or cleanup is part of adoption. Extend the compatible pending 0.3.9.

Previous files, manifest and review remain under
`target/shared-tooling-039-adoption/previous`, `previous.snapshot` and
`previous-review.json`, alongside fresh focused logs and upstream diff.

### Retained 0.1.6 adoption

The maintainer authorized adoption of committed Shared Tooling 0.1.6 on 2026-10-06.
That 44-file snapshot selects
[`a37771f1b6b5fc9a88ed6ab3b705bdda35cd8fa3`](https://github.com/dragginzgame/shared-tooling/tree/a37771f1b6b5fc9a88ed6ab3b705bdda35cd8fa3),
matching remote main at inspection and a clean read-only sibling source. The
unchanged Git-object exporter includes all shared audit methods, pinning rules/
checker, parser and IC pins, installers and their focused regressions. Product
records, package/MSRV/lock selections, release commands and receipt owners remain
unchanged. Setup/check targets are compatible additions in pending 0.3.7, under
[#3](https://github.com/dragginzgame/ic-backup/issues/3).

Explicit `make install-tools` prepares pinned jq/yq followed by six IC executables.
Each set activates independently after checksum/version checks; previous and failed
candidates remain retained. Make/CI select local bin paths. `make tools-check` is
offline. CI/release gates also check declarations and scoped existing exact-version
reasons in [the local overlay](../AGENTS.md#qualified-dependency-constraints).
Provisioning adds no transport/provider or IC runtime qualification.

The logger clears checkout/snapshot identity at target dispatch, preserving normal
release selections. Independent fixtures clear inherited Make/release/logger context.
Consumer-owned `scripts/ci/test-release-metadata.sh` reuses existing receipt,
failed-log and real-index cases for the immutable shared regression; Shared
Tooling's changelog-only adapter is not installed. No vendored file is patched.

Actual Linux setup, offline checks, declaration admission and focused shared
tooling/installer regressions pass. Evidence and previous snapshot/review remain
under `target/shared-tooling-037-adoption`; the review record binds file identities
and check scope. Native macOS setup/changed-source CI remains pending. Upstream's
[0.1.6 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37450707625)
subsequently completed successfully; upstream results do not qualify this consumer.
Shared audit methods are available offline; no product audit, report migration or
automatic broad gate was run by adoption.

### Earlier inspected and adopted revisions

A subsequent maintainer-requested update inspection on 2026-10-06 confirmed
remote main at [committed 0.1.5](https://github.com/dragginzgame/shared-tooling/commit/a7efade1a68e43f148252a1a73908a46c4cbe9e9).
That revision was inspected before the current adoption. Its new
dependency-pinning rules require the exact shared declaration checker/jq module,
reviewed parser preparation, exact-scoped compatibility exceptions and CI/release
gate wiring. It also documents locked preparation for every affected workspace
and isolates validation logger checkout/snapshot identity at target dispatch.
Independent fixtures retain their own Make-selection responsibilities.

Executing the exact committed checker read-only against this consumer reports
`command-fds =0.3.3`, `ic-management-canister-types =0.11.0` and
`ic_principal =0.1.5` as requiring compatibility-reason records. Existing source
qualification needs deliberate review under the new exception contract; inspection
does not authorize relaxing requirements or changing locked versions. The published
`ic-metrics` requirement was already compatible rather than exact. At that inspection
no checker, exception or parser installer was adopted. The then-dirty tool changes
were retained separately. Historical evidence is under
`target/maintenance-037-review.B63Hd3`, including
the committed diff, dirty source status, exact checker bytes and its findings.

The inspected upstream repository is
[`dragginzgame/shared-tooling`](https://github.com/dragginzgame/shared-tooling).
The previous committed baseline and tooling revision was
[`cb86188c5956866564de4fb6ec6be67b27981ab9`](https://github.com/dragginzgame/shared-tooling/tree/cb86188c5956866564de4fb6ec6be67b27981ab9),
reviewed on 2026-10-06. A read-only remote query confirmed that exact `HEAD`/`main`;
the read-only sibling checkout was clean. The unchanged upstream Git-object
exporter supplies all twenty-three exact files. The validation runner retains raw
temporary logs if copying them to the configured failure directory fails. Consumer
release admission checks staged and working paths independently, including original
metadata hidden by restored working bytes. Existing candidate/changelog and whole
prepared-index checks remain enforced. Real private-index cases reuse existing
history without creating commits. Conditional Git/formatter substitutes explicitly
reject missing evidence; Linux reproduces the shell condition behind 0.3.5's macOS
completed-tag failure. Native macOS confirmation remains pending.

The release runner reconciles saved intent through normal targets before
selecting another increment. Its late adapters receive `RELEASE_COMMIT`, which may
precede HEAD. Hook refresh preserves formatter failure and canonicalizes checkout
paths. Public commands remain the same; these are compatible recovery corrections.
No vendored file is patched locally. See [the review provenance](shared-tooling-review.json).

The previous `9437bab201bb6071da0bdc4de0336daf553113f5` files, manifest and audit
are retained under `target/shared-tooling-036-review/previous`, `previous.snapshot`
and `previous-review.json`. Current focused Linux tooling evidence remains in that
same directory; snapshot artifact evidence is under `target/snapshot-artifact-review`.

The previous `c0206f1943238e21bd00fbe01658e6a0864c24fa` files, manifest and audit
are retained under `target/shared-tooling-035-review/previous`, `previous.snapshot`
and `previous-review.json`. Those are historical source/qualification paths; native
macOS qualification and complete CI remain separate.

The previous `b8537873ac124ad17b30e32aa23e9006a3e6ec21` files, manifest and audit
record are retained under `target/shared-tooling-refresh.BOQlFZ/previous`,
`previous.snapshot` and `previous-review.json`. That audit's separately hashed dirty
proposals and all earlier evidence remain historical; this revision now commits
the automatic changelog/root dependency rules plus the new hook contract.

The earlier snapshot at `41e1fd0ba41460bd2127cbf98ac8a4b2b2020d3e` and all its
exact files are retained under `target/shared-tooling-review.ztA5F9/previous`;
its manifest is `target/shared-tooling-review.ztA5F9/previous.snapshot`. The
refresh records the same canonical HTTPS source. Product choices stay within
the shared baseline; no exception is claimed for required macOS support,
release ordering, artifact retention or effect authority.

[The snapshot manifest](../.shared-tooling.snapshot) binds the committed source,
exact SHA-256 bytes and executable modes of the vendored principles, consumption
guide, host matrix, checksum verifier, snapshot verifier, validation runner and
runner regressions and common baseline. CI and releases use only these local
files, without fetching Shared Tooling or reading a sibling.

The original adoption pinned tools at
`956236a3848c2cfae6ae05f5c77e9c37b01b3366` and separately reviewed dirty upstream
AGENTS.md bytes with SHA-256
`142be0235013d766114c3036f903f773caabc758e87c2c76d2baa250e2ca1a6d`.
Those are historical source identities, not the current rules. The refresh
retains the previous manifest and files under the evidence directory linked
from [the handoff](status/current.md).

Keep vendored files unchanged. Refresh from a reviewed clean revision using
[the upstream consumption procedure](consuming-snapshots.md), inspect the diff
and rerun affected consumer checks. Keep consumer adapters outside the snapshot;
`scripts/ci/test-tooling.sh`, the Makefile and release helpers are consumer-owned.
Exclude all manifest-declared paths from local banner, navigation and prose
rewrites. Consumer explanations belong here, in [the documentation index](README.md)
or [development](development.md); the manifest continues to bind exact upstream bytes.
Earlier [tooling provenance](tooling-provenance.json) remains historical evidence
for the adapted release/hook helpers, not a Shared Tooling snapshot identity.
The inspected Shared Tooling revision contains no standalone license file; retain
its source attribution rather than assigning it a sibling's license notice.

## Common release workflow adoption

Current adapters verify original validation and receipt source/date/version/member
hashes from the exact selected release commit. Normal targets finish an unchanged
same-kind saved release only; newer committed fixes or another requested kind then
require fresh preflight and complete validation before the next increment. Explicit
resume selects one saved release, with post-completion annotated-tag and original
validation checks even when the shared plan is already complete. No old proof is
rebuilt from current source. Unknown remote history or conflicting intent stops.
See [the maintained release guide](releasing.md) and the adopted
[issue #2](https://github.com/dragginzgame/ic-backup/issues/2).

The initial workflow adoption below records the reviewed 0.3.0 transition.

The maintainer selected pending `0.3.0` because replacing the public release
workflow is breaking before 1.0. Existing pending implementation notes move into
that same entry; package metadata remains `0.2.3`. The public patch/minor/major
entry points invoke the exact vendored runner with explicit branch/remote inputs.
`release-resume VERSION=X.Y.Z` continues its saved source/date/version/destination
and phase. Conflicting selections reject; `make -n` remains read-only because local
recipes omit the upstream example's recursive `+` prefix. The shared runner still
owns all staging, commits, annotated tags and the exact atomic branch/tag push
with `--no-follow-tags`. Consumers retain original metadata, validation evidence
and build artifacts. See [the local release guide](releasing.md).

The obsolete public preparation aliases and separate stage/commit/push workflow
are removed, including arbitrary one-shot `release-x`. There is one current
workflow, without compatibility wrappers. Consumer adapters reuse the existing
v1 package receipt owner and retain original-input validation evidence under the
runner's Git state directory. Bounded package version parsing and exact inherited
member ownership remain unchanged. Preparation edits only the package's lockfile
version; no Cargo update or dependency reselection occurs. Ordinary failures restore
original metadata; interruption evidence/metadata backups remain retained.

Standalone publication still delegates package admission to Cargo, and read-only
version, plan and tag inspection remain available. Resume additionally checks
retained validation/receipt evidence and the exact completed local tag, including
when the shared plan is already complete. Consumer checks use actual Make entry
points with Git/Cargo substitutes; they establish no real release/publication.
Native macOS qualification remains pending. Snapshot integrity, consumer behavior
and actual host/publication evidence are distinct.

The committed rules now require automatic numbered pending headings and an explicit
root Cargo dependency catalog. The selected `0.3.0` remains the next minor from
finalized `0.2.3` because the complete batch breaks public release/setup commands;
compatible slices extend it. All nine direct dependencies already inherit the root
catalog. Cargo-sort 2.1.4 sorts both manifests without changing effective package
metadata, dependency/feature selection or the lockfile. Package version selection
and release transactions remain separately authorized.

`make install-hooks` replaces `hooks-install` and activates the unchanged shared
hook locally. Formatting uses an isolated exact index export, refreshes only selected
files and rejects partial staging while retaining unrelated edits. The member license
is now an identical regular copy of the root notice because the standard hook rejects
symlinks anywhere in its tracked snapshot. The obsolete local installer is removed;
its protected-hook-path obligation is covered by the stricter shared installer.
Both `fmt` and `fmt-check` use the pinned manifest sorter before Rust formatting;
the latter also admits prepared release metadata before staging. Setup installs
tools explicitly and independent CI/release checks never mutate formatting.

## Local choices and evidence

The new [agent maintenance rule](../rules/agent-maintenance.md) defines user-triggered
CI/issue inspection and carries explicit session activation forward. Checks inspect
and report; repairs and GitHub writes retain their separate authority. One current
inspection reviewed all workflows/runs for local HEAD, open issues/discussion and PRs.
The existing description matches the implemented scope; issue #1 remains open with
already-satisfied metadata. No issue was created, commented on or closed.

Backup contracts, v1 records, same-ID/same-release recovery, finite call authority,
target ownership and the targeted-check boundary remain local. Rust stays at
edition 2024, development 1.99.0 and MSRV 1.91.0. The library API, dependency
selection, package version and live-effect authority are unchanged by this tooling
adoption. Its release command hard cut is described above.

The shared baseline and maintainer-provided local tracking instructions require
GitHub issues as the sole record for bugs and follow-up work. For reusable Shared
Tooling feedback, record the reviewed revision, affected
owner/callers/hosts, symptom, focused evidence, smallest proposal and disposition
in the owning repository's issue, then link it from the handoff. Do not create a
competing local feedback list. Sending feedback or changing upstream requires
separate authorization. This document records adoption, not outstanding issues.

`make shared-tooling-check` verifies the snapshot offline. `make tooling-check`
runs upstream runner cases against the vendored implementation plus local
byte/mode/missing-file/symlink/manifest rejection and evidence-retention cases.
`make release-check` exercises the actual Make gate with substituted Cargo and
checks failed dependency preparation stops later targets while retaining logs.
Fixture-based checks bind their repository, failure-log and GitHub summary paths
locally, including when a parent CI runner supplies a different context. Expected
fixture failures do not alter the parent's retained logs or reported results.
These native tool checks establish no IC behavior. The shared host matrix
describes upstream evidence; [the consumer matrix](development.md#supported-host-scope)
requires macOS 15 on Apple Silicon and Intel alongside Linux. CI covers those
three hosts and separately invokes consumer checks with macOS system Bash 3.2.
Passing native macOS evidence remains pending; Linux checks and configured jobs
alone cannot qualify it. Local release/hook regressions use portable SHA-256
selection and Perl editing; the refreshed verifier handles empty Bash 3.2 arrays.

The stale design-only GitHub description was corrected with explicit maintainer
approval and read back on 2026-10-05: "Host-side Rust foundations for Internet
Computer backup and same-release recovery: local artifacts, journals, plans and
bounded IC codecs. Transport and runners are not yet implemented." Initial
automatic approval review rejected the remote metadata effect under audit-only
authority; the update ran only after the maintainer separately approved its exact
text. No issue, commit, tag, push or package publication was created.

`make ci` preserves target order and stops at the first failure. It adds live
target-labelled errors, a timing/result summary, a GitHub step summary when
available, and full/highlighted failure logs under `target/validation-failures/`.
Consumer build/evidence artifacts survive success, failure and retry.
Full CI remains a separately authorized/configured gate, not routine validation.
