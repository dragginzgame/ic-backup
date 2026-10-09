# Testkit server ownership cut — 2026-10-09

The pending 0.11.0 draft replaces Shared Tooling's PocketIC selection with the
published, exact locked Testkit CLI. Package versions/receipt remain 0.10.1;
all work is uncommitted. Original stage resume work remains in this draft.

Canonically refresh from clean committed Shared Tooling
`8140e3dd1b44409d682c721889ab702f438c6a17`: 92 selected paths, nine changed and
three retired. The upstream changelog selects 0.2.0; committed VERSION still
says 0.1.38 and exact owner CI is queued. Selected bytes/modes match the source.

`make install-testkit-server` selects the unique locked Testkit version from Cargo
metadata, delegates exact CLI installation/receipt admission to the shared Cargo
installer and invokes Testkit setup. `make testkit-server-check` checks that same
CLI and invokes Testkit's offline check, printing its admitted absolute server
path. Both simulator suites use it before managed startup. No duplicate server
catalog, version equality, bundle-receipt projection, cache search or old-route
fallback remains. Product topology, private output, digest evidence and original
journals stay local. CI prepares/checks the route on all three declared hosts.

Actual Linux Testkit setup/check and shared five-tool reinstallation pass. All 21
files in the previous six-tool bundle remain byte-identical. The actual offline
missing-server check refuses and creates no directory. Synthetic Make fixtures
separately qualify setup/check delegation, absent/ambiguous locked packages and
CLI/server refusal without setup or fallback. Canonical shared IC fixtures cover
old-bundle rejection, explicit activation and evidence retention.

A concurrent incoming direct Host 0.9 selection changed the lock while the first
results were being recorded. Preserve it, bind the final manifest/lock/source
hashes, and rerun affected checks rather than relabel earlier logs. The public
`PersistenceError::Publication` carries the new Host 0.9 Rust identity. Both
published archives and 40 Rust/original-manifest files match committed/cached
source. Host's Rust API implementations are unchanged by its tooling minor;
no new runtime offload or local compatibility adapter is introduced. Product
Host 0.9 and Testkit's published dev-only Host 0.8 graph remain distinct.

On the final graph, thirteen actual core snapshot/download/upload/restore journeys,
three Agent gateway journeys and seventeen original-stage cases pass. Lost and
malformed replies, wrong-root refusal, pending spending and retained state keep
their original assertions. Both-library Clippy and Rust 1.88, including independent
normal consumers, pass. Snapshot/pin/ShellCheck and consumer tooling fixtures pass.
These are isolated Linux effects, not production IC, full runners or application
safety proof. Metrics 0.2.20 and Testkit 0.25.5 remain selected.

Initial setup appended a basename to the installer's returned executable path;
corrected caller passes. The fixture's inline trap failed ShellCheck; scoped
cleanup passes. Agent compilation caught a removed Path import still used by an
evidence helper; the first replacement missed its compacted line, and exact
restoration passes. Failed/inconclusive attempts remain separately retained.

Removed functions: `usage` in `scripts/ci/check-pocketic-alignment.sh` and
`expect_failure` in `scripts/ci/test-pocketic-checks.sh`. Testkit CLI check replaces
the retired alignment interface; owner/shared provisioning tests and local
command/refusal fixtures replace the duplicate test. `check-pocketic-binary.sh`
contains no function. No Rust function/method/type is removed. No retained effect
record or schema is replaced or discarded.

[#32](https://github.com/dragginzgame/ic-backup/issues/32) remains open until exact
committed consumer Linux, Intel and Apple Silicon CI passes. Testkit owner
acceptance is separate. No full CI/release gate or root Git write ran.
[The evidence record](qualification.json) binds inputs, commands, failed attempts,
retained bundles and source hashes. Prior dependency checks stay in
[their original review](../../updates-0102/01/report.md).
