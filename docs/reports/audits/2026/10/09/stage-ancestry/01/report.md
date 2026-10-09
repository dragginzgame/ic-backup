# Complete ancestor-history stage admission

Continue the compatible 0.10.1 draft at released 0.10.0 HEAD
`d909fb2f2ecbcc40e8bea920bd3fa45a0a5ea57b`. Local stage admission checked direct
predecessors only: an intact settled intermediate stage could hide a changed or
missing earlier journal. Two fresh regressions reproduced erroneous descendant
creation and retained-layout access before repair. Retain both failures separately.

The existing `validate_predecessors` now traverses the original ancestor DAG
iteratively. Admit each distinct stage's exact binding/plan and complete chronological
Applied settlement through the existing journal reader. Retain at most one expected
row per bounded catalog node; shared edges must agree on binding/settlement identity
while their learned-input evidence remains edge-specific. Release each ancestor
layout and all journal locks before the next admission. Sequential local checks
do not establish atomic evidence or noncooperating custody.

All four stage entrypoints share this admission. Changed/missing ancestral evidence
rejects new allocation, reopen and retained-layout access without repair or extra
spending. The real planned-download driver additionally uses complete preparation
for capture and metadata, removing their manual journal creation. No function,
method or type is removed; records, digests, public APIs and original allowances stay
unchanged. No runtime application permission or terminal/release authority appears.

Thirteen focused stage cases, one public consumer journey and three actual Testkit
planned-download cases pass on the preserved incoming Host 0.8.9, Metrics 0.2.18 and
Testkit 0.25.4 graph. Both-library warnings-denied Clippy passes. The initial offline
attempt lacked Host 0.8.9 cache entries; locked cache preparation preserves selection.
The first simulator attempt failed before fixture ingress because the sandbox denied
localhost binding; the permitted loopback run passes. Logs, source identities and
remaining checks are retained in [qualification](qualification.json) and
`target/stage-ancestry-0101/`.

[#29](https://github.com/dragginzgame/ic-backup/issues/29) remains open for installed
orchestration, fresh application admission, complete upload/restore and terminal
qualification. This uncommitted candidate has no matching native CI. Package versions
and receipt remain 0.10.0; no sibling edit, Git write, release, registry publication
or production IC effect occurs. Earlier batch evidence keeps its original graph.
