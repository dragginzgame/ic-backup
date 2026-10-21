# Current handoff — 2026-10-21

## Current boundary

Core owns backup records, bounded spending, guarded persistence, reconciliation
and coordination. Agent owns reserved signed transport. Platform owns external
IC types/codecs/SDKs. IC Tooling owns Host, Build and Testkit: generic host
IO/process custody, release metadata rendering and simulator assets/processes.
Applications retain fresh authority, consistency, fencing and external-effect
settlement. The [implementation contracts](../implementation-contracts.md) remain
mandatory; this batch changes no Rust, storage, wire or recovery behavior.

## Current batch and selections

Task-entry HEAD is **`1ef6227` / 0.36.1**, with a clean working tree. The workspace
now requires Platform **0.6**, resolved to published **0.6.0**. The only added
package is Platform 0.6.0; all prior package versions/checksums remain selected.
Host/Testkit and the IC Build release editor remain **0.62.1**, Candid/derive
**0.10.39**, Agent SDK **0.49.2**, and development Metrics **0.15.0**. Package
versions remain **0.36.1**; compatible pending notes select **0.36.2**. No index,
commit, ref, release receipt or snapshot changes are made.

The registry archive checksum matches Cargo.lock and all 61 downloaded source
files match Cargo's actual cached source. Its recorded source revision is
`73dda4a5ee17abb78d905562b102a2af51031dda`; migration guidance is verified against
that committed changelog and the published README. All 23 files in the consumed
principal/account, Candid, management DTO, Agent and PocketIC modules are unchanged
from 0.5.1. The breaking change gates principal/account availability behind
`principal`, already included by our codec/DTO/client features. No new feature,
source adapter or consumer API change is needed.

Testkit retains its supported **Platform 0.5.1** development dependency. Its client
exports retain original upstream principal/PocketIC identities, so fixture calls
continue without conversion shims. Both libraries' independent production graphs
select only **Platform 0.6.0**. Leave Testkit/Host upgrades to their own batch;
removing this development-only duplicate does not justify a wider migration.
See [Platform guidance](../ic-platform.md) for maintained contracts and enforcement.

## Contributor tooling and retained history

Shared Tooling remains **0.25.9**, revision
`85aded191d85ac4a8c0e5deab207f6be18895862`, with 119 canonical payloads. Required
setup orders common host/Cargo tools, locked fetch, Testkit and the release editor.
Offline checks admit required selections. Optional IC executables remain outside
aggregate setup, CI reuse/PATH and release preflight. Manual IC setup/check
commands and retained bundles/evidence remain available. Testkit's authenticated
server, fixture custody and offline admission remain mandatory.

[Shared Tooling guidance](../shared-tooling.md), [development](../development.md)
and [release preparation](../releasing.md) describe current commands. Complete
adoption records remain in the [Shared archive](archive-shared-tooling-2026-10-21.md)
and [Platform archive](archive-platform-adoption-2026-10-21.md). Manifests, the
snapshot and this handoff own current selections and evidence; exact external
principal **0.1.5** and management DTO **0.11.0** constraints remain mandatory.

## Qualification and remaining limits

All **11 focused targets** pass: snapshot, direct dependency boundaries and
inheritance, formatting, early strict Clippy, all-target compilation, tests,
Rustdoc, Rust **1.91** with independent consumers, exact package archives and
documentation links. All **638 native tests** pass, including **38 Core + 12 Agent
PocketIC** and **14 HTTP** cases. Native/Wasm Rust 1.91 probes preserve all 24
management DTO and external principal identities; native also checks public Core
metadata/status projections. The new principal feature is active on all inspected
targets without expanding codec/runtime/parser selection.

Current evidence: `/tmp/backup-platform-06-20261021-r9dxe81w/`, including registry
metadata/archive, source comparisons, `validation/validation.qDTYqV/timings.tsv`,
`platform-identity/` and eight normal/build target trees with enabled features.
Independent consumers remain at `target/msrv-consumers.JaDLnQ/`, exact archives at
`target/package-check.cvW0Jn/`. Core stays Agent-free. Linux, Intel/ARM macOS and
Wasm tree inspection excludes development Platform, simulator, parser/CDK, WAT and
CLI from production; target inspection is distinct from native qualification.

Unchanged tooling evidence retains its original scope at
`/tmp/backup-local-simplification-pz964fhf/`: real setup/offline admission with
optional IC commands unavailable, actual offline CI reuse and 90 consumer release
cases (`target/shared-release-tests.ixoGB9/`). These tooling receipts are not
qualification of the new product graph. Earlier archives and failed evidence stay
retained. No full local `make ci`, new native macOS, installed-IC, hosted artifact
delivery, release or publication is claimed.

The direct boundary gate checks aliases and normal/build/dev/target declarations,
not transitive features or deployed bytes. Candid remains required for management
and SDK contracts, including upstream printer selection; development PocketIC's
DTO identity remains separate. The earlier Shared 0.26 optional macOS installer
hold is recorded in [the review](../shared-tooling-review.json); no snapshot repair
is adopted here. PocketIC retains its exact thiserror 2.0.18 requirement.

## Remaining application acceptance

Complete application acceptance still requires integration-owned current
membership/control/read authority, whole-selection fencing, retained source and
external-obligation custody, and terminal evidence. Local all-Applied checkpoints
and providers do not establish those facts. See
[application admission](../implementation-contracts.md#application-admission-and-fencing).
No new sibling or application qualification is claimed; earlier Canic observations
retain their recorded identities. Earlier migration history remains in
[the October 19 archive](archive-2026-10-19.md).
