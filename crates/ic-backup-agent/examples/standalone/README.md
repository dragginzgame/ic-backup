# Standalone backup and restore assembly

This maintained Rust example assembles the public Core/Agent APIs into one bounded
host-driven workflow. It freezes capture and restore catalogs before effects,
captures and downloads an original snapshot, publishes its manifest, uploads a new
source-bound destination, loads the same existing canister, requires independently
verified application acceptance before start, and releases its exact source
reference under terminal evidence. Inspection and cold continuation use the same
original catalogs and journals.

The [workflow module](workflow/mod.rs) is the assembly used by the independent
consumer qualification. Its [application boundary](workflow/application/mod.rs)
has required callbacks and no default authority. The test adapter, fixed test keys,
PocketIC setup, oracle bytes and fault modes live under `tests/independent`.

## Embed the example

Copy the `workflow/` directory into a host application's source and declare
`mod workflow;`, or adapt that same source within the consumer. The example is
compiled as an `rlib` Cargo example; it adds no installed CLI or workspace crate.
Its direct dependencies are `ic-backup`, `ic-backup-agent`, `ic-platform` with
`agent`, `candid` and `management-canister-types`, `ic-host-fs` and `serde_json`.
Align supplier identities with the owning workspace. Supply an async executor
for Agent network/timer work; the assembly imposes no `Send` requirement.

The caller supplies an actual `Arc<dyn ic_platform::agent::Identity>`, a trusted
root key provisioned out of band, exact declared context, target, endpoint and
finite timeout. No identity is compiled into this example and no trust root is
fetched. `Connection::new` rejects signer/context mismatch, invalid targets and
invalid transport inputs before catalog creation.

The following sequence uses application-owned values and an implementation of
`workflow::Application`. `application.verify_stopped_restore()` represents that
application's own separately accounted verification and durable acceptance owner;
it is deliberately not a generic success flag supplied by this module.

```rust,ignore
use std::num::NonZeroUsize;
use workflow::{Connection, Driver, Phase};

let connection = Connection::new(
    context, target, endpoint, identity, trusted_root, timeout,
)?;
let mut run = Driver::new(private_root, connection, &application);
run.create()?; // Once, before capture; occupied/partial originals refuse recreation.
run.advance(Phase::Capture, NonZeroUsize::new(3).unwrap()).await?;
run.advance(Phase::Restore, NonZeroUsize::new(3).unwrap()).await?;
// Allocation, upload and load have settled; start is still absent.
application.verify_stopped_restore().await?;
run.advance(Phase::Restore, NonZeroUsize::new(1).unwrap()).await?;
run.release()?;
```

On a later invocation, construct the driver from the same original inputs and
root, inspect with `run.inspect(Phase::Restore)`, then call `advance` without
`create`. The positive stage bound counts new stages, not HTTP calls. An occupied
pending or partially prepared stage refuses before another dispatch. Retain the
original inputs, signed envelopes, raw arguments/replies, learned evidence,
manifest, partial artifacts and references for explicit original-authority
reconciliation. Recreating the catalogs or increasing allowances is not recovery.
A successful `release` also freshly qualifies already-released replay and keeps
unrelated references; it performs no remote call or artifact cleanup.

## Implement the application boundary

| Callback | Required application fact |
| --- | --- |
| `admit` | Actual fresh membership/release, direct controller or read permission, capture consistency, and exclusive never-dispatched custody. Every remote check has its own prior accounting. |
| `retain_ingress` | Durable original signed envelope/request ID before submission, refusing any prior or uncertain dispatch witness. `workflow::retain_ingress` provides create-only local retention after custody is qualified. |
| `qualify_reply` | Actual original-request attribution and application meaning, plus durable exact argument/reply retention before cancellable qualification work. `retain_applied_reply` is available after qualification; decoding or matching bytes alone cannot justify it. |
| `restore_requirement` | The explicitly selected safety lane, both original plans, exact source manifest and any original fence binding. There is no no-irreversible-effects default. |
| `restore_safety` | Current stopped lifecycle and whole-selection safety, external obligations, required fence custody and accepted restored state before start. |
| `acceptance` | Independently verified stopped post-load acceptance bound to the original restore, source metadata and actual allocated destination. Called before preparing start and again during terminal replay. |
| `terminal` | Qualified complete application/effect settlement and the complete original command-custody set. Empty commands require actual proof of completeness. |

Applications own quiescence, current observations, evidence freshness and stable
noncooperating byte custody. A status response or operator assertion cannot prove
absence of payments, timers, outbound work or obligations. The assembly accepts a
fenced requirement only with the application's corresponding original binding and
qualified current safety evidence; it does not acquire or release an application
fence. Root-paid application protocols retain their own integration owner.

The reference catalog targets one existing canister on the same declared network
and release. It admits eight data reads and eight data writes, with 1 MiB chunks;
metadata exceeding those original finite ceilings refuses without expanding them.
The reference catalog requires a nonempty data plan; read-free metadata refuses
explicitly. Choose any different transfer ceilings before freezing a new catalog,
and retain that exact profile for every continuation.
Original v1 token/purpose labels remain unchanged so this promotion does not rebind
retained authority. This is not multi-canister or partial-transfer resume support.
Errors preserve their typed source inside `io::Error`, including bounded raw
responses carried by Core post-reply errors. Consumers must retain that evidence;
no error grants retry, rollback, budget reset or cleanup authority.

## Qualification

Build the example with:

```bash
cargo build --offline --locked -p ic-backup-agent --example standalone
```

After explicit Testkit setup, run its real consumer qualification with:

```bash
cargo test --offline --locked -p ic-backup-agent --test pocketic_transport independent:: -- --nocapture
```

The isolated normal-dependency build copies the exact example sources and excludes
Testkit/PocketIC from the child graph. Its application adapter qualifies an
exclusively owned inspected fixture, including a different supplied signer for the
successful journey. Cold stage continuation, lost allocation/upload/load/start
replies, ingress/reply/partial-output refusals, changed acceptance and guarded
terminal replay exercise this same assembly. Candidate-path simulator acceptance
is distinct from registry-only, installed application, live IC and native macOS
qualification; see the [handoff](../../../../docs/status/current.md).
