# Current handoff — 2026-10-31

## Upstream adoption

Adopted published Host Artifacts/FS/Process **0.73.1**, Platform **0.15.2** with
canonical Types **0.15.1**, Testkit **0.75.0** and release editor Build **0.73.1**.
Testkit adds arithmetic-only Metrics **0.23.2**; optional Wire remains disabled.
Old Platform **0.11.1**/**0.13.0** selections are removed. Candid **0.10.39**, Agent
**0.49.3**, external Principal **0.1.5**, native management DTO **0.11.0**, PocketIC
**16.1.0** and every unrelated package/version/source/checksum are preserved.
All non-workspace dependencies remain registry-selected; no sibling patch exists.

The user released the preceding cleanup as **0.52.3**, commit
`4a071b6d781a1c6a7025d719fb0a57322c95db25`. Own package versions remain **0.52.3**.
Draft **0.53.0** records the public Host Rust identity change: consumers exchanging
checksum/publication errors must align direct Host dependencies to **0.73**.
Error payloads/source chains and record/wire/hash contracts are preserved.
No Backup Rust files changed. This adoption is unstaged/uncommitted; no release,
publication, sibling mutation, live IC operation or artifact cleanup occurred.

Platform keeps existing facade imports over its canonical owned Principal supplier.
The prior Principal implementation and Types source differ only in documentation.
Host Artifacts/Process production source is identical; FS changes only an upstream
test. Native external DTO/SDK identities and explicit conversions remain.
[The Platform boundary](../ic-platform.md) records consumer constraints.

## Shared source and setup

Shared revision **143a9abb9ae81feae9400ff3844b03b1ddfdd75a** was independently
selected from GitHub main and exported from a clean isolated source. Its committed
VERSION is **0.27.21**, while its title/leading notes say **0.27.22**. The snapshot
preserves that source label and exact revision: **134 payloads**, two additions,
no removals, with source bytes/modes verified. Dirty sibling edits were excluded.
[The adoption review](../shared-tooling-review.json) retains exact provenance,
archive checksums, committed-source comparisons and compatibility assessment.

Shared's release-group helpers remain inactive for Backup's single coordinated
version/root changelog. Existing publication, direct delivery and exact resume
adapters remain. Common host/Rust/optional IC executable catalogs are unchanged.
`make install-tools` and offline `make tools-check` pass for the selected Testkit
CLI/server and Build editor. Prior bundles and failed evidence remain retained.
PocketIC's exact server stays **16.1.0** under Testkit-owned admission/lifecycle.

## Fresh focused qualification

Evidence is `/tmp/backup-adopt-upstream.sKsOIs/`, including incoming manifest/lock,
prior handoff/review, registry responses/archives, clean Shared source, source
diffs and committed-file proofs. No library graph relies on these temporary paths.

Root and three independent consumer graphs pass locked metadata admission.
Six normal/build observations cover both libraries on Linux x86-64 and both macOS
architectures. Library graphs exclude Testkit/PocketIC/WAT/Clap/runtime guard;
Core excludes Agent. Normal graphs have one Platform/Types and aligned Host
identities. Independent fixture locks retain original package selections.
These graph observations do not establish native macOS execution.

Strict all-target/all-feature Clippy passes for both packages (`clippy.log`).
Three independent consumer Clippy profiles and actual Rust **1.91.0** Core/Agent
public probes pass (`consumer-checks.log`), including canonical Host error payloads.
ShellCheck/syntax and read-only codec-vector checks pass (`shell-check.log`,
`codec-artifacts.log`); no fixtures were regenerated.

All **153 selected native cases** pass (`native.log`): **122 Core unit**,
**11 public**, **four Agent unit** and **16 HTTP** cases. They cover canonical
identity/hash admission, exact codecs/bounds, retained artifact/custody/recovery,
original spending and signed retention before single submission.

All **five selected real PocketIC cases** pass (`core-pocketic.log`,
`agent-pocketic.log`): connected same-ID round trip; independent cold
capture/restore/terminal replay and death after authenticated load; complete Agent
transfer/restore; wrong-root rejection. Retained Core root:
`/tmp/ic-backup-pocketic-2364636-0`. Agent roots:
`/tmp/ic-backup-agent-pocketic-2402392-*`; independent normal build:
`/tmp/ic-backup-agent-independent-build-2402392-0`.

Consumer tooling/tamper, graph/refusal, input-change stopping and failure retention
pass (`tooling.log`). Actual Build metadata preparation and exact receipt/resume
checks pass with isolated substitutes (`release-preparation.log`,
`release-receipts.log`); traces remain in `target/shared-release-tests.FmJgd9` and
`target/shared-release-tests.6CWn2D`. Final formatting, links, regular-file/licenses,
source provenance, diff and **506 frozen source/build inputs** are checked in
`final-checks.log`; manifest/lock selections remain frozen through qualification.

No full CI gate, full MSRV/macOS matrix, package/registry-only release acceptance,
installed Canic migration or live qualification was run. Focused results grant no
complete-product, application-fence or terminal/reference-release authority.

## Retained obligations and earlier evidence

The preceding complete handoff and adoption review are `prior-status.md` and
`prior-review.json` in this adoption's evidence directory. They retain original
cleanup/adoption qualification scopes, earlier failures and source identities.
[Implementation contracts](../implementation-contracts.md) and the
[operator map](../canic-adoption.md#reusable-operator-composition) remain canonical.
Preserve all `/tmp/backup-*`, simulator/independent-build roots, original Canic
journals, `target/msrv-consumers.*`, release fixtures, validation failures and
old/failed tool bundles. No disposal or live effect is authorized by this handoff.
