# Current handoff — 2026-10-31

## Metrics and Testkit adoption

Adopted published **Testkit 0.75.1** with transitive, arithmetic-only
**Metrics 0.24.1**. The root Testkit requirement now selects this patch; the
incoming lockfile already resolved both and is preserved byte-for-byte.
Only these two packages differ from the preceding Host/Platform adoption;
every unrelated package/version/source/checksum is unchanged. Metrics stays
behind Testkit with defaults disabled; no direct or production dependency was added.

Testkit's optional Wire interface and Metrics' native Wire metadata remain disabled.
No Wire package enters the graph. Core and Agent normal production trees are
byte-for-byte identical to the prior adoption; neither includes Metrics, Testkit,
PocketIC, WAT, Clap or the runtime guard. Core remains Agent-free.

The exact managed **Testkit 0.75.1 CLI** is prepared through
`make install-testkit-server`, using an admitted cache entry. Its standalone
offline check passes and returns the authenticated **PocketIC 16.1.0** binary.
CLI receipts, compiler/features/source admission, server selection and lifecycle
remain with Shared Tooling/Testkit. Old tools and bundles are retained.

Official crates.io sparse-index responses authenticate both selected releases and
archive checksums. Published source matches Metrics commit
`1e7193f77107a528cd99492b40df6750ddd445b5` and Testkit commit
`7898f58fc0ee0a909484c192a81cf454683120b4`: **60** source/manifest files match
committed bytes and **93** cached files match authenticated archives. Testkit's
source and Metrics' arithmetic Rust source are unchanged; Metrics changes docs
and its unused native Wire dependency generation. No Backup Rust migration is needed.

Own package versions remain **0.53.0**, released at `bd11ee0`. Pending **0.53.1**
covers the accepted ten simplifications and both dependency adoptions. Existing
Rust cleanup, public APIs, record/wire/hash formats, spending and recovery remain
intact. Changes are unstaged/uncommitted; no product consumer migration is required.
Contributor notes require refreshing the exact Testkit CLI after this update.

## Fresh focused qualification

Evidence is `/tmp/backup-adopt-metrics-testkit.YY4bZ2/`: incoming/prior locks,
prior handoff, official index responses, archive/committed/cache source review,
metadata, exact normal-tree comparisons, setup and command logs.

Strict all-target/all-feature Clippy passes for both libraries (`clippy.log`).
Formatting, inheritance/pins and direct IC dependency boundaries pass
(`source-checks.log`). Actual managed setup and offline server admission pass
(`setup.log`, `server-check.log`). The focused original-selection, busy/refusal
and setup/check Make dispatch fixtures pass (`testkit-routing.log`).

All **five selected real PocketIC cases** pass (`core-pocketic.log`,
`agent-pocketic.log`): Core connected same-ID round trip and effect-free replay;
cold recovery at allocation/load/start; actual Agent complete transfer/upload/
restore; wrong trusted-root rejection; discarded stop reply preserving pending
original spending without reissue. Core roots are
`/tmp/ic-backup-pocketic-3831212-*`; Agent roots are
`/tmp/ic-backup-agent-pocketic-3902746-*`. Both suites use the newly locked library's
offline asset check and bounded managed lifecycle with private original output.

Final documentation/diff checks and **372** frozen Rust/manifest/lock/toolchain
inputs are recorded in `final-checks.log`; no input drift occurred. Prior native,
public, HTTP and independent Rust 1.91 consumer evidence retains its original
scope; unchanged production graphs/sources were checked explicitly here.
No full CI/release gate, native macOS/all-target MSRV matrix, package acceptance
or installed-IC/live Canic migration was run. These focused results establish no
complete-product, generic application-fence or reference-release authority.

## Preserved dependencies and evidence

Host Artifacts/FS/Process remain **0.73.2**, Platform **0.15.3**, Types **0.15.1**,
Candid **0.10.39**, Agent **0.49.3**, external Principal **0.1.5**, management DTO
**0.11.0**, and Build editor **0.73.1**. Shared revision
`143a9abb9ae81feae9400ff3844b03b1ddfdd75a` remains unchanged: **134** payloads,
committed VERSION **0.27.21**, commit title **0.27.22**. No sibling patch exists.

The preceding handoff is retained as this adoption's `prior-status.md`.
Earlier evidence remains `/tmp/backup-adopt-host-platform.ZA12oo/`,
`/tmp/backup-cleanup-ten-next/` and `/tmp/backup-adopt-upstream.sKsOIs/`.
[Implementation contracts](../implementation-contracts.md), [the Platform boundary](../ic-platform.md)
and [the operator map](../canic-adoption.md#reusable-operator-composition) remain canonical.
Preserve all backup/simulator/independent-build roots, original Canic journals,
`target/msrv-consumers.*`, release fixtures, validation failures and old/failed
bundles. No staging, commits, release, publication, sibling mutation, live IC
operation or artifact cleanup occurred. No disposal is authorized.
