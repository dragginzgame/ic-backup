# Complete original snapshot data-upload coordination

The compatible 0.14.1 draft implements complete bounded data-upload planning and
fresh-stage coordination under [#29](https://github.com/dragginzgame/ic-backup/issues/29).
Released base and package version remain 0.14.0 at
`eaefe8b0888759603cd19acaf94eabd039d7499a`. Changes remain uncommitted.

Guarded source preparation counts exact complete extents against the original
stage ceiling before source IO, then retains canonical request digests while
buffering one payload at a time. Download and upload reuse one extent declaration
owner. The ephemeral data plan binds exact metadata/source checksum/destination,
the original singleton allocation reply/predecessor and unchanged allowances.
Each ordered operation receives one update and zero observations; spare headroom
stays unassigned. Source network/release remain exact, with independently qualified
current caller permissions. Empty regions need no placeholder; known empty chunks
still require their hash-bound write.

The public coordinator admits the complete unused original journal set, freshly
prepares each payload and delegates reservation, mandatory fresh admission and
one provider invocation to the existing single-upload owner. Independent
qualification must durably retain original request/reply bytes before returning
an exact Applied receipt. Only retained Applied prerequisites admit successors.
After all writes, the existing stage checkpoint binds full original histories
and ordered canonical reply evidence. No record schema or existing digest changes.

Actual Testkit upload journeys delegate complete data assembly to these APIs and
independently compare destination metadata and bytes. Eight focused journeys cover
complete upload, lost/malformed metadata, lost/malformed first and second writes,
and insufficient allowance before any data dispatch. Local custody cases cover
qualification refusal, wrong receipts, admission refusal, source drift after the
first receipt, missing journals and occupied checkpoints. Reentry never reissues
a consumed stage; all original source bytes, pending spending, returned evidence
and unfinished references remain retained.

An incoming lock update selected Host Artifact/FS 0.12.1 during the first gate;
that run was stopped and retained. Published Rust source trees match 0.12.0;
release tag `v0.12.1` resolves to `e5ecfa06c14d144cfeb85ea89d65906b1bf81636`.
The final graph retains Testkit 0.31.0 with independent test-only Host 0.11,
Metrics 0.5.0, Agent 0.49.2 and the reviewed Shared Tooling 0.3.0 snapshot.
No sibling patch or sibling edit follows. The incoming final lock is preserved.

All documented delivery gates pass. The final-graph `make ci` passed unchanged
snapshot/tool/pin/link/fetch/shell/tooling/release/hook checks before a new caller
test failed: its metadata exceeded its declared allowance. After bounding that
fixture, all affected and remaining gates pass through
`make ci CI_TARGETS='fmt-check check clippy test doc check-msrv package'`.
This includes 500 core unit cases, 25 core simulator journeys, eight Agent HTTP
cases, three Agent simulator journeys, warning-denied Clippy/docs, both MSRV
consumers and both packages. An additional independent Rust 1.88 consumer compiles
the new complete public API with its locked Host 0.12.1 graph. Initial fixture,
Clippy and sandbox server-bind failures remain separately retained.

[qualification.json](qualification.json) binds technical inputs, final graph,
CLI identity, archive and retained log hashes. Evidence remains under
`target/continuation-0141/`. Documentation completion receives narrow snapshot,
link and whitespace checks without repeating unchanged delivery gates.

Released 0.14.0 [native CI](https://github.com/dragginzgame/ic-backup/actions/runs/38048572026)
passes Linux; both macOS jobs remain queued. The uncommitted candidate has local
Linux qualification only. #29 remains open for same-ID restore orchestration;
#25 retains async Agent/application integration. Authentic complete backend
transfer, command/byte custody, load/start safety and terminal/fence/reference
release remain independently qualified. No partial-upload resume, hidden
observations/reissue, default provider, release, registry publication, production
IC effect, root Git write or recovery cleanup is introduced.
