# Metadata-stage coordination and Shared formatting adoption

Released base: `f1bba34b667a724274a65ac9653b4de88f30414f` (0.12.0).
Select compatible 0.12.1: the new coordinator is additive and the shared change
only condenses human formatting diagnostics. Package versions/receipt remain 0.12.0.

## Product owner

`workflow::ic_snapshot_metadata::read_snapshot_metadata` accepts an exact singleton
original metadata stage. Reuse the canonical single-read owner for durable original
reservation, mandatory fresh admission and one provider call. Under the selected
journal lock, re-admit the reply and require explicit integration qualification of
actual attribution/authentication and durable original request/reply retention.
Record only its exact Applied receipt; release the lock before canonical checkpoint
publication. Learned evidence binds the existing metadata request/raw-reply digest.

Return original response/predecessor for the existing decoder/download-plan binding.
Successor stage/writer creation stays explicit. Qualification/receipt/checkpoint errors
retain bounded replies and original spending, including Applied history after failed
checkpoint publication. Pending/Applied reads reject repetition; explicit original
checkpoint recovery never calls a provider. No new schema, byte store, allowance,
default async Agent bridge or terminal/fence/reference release is introduced.

The actual stopped Testkit download caller delegates metadata receipt/checkpoint
coordination to this public API. Lost/malformed metadata cases retain pending original
journals/references across reopen, with no data stage or reissue. The complete download
and capture/data safe-stop cases continue to pass. Full upload/restore orchestration,
application-qualified async Agent wiring and product manifest/terminal/custody proof
remain with [#29](https://github.com/dragginzgame/ic-backup/issues/29).
No Rust function, method or type was removed; caller-side manual assembly was replaced.

## Shared owner

Canonical export from clean committed Shared Tooling 0.2.10
`43a0dc46cdc3c77e70a68e192561642ed50a3e0f` adds the explicitly selected formatting
reporter (95 files). Include it in every local hook/release/Testkit fixture.
Keep success to one line; retain complete stdout/stderr and status on failure,
prepared tool admission, sorter-before-rustfmt ordering and default Make help.
Preserve the specialized post-run release-resume check. CI collects `formatting.*`
logs alongside existing evidence; an executed copy of the actual collector verifies
both formatting streams and existing validation bytes in the retained archive.

Actual hook/index/partial-stage preservation and output/refusal cases, Testkit routing,
release adapters, formatter/snapshot checks, ShellCheck and workflow lint pass.
Source/version annotation, shared evidence/guidance and exact file modes are reviewed;
no uncommitted sibling source, unused registry observer or tool pin is adopted.

## Qualification and limits

[qualification.json](qualification.json) binds selected inputs, source files and
retained logs under `target/continuation-0121/`. All 58 workflow/stage unit cases
(including nine new metadata cases), seven actual core planned simulator journeys,
four metrics cases, strict core tests Clippy/rustdoc and both independent Rust 1.88
consumers pass.
A separate Rust 1.88 consumer compiles the new public coordinator with explicit
provider/admission/qualification parameters. Initial test-only Clippy failures are
retained separately; corrected final source passes.

Preserve the incoming lock (including its pre-existing cc/syn/smallvec updates).
The tested graph uses Host 0.10.1, Testkit 0.28.0, Metrics 0.3.6 and Agent 0.49.2.
No dependency reselect, broad local gate, root Git write, release or production IC
operation occurred. Hosted CI retries concern released 0.12.0, not this dirty draft;
current native evidence/next actions are recorded in the handoff and owning issues.
