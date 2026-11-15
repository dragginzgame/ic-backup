# Backup restart

Status: local inventory, payload integrity, capture intent, transfer planning
and bounded reply decoding are implemented, together with direct-host status/list/take
and settlement from surviving certified replies. Full capture/restore remains proposed in the
[library design](library-design.md); the [current handoff](../status/current.md)
links each implemented contract.
The user selected a single-canister library first: capture one already-stopped
canister and restore to the same existing ID, leaving application coordination
and restart to integrations. The [operation contract](operation-contract.md) refines API, transfer and
provider semantics. The [local-store contract](local-store.md) selects local
bundles, backup history and independent concurrent targets for v1. Canic owns
concurrency limits and fleet consistency; Blob integration is v2. Record field
encodings and provider qualification remain implementation prerequisites.

Start with a standalone host-side library and a narrow vertical slice. Canic is
a required integration alongside independent consumers. Design its adapter
boundary with the standalone slice; the [sibling review](sibling-integration.md)
records current signing, controller topology and supplier constraints.

Before implementation, turn the selected scope into exact input/output contracts,
management and application authority, bounds, custody, crash reconciliation and
acceptance evidence. Do not recreate the legacy feature set or release machinery
by default. A CLI, Agent wrapper, persisted schema and simulator workspace are
not required merely to establish the library. The implemented inventory has a
bounded manifest reader, initial operation records and a pure transfer planner.
The direct provider and private PocketIC qualification cover status, snapshot
list/take, surviving reply settlement and offline replay. Native injected providers
prove persistence ordering only; complete download and restore remain unfinished.

The previous library release is 0.67.2. The user selected 0.100.0 for the
incompatible rewrite, superseding the unpublished 0.68.0 draft. This separates
the new implementation from legacy without implying maturity or compatibility.
The GitHub-only 0.101.2 checkpoint includes the direct provider and durable snapshot
capture. The 0.101.3 GitHub checkpoint adds retained-reply recovery and local
lookup/certificate authentication with the reviewed dependency refresh.
Local unpublished 0.101.4 now retains one durable recovery lookup per operation.
Terminal retained observations now settle locally with distinct timing and
effect-free replay; dependent actions still require live application admission.
The 0.101 minor accounts for Host 0.79 public Rust type identities. The pending
patch opens compatible development; it does not claim a finished capture/restore
product or pre-authorize a breaking API change.
Publication remains disabled; no registry name/version is reserved by a local
manifest. Legacy backups and unfinished operations have no automatic migration.

The [legacy repository](https://github.com/dragginzgame/ic-backup-legacy) retains
history. Preserve any original local journals and recovery evidence; the former
`/home/adam/projects/ic-backup-legacy` path was absent during this review. Existing consumers must use their original qualified
integration until an explicit migration is designed. Do not delete artifacts,
rewind external obligations or infer compatibility from matching package names.

The current native tests cover local filesystem and completion-record behavior.
They do not qualify IC effects; the private PocketIC cases qualify only their
named management and recovery boundaries. End-to-end backup/restore, Canic
integration and release/publication acceptance remain contract-specific work.
