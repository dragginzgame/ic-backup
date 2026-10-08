# Direct Agent transport hard cut

The maintainer selected direct registry `ic-agent` on 2026-10-08. The uncommitted
0.8.0 draft adds actual `ic-backup-agent` preparation/single-update submission and
retires the ICP destination/probe. Package/receipt stay 0.7.0. Core v1 records,
original budgets, obligations, references and independent descriptor custody
remain unchanged. Agent initially preserved existing resolved packages. A later external workspace
update selected Host 0.6.0, crypto-common 0.1.6 and generic-array 0.14.9. Preserve
and freshly qualify that graph; do not relabel earlier 0.5.2 evidence.
[Host source proof](host-060-source-proof.json) verifies every published Rust/manifest
blob against official committed source and non-yanked archive checksums. Artifacts/FS
Rust are unchanged; Backup retains direct capture rather than new group capture.
[Earlier Host review](previous-host-052-review.json) remains historical.
A subsequent external update selected Host 0.7.0 and Metrics 0.2.12;
[final source proof](host-070-metrics-source-proof.json) and fresh consumer checks
own that actual graph. Prior 0.6.0 logs/review remain unchanged. `ic-host-process` is removed from
both manifests after confirming its sole caller was the retired ICP probe.
Final selected Host packages are artifacts/FS only; existing descriptor custody
and local bounded test capture retain their original owners. [Official API source proof](agent-api-source-proof.json)
retains exact 0.49.2 archive/source association, not a sibling patch.

[The maintained contract](../../../../../../../agent-transport.md) owns API, bounds,
trust and interruption semantics. There is no retry/fallback, mutable Agent escape,
automatic poll/root fetch or journal settlement. The transport accepts only exact
existing reserved request owners and returns passive verified replies or Pending.
Integrations retain durable original ingress and independently qualify actual
network trust, fresh permissions/application safety and never-dispatched custody.

Local actual HTTP boundary tests pass across eight controlled response/failure
conditions plus original context/reservation admission. Three actual PocketIC
Agent gateway cases pass: complete capture/metadata/1 MiB region transfers/upload/
same-ID restore through an actual Ed25519 controller, discarded stop response with no repeat, and wrong trusted-root
failure after the effect applied. This is actual transport/fixture evidence, not
an installed generic runner or Canic adapter. Native macOS and real production
identity/endpoint evidence remain separate. All retained old ICP evidence keeps
its original source/graph; removal does not erase unresolved effects.

Focused release fixtures pass with a real two-member preparation/receipt case:
both local lock versions and the internal registry requirement change together,
both manifests bind validation/receipt, selected source ignores corrupt working
metadata, and changed transport metadata rejects. Command effects use substitutes
and private index boundaries; no root Git or live publication runs. CI/package/
explicit publish commands select both libraries. [Qualification](qualification.json)
binds final source, graph, commands, logs and actual failures/corrections.

The frozen final graph passes 158 focused Rust cases and 61 release fixtures.
Both libraries pass Clippy and Rust 1.91; strict Agent rustdoc, both verified
package archives, formatting, documentation, dependency/snapshot and shell checks
pass. Metrics diagnostics/histogram cases run under the final 0.2.12 selection.
Exact-source Metrics owner CI remains queued; current consumer native macOS
acceptance still requires committed source. Earlier graph logs and archives stay
separate from this final qualification.

Removed functions from `crates/ic-backup/tests/pic_journey/icp/mod.rs` are
`retain`, `admit_tool`, `status_control`, `assert_pending_reopen` and
`generic_management_route`. Its caller
`pinned_icp_generic_management_route_failure_retains_pending_original_spending`
in `tests/pocketic_snapshot.rs` is also removed. These exclusively qualified the
retired CLI route; real Agent HTTP/gateway cases replace that maintained acceptance.
The now-unused `Backend::new_with_nns` is removed from
`tests/pic_journey/backend/mod.rs`; `Backend::new_with_network` is also removed by
inlining its sole remaining application-only constructor into `Backend::new`.
The independent core fixture retains its original network. No production core
function, method or type is removed. No sibling, Git, release,
version, publication or retained-evidence cleanup effect occurs.
