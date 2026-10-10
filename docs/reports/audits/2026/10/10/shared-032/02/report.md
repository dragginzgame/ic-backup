# Final Testkit 0.32 consumer graph

The incoming root manifest/lock changed to Testkit 0.32.0 after the completed
[Shared/Host review](../01/report.md). Preserve that earlier proof for its exact
Testkit 0.31/independent Host 0.11 graph. The new selection joins Testkit's four
test-only Host crates with direct Host Artifact/FS on 0.12.2. Metrics 0.5.1,
Agent 0.49.2, PocketIC 16.1.0 and Shared Tooling 0.3.2 remain selected.
Package/receipt stay 0.14.0; the compatible draft remains 0.14.1.

Published Testkit Rust source is unchanged from 0.31.0. Archive VCS evidence and
GitHub main identify release `1502f667bc6b7f54067dd57d1841cd0a150110a7`.
The canonical explicit installer prepares the exact 0.32 CLI and admits the
existing server bytes. Earlier CLI selections, receipts and failure evidence
remain retained. Missing selection refusal remains an explicit offline failure,
with no fallback or installation during ordinary checks.

All affected gates pass on this final graph through `make ci` with the explicit
graph/tool/pin/link/fetch/format/build/Clippy/test/doc/MSRV/package target list.
Reuse unchanged passing shell/tooling/release/hook evidence from the prior full
suite. This includes 500 core unit cases, 25 core simulator journeys, eight Agent
HTTP cases, three Agent simulator journeys, warning-denied docs, both independent
Rust 1.88 consumers and both packages. Normal consumers retain no Testkit or
PocketIC dependency/features. The prior standalone public data-upload API proof
retains its unchanged normal Host 0.12.2/Metrics 0.5.1 graph.

[qualification.json](qualification.json) binds exact final inputs, graph, archive,
CLI selection and retained log hashes. Incoming manifest/lock and validated
technical inputs remain unchanged. Documentation completion receives narrow
snapshot/link/whitespace checks. No production
record, spending or IC permission changes follow; matching local simulator
qualification remains separate from native macOS acceptance and full runners.
No root Git write, release, registry upload, sibling edit or recovery cleanup ran.
