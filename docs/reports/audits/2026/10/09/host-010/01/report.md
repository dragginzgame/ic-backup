# Host 0.10 upgrade and 0.12.0 qualification

Released base: `6b5714aed1b3f69636b037fafebdee03b627a418` (0.11.5).
The requested Host upgrade extends the preceding undated 0.11.6 tooling batch.
Select 0.12.0 for the complete pending batch because the public
`PersistenceError::Publication` now exposes Host 0.10's Rust error identity.
Consumers directly sharing this value must use the same compatible Host line.
Package versions, internal package requirement and release receipt remain 0.11.5.

## Selected graph and API migration

The incoming manifest/lock initially select registry `ic-host-artifacts` and
`ic-host-fs` 0.10.0 with default features disabled. During qualification the new
0.10.1 patch is published and verified as the latest stable, unyanked Host release.
Run four narrowly scoped `cargo update --package NAME --precise 0.10.1` commands;
the concurrent incoming update already selects those versions, so no further
package changes are needed. Preserve the final manifest/snapshot and freeze the
selected lock for qualification. Archive checksums match registry and lock.
Published final source points to `c7bdc3d4e1c658957202eebd76bff2c51e22f645`.

Replace the retired non-Unix pathname `write_typed_with` call with canonical
`write_with(path, options, producer)`. Supported Unix publication keeps
`write_at_with` under its held directory descriptor. Original producer/cleanup/
visibility errors remain intact, with serialize-once-before-effects byte buffers,
private 0700 parents/0600 files, synchronized pre-publication staging and the
post-completion barrier. Record schemas, checksum framing, bounds, original
spending, obligations and references are unchanged. No compatibility wrapper or
new persistence owner is introduced and no local Rust symbol is removed.

Host's new generic lock publication errors do not move Backup's distinct
layout/journal/command-custody owners. Host 0.10.1 finishes synchronizing formerly
missing parents even when a competing
writer wins mkdir, before staging proceeds. This upstream durability fix is part
of the final selection. Review the
[published Host release notes](https://github.com/dragginzgame/ic-host-tooling/blob/c7bdc3d4e1c658957202eebd76bff2c51e22f645/CHANGELOG.md).

All four Host crates have a published 0.10.1 release. The initial phase retains
published Testkit 0.27.2's separate Host 0.9.7 test-only graph. During final review
an external manifest/lock update selects freshly published Testkit 0.28.0 and
Metrics 0.3.5, unifying all four Host crates at the verified patch selection 0.10.1.
Preserve both phases of inputs/evidence without broad reselection or sibling patches.
The exact new CLI is prepared and the final graph is independently requalified. Process/Tools remain
Testkit-only dev dependencies; normal consumers use Artifact/FS alone. Optional
gzip/archive/Wasm parser inputs remain absent from the core normal consumer,
as in the earlier qualified normal graph; this is no new footprint claim.

## Qualification

[qualification.json](qualification.json) binds incoming/final inputs, published
archives, source hashes and final logs under `target/host-upgrade-0120/final/`
with initial phase evidence retained in its parent. All 221 focused
artifact, persistence and workflow unit cases pass, including private publication,
crash barriers, cleanup evidence, locks and original journal invariants. Five core
and three Agent actual simulator cases pass. Warning-denied both-library tests
Clippy and rustdoc pass, as do both independent Rust 1.88 consumers.

An extra compile-only Rust 1.88 consumer directly depends on Host FS 0.10.1 and
Backup: its extracted public publication error is exactly the Host 0.10 type,
and the canonical pathname producer signature compiles. It executes no writer
and does not qualify unsupported Windows runtime behavior. Its selected packages
remain within the original graph; no simulator/process/tool owner is linked.
Explicit Testkit 0.28.0 CLI preparation and PocketIC 16.1.0 admission, formatter,
dependency pins, 94-file Shared 0.2.8 snapshot and local links pass on the final
selection. Preserve the older CLI selections and retained server/recovery evidence.

Earlier 0.11.6 tooling/Host 0.9.7 evidence remains at
`docs/reports/audits/2026/10/09/continuation-0116/01/`; it does not qualify Host 0.10.
No full workspace/CI/release gate, native macOS acceptance, root Git write, version
transaction, release, production IC effect, sibling edit or recovery cleanup ran.
The dirty candidate has no remote CI result. Keep pending native/product acceptance
with the existing owning issues; this dependency migration grants no runner,
permission, retry, new allowance, terminal or fence/reference-release authority.

Released 0.11.5's exact-source CI has Linux success and both macOS jobs queued at
final inspection: [37958355952](https://github.com/dragginzgame/ic-backup/actions/runs/37958355952).
This does not qualify the dirty 0.12.0 candidate. Preserve native acceptance with #32.
