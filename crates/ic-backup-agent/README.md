# ic-backup-agent

Direct Rust `ic-agent` transport for exact original `ic-backup` requests. This
package signs and submits one previously reserved replicated update with separate
receiver/effective target, explicit endpoint, caller, trusted root and deadline.
It disables redirects, HTTP/TCP retries, automatic polling and root-key fetching.

Create `AgentTransport` with the exact plan context and an integration-owned
identity/root key, then configure the appropriate provider below. Providers sign,
invoke mandatory durable ingress retention, and submit once. Signing and dispatch
are internal; the former public `prepare`, `submit` and `UpdateOutcome` API is
removed in 0.28. Retention callbacks receive a passive `PreparedUpdate` view with
exact `envelope()` and `request_id()` access. An async network/timer executor is required.

For capture/load/start coordination, configure `AgentMutationProvider` with that
transport and a mandatory fallible callback retaining the exact signed envelope
and request ID privately and durably under the original plan/attempt. Retention
failure stops before submission. Await the core `capture_snapshot` or
`restore_snapshot` coordinator with async fresh admission and, for restore,
independent reply qualification. The selected journal stays locked across awaits;
cancellation preserves pending spending and denies reentry.

For metadata/data reads, configure `AgentSnapshotTransferReadProvider` with the
same mandatory durable signed-ingress retainer. Await core `read_snapshot`,
`read_snapshot_metadata` or `download_snapshot` with async fresh admission and,
for metadata/download, independent qualification. Retain replies before awaiting
cancellable qualification. Download cancellation preserves partial bytes, earlier
receipts and pending spending.

For source-bound metadata/data uploads, configure `AgentSnapshotUploadProvider`
with the same mandatory signed-ingress retainer. Await `upload_snapshot`,
`allocate_snapshot` or `upload_snapshot_data` with async fresh admission and
independent allocation/data qualification. Integrations own authentic complete
source, destination attribution and stable bytes. Retain replies before cancellable
qualification; cancellation preserves source evidence, earlier receipts and pending
spending.

Configure `AgentObservationProvider` for independently reserved status/list/readback
recovery. Await each of the three observation ports with the current guarded journal;
its mandatory retainer receives the exact reserved request and signed preparation.
Retain durable original identities and ingress under never-dispatched custody, refusing
prior dispatch. Cancellation and passive replies keep both attempts pending.

For original opaque application acquisition updates, configure
`AgentFenceAcquisitionProvider` with both durable signed-ingress and exact raw-reply
retainers. Await `acquire_fence` with the current guarded journal. Reply-retention
failure stays Indeterminate; no application outcome or Active fence is inferred.
Application reconciliation is also async with the current original record; its
actual read method and whole-unit attribution remain application-owned.

A certificate-verified reply is passive evidence for the existing method-specific
codec and association checks. Accepted/processing responses return the provider's
Indeterminate result without polling; the retainer already owns the ingress ID.
Errors, cancellation and lost replies leave original reservations pending;
none authorizes repetition, refunds spending or proves nonapplication. There is
no signed-envelope re-import/reissue API or automatic journal settlement.

The integration must prove actual network trust, fresh permissions/prerequisites,
never-dispatched custody, stable source bytes, complete transfer and application
restore/fence safety. Matching context declarations and a pending journal are
insufficient. Retain all original obligations and source references. This package
supplies transport, not a complete backup/restore runner or Canic adapter.
