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
