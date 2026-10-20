# Current handoff — 2026-10-20

## Current boundary

Core owns backup records, bounded spending, guarded persistence, reconciliation
and coordination. Agent owns reserved signed transport. Platform owns external
IC types/codecs/SDKs; Host owns generic host IO/process custody; Testkit owns
simulator assets/processes. Applications supply fresh authority, consistency,
fencing and external-effect settlement. The
[implementation contracts](../implementation-contracts.md) remain mandatory.

## Current batch

Task-entry HEAD remains **`5614847` / 0.35.1**. Incoming work already selected
Host **0.32.0**, Platform **0.5.0**, Metrics **0.14.11** and Shared Tooling
**0.25.7** (`d39e8a84bf4fd7e42776d2bb8e589ad09aacb02b`). Testkit changed concurrently
from 0.59 to **0.60.0**; preserve that selection and qualify the current graph.
No manifest/lock edits are made by this pass. Original and concurrent inputs,
staging and unrelated pending work remain preserved.

Artifacts, FS and Process **0.32.0** are the latest published Host releases.
All 167 registry archive files match their checksums/unpacked sources at revision
`c3f6f605a47f248e18bc3ad337e431a990a7a619`. All 136 Rust source files match 0.31.2;
FS/Process update their internal Host dependency identities, without feature changes.
Backup needs no source adapter or new production feature. However its public
`PersistenceError::Publication` and `ArtifactError` checksum-error conversion now
use Host 0.32 types. Consumers sharing those values must align their direct Host
FS/Artifacts dependencies. Pending notes therefore select **0.36.0**; package
versions remain **0.35.1**. Wire/storage formats and custody/recovery policy stay
unchanged. See [consumer guidance](../development.md) and [notes](../../CHANGELOG.md).

Published Testkit **0.60.0** is checksum-verified across all 51 archive files;
its Rust sources match 0.59.0. It shares Platform 0.5.0 with production, including
decoder limits/errors, removing development-only Platform 0.4.1. Testkit retains
Host 0.31.2 solely in development. The exact 0.60 CLI is explicitly prepared and
admits the existing PocketIC **16.1.0** server. No local patch or compatibility
shim is introduced. See [the Platform boundary](../ic-platform.md).

The pending Shared Tooling adoption retains its canonical 119-file closure and
early Clippy ordering for CI/release and publication/dry runs. Saved-source
recovery and exact receipts retain their semantics. The
[review](../shared-tooling-review.json) records canonical provenance; unchanged
metadata failure checks retain their [release evidence](../releasing.md).

## Qualification and remaining limits

All ten focused targets pass: dependency boundaries and inheritance, formatting,
early strict Clippy, all-target compilation, tests, Rustdoc, Rust **1.91** with both
independent consumers, both exact package archives and local documentation links.
All **638** native tests pass, including **38 Core and 12 Agent PocketIC** cases
and **14 HTTP** cases. The independent Core consumer remains Agent-free and
compiles both public Host error conversions using 0.32. An isolated negative
control proves both conversions reject 0.31 identities with typed compiler errors.
Archives preserve exact source/license bytes and compile together.

Evidence: `/tmp/backup-host032-4pl4im4n/`, with
`validation-current/validation.uBtis0/timings.tsv`, registry/source comparisons,
`negative-host-identity/`, and eight normal/build target trees. Those trees cover
Linux, Intel macOS, Apple Silicon and Wasm for both packages; production contains
only Host 0.32.0 and excludes simulator/parser/CDK, WAT, CLI and opt-in filesystem
backends. Independent consumers remain at `target/msrv-consumers.cQMT5v/` and
archives at `target/package-check.r9ONVn/`. Manifest/lock bytes remain identical to
the preserved concurrent inputs. The initial `validation/validation.KtJRuI/` refusal
for an unprepared 0.60 CLI remains retained; explicit setup and fresh validation
resolved it without weakening offline admission.

Earlier Platform identity/Wasm evidence remains at
`/tmp/backup-platform050-_nqizc_l/`; Shared's six focused checks and 90 release cases
remain at `/tmp/backup-shared-0257-gq1cn_l2/`. The unchanged metadata checker's
controls remain at `/tmp/backup-shared-0256-b14_rsos/`. Receipts retain their
original source/graph scope. No new full local `make ci` or release is claimed.
The pushed 0.35.1 source's
[hosted CI run](https://github.com/dragginzgame/ic-backup/actions/runs/39414777205)
passed Linux and both macOS hosts; it does not qualify this dirty batch.
Cross-target tree inspection is separate from native macOS, Wasm compilation,
installed-IC and application qualification.

Direct dependency admission includes aliases and normal/build/dev/target edges;
transitive features still need separate review. Candid remains required for
management/SDK contracts, including upstream printer selection. Development-only
ic0 and PocketIC DTO 0.8 remain separate from production DTO 0.11. Independent
consumers enforce simulator/WAT exclusions, not role-comment-derived exclusions or
a development-only Clap ban. Deployment admission belongs to consuming applications.

## Remaining application acceptance

Complete application acceptance still requires integration-owned current
membership/control/read authority, whole-selection fencing, retained source and
external-obligation custody, and terminal evidence. Local all-Applied checkpoints
and test providers do not supply those facts. See
[application admission](../implementation-contracts.md#application-admission-and-fencing).
Earlier Canic source observations retain their recorded identities; no new sibling
inspection or application qualification is claimed. Migration history remains in
the [October 19 archive](archive-2026-10-19.md).
