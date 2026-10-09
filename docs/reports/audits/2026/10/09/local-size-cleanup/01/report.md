# Local persistence size-check cleanup — 2026-10-09

Remove nine private forwarding helpers in the pending 0.11.0 draft. Each only
passed its record and model-owned byte limit to `json::check_json_size`. Call that
canonical checker directly at the original admission sites, including download
manifest replay and fresh integrity verification. The model still owns each
limit; no new helper, generic registry, branch or state transition is introduced.

| Flow | Owner and retained contract |
| --- | --- |
| Byte limits | Existing model constants; exact original values reused at each call |
| Pretty-JSON size admission | Existing bounded `json::check_json_size` serializer and typed errors |
| Create/read/transition callers | Same check sites, lock/effect order, rejection and recovery behavior |

Removed functions, all replaced by direct `json::check_json_size` calls:

- `ops::persistence::attempt_journal::check_size` — `crates/ic-backup/src/ops/persistence/attempt_journal/mod.rs`
- `ops::persistence::consistency::check_size` — `crates/ic-backup/src/ops/persistence/consistency/mod.rs`
- `ops::persistence::download_journal::check_size` — `crates/ic-backup/src/ops/persistence/download_journal/mod.rs`
- `ops::persistence::effect_graph::check_size` — `crates/ic-backup/src/ops/persistence/effect_graph/mod.rs`
- `ops::persistence::execution_settlement::check_size` — `crates/ic-backup/src/ops/persistence/execution_settlement/mod.rs`
- `ops::persistence::fence_obligation::check_size` — `crates/ic-backup/src/ops/persistence/fence_obligation/mod.rs`
- `ops::persistence::inventory::check_size` — `crates/ic-backup/src/ops/persistence/inventory/mod.rs`
- `ops::persistence::operation_plan::check_size` — `crates/ic-backup/src/ops/persistence/operation_plan/mod.rs`
- `ops::persistence::restore_safety::check_size` — `crates/ic-backup/src/ops/persistence/restore_safety/mod.rs`

Keep independently protective identity, retained-byte, journal/layout, pre/post
and serialization checks. Record-only stage inspection and complete resume have
distinct recovery obligations and are retained. Public functions, record schemas,
exposed errors and original spending semantics are unchanged. This is an internal
indirection cleanup, not a performance claim or a whole-repository audit verdict.

All 151 persistence cases pass, including original quota refusal, no-follow IO,
lock contention, lost publication, journals and stage history. Both-library
warnings-denied Clippy and Rust 1.88 all-target/all-feature checks pass, as do
formatting and exact vendored snapshot verification. No new mirrored tests,
full CI/release gate or simulator run is needed for this internal delegation.
[The source-bound evidence](qualification.json) retains commands, unchanged graph
identity and the removed-function inventory. All work remains uncommitted with
package versions/receipt at 0.10.1; prior dirty work and evidence remain retained.
