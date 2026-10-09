# Original-journal checkpoint and Shared installer continuation — 2026-10-09

Released 0.11.0 is confirmed at `d13823bcbf52649e3935ba8a48b1540b889174e9`.
The compatible 0.11.1 draft adds `checkpoint_execution_settlement`: derive the
existing checkpoint from the exact retained plan and complete original journals,
then use the existing immutable publisher to recheck every full history before
writing. The existing v1 schema, digest, numeric bounds, spending owner and
publication/replay errors remain unchanged. A separate typed derivation error
admits constructor failures without extending the existing public error enum.

The public learned-stage caller and three isolated Testkit download cases now use
that owner. Removed `pic_journey::planned_download::settle` from
`crates/ic-backup/tests/pic_journey/planned_download/mod.rs`: its manual journal
scan/record assembly is replaced by the library entrypoint. No other function,
method or type is removed; existing exact caller-supplied publication and
identity-bound replay retain distinct recovery contracts. The second sequential
journal scan is deliberately retained as publication admission. Active journal
guards must be dropped before calling; an occupied checkpoint is never replaced.

Focused qualification passes: 18 settlement cases, 17 stage persistence cases,
one public learned-stage case, three actual simulator download cases (success,
lost reply and malformed reply), four local metrics cases, warnings-denied Clippy
for both libraries and changed integration targets, and both independent Rust
1.88 consumers. Failure cases retain original journal bytes/spending; no zero
consumption is supplied for missing evidence. The simulator uses the selected
Testkit CLI's actual offline server admission and managed isolated topology.
No live IC effects or downstream Canic edits were performed.

Canonically refresh the same 92-file Shared snapshot from a clean isolated
committed source at `ee48bb37c98c771e77b92fd891f0757d8c1c8b99` (0.2.2 handoff).
Six selected files change, with exact committed bytes/modes; no pin changes or
vendored in-place edits. The final IC pin row is processed without a newline.
CI installers use exact-path atomic publication: reject a late directory and
replace a late symlink entry without modifying its target. Rejected publication
retains the staged candidate and destination contents; Perl is checked before
setup. Consumer IC fixtures, the exact upstream CI installer companion suite,
actual consumer snapshot/pin/ShellCheck, Testkit offline admission, formatting and
Markdown links all pass. The companion suite exercises the exact shared owner,
not a new local copy or a whole-consumer CI verdict.

The incoming lockfile remains byte-identical: Host 0.9.1, Metrics 0.3.1, Testkit
0.26.0 and tokio-util 0.7.20. No dependency is reselected. Independent consumers
preserve the selected graph without simulator/dev feature unification; Metrics
keeps its instruction-reader feature absent. Preserve earlier qualification and
failed release evidence separately from this graph and source.

Released 0.11.0 Linux CI passes; Intel and Apple Silicon macOS jobs remain queued
in [the exact release-source run](https://github.com/dragginzgame/ic-backup/actions/runs/37922374746).
[Exact Shared source CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37918955655)
also remains queued. These runs do not qualify this uncommitted 0.11.1 candidate.
Issue #32 stays open for both remaining native consumer runs. Issue #29 stays open
for provider-driven full orchestration and fresh application authority/fencing,
complete manifests/transfer, command custody and product terminal proof. Local
all-Applied checkpointing supplies none of those permissions or release authority.

All changes remain uncommitted; manifests and release receipt remain at 0.11.0.
No full CI/release gate, Git write, tag, push or package publication was run.
[Source-bound evidence](qualification.json) records exact inputs, commands and logs.
