# Backup restart

Status: workspace scaffold with a [library design proposal](library-design.md).
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
not required to establish this scaffold.

The previous library release is 0.67.2. The new unpublished package starts at
0.68.0 because an incompatible replacement requires a minor increment before
1.0. No registry name/version is reserved by a local manifest.

The [legacy repository](https://github.com/dragginzgame/ic-backup-legacy) retains
history. Preserve any original local journals and recovery evidence; the former
`/home/adam/projects/ic-backup-legacy` path was absent during this review. Existing consumers must use their original qualified
integration until an explicit migration is designed. Do not delete artifacts,
rewind external obligations or infer compatibility from matching package names.

The new scaffold has no product behavior to qualify. Compilation and documentation
checks prove the foundation only; native IC effects, public consumer usability
and release/publication acceptance remain future contract-specific work.
