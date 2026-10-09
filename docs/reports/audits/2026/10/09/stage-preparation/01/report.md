# Complete original stage-journal preparation

The compatible 0.10.1 draft follows pushed 0.10.0 at
`d909fb2f2ecbcc40e8bea920bd3fa45a0a5ea57b`. Package versions and release metadata
remain 0.10.0. The released lock already selects Host 0.8.8, Metrics 0.2.17 and
Testkit 0.25.3. A concurrent update subsequently selects Metrics 0.2.18 and
Testkit 0.25.4; preserve the incoming graph and keep earlier results separate. Shared Tooling remains the reviewed
0.1.34 snapshot; the agent performs no refresh or dependency selection.

For [#29](https://github.com/dragginzgame/ic-backup/issues/29),
`ExecutionStageGuard::prepare` creates an exact new stage and every original child
journal before returning. Bulk authority derivation precedes allocation. The
existing stage admission and durable create-only journal publisher retain their
own errors and lifetimes; locks remain sequential. Re-admit original stage and
predecessor records after journal publication. The new typed preparation error
preserves stage, authority and journal evidence. No schema, digest, accounting
transition, provider, receipt or effect authority is introduced.

The record/layout-only `create` primitive remains distinct. Repeated preparation
refuses occupied stages. Partial publication and process death retain every
original record; open creates no journal and complete execution admission rejects
missing evidence. Complete original journals permit local reopen after a lost
preparation response without allowance reset or another publication.

The public learned-ID journey and all three existing real Testkit planned-download
cases use this owner. The download caller's manual journal-creation loop retires;
no function, method or type is deleted. Fixture ingress custody, authentic replies,
stopped/no-external-effects safety and explicit receipt qualification remain
integration-owned. This entrypoint does not install a generic runner or qualify
Canic, upload/restore orchestration or terminal/fence/reference release.

Focused evidence is retained in `target/continuation-0101/`. Ten stage cases cover
complete originals, spare headroom, pending reopen, changed bindings, occupied
later journals and acknowledged death after first/last journal publication. The
public learned-ID case exercises prepare, reservation and unchanged pending reopen.
The same fourteen focused cases pass on the preserved incoming graph.
Real simulator success/lost/malformed cases retain the original one-call spending,
partial bytes, references and refusal behavior. Clippy, Rust 1.88 with independent
normal consumers, strict rustdoc, formatting, snapshot and documentation checks
are recorded with their actual results in [qualification](qualification.json).
The initial Clippy failure on private barrier variant names remains retained
separately from corrected verification.

Released 0.10.0 Linux CI passes; both macOS jobs are still queued at inspection.
[#30](https://github.com/dragginzgame/ic-backup/issues/30) and
[#31](https://github.com/dragginzgame/ic-backup/issues/31) retain native acceptance;
this new uncommitted candidate has no matching hosted evidence. Work remains local
and uncommitted, with no sibling edits, root Git writes, release, publication or
live IC effects.
