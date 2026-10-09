# Selected Testkit 0.27.0 preparation — 2026-10-09

The reported release-validation failure was a missing selected Cargo CLI, not
failed server-byte admission. The manifest/lock had moved from Testkit 0.26.0 to
0.27.0; the old installed CLI could not satisfy that exact selection. Package-cache
lock waiting preceded the refusal but did not change the missing prerequisite.
The original failed target and combined logs remain retained without modification.

Explicitly run `make install-testkit-server`. The canonical shared selected Cargo
installer prepares `ic-testkit-server` 0.27.0 in release profile and publishes its
original receipts/selection. Testkit admits the existing PocketIC 16.1.0 asset
through its own setup owner; retain previous versioned CLI installations and all
server evidence. No default validation path downloads or changes dependencies.
Repeat this explicit preparation after the selected Testkit package changes.

`make testkit-server-check` and the actual validation-runner target now pass,
printing the admitted server path. All three core planned-download and three
Agent simulator cases pass with actual managed startup: complete isolated
transfer/restore, lost or malformed replies, pending spending/no reissue, and
wrong-root refusal. This is the unchanged Host 0.9.2 / Testkit 0.27.0 / Metrics
0.3.1 graph under Shared 0.2.3. Earlier 0.26.0 evidence remains separate.

[Qualification](qualification.json) retains exact manifest/lock/snapshot identity,
CLI/server fingerprints, commands and original failure-log fingerprints. This
fix prepares the local prerequisite only; no source API, script, schema, record,
spending owner, function, method or type is changed or removed. Keep the 0.11.1
draft uncommitted and package/receipt at 0.11.0. No full release/CI gate, live IC
operation, Git write, publication or sibling edit was performed. Native/downstream
and application/terminal qualifications remain distinct.
