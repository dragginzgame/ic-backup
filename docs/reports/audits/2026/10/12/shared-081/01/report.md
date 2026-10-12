# Shared Tooling 0.8.1 focused consumer adoption

The uncommitted Backup 0.20.1 draft now selects committed Shared Tooling 0.8.1
`c97f9f9c7b7c706fffdcfc5925cf58759b58093f`. The source was checked out
cleanly in isolation with the recorded GitHub remote. Canonical preview and
export retained all 102 existing entrypoints and payloads, updated five selected
files, and retired none. The updated files are `docs/consuming-snapshots.md`,
`docs/local-setup.md`, `scripts/ci/verify-shared-tooling-snapshot.sh`,
`scripts/dev/install-rust-tools.sh` and `scripts/ci/test-rust-tools.sh`.
The manifest digest is
`a21649b448d246539dc01353bd8c2c99ab1bbd57dfc2459dc88e11c2363b88e4`.

The selected verifier now rejects malformed exact tab-separated snapshot
records. A separate copy of the real manifest with a trailing tab in its
`format` record was rejected as malformed; the real 102-file snapshot verified.
The Rust-tool fixture exercised changed compiler, native environment and policy
identities and passed, including retained refusal evidence. The existing
`cargo-sort-derives` 0.14.1 pin and admitted selection remain valid. The
incoming Host 0.14.5, Testkit 0.34.3 and Metrics 0.7.1 graph is unchanged;
`Cargo.lock`, `ci/tool-versions.env` and root `Cargo.toml` match their
pre-export hashes. Package versions remain 0.20.0 and no Backup runtime or
public contract changes follow from this snapshot.

Focused checks passed: `make shared-tooling-check`,
`bash scripts/ci/test-rust-tools.sh`, `bash scripts/ci/test-tooling.sh`,
`make tools-check`, `make check-doc-links`, `make shell-check`, and
`git diff --check`. Preview, export and check logs are retained under ignored
`target/shared-081/`. At the maintainer's request, the full `make ci` gate was
**not** rerun for this 0.8.1 candidate. The earlier
[0.8.0 review](../../continuation-0201/02/report.md) owns its own complete
16-target gate and does not qualify these new source bytes. This uncommitted
candidate has no hosted/native result. Backup
[#36](https://github.com/dragginzgame/ic-backup/issues/36) remains open for
exact consumer native acceptance.
