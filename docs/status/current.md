# Current handoff — 2026-10-04

The maintainer completed `0.1.3` at `9cc4be7` (Release 0.1.3), requested a
`0.1.4` changelog draft and continued extraction. This session extends the retained
uncommitted inventory/selection batch with explicit operation dependency graphs,
deterministic planning order and pure causal declared-progress policy. Source,
regressions, schemas and documentation remain uncommitted for review. Canic and
siblings remain read-only. No live IC effects, release/version transactions,
commits, pushes or package uploads ran.

`EffectGraphRecord` retains v1 unique opaque u64 operation sequences and sorted
explicit prerequisites. Zero, nonconsecutive identities and u64::MAX are admitted;
sequences are not array indices or numeric dispatch order. Bounds are 8,192 nodes,
1,024 direct prerequisites per node and 65,536 total edges. Decode bounds aggregate
edge retention while parsing. Duplicate/missing prerequisites and self/longer cycles
reject. Iterative topological validation admits a maximum chain depth of 8,191;
smallest currently ready sequence wins ties. Computed order is private cached
projection and never serialized. A domain-separated binary digest binds exact
canonical identities/edges with u32 counts and u64 big-endian sequence values.

`policy::effect_order::readiness` admits caller-declared completed identities only
when bounded, unique, known and causally closed over prerequisites. Read-only views
bind the original graph digest and project incomplete ready/blocked nodes in
planning order. Policy performs no IO, record writes, scheduling or mutations.
Declared completion is not a qualified receipt; ready is not fresh authority,
spending allowance, custody, external-effect settlement or terminal completion.
Application dependency semantics must be explicitly qualified by integrations;
parent-before-child or reverse restart order is never inferred from inventories.
The graph is one component of a future complete plan, without operation payloads,
target/request/network/release binding, budgets, actual receipts or runner wiring.

Persistence ops durably create fixed `effect-graph.json` under layout/journal
exclusion without replacing evidence. Local reads require the validated original
graph and exact expected digest. Lost creation replies reconcile by reading that
exact declaration, without repair or progress reset. Encoded input and canonical
pretty-output each admit at most 1 MiB; dense valid graphs exceeding canonical
output reject before publication. Unsafe entries and replaced roots reject.
See [the maintained boundary](../extraction-boundary.md) and
[the v1 graph schema](../contracts/effect-graph.schema.json).

The prior uncommitted `InventoryRecord`/pure selection batch remains intact.
Physical forests admit 1–1,024 unique canonical principals, closed acyclic parent
edges and required nullable parent/role/module fields. Role text admits 256 UTF-8
bytes; null, empty and literal "null" remain distinct. Its unambiguous binary hash
binds declared fields only. Exact/DirectChildren/Descendants selection retains
original parent links and full inventory digest, including unselected targets.
Immutable `inventory.json` publication preserves prior evidence and has independent
1 MiB input/output bounds. Matching inventories prove no fresh membership,
controller authority, revision continuity or application consistency. See
[the inventory schema](../contracts/inventory.schema.json).

Earlier qualified owners remain intact: artifact checksums/staging/publication,
bounded JSON and journal locks, stable layout exclusion and conservative restore
dependencies, inherited command custody, exact local download lifecycle and finite
per-operation mutation/observation accounting. Qualified receipts and backend
complete-transfer evidence remain caller-owned. Lost paid responses block blind
retries; budgets never refund/replenish. Uncertain requires a qualified settled
observation unable to resolve mutation; a lost observation reply stays pending.
Local replay performs no remote calls. Durable artifact progress, Applied operation
evidence and dependency readiness do not establish full terminal completion or
permit restore-reference release.

Fresh targeted Linux checks passed: 93 unit tests and five public-API integration
journeys, warning-denied Clippy/rustdoc, formatting, Rust 1.91.0 all-target/all-feature
compilation and standalone Cargo package verification. Adapted ordering cases
exercise explicitly qualified parent-first/child-first edges. Fresh regressions
cover canonical golden bytes, changed identities/edges, deterministic ties, causal
progress, malformed schemas/cache injection, cycles/closure, maximum chain/count,
exact direct/aggregate edge bounds, independently bounded IO, unchanged bytes,
contention and unsafe/replaced paths. A public journey reopens the original graph
and projects declared progress locally. Existing inventory, artifact/download,
attempt, layout and real process/custody regressions also passed. Machine schemas,
required fields/examples and independent binary golden encodings validate.
This is native graph/declaration/filesystem/process qualification only; no
PocketIC, live authority, actual completion proof or IC backend was used.
No broad gate ran.

Fresh logs are `target/effect-graph-tests.log`, `target/effect-graph-clippy.log`,
`target/effect-graph-msrv.log`, `target/effect-graph-docs.log` and
`target/effect-graph-package.log`. Current verified source/archive remain under
`target/package/`; prior package evidence is retained under
`target/effect-graph-package-evidence.asr33i_v/`,
`target/inventory-package-evidence.a7jne1oj/`,
`target/attempt-journal-package-evidence.kqp9ti8e/` and
`target/download-journal-package-evidence.45iah_5j/`. Earlier logs, recovery fixtures
and release evidence remain retained. The member license still archives exact
regular MIT contributor notices from the maintained root file.

[Graph provenance](../effect-graph-source.json) records 12 inspected Canic files,
44 exact consumer references and dispositions for full backup phase projections,
restore application ordering and CLI resume/reporting. Canic HEAD was
`3978e02d28fea9022c7a6e84a7ec6d6e4a0d4af3` with dirty working-tree source at
inspection; hashes identify inspected files/sections, not qualification. No
consumer or source was changed/removed. [Inventory provenance](../inventory-source.json)
retains 10 inspected files and 48 consumer references. Earlier
[planning](../source-baseline.json), [artifact](../extraction-source.json),
[layout](../layout-source.json), [command custody](../command-custody-source.json),
[download](../download-journal-source.json) and [attempt](../attempt-journal-source.json)
provenance remain unchanged.

Cargo remains `0.1.3`, Rust 2024, development Rust 1.99.0 and MSRV 1.91.0.
No dependencies/lockfile changes, Canic imports, sibling patches, unsafe Rust or
shared target were introduced. The sole populated changelog draft is undated
`0.1.4` beneath empty Unreleased. Dated `0.1.3`, older changelog history and
`docs/release.json` remain unchanged. Maintainers own release preparation; see
[development](../development.md) and [releasing](../releasing.md).

Full B1/B2 completion and independently usable canister backup/restore remain
unestablished. Authoritative membership/revision/fence ports, complete authenticated
plans and exact operation payload/budget bindings still precede runner extraction.
Fresh authority/consistency, full backend metadata/transfer extents and bounded
executor calls/lost create-upload-load reconciliation remain integration/port work.
Full execution/restore journals, completion manifests, terminal reference release,
prune, transport and CLI remain proposed. Canic's backup executor preflight still
rejects. Read [the design](../extraction-design.md) for the maintained sequence.
Canic adoption/live IC effects need their own instructions; the no-commit rule remains.
