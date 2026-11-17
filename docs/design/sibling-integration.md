# Sibling dependencies and Canic integration

Design review: 2026-11-14. This refines the [library design](library-design.md)
from read-only inspection of the current sibling worktrees. The selected first
slice remains one already-stopped canister. **Canic is a required consumer**, with
its own application coordination and management execution adapter; standalone
use remains equally supported. No dependency, supplier code or product API was
changed by this review. The subsequent [operation contract](operation-contract.md)
turns these findings into proposed public-operation and provider semantics.

## Evidence and scope

Canic is now the 0.200.x rewrite, not the Fleet/Root implementation described by
Backup's legacy documentation. Several sibling worktrees contain active changes.
The source observations below do not establish registry freshness, a released
integration or new qualification. Use published dependencies when implementing;
recheck available releases then. No sibling path patches or shared targets.

| Repository | Inspected base | Relevant observation |
| --- | --- | --- |
| Canic | `c617d902c6b9a0c29634861b826e215fdc072bea` plus local changes | Native host/CLI, external signing, profile manager/worker; checkpoint design exists, Backup adapter does not. |
| Platform | `c90ce9fce0948da0c05e6f5ac333c520fa46d801` plus local locks/qualification | Platform 0.32.3, Types 0.20.3, Runtime 0.22.3 in inspected manifests. |
| IC Tooling | `d02fc37af14a83845903bfb6d9b524a29693b9c0` | Host 0.77.2, Testkit 0.104.0 and separate Testkit Wire 0.1.0. |
| Auth | `7af3ccb3fe9fc01d2841c9ee8512785fe9a014b4` plus local changes | Session/denial rollback requires surviving authority. |
| Blob | `212452e73896376f46e8e7e04906f50feba62e74` plus local changes | Historical snapshot activation remains unsupported. |
| Scheduling | `7178d65b48664445fcd44d0d4632c23afcbeb416` plus local locks | Jobs and Timers are canister lifecycle participants. |
| Entropy | `0c8b68b580e46e6c5aebb3a02c5e19f26041cf59` | Runtime reconstruction and fresh entropy belong to the application. |
| Storage | `d2762f2dc7db339a88446e9c7c620ab95785711b` plus local locks | Canister stable allocation/collections, not the host backup store. |
| Wire | `6b8df811dca1e1f289b290b6b0040314b01714bd` plus local changes | Bounded owned RPC; external management stays Candid. |
| Observability | `bd13bd466d2a872ed6439d1f71790a7e78dfda10` plus local lock | Logging is optional; Metrics is application instrumentation. |
| Relay | `50ef15f286830f1c03cc85f0804f23a622bd09bc` plus extensive local changes | Retained channel history does not by itself qualify stale-snapshot recovery. |
| IC DB | `18dba6cc3c3bd9f5243d81dc8a693a4dfe56797d` plus local implementation | Current source has a small schema/store API, ahead of its scaffold README. It is an application participant, not Backup's host database. |
| IC Query | Uncommitted fresh worktree | `ic-topology` has a bounded placement report; `ic-sns-migration` is a shell. No umbrella `ic-query` library. |

Committed links below were checked against unchanged relevant source files even
where unrelated worktree changes exist. Findings about IC DB, Query and Relay
use the local source inspected during this review; they are not claims about the
base commit's complete implementation. There is no separate local Testkit repo:
its package lives in IC Tooling. Toko Tooling remains the installed development
baseline, not a Backup runtime dependency.

## Dependency selection by owner

### Platform rebuild adoption review — 2026-11-16

Supplier work is tracked in [Platform #11 — snapshot/status codecs](https://github.com/dragginzgame/ic-platform/issues/11)
and [Platform #12 — signed ingress/recovery](https://github.com/dragginzgame/ic-platform/issues/12).
These own implementation acceptance; the evidence below records the reviewed consumer boundary.

Current adoption (2026-11-17) is complete: Platform 0.34.11 owns ingress,
direct-key and certificate verification; Protocol 0.3.3 owns management request
encoding and reply decoding; Types 0.21.2 owns public identities. Old Platform
and Types versions are removed from both locks and test dependencies. The
[handoff](../status/current.md#current-platform-adoption--2026-11-17) owns current
consumer qualification; the following probes are historical.

Registry trial of **Platform 0.34.7 / Protocol 0.2.6 / Types 0.21.2** passed.
Platform/Protocol source is `a813d4ea5378515984a547a9f4e9d37f2124cf14`; archives
match their registry checksums. Protocol now supplies `management-snapshots`.
The isolated public-API probe uses no old Platform implementation: it reads the
original wire requests from the qualified 20 MiB fixture, existing independently
retained golden replies and explicit byte fixtures. All 110 ingress envelopes
parse with original-byte custody and computable request IDs; 11 status/list and
two take argument encodings match original requests exactly. All eight golden
status/list/take/metadata replies decode; metadata checks preserve full-width
integers, NaN/signed-zero bits, chunk order, optional absence, timer/hook and
provenance. Full 1 MiB data and empty-result decoding pass, along with byte-cap
and trailing-data refusals. Evidence and exact probe source/lock are under
`target/evidence/platform-retry/`; no source, operation or wire record was changed.
This removes the previous missing-API observation, but does not qualify a complete
consumer port, certificate authentication, every request shape or installed IC
effects. Product and qualification still use the old cohort until coordinated
caller migration and recovery qualification are complete.

Earlier recheck: verified Platform **0.34.6** / Protocol **0.2.5**, source
`9dfdc0083fa5d6e56da7b90c04c20de4b2d3cc9f`, now supplies `ingress-packets` with
strict content/envelope decoding, original bytes, request IDs and response parsing.
This advances #12; actual Backup migration and regression qualification remain.
Protocol's `management-chunks` handles `stored_chunks` / `upload_chunk`, not
snapshot metadata/data, allocation/load or status. #11 therefore still blocks
complete adoption. Types 0.21.2 is available, but both maintained workspaces still
need the old facade's Types 0.20.4 at their shared public boundary. No independent
qualification migration is claimed. Evidence: `target/evidence/platform-tooling-refresh/`.

The preceding review found gaps at the management codec and signed-ingress
boundaries. That registry/archive review verified Platform **0.34.5** and
Protocol **0.2.4** at `1e92703bc803b81dbe1e3d815fd1077d440d62b4`; Types remains
**0.21.1**. Original archives, index checksums and the fresh offline resolution
refusal remain under `target/evidence/platform-refresh-next/`.
The earlier review verified Platform **0.34.3** at `3f972ec7d43e2fcd087f69bae90767f6bc405c85`, Types
**0.21.1** at `2797582e24f782e442df841d24db1ea0fac7bb92`, and Protocol **0.2.3** at
`cbbf26ade6644ed6fbcadfe3ccbafe809c362f25`. Protocol 0.2.2 at the Platform release
revision was also inspected. These are deliberately narrowed rebuilds, not
compatible facade updates. Original archives, index checksums and an offline
resolution refusal remain under `target/evidence/platform-refresh/`.

| Backup requirement | New published surface / gap |
| --- | --- |
| Snapshot/status DTOs and bounded Candid encoding/decoding in `capture/status/wire`, `capture/transfer` and `restore/procedure` | Protocol exposes selected Cloud Engine, Registry and HTTP-request formats; no management snapshot/status DTOs or replacement for the used codec API. |
| Original call/read-state envelopes, request IDs and response parsing in `capture/advance/request`, `request/lookup` and `capture/advance/http` | Platform 0.34.5's anonymous `native-read` client privately uses upstream envelope types; it exposes no replacement for Backup's signed ingress/recovery parsing. |
| Strict Ed25519 and low-S secp256k1 verification in `capture/advance/request` | Supplier gap resolved: Platform 0.34.4 restored `ed25519`/`ecdsa`, retained in 0.34.5. Backup consumer qualification still belongs to the coordinated migration. |
| One public Principal identity shared by DTOs, requests, plans and fixtures | New Types 0.21.1 is distinct from the Types 0.20.4 used by the required old facade; adding it alone does not migrate those boundaries. |

Cargo rejects the existing feature selection against 0.34.5, beginning with the
removed `certification-verify` feature. Renaming that feature alone cannot restore
the other missing APIs. Keep Platform 0.33.7 / Types 0.20.4 until the supplying
owners implement the required boundaries and Backup qualifies a coordinated
migration. Do not import the old facade, duplicate wire/crypto ownership or add
an unused parallel Types dependency just to claim adoption.

The [0.34.5 release notes](https://github.com/dragginzgame/ic-platform/blob/1e92703bc803b81dbe1e3d815fd1077d440d62b4/releases/platform/CHANGELOG.md)
also add certificate construction and separately selected fixture signing. The
new crypto modules re-export upstream Dalek/K256 types; no missing-crypto claim
remains. Anonymous query/subnet observations are not a substitute for exact
externally signed update envelopes, original request IDs and retained recovery
proofs. No dual Platform dependency or consumer-side protocol implementation was
added to work around the remaining supplier gaps.

The [new Platform changelog](https://github.com/dragginzgame/ic-platform/blob/3f972ec7d43e2fcd087f69bae90767f6bc405c85/releases/platform/CHANGELOG.md)
records immutable bounded certificate/authentication APIs and independent
direct/delegated, subgroup and mainnet replay evidence. These are useful future
migration inputs, not proof that a known Backup defect has been fixed. Backup
already uses historical root/target/signing/delegation checks. The
[supplier requirements](https://github.com/dragginzgame/ic-platform/blob/3f972ec7d43e2fcd087f69bae90767f6bc405c85/docs/requirements.md)
retain Backup's signed recovery inspection contract as a separate requirement.
Canic signing, manager execution and fenced recovery remain consumer work.

### Selected dependency boundaries

| Dependency | Proposed owner/use | Selection rule |
| --- | --- | --- |
| `ic-platform` | Backup's external management DTOs and bounded Candid encoding/decoding; Agent adapter's transport/signing/certificate interfaces. | `default-features = false`; select `management-canister-types` and `candid-codec` at the management boundary, `agent` only in the native adapter, and `ic-protocol` when inspecting signed envelopes/read-state messages. |
| `ic-platform-types` | Canonical Principal/Account owner. | Platform reexports the same Principal. A future pure identity-only package may depend on Types directly; do not add a redundant dependency to a package already using Platform merely to duplicate that import. |
| `ic-host-fs` | Selected 0.80.2: held directories, no-follow reads, retained files, descriptor locks and explicit publication outcomes. | Direct storage dependency; the old `capability` feature is retired. |
| `ic-host-artifacts` | Selected 0.80.2: complete bounded reads/copies and producer hashing. | Backup owns its commitment encoding and expected-digest comparisons; approved direct `sha2` 0.11.1 supplies SHA-256 and the fixed module stream. |
| `ic-host-http` | Selected 0.80.0: bounded complete-body consumption. | Backup directly owns Reqwest 0.13.5 with Rustls, endpoints, retries, trust, deadlines and IC evidence. |
| `ic-host-process` | External signer execution/custody. | Canic's signer adapter already needs it. Core does not spawn signers or management commands. |
| `ic-wire` | Canic-owned recovery/control RPC and any manager execution protocol. | In Canic's protocol/adapter, not a replacement for management Candid or a mandatory host artifact format. |
| `ic-platform-runtime` | Canic's canister-side execution and lifecycle. | Canic dependency only; never pulled into host Backup for a manager route. |
| `ic-testkit` | Real simulator, managed assets/processes and failure fixtures. | Private qualification workspace only, through `ic_testkit::pocket_ic`; no direct extra PocketIC dependency. |
| `ic-testkit-wire` | Optional typed Canic fixture calls. | Private qualification only if those fixtures use Wire. Use raw calls when exact retained reply bytes are needed. |
| `ic-logging` | Optional redacted diagnostic recording/export. | Add when a named diagnostic use needs its API; structured progress is already a Backup view. |

Platform's [management exports](https://github.com/dragginzgame/ic-platform/blob/c90ce9fce0948da0c05e6f5ac333c520fa46d801/crates/ic-platform/src/management_canister_types/mod.rs)
and [feature declarations](https://github.com/dragginzgame/ic-platform/blob/c90ce9fce0948da0c05e6f5ac333c520fa46d801/crates/ic-platform/Cargo.toml)
provide the concrete dependency boundary. Use owned DTOs and codecs; do not add
direct registry management declarations, Candid, `ic-agent` or an external Principal
alias in Backup. Convert validated principal bytes at the private native SDK edge
when supplier identities differ. Feature unification must not accidentally enable
canister runtime or native SDK code in the wrong consumer graph.

Host already provides the primitives behind the proposed store. In particular,
[retained publication](https://github.com/dragginzgame/ic-tooling/blob/d02fc37af14a83845903bfb6d9b524a29693b9c0/crates/ic-host-fs/src/durable/retained/mod.rs)
keeps failed partial files, while [artifact streams](https://github.com/dragginzgame/ic-tooling/blob/d02fc37af14a83845903bfb6d9b524a29693b9c0/crates/ic-host-artifacts/src/artifact/mod.rs)
provide bounded hashing. Host does not provide Backup's transactional ledger,
record framing, attempt transitions or multi-file recovery. Those remain local;
reuse of a file-write helper does not prove journal crash consistency.

Do not use `ic-storage` or `ic-db` for the operator's filesystem ledger. Do not
add Jobs/Timers as a host scheduler, Blob as an initial backup destination, Auth
as management-controller authority, Relay as an IC management proxy, or Topology
as proof of application membership. These are different responsibilities.
`ic-build`, host tool installers and Wasm inspection may support fixtures or
application artifact preparation; they are not required to transfer snapshot bytes.

## Canic's current constraints

### Management identity is not the operator identity

The [profile manager](https://github.com/dragginzgame/canic/blob/c617d902c6b9a0c29634861b826e215fdc072bea/apps/profile/manager/src/canister.rs)
creates its worker with `controllers: [manager]` and later verifies that exact
set. The operator controlling the manager is not thereby a controller of the
worker. The current worker therefore cannot use Backup's direct-host profile.
Do not add a controller, change visibility or substitute a proxy principal merely
to make the standalone path pass.

The public management API permits canister-originated snapshot operations.
The earlier native-only DTO gap is resolved in the selected registry Platform
0.33.7: `management_canister_types/snapshot/{mod.rs,codec/mod.rs}` and its parent
export snapshot records/codecs on Wasm; only native client extensions remain
target-gated. A temporary caller compiled take/load/read/upload DTOs and owned
Candid encoding for `wasm32-unknown-unknown` with only the management feature.
Its locked graph contains no Agent/transport dependency. Source is the same verified
0.33.7 archive already selected by Backup; product/qualification locks did not change.
Evidence: `target/evidence/reduction-followthrough/platform-wasm.log`.
This proves that supplier target boundary, not a Canic executor or live IC calls
from Canic. No local DTO copy or new maintained package was added.

Propose two explicit, immutable execution bindings:

- **Direct host:** approving caller and actual management sender are the selected
  host identity. The existing narrow capture/restore profile applies.
- **Canic executor:** the host initiator is authorized by a selected live Canic
  executor; that canister is the actual management sender. The plan also binds
  executor identity/generation, target, approved operation and receipt protocol.

Both use the same Backup transfer state machine and artifact format. The provider
owns the route-specific authentication and effective controller evidence. A
manager reply is not a raw management ingress certificate; it needs an authenticated
Canic receipt bound to the underlying retained effect. Source identity and
management sender remain separate fields. Never silently fall back between routes,
change a pending route, or infer execution authority from a principal's role label.

### Replacement follow-up, 2026-11-15

Read-only recheck of Canic at `fe3a73d297f268932d54e8109038f58198042582`
with local changes found the same controller distinction in the newer managed
starter. Its [system design](https://github.com/dragginzgame/canic/blob/fe3a73d297f268932d54e8109038f58198042582/docs/design/canic-system-design.md)
(section 4, infrastructure roles) identifies Root as its application's sole
controller and the external operator as Root's controller. Section 13 retains
application-only checkpoint recovery with surviving control-plane authority;
section 17 still describes the Backup adapter as future consumer-gated work.
These are design/source observations, not fresh IC qualification. No sibling files
were changed and this recheck does not requalify the earlier Platform DTO gap.

Consequently the [replacement](restart.md) implements the direct route only where
its signer is an actual target controller. The same host consumer loop is suitable
for fleet coordination, but it is not yet a working Canic-managed application
adapter. The existing [Canic issue](https://github.com/dragginzgame/canic/issues/1)
remains the integration owner. Do not add an unused executor variant or change
controllers to hide this missing authenticated management-result protocol.

### External signing and retry semantics

Canic's [signed-attempt module](https://github.com/dragginzgame/canic/blob/c617d902c6b9a0c29634861b826e215fdc072bea/crates/canic-host/src/signed/mod.rs)
uses ICP CLI in sign-only mode, inspects the returned envelope and retains exact
update/read-state bytes. It keeps private keys outside Canic. Its deployment
continuation can resubmit the same envelope within the original validity window
and poll status in a loop. That is not the draft Backup contract, which reserves
one submission and separately accounts every observation.

Do not call Canic's private `signed::execute` as Backup's provider or copy its
retry loop. Define a bounded prepare/sign boundary accepting an exact reserved
request and returning inspected original signed bytes plus status authority.
Backup retains those bytes before its one submission; a separately invoked
observation consumes its own allowance. An external signer is an adapter concern,
not a requirement to export a key into an `Arc<dyn Identity>`. Its process must
only sign; it has no authority to submit, retry or renew a management request.
Unexpected signer loss preserves custody and blocks dependent dispatch.

Platform's [Agent boundary](https://github.com/dragginzgame/ic-platform/blob/c90ce9fce0948da0c05e6f5ac333c520fa46d801/crates/ic-platform/src/agent/mod.rs)
explicitly leaves retry/deadline/trust policy to the consumer. Configuring TCP
retries alone does not establish absence of HTTP middleware retries. Qualify the
actual submission path and retained authenticated outcome, including external
signatures, expiry and missing status replies.

### Discovery is an input, not a freeze

Canic's current `Fleet` views report local deployments, certified module/controller
observations and optional manager-reported workers. The source documents partial
inventory and uncertified manager reports. Do not feed this view into Backup as an
authenticated complete frozen membership. Canic must resolve an exact selected
target and independently establish live authority and continuous maintenance
custody. `ic-topology` can contribute placement evidence; it cannot replace this
application admission or provide a recovery fence.

## Required Canic integration contract

Keep the integration in Canic, initially as a real module in `canic-host` plus
Canic-owned application/executor protocol and handlers when needed. A separate
`canic-backup` wrapper crate and a framework dependency in Backup are unnecessary.
The host engine remains outside canisters even when management effects are
performed through a canister controller.

| Boundary | Required owner and binding |
| --- | --- |
| Approval | Canic binds its retained operation ID to one exact Backup plan digest, source, network, target and execution route. Canic display names never replace physical IDs. |
| Admission | Canic provides current release/schema identity, target membership, controller/read authority, maintenance custody and explicit rewind safety. |
| Pre-capture fence | Canic persists a recovery fence in application state before capture and keeps its witness outside the rollback set. It stops the selected application through its own accounted operation. |
| Snapshot execution | Backup owns capture/transfer/load requests and original host allowances. A Canic executor reserves its own real canister-call liabilities and retains exact results under the same logical attempt binding. |
| Interruption | Host timeout or duplicate continuation looks up the original executor operation; it does not ask the manager to issue a second management call. Unknown underlying outcome remains pending. |
| Completion | Backup returns `BackupVerified` or `LoadedStopped`, with exact immutable evidence; Canic does not translate these into application readiness. |
| Reopening | Canic starts only the checkpoint that already contains its recovery fence, reconciles participants, obtains fresh surviving authority, accepts the application and explicitly opens normal work. |
| Retention | Keep source references until Canic's accepted terminal outcome and external obligations permit release. The first Backup slice retains references; a release API is separate designed work. |

The host journal and executor ledger protect two different effect boundaries;
they are not interchangeable replicas of an allowance. Define a stable join of
Backup operation/step/attempt and Canic executor operation/generation. A replay of
that join returns the original result or unresolved state. It cannot renew budget.
Budgets include both host-to-executor and executor-to-management requests, reply
framing, streaming windows, readback and separately authorized observations.

Canic's [checkpoint design](https://github.com/dragginzgame/canic/blob/c617d902c6b9a0c29634861b826e215fdc072bea/docs/design/canic-system-design.md)
requires surviving control-plane authority outside the application rollback set.
Keep coordinator/executor rollback and whole-Fleet historical restore excluded.
Even a one-worker restore must fence its manager's topology/producers when they
could invalidate the selected target. Adding more per-canister Backup calls later
does not by itself produce a distributed checkpoint.

## Application participants are not Backup dependencies

| Participant | Canic/application obligation before reopening |
| --- | --- |
| Storage and IC DB | Admit the exact storage/schema layout and restore the owning application. Backup copies full snapshot bytes without decoding database rows or performing schema migration. |
| Auth | Preserve current denials/revocation and obtain fresh authenticated authority; an older internally valid session image is not safe merely because it decodes. |
| Blob | Keep the current external storage owner and reconcile restored consumer references against it. Active restoration of an older Blob-owner snapshot is unsupported; a successful load must not enable it. |
| Jobs and Timers | Preserve original effect identities and reconcile Running/Uncertain work against surviving evidence. Restore scheduling only under the recovery fence; rewind must not repeat completed payments or jobs. |
| Entropy | Discard restored seeds/streams and reconstruct under a fresh authorized lifecycle before security-sensitive work. Freshness is not obtained by reusing a saved random generator. |
| Relay | Reconcile incarnation, sequence/ACK state, retained charges and outstanding peer work. A locally consistent channel image is not proof of a fresh peer history. |
| Logging and Metrics | Restore diagnostic state only under its own contract; logs/counters cannot substitute for effect or authority receipts. |

These are substantive safety requirements from the siblings, not a generic list
of hooks that already exist in the new Canic runtime. See
[Auth's recovery boundaries](https://github.com/dragginzgame/ic-auth/blob/7af3ccb3fe9fc01d2841c9ee8512785fe9a014b4/docs/integration/canic-adoption.md),
[Blob's maintained service contract](https://github.com/dragginzgame/ic-blob-storage/blob/212452e73896376f46e8e7e04906f50feba62e74/docs/architecture/service-contract.md),
[Jobs safety](https://github.com/dragginzgame/ic-scheduling/blob/7178d65b48664445fcd44d0d4632c23afcbeb416/SAFETY.md)
and [Entropy lifecycle](https://github.com/dragginzgame/ic-entropy/blob/0c8b68b580e46e6c5aebb3a02c5e19f26041cf59/crates/ic-entropy-ic/README.md).
Snapshot load does not run normal upgrade hooks; Canic must explicitly qualify
its fenced recovery entry, including restored timers and callbacks.

## Implementation order and acceptance

1. Freeze the standalone operation contract together with a small Canic consumer
   contract test. Bind approval/source/target, separate initiator from actual
   management sender, and demonstrate external signing without moving private keys.
2. Implement and qualify the native capture/verify and upload/readback/load paths
   using published Platform and Host APIs. Run both the independent Rust consumer
   and a directly controlled Canic test application through the same public API.
3. Consume the verified Platform Wasm DTO boundary and implement the Canic-owned executor
   protocol for the actual manager-controlled worker. Exercise lost replies on
   both network hops, original-result lookup, authority changes and no redispatch.
4. Qualify the Canic fence/stop/capture and load/start-fenced/accept/open journey,
   with surviving manager/control-plane state. Initially use an inspected small
   application; add participant-specific acceptance before claiming support for it.

The first step remains a single-canister library. Successful native steps can be
reported independently, but full Canic compatibility requires steps 3 and 4 for
its manager-controlled topology. The present manager lacks that snapshot executor
and recovery protocol. These are concrete integration gaps, not already-supported
callbacks. No broad Canic, Blob or whole-Fleet recovery claim follows from a simple
fixture round trip. Keep both consumer graphs in acceptance; do not wait until the
library is finished to discover that its signer or authority model cannot fit Canic.

### Reduction follow-through — 2026-11-16

Read-only Canic review at `8412c1edca540e6f299ab1cf9fc2d3e9f2c3ed5a`
(0.216.3; unrelated CLI edits present) revisited
`canic-management/src/engine/mod.rs` controller creation/checks and
`canic-host/src/signed/mod.rs`. Management still makes Root the actual controller;
no Backup/checkpoint executor is implemented. Existing host signing/submission is
an operation with its own custody and polling policy, not Backup's sign-only adapter.
The adapter must use bounded process custody and produce the exact reserved
envelope/expiry. Its selected signer admission requires ICP CLI 1.7.0; the ambient
binary inspected here was 1.6.0, so no claim about a qualified 1.7.0 signing path is
made. No sibling source, tool installation or controller was changed.

Keep one Canic-owned host adapter, not a `canic-backup` wrapper crate, new key
custodian or copied retry engine. The necessary two-hop executor state protects a
different effect boundary and cannot be deleted as a duplicate of the host journal.
Bind original Backup approval/step/arguments and finite liabilities to Root's
surviving operation/generation; retain an exact authenticated result or unresolved
outcome. Repeated host continuation must look up that join without issuing another
management call. A Boolean executor success or direct-host certificate cannot
substitute for this protocol. Continuous application fence/rewind acceptance stays
with Canic outside the application's rollback set. These remain design obligations,
not newly implemented behavior.

## Direct-provider continuation review — 2026-11-14

The focused frozen-legacy review revisited
[Agent construction](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup-agent/src/lib.rs),
[single submission](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup-agent/src/submission/mod.rs),
[reply digests](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup-agent/src/reply_evidence/mod.rs)
and [HTTP boundary tests](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup-agent/tests/http_boundary/mod.rs).
They retain exact signed bytes before one request, disable redirects/proxies and
Agent/HTTP retries, bound responses and leave acceptance, disconnects, malformed
responses and timeouts unresolved. Keep these guarantees and re-author their
actual network-boundary tests with the eventual concrete provider. They are not
qualified by the current injected provider's native call-count tests.

Legacy returns raw reply bytes after `Agent::update_signed` verifies the live
certificate, then records a domain-bound reply digest. That digest is not the
certificate. Copying this path would lose the proof required by the new historical
replay contract. Platform 0.33.2 supplies native `ic_protocol::decode_call_response`
and strict certificate decoding, so a new adapter can retain original certificate
bytes. Native identities and external wire declarations must remain Platform-owned.

The inspected Agent 0.49.3 implementation's `update_signed` returns the reply or
pending request identity and discards the certificate. Its public `verify` checks
signature/delegation and then compares certificate time to the current clock;
Platform's `certification::verify::authenticate_certificate` delegates to it.
Calling that live verifier unchanged on an old receipt would reject valid retained
evidence. The canister-signature API verifies a different witness protocol and is
not a substitute for management ingress certificates. A concrete provider must
select and qualify historical authentication under original trust and admitted
time, separately from fresh-observation policy. Do not change signed ingress
expiry or suppress a verification failure to manufacture historical acceptance.
No cryptographic implementation, extra dependency or concrete transport was added
in this review; no unsupported supplier API is assumed available.

The current capture replacement exposes one `BackupConnection`, bounded
`BackupSigner` and action-aware `BackupApplicationGuard`. Exact original
request/certificate/HTTP checks are private shared primitives. The public
`advance` loop derives calls; `inspect` authenticates original progress without
effects. Explicit reply import and one original-request observation preserve
separate historical windows and spending. The
[capture contract](../architecture/capture-intent.md) owns these guarantees.

Actual PocketIC qualification covers the complete multi-extent capture, target
succession, full older-source restore, capture after restore, interrupted
allocation/upload/load and take/data recovery, plus offline
replay with a direct controller fixture key. Canic still owns its actual signer
process, manager-controlled execution, continuous maintenance and external
settlement. [Canic #1](https://github.com/dragginzgame/canic/issues/1) remains the
integration umbrella. Its implementation owners are
[Canic #5 signing](https://github.com/dragginzgame/canic/issues/5),
[Canic #6 manager execution and original-result lookup](https://github.com/dragginzgame/canic/issues/6)
and [Canic #7 fenced application recovery](https://github.com/dragginzgame/canic/issues/7).
These issues distinguish completed supplier prerequisites and locally qualified
Backup from Canic's actual implementation/qualification. No Canic execution enum
promises an unsupported route.

For live original-request recovery,
[Platform #167 — Add bounded strict read-state content decoding for signed recovery lookups](https://github.com/dragginzgame/ic-platform/issues/167)
is resolved by adopted Platform 0.33.7, source
`663cf913997490f448553119a6caf66b3cdecc81`. Its strict read-state content reader
preserves the existing envelope identities and nested path bytes, refusing unknown
or duplicate content fields and malformed/oversized input. The bound is raw input,
not a total work/allocation quota. Backup still owns exact request-status paths,
signature/binding checks, finite observation accounting, timing, retention and
settlement; the direct-host workflow is now qualified with fixture authority. No supplier edits or local
wire schema were added.

The subsequent coordinated adoption review found a remaining construction gap in
that same 0.34.7 archive: `ingress` has no public bounded encoder for an externally
signed envelope. The unchanged `qualification/ic-backup-consumer/src/authority/mod.rs`
needs both Call and ReadState before Backup records its original bytes. Platform's
`native::Ed25519Identity::prepare_update` covers calls only; its
`ReadClient::observe_request` creates and sends a read-state internally and adds
a time path, bypassing the required original custody boundary. The locked offline
compile refusal is retained in `target/evidence/platform-adoption/encoder-probe.log`.
[Platform #12](https://github.com/dragginzgame/ic-platform/issues/12#issuecomment-6928914126)
now records that exact missing capability and acceptance. The successful decoder
trial does not establish complete API availability. Both maintained graphs remain
on the working old cohort; no consumer CBOR codec or sibling patch was introduced.

The focused review also re-read frozen legacy metadata `wire/mod.rs` and
`tests/mod.rs` at `4ae087fad8c23de712aa60bf569bde4b2b932651`, and its migration history
`1af7e219c3073395992c316ecaa5e83e74b1aecc`. Separate allocation/work/skipping limits,
optional distinctions, exact floating-point bits and v128 bounds remain required.
New Protocol owns those decoders; Backup retains capture/restore policy and
original evidence. Existing independent golden vectors remain the regression
source; no legacy implementation or historical pass was imported as qualification.

## Manager exchange proposal — 2026-11-17

Status: Backup-owned semantic proposal for
[Backup #1 — Manager-route admission and retained-result replay](https://github.com/dragginzgame/ic-backup/issues/1),
for agreement with [Canic #6 — Manager executor and original-result lookup](https://github.com/dragginzgame/canic/issues/6).
This is not an implemented route, a frozen wire ABI or actual-IC qualification.
It supplies the concrete next contract before either repository adds effects.

### Current evidence and ownership

Backup 0.105.3 at `709d8eeb686c62d8181b6a0b00cb51252df6f5e4` binds
`CaptureRequest::sender`, direct envelope authentication, controller policy and
certificate target to one management caller/target pair. `operation/runtime`,
`operation/intent` and the fixed capture/restore procedures have no manager join.
Repurposing `sender` would conflate the host signer with the controlling manager.
Existing direct intent/receipt bytes must continue to select their original reader.

Read-only Canic review used base `d652c55e4cc455a642f44645bf2e16a6d08baefc`
and its active working source. `canic-management-protocol/src/lib.rs` exposes
bounded staging/status/reconciliation for creation and upgrades, with no Backup
snapshot exchange. Engine creation/status checks still select the manager as sole
child controller; upgrade observation is separate from dispatch. The inspected
engine files are dirty relative to that base, with these SHA-256 identities:

| `canic-management/src/engine/` file | Reviewed working bytes |
| --- | --- |
| `mod.rs` | `473ef3545e8cc310bfc14e480e0d24102d441ec2850af532d91aede0e50e4f30` |
| `upgrade.rs` | `b43a43000b1fc1cc0709088d76ead8a9a54040ddb294106edf5a3fb6087b071f` |
| `delivery.rs` | `0e031124d09cc8526048578c49ca4f41f060316596900646576e49f1e8fe552e` |

These observations do not claim committed supplier support. No sibling file was
edited. Canic owns its endpoint schema, handler, stable effect ledger and adapter;
Backup owns host admission, original evidence and fixed procedure interpretation.
The new owned endpoint exchange uses the `ic-wire` application boundary; inner
management bytes remain published Platform DTO/Candid bytes. Final schema,
method/endpoint identifiers and a portable dependency arrangement must be agreed
before coding. Backup must acquire no Canic runtime/control-plane dependency,
key/process owner, generic workflow callback or private management DTO copy.

### Immutable identity and one effect slot

An approved manager route must bind network/trust root, exact manager principal,
protocol revision, provisioned manager implementation/authority policy, original
surviving authority generation and application witness. Host envelope signer and
inner management caller are separate: the latter is exactly the manager principal.
The target remains the existing child, with its original release/module and source.
Changing any route authority requires a new approval, never rebinding old work.

The following is the semantic request record, not a Serde representation:

| Binding | Required meaning |
| --- | --- |
| Operation | Capture/restore kind, original 16-byte operation ID and exact 32-byte Backup approval digest. Restore approval already binds its source. |
| Step | Closed capture/restore step tag and ordinal where applicable; derived by the fixed procedure, never supplied as arbitrary dispatch authority. |
| Route | Network identity, manager principal, protocol revision and original authority generation. |
| Effect | Existing child principal, allowed management method, complete argument length, SHA-256 and original raw argument bytes. |
| Allowance | Absolute last inner-dispatch time, one inner-call reservation, finite cycles, reply/retention bounds and other immutable manager liabilities. |

The manager first pins one approval/target/route binding per
`(network, operation kind, operation ID)`, then one record per fixed step/ordinal.
Generation, approval or argument differences are conflicts, not new lookup keys.
This prevents a changed generation or digest from opening a second slot for the
same original operation. There is no attempt counter that authorizes retries.
A separately approved new operation has a new ID and cannot adopt an unresolved
old target obligation. The manager authenticates the actual caller and validates
membership/controller authority before accepting new work.

The manager checks argument length/digest and the selected method's DTO/target,
then reserves its liabilities before dispatch. All admissions that can change
across awaits are rechecked at dispatch. `sender_canister_version`, if present,
means the executing manager's version, never the child's version; its exact
original argument bytes cannot be rewritten at continuation. Unknown methods,
steps, schemas and oversized inputs refuse before effects.

### Execute, lookup and terminal evidence

Use two finite semantic operations: execute the exact original slot once, and
lookup the retained record for that same binding. Names/endpoint IDs are pending
Canic agreement. Lookup is an authenticated update producing an ingress-certified
reply; an ordinary uncertified query is not recovery evidence. Lookup never issues
an inner management call, repairs a missing record or refreshes old authority.
Duplicate execute returns the original record or a conflict; it never redispatches.
Backup itself still sends its original execute only once.

| Manager record | Backup interpretation |
| --- | --- |
| No record / unavailable | Unresolved. Neither absence nor ingress expiry proves no inner effect. |
| Reserved / dispatched / continuation lost | Pending with original spending retained. No second management call. |
| Completed reply | Exact original bounded raw management reply plus its complete immutable binding. Only this payload enters the owning management decoder/policy. |
| Completed management rejection | Exact original bounded management rejection code/message and binding; terminal rejection, no refund or retry. |
| Refused before inner dispatch | Distinct retained manager refusal with no issued inner call. It blocks the operation; it is not a fabricated management rejection or retry grant. |

A callback trap, lost continuation, upgrade, oversized unretained result or failed
result persistence leaves the inner effect unresolved. Terminal success/rejection
requires durable original outcome custody; uncertainty is not turned into a
success/refusal flag. A completed record cannot regress or change bytes. The
manager's ledger remains outside the restored child and survives manager upgrades;
rollback/replacement of that ledger is outside this profile.

Backup authenticates the original outer envelope and its IC certificate against
the **manager** principal and original root/time policy. It then strictly decodes
the bounded manager record and compares every binding above against its own
derived step and arguments. The original outer request ID lives in host evidence:
putting it in its own signed arguments would create a circular request-ID binding.
A lookup's distinct outer request ID is retained separately; matching the original
logical slot and argument commitment correlates its result to execute.

Outer success is only a manager reply, not management success. Crucially, outer
**rejection** also does not establish inner rejection: the handler may have issued
an inner effect before an outer callback/trap failure. Both cases require the
retained inner record or remain unresolved. Backup must not feed the outer result
flag directly into the existing direct-call rejection policy.

An IC certificate authenticates the manager's statement, not an independently
certified management response or proof of the manager's implementation correctness.
The integration must provision trust in that manager implementation and surviving
ledger policy. Protocol/generation fields alone cannot establish that trust. Fresh
application authority is required for dependent effects; retained completed replay
continues to use original evidence without requiring a live signer or guard.

### Finite recovery and bytes

Separate three recovery facts: original execute ingress status, manager inner
outcome lookup, and any original lookup-ingress status. They have distinct original
requests, timing and spending. Propose at most one explicit manager lookup per
pending step for the initial profile, with at most one original-request status
observation per admitted outer request. Missing replies remain unresolved after
those slots are spent. No hidden polling, fresh-lookup loop or renewal is permitted.

Approve and reserve host signing/transport/observation liabilities separately from
manager inner-call/cycle/result liabilities. Each execute includes one host hop
and one manager-to-management hop; a manager lookup adds a host hop but zero inner
calls. Signer-service network use is also finite. Lookup success never refunds
execute or manager spending. Manager dispatch uses its retained original deadline;
expired or revoked work can expose retained evidence but cannot dispatch anew.
For the initial proposal, the inner-dispatch deadline equals the exact original
execute-ingress expiry; host admission checks that equality before signing.

Encode bounds must include both the complete outer protocol frame and inner raw
Candid result, request signatures/certificates, decoded allocations, journals and
manager retention. Existing 2 MiB request / 4 MiB HTTP result / 2 MiB certificate
ceilings are independent caps, not a guarantee that arbitrary nested replies fit.
Keep the 1 MiB extent cap and 8,192 total request ceiling; derive two-hop counts
and reject an unrepresentable manager plan. No automatic extent reduction or
profile enlargement occurs during recovery. The direct route's 20 MiB evidence
does not qualify this manager route's capacity.
Canic's inspected general observation limit is 4 KiB, and its artifact staging
limit is 512 KiB plus framing. Neither admits a full 1 MiB snapshot-data result;
the snapshot protocol needs its own explicit bounds, not reuse of staging limits.

### Implementation seam and acceptance

Once the exchange and actual Canic consumer are fixed, extend the existing host
recorder's admission/projection for the explicit manager profile. Keep one public
advance/reconcile loop and the same capture/restore policy, schedule, custody and
publication owners. New route-aware intent/evidence needs an explicit format
boundary: old direct records must never be reinterpreted or migrated on open.
No new public phase commands, dynamic executor registry or second recovery engine.

| Required case | Owning proof |
| --- | --- |
| Host signer differs from sole child controller | Actual manager/child capture/restore; child status must admit manager, while outer signing admits the host. |
| Same slot, changed target/method/arguments/digest/generation | Manager refuses conflict and Backup rejects mismatched certified record; zero additional inner dispatches. |
| Lost execute outer reply, including certified outer rejection after inner dispatch | Explicit lookup imports the original inner result; no resubmission or inferred failure. |
| Lost lookup reply / lookup-ingress observation | Original lookup evidence and its own finite slots survive host restart; no new lookup allowance. |
| Callback trap or manager upgrade while inner outcome is pending | Surviving ledger cannot reissue; terminal original bytes remain importable if retained. |
| Completed record rewritten/regressed or unknown schema | Bounded strict refusal with all original evidence retained. |
| Late/revoked/generation-changed dependent work | No new effect; original completed offline replay still authenticates its own evidence. |
| Result/frame/cycle/request/evidence ceiling reached | Refuse before the next effect, or retain uncertainty after an already-issued effect; no enlarged budget. |
| Full normal consumer path | Real IC take/read, allocate/upload/readback/load, stopped completion, offline replay and direct-route regressions. |

The first executable qualification belongs with Canic's real manager/worker and
published protocol, not a native fake claiming IC effects. Canic #7 separately
owns the continuous fence, stop/start-fenced/reconcile/accept/open lifecycle.
Until that supplier exists, this review does not add an unsupported production
route enum or a success-returning adapter stub. Backup #1 remains open.

Focused legacy review used frozen `4ae087fad8c23de712aa60bf569bde4b2b932651`:
`ic-backup-agent/src/submission/mod.rs` retains the original envelope before one
dispatch and treats failed reply retention as indeterminate. Attempt-journal
observation and persistence tests retain exact request/authority/limits across
reopen and uncertain observation. Relevant submission history is
`1af7e219c3073395992c316ecaa5e83e74b1aecc` and
`278016b369f4312b4311410bbb7a0ad770ba3de6`. Reuse those invariants; the new
assumption is a second effect owner whose inner outcome is not the outer ingress
outcome. Legacy's direct transport cannot supply that missing proof. No code or
legacy recovery artifacts were copied, and no new qualification is claimed here.
