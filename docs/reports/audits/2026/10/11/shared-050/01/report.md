# Shared Tooling 0.5.0 consumer adoption

Adopt committed `a306180ce534227fcb98b882fba9a526d5d747e0` through clean canonical
preview/export, including both required store companions (98 selected paths).
Exclude the sibling's dirty 0.5.1 work. Released Backup remains 0.18.1 at
`7e9eaf2da8a6e89ba790210d895effc1a9b0898d`; compatible pending notes select 0.18.2.

Host/IC setup reuses authenticated per-user bundles with independent pins and
unchanged `.tools` command paths. Contributors rerun explicit setup and offline
checks. Common Cargo tools use `.tools/rust/common/bin`; their slots and Testkit
storage remain local. Original local bundles and failed evidence remain retained.
Actual Linux setup/admission and two-checkout native reuse pass with the second
consumer's downloads disabled. CI isolates one canonical store per native job.

Eight actual workflow/shared-action collector cases check original bytes, bundle
deduplication, unrelated-store exclusion and link boundaries. Local CI archives
logs before shared selection, retaining uploadable diagnostics and original failure
status on rejected paths. The unchanged shared action loses its archive output in
those rejection cases; [upstream #121](https://github.com/dragginzgame/shared-tooling/issues/121)
owns that separate fix and full/compact/native acceptance. These are local archive
round trips, not hosted upload results. Workflow lint passes.

All 16 delivery targets pass on the frozen initial Host 0.13.1 graph. After an
external lock refresh, the user paused updates; all ten affected graph gates pass
on Host 0.13.2, Metrics 0.6.2, Testkit 0.33.5 and PocketIC 16.1.0. They include
strict Clippy, actual simulator journeys, Rust 1.88 independent consumers and
exact paired-package builds. Unchanged tooling/release/hook proof is reused.
Host 0.13.2 and Testkit 0.33.5 Rust sources match their previous patches; Metrics
0.6.2 adds an exact checked-u64 rank branch with the existing wide fallback.
No measured speed claim follows. Technical inputs and index stay unchanged within
each passing phase; source/graph/case inventories remain ignored.

Released 0.18.1 CI passes Linux and Apple Silicon; Intel macOS is still running.
The uncommitted adoption has no hosted result; matching macOS acceptance remains
pending under [#36](https://github.com/dragginzgame/ic-backup/issues/36).
No package-version, Git/release, production IC, sibling edit or recovery cleanup
occurs. See [the compact proof](qualification.json); detailed evidence remains
under `target/shared-050`.
