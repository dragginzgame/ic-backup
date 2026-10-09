# 0.10.2 tooling and published-package follow-up

Preserve the pending 0.10.2 implementation after released 0.10.1
`5c4bd9df50d2cf4f954a236ab9b155778b33d645`. Canonically refresh the unchanged
95-file selection from committed Shared Tooling 0.1.37 to the 0.1.38 fix
`926a20606591214ab29faa236b0b584e4857439e`. Three selected files change; all bytes
and modes match a clean isolated committed checkout. The sibling's dirty VERSION
preparation is excluded. No vendored helper is patched or symbol removed.

[Shared #86](https://github.com/dragginzgame/shared-tooling/issues/86) closes a
multi-document exception-catalog bypass: admission and suppression require one
validated JSON array. Fixtures check conflicting first/last documents, malformed
streams and valid unchanged catalogs, preserving original manifest, lock, exception
bytes and private index. Real consumer declaration/snapshot checks and ShellCheck pass.

The incoming lock first selected Metrics 0.2.20/Testkit 0.25.5 with Host 0.8.9,
then independently moved all four Host packages to 0.8.10. Preserve both graph
identities and their evidence; the agent performs no dependency reselection.
Locked cache preparation supplies missing inputs. The first final-graph Clippy
attempt lacked Host 0.8.10 cache entries and remains retained separately.

Crates.io's summary API refused requests with HTTP 403; its official sparse index
and static archives supplied independent published-version/checksum evidence.
Metrics 0.2.20 and Testkit 0.25.5 archives match registry checksums and all 75 Rust/
original-manifest blobs at their clean committed VCS identities. Host 0.8.10's four
archives likewise match registry checksums and 82 committed blobs; their Rust and
original member manifests equal 0.8.9. Metrics arithmetic is unchanged from 0.2.18.
These updates add no new Host runtime offload or Metrics accounting behavior.
Testkit 0.25.5 adds an optional operation-idle lifetime, keeping our existing managed
server defaults. Testkit's sibling 0.25.6 proposals are excluded.

On the final Host 0.8.10/Metrics 0.2.20/Testkit 0.25.5 graph, seventeen stage cases,
the public learned-ID journey, four diagnostics cases and all three real Testkit
download success/lost/malformed cases pass. The latter preserve original calls,
pending spending, partial artifacts and references. Remaining focused verification,
exact source/graph/log hashes and published archive proof are retained in
[qualification](qualification.json) and `target/updates-0102/`.

Upstream 0.1.38 CI is queued at inspection; released 0.10.1 runs are queued/in progress.
Neither supplies native qualification for this uncommitted candidate. #25/#29 keep
their broader installed transport/runner/application/restore/terminal acceptance.
Earlier 0.10.2 evidence remains on its actual original graph. Package versions and
receipt stay 0.10.1; no full gate, root Git write, sibling edit, registry publication
or production IC effect occurs.
