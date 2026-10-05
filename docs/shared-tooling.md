# Shared engineering and tooling adoption

The maintainer requested adoption of Shared Tooling as the top-level Dragginzgame
engineering rules on 2026-10-04. [AGENTS.md](../AGENTS.md) identifies the local
product overlay; [DRAGGINZGAME.md](../DRAGGINZGAME.md) is the reviewed, byte-for-byte
local copy of the upstream engineering rules. Its upstream-only validation section
does not prescribe consumer commands.

## Reviewed sources

The inspected upstream repository is
[`dragginzgame/shared-tooling`](https://github.com/dragginzgame/shared-tooling).
The committed baseline is
[`956236a3848c2cfae6ae05f5c77e9c37b01b3366`](https://github.com/dragginzgame/shared-tooling/tree/956236a3848c2cfae6ae05f5c77e9c37b01b3366).
At final review, its `AGENTS.md`, CONTRIBUTING/README and two principle guides were
dirty. The copied rules include the reviewed `AGENTS.md` working-tree changes,
with SHA-256
`142be0235013d766114c3036f903f773caabc758e87c2c76d2baa250e2ca1a6d`.
Those bytes are separately reviewed local guidance; they are not attributed to
the committed revision, and no upstream acceptance or publication is claimed.
Future upstream edits do not change the adopted copy automatically.
The common rules are mandatory. The committed principle guides predate these
working-tree authority clarifications; the adopted baseline controls their
interpretation. Product choices stay within its delegated scope; no local
exception is claimed for tracking or validation.

[The snapshot manifest](../.shared-tooling.snapshot) binds the committed source,
exact SHA-256 bytes and executable modes of the vendored principles, consumption
guide, host matrix, checksum verifier, snapshot verifier, validation runner and
runner regressions. These files came from an isolated clean local checkout of
the named commit. CI and releases use only these local files, without fetching
Shared Tooling or reading a sibling. The working-tree rules copy is deliberately
separate from this committed tooling manifest.

Keep vendored files unchanged. Refresh from a reviewed clean revision using
[the upstream consumption procedure](consuming-snapshots.md), inspect the diff
and rerun affected consumer checks. Keep consumer adapters outside the snapshot;
`scripts/ci/test-tooling.sh`, the Makefile and release helpers are consumer-owned.
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
These native tool checks establish no IC behavior or additional supported host.
The shared host matrix describes upstream evidence; consumer CI remains Linux.

`make ci` preserves target order and stops at the first failure. It adds live
target-labelled errors, a timing/result summary, a GitHub step summary when
available, and full/highlighted failure logs under `target/validation-failures/`.
Consumer build/evidence artifacts survive success, failure and retry.
Full CI remains a separately authorized/configured gate, not routine validation.
