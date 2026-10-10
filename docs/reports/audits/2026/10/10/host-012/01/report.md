# Host 0.12 and Testkit 0.31 qualification

Preserve the incoming direct registry Artifact/FS 0.12.0 and Testkit 0.31.0
requirements and lockfile exactly. Released Backup base remains 0.13.0 at
`bf5138ec99c2f02650b8af618fc30195ef50dada`; package/receipt stay 0.13.0.
The existing 0.14.0 draft also covers the exposed Host publication-error Rust
identity change. Consumers sharing `PersistenceError::Publication` must align
their direct Host dependency to 0.12. No compatibility adapter is introduced.

Verified upstream release tags and GitHub main resolve to Host
`1ba4591868a83b367d56bae9e3c213418c66a192` and Testkit
`f1ae9e6d3b0f3f20ec1e1f1b49c8b1dea3155e0a`. Published archive hashes match
the selected lockfile and cached VCS evidence. All four published Host Rust source
trees match 0.11.0; Testkit's Rust source matches 0.30.0. These releases adopt
Shared Tooling 0.3.0's complete common-tool contract, already selected here at
`88a73139a0f083344c41a6f6f4b5c3a8aca7dc1d`. No new Rust API or redundant local
implementation was found to offload. Common executable pins remain unchanged.
Dirty sibling release/tooling changes are inspected separately and are not adopted.

Normal Backup consumers retain only Host Artifact/FS 0.12 with optional
archive/Wasm features disabled. Testkit 0.31 independently selects all four Host
0.11 crates in its test-only graph; preserve that published selection. Metrics
0.4.0, Agent 0.49.2 and Testkit's PocketIC 16.1.0 selection remain unchanged.
No sibling patch or direct production Process/Tools dependency is added.
Original records, allowances, private publication permissions and synchronized
publication barriers retain their existing owners.

Explicit canonical setup prepares the exact Testkit 0.31.0 CLI. Offline aggregate
admission passes. A standalone Rust 1.88 consumer directly converts and matches
Host 0.12 publication errors, verifying retained primary IO identity and displayed
secondary cleanup evidence without filesystem publication. Its first fixture
used the wrong cleanup field name; corrected public admission passes and the
initial compiler log remains retained separately. No production source fix follows.

The complete `make ci` delivery suite passes on this final graph: snapshot,
common/product tool admission, pins/links/fetch, shell/tooling/release/hook checks,
formatting, native compilation, strict Clippy, every unit/public integration and
simulator case, warning-denied docs, both independent Rust 1.88 consumers and both
packages. This includes 492 core unit cases, 22 core simulator journeys, eight
Agent HTTP cases and three Agent simulator journeys. Earlier Shared adoption
proof remains qualification of its original graph, without relabeling.

[qualification.json](qualification.json) binds exact technical inputs, selected
graph, CLI identity, archive evidence and log hashes. Incoming manifest/lock,
validated technical inputs and 41 captured older tool/receipt/server files remain
unchanged. Documentation-only completion receives snapshot/link/whitespace checks.
Evidence remains under `target/ic-tools-review/`, including the retained first
consumer rejection. No recovery evidence or earlier installation is removed.

Released 0.13.0 [CI run 38044109719](https://github.com/dragginzgame/ic-backup/actions/runs/38044109719)
passes Linux; Intel and Apple Silicon jobs remain queued at inspection. This
uncommitted 0.14.0 candidate has no hosted result. Native acceptance stays with
[#32](https://github.com/dragginzgame/ic-backup/issues/32) and
[#36](https://github.com/dragginzgame/ic-backup/issues/36). No root Git write,
release, registry publication, production IC effect or sibling edit ran.
