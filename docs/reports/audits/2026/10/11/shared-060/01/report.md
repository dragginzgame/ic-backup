# Shared Tooling 0.6.0 consumer adoption

Adopt committed `75180980a0c4e4252d8a915aaecfaefd92bf9a4e` through clean canonical
preview/export, adding the required Cargo-build companion (99 selected paths).
Released Backup remains 0.18.2 at `e2e19b168630cdc808a7477c4a50b18e9b6222d4`;
compatible pending notes select 0.18.3. Sixteen selected files change from 0.5.0.

Common/selected Cargo tools reuse exact shared generations under standard compiler,
native environment, source and packaged-lock identities. Application flags and
wrappers stay local; command paths remain. Testkit keeps server storage/lifecycle.
Explicit complete setup and offline admission pass. Old generations and failed
attempts remain retained; no store, compatibility shim or sibling PATH is introduced.
Actual Linux host/IC/common/selected Cargo reuse passes offline for two consumers
with different application flags. CI rejects missing executable identities.

Twelve actual workflow/action full/compact collector cases retain local records,
shared Rust payload bytes, original rejection diagnostics and incomplete status;
bundle deduplication and external-link/unrelated-store exclusion pass. The canonical
selector owns local Rust evidence and referenced shared generations. Local logs
are archived first; safe partial selections are archived before original failure
propagation. The shared action's completeness step also refuses after retention.
These are local archive round trips, not hosted upload results.

All 16 delivery targets pass on frozen Host 0.13.2, Metrics 0.6.3, Testkit 0.33.6,
PocketIC 16.1.0 and Agent 0.49.2, including strict Clippy, actual simulator tests,
Rust 1.88 consumers and paired current archives. Both incoming patches' Rust
sources match their predecessors. Final CI assertion changes reuse unaffected
full-gate evidence and receive focused qualification. Real index/HEAD stay intact.

Known limitation: the common installer suppresses failures under a held shared
lock, prints success and can leave an incomplete selection. Actual native setup
and an isolated substitute-compiler fixture reproduce it; retained evidence and
the explicit propagation proposal are on
[#120](https://github.com/dragginzgame/shared-tooling/issues/120#issuecomment-6108794633).
Do not claim fail-closed common setup under contention. Preserve offline admission;
no vendored patch or lock removal is applied. Matching macOS qualification remains
pending under [#36](https://github.com/dragginzgame/ic-backup/issues/36).

No package-version, Git/release, production IC, sibling edit or recovery cleanup
occurs. See [the compact proof](qualification.json). Detailed source/graph/case/log
inventories remain under ignored `target/shared-060`.
