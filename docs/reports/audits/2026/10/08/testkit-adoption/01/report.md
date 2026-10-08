# Published Testkit adoption and qualified simulator pins

Released base is IC Backup 0.8.1 at
`8d1656e7c6c9c29032ad063fe351fc040d316252`. The maintainer requested Testkit
instead of a direct PocketIC dependency. This batch remains uncommitted; package
versions and release receipt stay 0.8.1. The changelog selects 0.9.0 because the
incoming Host 0.8 graph changes the exposed publication-error Rust type identity.

Both library dev dependencies and fixture imports now use published `ic-testkit`
0.25.1. It supplies the complete PocketIC 16.1 API and explicit managed startup
with a 60-second readiness deadline. Each original fixture retains admitted
server checksums and private raw stdout/stderr under its caller-owned root.
Instance teardown precedes managed-server cleanup. Exact application calls,
original reservations, reconciliation, byte checks and fixture safety remain in
Backup. No Testkit snapshot retry, baseline reset, funding policy, new production
process runner or spending owner is adopted. No function, method or type is removed.

Preserve the incoming Host artifacts/FS 0.8, Metrics 0.2.14 and PocketIC 16.1
selection. Testkit 0.25.1 joins a single Host 0.8 graph, with process/tools dev-only.
[Published source proof](published-source-proof.json) verifies 151 Rust/original
manifest blobs against non-yanked registry archives and exact official committed
source. The final concurrent update selects all four Host packages at 0.8.1 and
Testkit at 0.25.1. Testkit/Metrics Rust bytes are unchanged. Host improves bounded
buffer allocation, adds an immediate path-lock API and resource-report example;
Backup does not select those new APIs. Fresh qualification preserves its original
record/bounds/publication/spending contracts. Earlier Host 0.8.0 / Testkit 0.25.0
inputs, logs, source proof and packages remain under
`target/testkit-adoption-090/qualified-0250`, separate from final patch evidence. Direct Host production declarations still disable
default features; Testkit enables archive/Wasm and Candid-extraction features in
the full dev graph. Their exact feature selections are recorded, without claiming
that the development graph has empty features.

The shared guide explicitly delegates qualified consumer pin selection outside
the immutable snapshot. Transfer only `ci/ic-tools.tsv` to consumer ownership;
[shared source proof](snapshot-source-proof.json) retains the exact other 79
source bytes/modes at `db039347d2372b877c1c46dcdd2b5c3aa9412009`. No vendored
helper or common rule is patched. The one matrix pins all three official PocketIC
16.1 archive hashes; other tool rows and shared install/check behavior stay the
same. Explicit Linux preparation and offline full-bundle hash/version admission
precede simulator tests. [Shared #76](https://github.com/dragginzgame/shared-tooling/issues/76)
owns the upstream default update. macOS asset declarations are not native execution
qualification. The immutable guide's default 16.0 table keeps its source meaning;
[consumer guidance](../../../../../../../shared-tooling.md#consumer-owned-ic-pins)
owns this reviewed 16.1 override.

All 158 focused Rust cases pass on the final graph: five Agent HTTP/gateway,
ten core simulator, 134 persistence and nine public recovery/source/settlement
cases. Both-library all-target/all-feature Clippy, Rust 1.91, strict rustdoc and
Cargo package verification pass. Full repository tooling, shell, formatting,
dependency declarations, documentation, snapshot and installed IC admission checks
pass locally on Linux. [Qualification](qualification.json) binds exact inputs,
commands and retained logs. These are focused checks, not a full CI/release gate.

The first full tooling run failed because its local isolated consumer copied only
snapshot paths and omitted the now consumer-owned matrix required by the closed
governance roster. Copy that explicit input in the local fixture; the corrected
full tooling target passes. Original failure and retained fixture remain separate.
An early documentation check ran before this report was written and failed on its
missing links; the corrected completed-document check is recorded separately.
A one-off preparation script initially guessed a reversed official asset filename;
it refused before matrix writes. The actual release JSON supplies the corrected
mapping. The original tool transcript retains that assertion, not a passed attempt.
The final matrix provenance comment also changed its exact installer receipt;
offline admission correctly refused that mismatch. Explicit re-preparation creates
a fresh bundle for those full matrix bytes while preserving identical tool versions
and server bytes, plus the prior complete bundle. The final offline retry passes.

Preliminary Testkit 0.24 / PocketIC 16.0 checks and package archives remain under
`target/testkit-adoption-090/preliminary-024`, bound to their original inputs.
They are not relabelled as final 0.25.1/16.1 evidence. Old simulator bundles, journals,
artifacts and incomplete recovery evidence remain retained.

Exact Testkit 0.25.1 and Host 0.8.1 owner checks are in progress/queued or
cancelled at the last inspection; Metrics exact owner CI passed. Prior Host
0.8.0 CI failed on both macOS hosts because its cleanup fixture invokes missing
`/bin/true`. Host 0.8.1 retains that fixture; upstream #5 owns the report. This
fixture path failure does not establish a production cleanup defect. None qualifies this uncommitted consumer on native macOS.
[Transport acceptance #25](https://github.com/dragginzgame/ic-backup/issues/25)
and [workflow/stage binding #29](https://github.com/dragginzgame/ic-backup/issues/29)
retain their separate acceptance. Testkit manages the local simulator; direct
Rust Agent remains the sole product transport. Generic application/fence safety,
complete workflows, downstream Canic adoption and terminal/fence/reference release
remain unfinished. No sibling files, public IC effects, Git writes, releases or
publication are performed.
