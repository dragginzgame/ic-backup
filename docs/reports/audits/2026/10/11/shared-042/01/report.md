# Shared Tooling 0.4.2 consumer adoption

Adopt committed `8db24059613c63ba9be36d4225fb74f598f49df2` through the canonical
clean exporter, with the same 96 selected paths. Released Backup is 0.18.1 at
`7e9eaf2da8a6e89ba790210d895effc1a9b0898d`; compatible pending notes select 0.18.2.

Common Cargo tools use the verified selected-tool slot owner and activate only a
complete set at `.tools/rust/common/bin`. CI, hooks, shell guidance and failure
archives use that contract. Contributors must explicitly run
`make install-rust-tools`, then `make rust-tools-check`, and update terminal PATH.
Old installations, receipts, builds and failed attempts remain retained.

The 0.4.2 policy assigns future per-user executable storage to Shared Tooling.
That store is not implemented; [upstream #120](https://github.com/dragginzgame/shared-tooling/issues/120)
owns its implementation and qualification. Current canonical `.tools` paths remain;
no consumer store, relocation or sibling PATH is introduced.

The full 16-target delivery gate passes on frozen intermediate 0.4.1 inputs,
including strict Clippy, actual simulator journeys, Rust 1.88 consumers and paired
package builds. Its executable/product inputs are identical in 0.4.2; only two
policy documents and the snapshot identity advance. Affected snapshot,
documentation and consumer-tooling gates pass again. Explicit complete setup,
offline admission and workflow lint pass. Both actual archive collectors preserve
exact slot/receipt/failure bytes and links, exclude redirected roots and never
follow external links. Detailed case/source inventories remain ignored.

Host Artifact/FS 0.13.1, Testkit 0.33.4, Metrics 0.6.1 and Agent 0.49.2 retain their
incoming locked selections. Published Rust sources, toolchain and the real index
are unchanged. No package-version, commit, push, release, production IC or sibling
write occurs. Matching native macOS candidate acceptance remains pending.
See [the compact proof](qualification.json); original logs remain under
`target/shared-041` and `target/shared-042`.
