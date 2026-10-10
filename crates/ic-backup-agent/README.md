# ic-backup-agent

Direct Rust `ic-agent` transport for exact original `ic-backup` requests. This
package signs and submits one previously reserved replicated update with separate
receiver/effective target, explicit endpoint, caller, trusted root and deadline.
It disables redirects, HTTP/TCP retries, automatic polling and root-key fetching.

Create `AgentTransport` with the exact plan context and an integration-owned
identity/root key. Bind an existing request using `ReservedUpdate`, then call
`prepare` with the currently guarded original journal. Retain its exact signed
`envelope` and `request_id` durably with original intent before consuming the
preparation through `submit().await`. An async network/timer executor is required.

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
receipts and pending spending. Upload, recovery and fence requests still use the
original explicit `prepare`/`submit` interface.

A certificate-verified reply is passive evidence for the existing method-specific
codec and association checks. `Pending` returns the original ingress ID without
polling. Errors, cancellation and lost replies leave original reservations pending;
none authorizes repetition, refunds spending or proves nonapplication. There is
no signed-envelope re-import/reissue API or automatic journal settlement.

The integration must prove actual network trust, fresh permissions/prerequisites,
never-dispatched custody, stable source bytes, complete transfer and application
restore/fence safety. Matching context declarations and a pending journal are
insufficient. Retain all original obligations and source references. This package
supplies transport, not a complete backup/restore runner or Canic adapter.
