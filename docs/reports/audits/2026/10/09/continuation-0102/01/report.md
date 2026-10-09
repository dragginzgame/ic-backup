# Complete stage resume and Shared Tooling 0.1.37

Released 0.10.1 is confirmed locally and on GitHub main at
`5c4bd9df50d2cf4f954a236ab9b155778b33d645`. Select compatible 0.10.2 notes; versions
and receipt remain 0.10.1. The released graph selects Host 0.8.9, Metrics 0.2.18
and Testkit 0.25.4 without dependency changes in this batch.

`ExecutionStageGuard::resume` joins exact retained stage/ancestor admission to the
existing complete original journal-progress reader, then re-admits stage/ancestor
records before returning the guard and canonical view. The new typed resume error
retains original stage/progress failures. Missing, changed or held evidence refuses
without creation or allowance reset. Resume reads local records only: artifact
verification and provider calls remain distinct. Record-only `open` keeps its
incomplete-preparation inspection purpose. No schema, digest, spending owner,
fresh permission, atomic custody, installed runner or terminal/release proof appears.

The public learned-ID consumer and actual Testkit success/lost/malformed download
recovery use complete resume. Four fresh cases cover pending original bytes and
unassigned headroom, artifact-free replay, missing/changed/held child journals and
changed ancestor history. Seventeen stage cases, one public journey and three
actual simulator cases pass, with original calls, journals and references retained.
Both-library Clippy and Rust 1.88 checks pass. Remaining focused results and exact
source/graph/log hashes are retained in [qualification](qualification.json) and
`target/continuation-0102/`. No function, method or type is removed.

Canonically refresh the unchanged 95-file selection from committed Shared Tooling
0.1.35 to 0.1.37 `dc4fdf0f78928d75b69bbf43b37c690c53a04d1e`. Seven selected files
change; every byte and executable mode matches a clean isolated committed checkout.
Installer fixtures use substitute Cargo to qualify receipt-stream refusal,
post-build path admission and original failure retention. Local hook/adoption
fixtures retain real pinned sorting/Rust formatting with a Cargo dispatch wrapper
for the checkout-local tool case. Missing/wrong pins preserve the private index and
unrelated edits; prepared tools are not staged. No registry tool install occurs.
Exact 0.1.37 upstream CI is queued at inspection, separate from local qualification.

Close [#30](https://github.com/dragginzgame/ic-backup/issues/30) and
[#31](https://github.com/dragginzgame/ic-backup/issues/31) using released 0.10.0
`d909fb2f2ecbcc40e8bea920bd3fa45a0a5ea57b`'s completed native
[tag CI](https://github.com/dragginzgame/ic-backup/actions/runs/37902207429): Linux,
Intel macOS and Apple Silicon all pass. Its main run still has Intel in progress;
new 0.10.1 runs are queued. These are separate evidence owners. Correct stale README
claims about transport; the public repository description already matches implementation.
Keep #25/#29 open for their broader acceptance. This uncommitted candidate has no
matching hosted CI, full gate, root Git write, sibling edit, release, registry
publication or production IC effect.
