# Shared Tooling 0.1.32 and Host 0.8.5 follow-up

The maintainer selected the existing compatible 0.9.2 draft, correcting an initial
0.9.3 reference. Release/package base stays 0.9.1; prior dirty implementation,
incoming Host 0.8.5/Metrics 0.2.16 selections and all retained recovery evidence remain.

Canonically adopt committed Shared Tooling 0.1.32
[`635a39a9dd5f8d021fa9c9196b591e00521a7e02`](https://github.com/dragginzgame/shared-tooling/tree/635a39a9dd5f8d021fa9c9196b591e00521a7e02)
from a clean isolated checkout. Eight files change in the unchanged 97-file
selection; [all source bytes/modes](shared-source-proof.json) match committed
blobs. The sibling's dirty 0.1.33 dashboard proposal is excluded.

The practical change is [shared #79](https://github.com/dragginzgame/shared-tooling/issues/79):
IC installation reuse compares complete validated tool/version/host/checksum rows,
so comments and order do not trigger downloads or rewrite installation receipts.
Changed records across any host, malformed/duplicate matrices, changed bytes and
unsafe links still reject. Focused fixtures cover preservation/refusal through the
shared installer; real offline checks of this checkout's selected bundle accept
commented and reordered copies with [unchanged link, pins and receipts](real-offline-reuse.json).
ShellCheck, dependency declarations, formatting, local documentation links and
locked PocketIC alignment pass. No actual bundle is installed,
replaced, removed or compacted by this review.

The baseline now places fleet reports in Shared Tooling and reusable arithmetic
in Metrics; no sibling dashboard or extra Metrics API/feature is appropriate here.
The upstream Cargo-install assessment remains qualification of a proposed broader
setup contract. Its scripts/workflow are not dependencies of our selected surface
and remain unselected. Our consumer CI already has no cancellation group, preserving
each pushed source. [Exact upstream CI](upstream-ci.json) passes on Linux and both
macOS architectures; this does not qualify the uncommitted consumer candidate.

All four official non-yanked Host 0.8.5 rows and archives match their
[registry checksums](registry-proof.json), clean release VCS identity and
[81 committed Rust/package files](host-085-proof.json). The complete `crates/`
trees are identical to released 0.8.4; Host's release carries the same tooling
reuse change, supplying no additional runtime code to offload. The incoming lock already selects 0.8.5, so preserve it and qualify that actual
graph; the agent performs no dependency reselection. Earlier 0.8.4 tests do not
stand in for this selected graph.
Three real planned-download Testkit cases pass on Host 0.8.5: complete durable
publication, lost reply and malformed reply, retaining original pending spending
and partial bytes without follow-up. Exact simulator roots/file hashes are recorded
separately from prior 0.8.4 evidence. Forty-two focused JSON publication/read, inherited descriptor custody, attempt
journal, stage/planner and public stage cases pass on the selected graph. Both
libraries pass warnings-denied Clippy/rustdoc and Rust 1.88 with independent normal
consumers. Existing publication errors, private IO, spending and command custody retain
exactly their previous owners. Host Process/Tools stay Testkit-only transitive
inputs, not a restored command backend.

Latest published Testkit is still 0.25.3. Its provisioning
[handoff #38](https://github.com/dragginzgame/ic-testkit/issues/38) remains open;
the published CLI lacks the proposed setup/check replacement. The adopted guide
requires its publication and native qualification before coordinated retirement
of the current six-tool setup. Preserve selected pins, explicit installation,
server alignment and prior bundles. This review creates no new provisioning path.

[Qualification](qualification.json) records exact inputs and retained commands/log
hashes. Work stays local and uncommitted: no Rust change, lock reselection,
sibling edit, Git/release transaction, publication, schedule or live IC effect.
