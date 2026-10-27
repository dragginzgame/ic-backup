//! One retained, single-dispatch update shared by the configured port adapters.

use crate::MAX_SIGNED_UPDATE_BYTES;
use crate::{AgentTransport, PreparedUpdate, ReservedUpdate};
use ic_backup::model::attempt_journal::AttemptJournalRecord;
use ic_platform::Principal;
use ic_platform::agent::{CallResponse, RequestId};

pub(crate) async fn submit_once<E>(
    transport: &AgentTransport,
    request: ReservedUpdate<'_>,
    journal: &AttemptJournalRecord,
    mut retain_ingress: impl FnMut(&PreparedUpdate<'_>) -> Result<(), E>,
    mut retain_reply: impl FnMut(&RequestId, &[u8]) -> Result<(), E>,
    unsupported: fn() -> E,
    indeterminate: fn() -> E,
) -> Result<(RequestId, Vec<u8>), E> {
    let wire = request.admit(journal).ok_or_else(unsupported)?;
    if wire.context != &transport.context {
        return Err(unsupported());
    }
    let receiver = Principal::from_text(wire.receiver).map_err(|_| unsupported())?;
    let target = Principal::from_text(wire.target).map_err(|_| unsupported())?;
    let signed = transport
        .agent
        .update(&receiver.into(), wire.method)
        .with_effective_canister_id(target.into())
        .with_arg(wire.arguments)
        .sign()
        .map_err(|_| unsupported())?;
    if signed.sender.to_text() != transport.context.caller()
        || signed.signed_update.len() > MAX_SIGNED_UPDATE_BYTES
    {
        return Err(unsupported());
    }
    let reply_limit = wire.reply_limit;
    let prepared = PreparedUpdate {
        transport,
        request,
        target: target.into(),
        envelope: signed.signed_update,
        request_id: signed.request_id,
        reply_limit,
    };
    retain_ingress(&prepared)?;
    let request_id = prepared.request_id;
    let response = prepared
        .transport
        .agent
        .update_signed(prepared.target, prepared.envelope)
        .await
        .map_err(|_| indeterminate())?;
    let reply = match response {
        CallResponse::Response(reply) if reply.len() <= prepared.reply_limit => reply,
        CallResponse::Response(_) | CallResponse::Poll(_) => return Err(indeterminate()),
    };
    // A failed post-dispatch retainer must never turn an uncertain effect into a retry.
    retain_reply(&request_id, &reply).map_err(|_| indeterminate())?;
    Ok((request_id, reply))
}
