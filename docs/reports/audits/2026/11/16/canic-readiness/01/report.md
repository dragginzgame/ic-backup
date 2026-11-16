# Code audit and Canic integration readiness

Date: 2026-11-16. Requested scope: audit Backup's code and determine whether
Canic integration can start. This is an audit, not authorization to repair,
commit, publish, or modify Canic.

Follow-up, 2026-11-16: the maintainer subsequently authorized fixing finding 1.
The local change after `747768b6fe50fc59a1246fb4be6d3eedeef501b4` uses the shared
request ceiling in lookup admission. A new native regression reproduces the old
`Bounds` refusal before correction; all six lookup tests then pass, and the
actual-IC interrupted-allocation case passes under the 2 MiB profile with exact
one-observation spending and offline replay. Evidence is retained under
`target/evidence/lookup-profile-fix/`. The correction is included in the
maintainer-authorized GitHub delivery batch, but remains unpublished on crates.io;
0.102.1 remains affected. Original findings and failed evidence below
remain historical; no original reservation or approval was reset.

## Verdict

**Ready to begin coordinated Canic integration work; fix finding 1 before pinning
Backup for the integration.** There is enough implemented and qualified capture,
restore, custody and replay behavior to build against. Another general cleanup or
framework redesign is not a prerequisite.

**FAIL for recovery across all admitted request profiles:** a fresh actual-IC
probe demonstrates that the documented 2 MiB restore request profile cannot
perform its original-result lookup. **BLOCKED for a production Canic-managed
round trip:** manager execution, bounded real signing and fenced application
recovery remain unimplemented/unqualified integration boundaries. These require
coordinated work in Backup and Canic; signer/guard adapters alone are insufficient.

The existing release tests remain valid for their exercised cases. They missed
the combination of a larger request ceiling and interrupted-restore lookup.
No second production correctness defect was demonstrated. This is not an
exhaustive security, supplier-cryptography or physical power-loss proof.

## Source, method and evidence

- Backup: clean `18d511b636e8919c640501cfd2926c32dc66e7e5`, version 0.102.1.
  Product source equals release `1db82ba22ac15c761906af0935e0e918acf40801`;
  the intervening commit changes only the handoff. This audit adds this report
  and updates the handoff; product, locks and qualification sources stay unchanged.
- Local overlay: `AGENTS.md` blob `20748d49875ffb50e27410153999d7e0e998993d`.
  Authorities: restart, operation/local-store, capture/transfer/inventory contracts
  and sibling integration review. The prior explicit publication authorization
  and current local overlay admit the experimental direct-controller release.
- Toko methods at `626da7cd9626cc2e152594862a2d82ea82ef51da`:
  `audits/README.md`, `code-hygiene.md`, `flow-convergence-and-duplication.md`,
  `ic-canister-applications.md`; referenced Rust hygiene and simplicity principles.
  This is a different scope from the earlier publication-readiness audit.
- Canic: committed `2ee4add03126aa2f5259c609fa05eae8a14bcd14`, also observed
  on remote main. Its dirty working tree was left untouched. Findings use
  committed signer/manager/design code; pending fleet/maintenance work is not
  treated as delivered integration. Existing issues #1, #5, #6 and #7 were read.
- Legacy: read-only `4ae087fad8c23de712aa60bf569bde4b2b932651` in
  `/tmp/ic-backup-legacy-loc-20261115`. Focused sources/tests/history below were
  inspected; legacy suites were not rerun and no legacy source/data was imported.
- Execution host: Linux x86-64; Rust 1.100.0,
  `0ef1bc386059596dada8fb65b50425aab2ea637a`. One focused actual-IC failure probe
  ran with locked/offline dependencies and the admitted Testkit server.
- Reused release evidence: `target/evidence/release-0.102.1/publication-corrected.log`
  records 232 native tests, five documentation tests and eleven actual-IC tests,
  plus strict lints, formatting, documentation, package and caller checks.
  `registry-verified.json` binds the archive to the release;
  `registry-consumer/replay.json` records successful completed capture/restore
  replay through the registry dependency. No broad suite or release gate was
  rerun for this audit. Hosted CI was not polled.
- Audit documentation: installed-tooling admission, local documentation links
  and whitespace checks pass. These checks do not qualify runtime behavior.

## Reviewed ownership and trust flow

| Boundary | Code inspected and conclusion |
| --- | --- |
| Local inventory | `store/mod.rs`, `manifest/mod.rs`: bounded traversal and strict declarations; inventory is explicitly not authenticated source or restore permission. |
| Payload custody | `payload/{mod.rs,source/mod.rs}`, capture source/assembly: exact member set, trusted manifest, held files, block checks, directory/name checks. Caller must exclude noncooperating writers; checks are not an OS write fence. |
| Approval and creation | capture plan/intent, restore plan, operation intent/facade/custody: canonical original identities, trust/profile/limits, explicit approval digest, same-source restore, target ownership and create-only records. |
| Finite spending | operation framing/journal/runtime: reserve before signing/submission, original evidence before dependent work, no refund/reset, uncertainty preserves original custody. Journal structural replay is followed by authenticated semantic replay. |
| IC request and proof | capture advance request/lookup, certificate/observation and HTTP: direct signed management request, exact sender/method/args/expiry, retained root and separate time windows, bounded authenticated terminal evidence. One-shot submission/lookup; finding 1 is the request-limit mismatch. |
| Capture | fixed procedure, status/snapshot/transfer codecs, metadata/data checks: stopped approved module/controller, non-replacing take, original take attribution, complete deterministic extents/chunks, exact module hash, final status before publication. |
| Restore | source transfer, restore plan/persistence/procedure: independent approval and spending, original source custody, fresh allocation, exact uploads and complete metadata/byte readback before load; distinct stopped-status checks; returns `LoadedStopped`. |
| Completion/restart | operation replay/completion, capture completion/succession: immutable manifest and exact journal/payload binding, retained partial writes, explicit exact reconciliation, authenticated predecessor before fresh ownership. Completed replay requires no live signer/guard/clock/network. |
| Consumer boundary | public connection/signer/guard, private qualification consumer, committed Canic callers: no production Canic/Testkit/PocketIC dependency. Real manager route and real signer remain missing. |

Capture and restore converge on one recorder, journal, reservation mechanism and
transport. Their distinct procedures express different ordering/source contracts;
they are not duplicate recovery engines. Payload one-shot and held-reader admission
share an owner. Rechecks around awaits enforce independent custody boundaries and
must not be deleted as repetition. The concrete avoidable policy duplication is
the stale lookup ceiling in finding 1. No new generic provider framework, stage
API, extra journal or persisted cache is justified by this audit.

## Findings

### 1. MEDIUM — admitted 2 MiB profiles cannot recover through lookup

**Owner:** Backup. **Disposition:** confirmed defect; fix before Canic dependency
pinning and before claiming this recovery path for the published capacity profile.

`store/limits/mod.rs:5` admits 2 MiB signed requests, needed around a 1 MiB upload.
`operation/intent/mod.rs:232` passes that immutable request ceiling into lookup
authentication. `capture/advance/request/lookup/mod.rs:95` instead rejects any
ceiling greater than 1 MiB, irrespective of the actual lookup envelope's size.
The test-only entry at line 75 repeats the same stale bound. Source history
`af240881239112b8a8a9250b4c37e3f00dc68b19` introduces the shared 2 MiB limit;
lookup did not converge on it.

**Consequence:** a valid capture or restore profile above 1 MiB fails with
`CapturePlan(Lookup(Bounds))` when observing a submitted call with a lost reply.
`operation/runtime/mod.rs:465` reserves the observation before signing and checking
the returned envelope. The failure leaves that observation/signing allowance spent,
with no prepared observation. Reopening does not authorize another signature.
The original management outcome remains uncertain; independently surviving original
reply evidence can still settle it. There is no demonstrated redispatch or data
corruption, but normal lookup recovery is unavailable. Lowering an existing
operation's immutable profile is not a valid workaround.

**Fresh reproduction:** copied the existing qualification workspace into ignored
`target/evidence/canic-readiness-audit/lookup-profile/`, pointed its dependency at
the unchanged product, and changed only the test's restore request ceiling from
the ordinary 64 KiB selection to 2 MiB. Ran the existing
`interrupted_allocation_observes_once_then_replays_offline` actual-IC case:

```bash
IC_BACKUP_TESTKIT_DIRECTORY="$PWD/.tools/testkit" cargo test --release --locked --offline \
  --manifest-path target/evidence/canic-readiness-audit/lookup-profile/Cargo.toml \
  --target-dir qualification/target -p ic-backup-qualification --test capture \
  interrupted_allocation_observes_once_then_replays_offline -- --exact --nocapture
```

It fails at copied `tests/capture.rs:685`, after `Http(Uncertain)`, with
`CapturePlan(Lookup(Bounds))`: zero passed, one failed, ten filtered; 8.33 seconds
test execution, 4.17 seconds build. Log: `target/evidence/canic-readiness-audit/lookup-profile.log`.
The retained restore journal ends at sequence 12, `observation_reserved` for
`allocate_snapshot`, following sequence 11 `call_submitted`; no observation file
exists. Original fixture evidence remains under
`lookup-profile/target/fixtures/capture-RestoreAllocationObservation-4045508-1794822550923827005/`.
`lookup-journal-summary.json` is only a diagnostic projection of those originals.

Original test SHA-256: `39cc907cbb370aa2757d4a978129ef5a6c6ae539516f2b989b1720064934fb95`.
Probe test: `39bf794a0915d6423e603f305ea1280c8837f595c32077af74256578ddc8d2a4`.
Failure log: `306ed1c82543dd147875b8dceaaf7c6de8bde982619764c94de245d2a7a62031`.
The release's original 64 KiB lookup case passes. Its 1 MiB-chunk/20 MiB cases
use the larger restore profile but do not exercise this restore-lookup combination.

**Smallest correction:** make lookup admission agree with the approved request
limit through its canonical owner, preserving actual-byte, sender, path, signature
and exact-expiry checks. Add the missing profile-boundary regression and qualify
interrupted restore lookup with the 2 MiB profile on actual IC. Do not replenish
already-spent observations or rewrite existing approvals as part of the fix.

### 2. MEDIUM — manager-owned targets need a new authenticated route

**Owners:** Canic executor and Backup route/admission/replay;
[existing issue #6](https://github.com/dragginzgame/canic/issues/6).
This is an acknowledged missing capability, not a regression in direct capture.

Committed Canic `crates/canic-management/src/engine/mod.rs:505` creates children
with the manager as their controller and line 620 checks that exact controller set.
Backup `operation/runtime/mod.rs:96` owns a concrete direct HTTP transport;
request authentication accepts supported self-authenticating host keys for a
management-canister call. Status requires that actual sender to control the target.
An operator who controls the manager is not therefore a controller of its child.

**Consequence:** wrapping the current Canic signer/guard cannot operate the normal
manager-owned child. An authenticated outer host-to-manager response also does not
prove the inner management result. Changing controllers to bypass the boundary
would change Canic's custody model, not complete the intended integration.

**Smallest next increment:** specify a Canic-owned executor request/result/lookup
contract, joined to Backup approval, operation, step/ordinal, exact target and
arguments, plus executor identity/generation. Keep its original-effect ledger
outside the restored application. Backup must admit and replay that route using
the same fixed procedures; Canic owns its management effects and their finite
liabilities. Qualify lost replies on both hops without redispatch. Avoid a generic
RPC or speculative provider family before this concrete contract exists.

### 3. MEDIUM — current Canic signing is not the Backup adapter contract

**Owner:** Canic host, [existing issue #5](https://github.com/dragginzgame/canic/issues/5).
The committed `crates/canic-host/src/signed/mod.rs:161` signer requires five minutes
of remaining plan time, invokes ICP CLI synchronously with inherited streams,
does not pass Backup's exact requested expiry, and bounds the result file after
the process exits. Its surrounding executor also owns submission and polling.

Backup's `OperationSigningRequest` requires exact expiry, a completion deadline,
bounded complete output and reserved signer network cost. Its asynchronous signer
contract requires cancellation custody and sign-only behavior. Existing Canic
deployment semantics are not automatically wrong, but reusing that entire executor
would violate Backup's original-call and accounting contract.

**Smallest next increment:** a narrow Canic-owned sign-only adapter using the
maintained process owner, with real-tool proof of exact call and read-state
signatures, supported identity, deadlines, output limits, cancellation and no
hidden submission/retry. Resolve actual tool expiry support first. Use an already
directly controlled small fixture for this boundary; it does not qualify manager
execution or authorize adding a controller to a managed production child.

### 4. MEDIUM — application recovery remains a required Canic protocol

**Owner:** Canic/application, [existing issue #7](https://github.com/dragginzgame/canic/issues/7).
Backup's synchronous guard checks an original witness; it does not acquire
maintenance, stop/start a canister, settle external effects or make restored state
safe to reopen. Canic's committed design section 13 specifies the fence, but the
inspected source has no qualified Backup consumer implementing that journey.

**Smallest next increment:** one application and a surviving manager/control plane.
Persist its recovery fence before capture, account stop and start separately,
restore to `LoadedStopped`, start into the captured fence, reconcile against
surviving authority, then explicitly accept/open. Snapshot load must work without
an upgrade-hook assumption. Exercise restored timers/callbacks and a post-checkpoint
external effect so rewind cannot repeat completed work. Keep whole-fleet/control-
plane rollback and active restoration of an older Blob owner excluded.

### 5. LOW — integration status prose trails the released source

Canic issue bodies still describe Backup 0.102.0 as unpublished/local and cite
older test totals. Current source, release records and handoff describe the later
authorized release and direct fixture qualification. These discrepancies can
misdirect integration work, but do not implement or remove a runtime capability.

Refresh the owning status prose when starting the integration, naming 0.102.1
and this newly discovered recovery defect, then the corrected version once
qualified. Retain the distinction between direct-fixture and Canic qualification.
No issue messages or instruction-file edits were made by this audit.

## Legacy guarantees and intentional scope changes

Focused review used legacy artifact `verification/mod.rs`, `member/mod.rs`, Agent
`standalone/transfer/mod.rs`, coverage tests, and
`workflow/ic_snapshot_restore/tests/mod.rs::failure_boundaries_retain_pending_or_applied_and_never_repeat_provider`.
History inspected includes `9f7b886a824eec67dda8f81792a560436419bb2a` (derive upload
bindings from verified original bytes and retain closing source checks) and
`d245b0f03a505ba5215d65c10bdebf1890c7dc37` (retain reply association with the original
reserved request; post-dispatch retention failure does not authorize retry).

| Legacy guarantee/problem | Present owner/evidence and changed assumption |
| --- | --- |
| Corrupt or substituted source after verification | Shared payload admission plus held block commitments and closing custody checks. Existing `source_tests` cover same-length corruption/replacement, changed trees and root rename. Original operation evidence remains independently authenticated. |
| Gaps, overlaps, duplicate/empty chunks | Fixed deterministic schedule, strict ordinal replay, exact region lengths, unique hash-bound whole chunks. Legacy allowed interleaved arrivals; current sequential schedule removes that choice while preserving exact coverage. Transfer tests and actual-IC full-chunk/capacity cases cover it. |
| Failed admission, lost/malformed reply, refused settlement | Shared reservation/runtime, typed surviving-certificate errors, original-only imports and one lookup. Runtime tests cover loss/cancellation/revoked guard/failed retention; actual-IC allocation/upload/load interruptions exist. Finding 1 exposes a missing cross-profile case. |
| Completed replay is not fresh authority | Immutable completion and original historical proof; registry consumer replays after backend shutdown. Fresh effects separately require the original live admission. |
| Application acceptance and start fencing | Deliberately moved out of Backup, not eliminated. `LoadedStopped` is narrower than legacy's application-assisted end-to-end finish. Canic must implement finding 4. |

No loss of module/heap/stable/globals/certified-data/chunk/timer/hook representation
was identified in the supported snapshot profile. Optional absence and exact float
bits remain explicit. Broader legacy composition, state-based reconciliation,
stop/start/acceptance and source release are not delivered Backup features.
Existing legacy operations still require their original reader, IDs and authority.

The existing [legacy assessment](../../../../../../../design/legacy-assessment.md)
and [inventory provenance](../../../../../../../architecture/local-inventory.md#focused-legacy-review--2026-11-14)
own the broader case mapping; historical legacy passes are not new qualification.
No LOC count was rerun: release preparation changed no production Rust. The earlier
46.99% app/library reduction remains a scoped comparison, not proof of full feature
parity, speed or Canic readiness.

## Concrete integration order and remaining qualification

1. Correct finding 1 in Backup, retain a regression for the admitted profile, and
   qualify its actual-IC interrupted restore. Prepare a patch release before Canic
   pins a recovery-capable version; preserve all original pending evidence.
2. Resolve the real signer contract and create a narrow Canic host consumer against
   an already directly controlled fixture. Bind exact plan/source/network/target,
   original evidence and current application witness through the public API.
3. Implement the actual manager request/result/lookup protocol with coordinated
   Backup admission/replay support. Prove both-hop uncertainty, finite spending,
   manager generation changes and offline receipt replay.
4. Complete one fenced application capture/restore/accept/open journey and its
   participant-specific reconciliation. This is the first meaningful Canic
   acceptance milestone; it is not a whole-fleet checkpoint.

The 20 MiB Linux fixture is useful capacity evidence, with its explicit 22 MiB
payload allowance, geometry, call/evidence budgets and measured costs. It is not
a guarantee for every 20 MiB shape or the missing recovery combination. Replay is
still quadratic; preserve the recorded measurements when evaluating the Canic
consumer rather than adding a cache without a measured need.

Production HTTPS, actual-IC macOS execution, real Canic process cancellation and
application-specific recovery remain unqualified here. Hosted native CI status
was intentionally not checked. Physical crash durability is not established by
process-kill tests. These constrain production acceptance, not the ability to
start the concrete integration increments above.
