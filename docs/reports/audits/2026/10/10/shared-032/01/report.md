# Shared Tooling 0.3.2 and Host 0.12.2 adoption

The compatible 0.14.1 draft selects committed Shared Tooling 0.3.2,
`c16444bf006f17c5bb4dda5ad070a0f345da9623`, verified against GitHub main.
Canonical export uses an isolated clean checkout, preserves product work and
binds 96 files, adding the README freshness task linked by the maintenance catalog.
The common baseline bytes remain unchanged. Older evidence remains retained.

Shared setup refuses unsupported platforms or unavailable Rust/Cargo before
downloads. Offline diagnostics identify the exact expected tool, selected path
and repair command. Formatting and Cargo tool recipes preserve Make jobserver
descriptors; standalone tool includes also refuse unsafe execution modes.
Fixtures require explicit completion before success or cleanup, including Bash
3.2's premature zero-status exits. README freshness is advisory, with no new
scheduler or delivery gate.

Local Cargo/Testkit/release recipes preserve jobserver access through their
existing execution guard. Local Testkit, release and hook fixtures enforce
completion at top-level and applicable individual-case boundaries. Parallel
consumer routing verifies live descriptors and refusal before download or
product setup. Root README setup uses the existing complete install owner.

Host 0.12.2 release is `e1ef99e6a4c6d05f0b0d8364f8586c6cc358dadc`.
All four Host release Rust source trees are unchanged. Published Artifact/FS
sources match 0.12.1, which matches 0.12.0; no new API
or byte owner can replace local backup contracts. Preserve the incoming 0.12.2
lock selection and independent Testkit 0.31/Host 0.11 graph. The incoming Metrics
0.5.1 selection is explicitly fetched before offline qualification. Root package
version remains released 0.14.0; no release/version transaction is performed.

The complete final `make ci` suite passes: snapshot/tool/pin/link/fetch/shell,
tooling/release/hook/formatter checks, native compilation, strict Clippy, all
500 core unit cases, 25 core simulator journeys, eight Agent HTTP cases and
three Agent simulator journeys, warning-denied docs, both independent Rust 1.88
consumers and both packages. A separate locked Rust 1.88 consumer also compiles
the complete new public data-upload API on this final graph. Injected premature
zero exits in actual final local fixture copies refuse and retain evidence;
these Linux tests do not establish native Bash 3.2/macOS acceptance.

[qualification.json](qualification.json) binds exact technical inputs, selected
graph, archives and retained log hashes. Source inputs and final lock remain
unchanged after validation. Documentation completion uses narrow snapshot,
parallel offline tool admission, link and whitespace checks.
Retain evidence under `target/shared-032-review/`, including offline cache
refusal, the early missing-report-link failure and interrupted fixture review.
Released source native acceptance remains distinct from this uncommitted local
candidate. No sibling edit, root Git write, registry upload, production IC effect
or recovery cleanup follows.
