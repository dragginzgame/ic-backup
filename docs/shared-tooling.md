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
local copy of the upstream engineering rules. Its upstream-only validation section
does not prescribe consumer commands.

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

The inspected upstream repository is
[`dragginzgame/shared-tooling`](https://github.com/dragginzgame/shared-tooling).
The current committed baseline and tooling revision is
[`41e1fd0ba41460bd2127cbf98ac8a4b2b2020d3e`](https://github.com/dragginzgame/shared-tooling/tree/41e1fd0ba41460bd2127cbf98ac8a4b2b2020d3e),
reviewed on 2026-10-05 from a clean read-only source checkout. The refresh uses
the upstream Git-object exporter, not copied mutable working-tree bytes. The
manifest's source was deliberately recreated with the checkout's canonical HTTPS
remote, replacing the earlier SSH spelling of the same repository. Adding
`DRAGGINZGAME.md` expands the declared set from eleven to twelve files and binds
the engineering rules and linked guides to the same revision as the tools.
Future upstream edits do not change the adopted copy automatically. Product
choices stay within the mandatory common baseline's delegated scope; no exception
is claimed for tracking, validation or required macOS support.

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

## Local choices and evidence

Backup contracts, v1 records, same-ID/same-release recovery, finite call authority,
target ownership and the targeted-check boundary remain local. Rust stays at
edition 2024, development 1.99.0 and MSRV 1.91.0. This adoption changes no crate
API, dependency selection, version or live-effect authority.

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

The public GitHub description was inspected during this adoption. It still says
the repository contains extraction design and contributor instructions, although
the library now implements local artifacts, journals, plans and bounded IC codecs.
Suggested replacement: "Host-side Rust library for Internet Computer backup and
same-release recovery foundations: local artifacts, journals, plans and bounded IC
codecs. Transport and runners are not yet implemented." Repository metadata was
read only; no remote write or issue creation was performed.

`make ci` preserves target order and stops at the first failure. It adds live
target-labelled errors, a timing/result summary, a GitHub step summary when
available, and full/highlighted failure logs under `target/validation-failures/`.
Consumer build/evidence artifacts survive success, failure and retry.
Full CI remains a separately authorized/configured gate, not routine validation.
