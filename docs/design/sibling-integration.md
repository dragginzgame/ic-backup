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

| Dependency | Proposed owner/use | Selection rule |
| --- | --- | --- |
| `ic-platform` | Backup's external management DTOs and bounded Candid encoding/decoding; Agent adapter's transport/signing/certificate interfaces. | `default-features = false`; select `management-canister-types` and `candid-codec` at the management boundary, `agent` only in the native adapter, and `ic-protocol` when inspecting signed envelopes/read-state messages. |
| `ic-platform-types` | Canonical Principal/Account owner. | Platform reexports the same Principal. A future pure identity-only package may depend on Types directly; do not add a redundant dependency to a package already using Platform merely to duplicate that import. |
| `ic-host-fs` | Private parents, no-follow reads, retained files, descriptor locks, durable publication and identity checks. | Direct Core storage dependency; use `capability` only if the selected implementation actually needs its held-directory API. |
| `ic-host-artifacts` | `Sha256Digest`, streaming `HashingWriter`/`BoundedWriter`, bounded byte copying and directory checksum composition. | Direct dependency where Backup uses these APIs. No gzip/Wasm features for opaque snapshot transfer. |
| `ic-host-http` | Selected 0.79.0: bounded complete-body consumption and Reqwest re-export for direct status HTTP. | Enable `rustls`; import Reqwest only through Host HTTP. Backup owns endpoints, retries, trust, deadlines and IC evidence. |
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

The public management API permits canister-originated snapshot operations; the
[IC management specification](https://docs.internetcomputer.org/references/ic-interface-spec/management-canister/)
is the protocol reference. However, Platform's current take/load/read/upload DTOs
are exported only under `not(target_family = "wasm")`. A typed Canic manager
executor needs a Platform-owned, Wasm-usable snapshot DTO/codec surface, qualified
without native Agent dependencies. This is a concrete supplier gap to resolve
before implementing that route, not permission to duplicate those DTOs in Canic.

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
3. Resolve the Platform Wasm DTO boundary and implement the Canic-owned executor
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

The adjacent local recovery gap is now closed by `inspect_status`: a restarted
caller can recover original admission and conservative reserved counts from the
complete original ledger. Re-authored native tests cover every cold stage, failed
status/rejection without refunds, malformed evidence and continuation using the
recovered admission. This retains legacy's progress-view guarantee and the
[a860d144 separation](https://github.com/dragginzgame/ic-backup-legacy/commit/a860d14421b9ff1c61c1dce079cdc6a19316bfd0)
between mandatory original admission and optional reporting. Views do not verify
certificates or grant dispatch authority.

The direct-key continuation supplies `authenticate_status_request` and applies it
before HTTP dispatch. Platform 0.33.4 already supplies the required Ed25519/K256
primitives, request domains and self-authenticating Principal derivation; no
supplier change or issue is needed for this initial profile. Exact standard
Ed25519/secp256k1 keys are supported; delegated, canister and other identity forms
refuse. This does not qualify the Canic sign-only process or application custody.
External preparation and live observation/application admission remain the next
composition work. The [owning contract](../architecture/capture-intent.md#direct-key-status-request-authentication)
records the focused legacy review, changed assumptions and native regressions.
