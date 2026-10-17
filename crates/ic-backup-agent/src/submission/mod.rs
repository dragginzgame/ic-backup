//! One retained, single-dispatch update shared by the configured port adapters.

use crate::{AgentTransport, PreparedUpdate, ReservedUpdate};
use crate::{MAX_SIGNED_UPDATE_BYTES, TransportError};
use ic_agent::{RequestId, agent::CallResponse, export::Principal};
use ic_backup::model::attempt_journal::AttemptJournalRecord;

pub(crate) async fn submit_once<E>(
    transport: &AgentTransport,
    request: ReservedUpdate<'_>,
    journal: &AttemptJournalRecord,
    mut retain_ingress: impl FnMut(&PreparedUpdate<'_>) -> Result<(), E>,
    mut retain_reply: impl FnMut(&RequestId, &[u8]) -> Result<(), E>,
    unsupported: fn() -> E,
    indeterminate: fn() -> E,
) -> Result<(RequestId, Vec<u8>), E> {
    let prepared = transport
        .prepare(request, journal)
        .map_err(|_| unsupported())?;
    retain_ingress(&prepared)?;
    let outcome = prepared.submit().await.map_err(|_| indeterminate())?;
    let UpdateOutcome::Replied { request_id, reply } = outcome else {
        return Err(indeterminate());
    };
    // A failed post-dispatch retainer must never turn an uncertain effect into a retry.
    retain_reply(&request_id, &reply).map_err(|_| indeterminate())?;
    Ok((request_id, reply))
}

impl AgentTransport {
    /// Sign an exact existing request after rechecking its original pending journal.
    ///
    /// No network call occurs. Retain the returned exact envelope/request ID with
    /// original plan, reservation and exclusive command custody before submission.
    /// Holding the original journal guard prevents cooperating record changes.
    /// Fresh authority and proof of no prior dispatch remain integration prerequisites.
    /// # Errors
    /// Rejects changed reservations/context, signing failures or oversized envelopes.
    fn prepare<'a>(
        &'a self,
        request: ReservedUpdate<'a>,
        journal: &'a AttemptJournalRecord,
    ) -> Result<PreparedUpdate<'a>, TransportError> {
        if !request.validate(journal) {
            return Err(TransportError::Reservation);
        }
        let wire = request.wire();
        if wire.context != &self.context {
            return Err(TransportError::Context);
        }
        let receiver =
            Principal::from_text(wire.receiver).map_err(|_| TransportError::Configuration)?;
        let target =
            Principal::from_text(wire.target).map_err(|_| TransportError::Configuration)?;
        let signed = self
            .agent
            .update(&receiver, wire.method)
            .with_effective_canister_id(target)
            .with_arg(wire.arguments)
            .sign()
            .map_err(|_| TransportError::Signing)?;
        if signed.sender.to_text() != self.context.caller() {
            return Err(TransportError::Context);
        }
        if signed.signed_update.len() > MAX_SIGNED_UPDATE_BYTES {
            return Err(TransportError::EnvelopeTooLarge);
        }
        let reply_limit = wire.reply_limit;
        Ok(PreparedUpdate {
            transport: self,
            request,
            journal,
            target,
            envelope: signed.signed_update,
            request_id: signed.request_id,
            reply_limit,
        })
    }
}

impl PreparedUpdate<'_> {
    /// Consume this preparation for one update HTTP request, without automatic wait or observation.
    ///
    /// A verified certificate reply is still passive method evidence, never an
    /// automatic Applied receipt. Accepted/processing results return Pending; callers
    /// must account separate recovery observations. Every error, cancellation or lost
    /// reply leaves original spending pending; no error establishes nonapplication.
    /// # Errors
    /// Rejects stale reservations before dispatch; transport/authentication failures
    /// and over-limit replies retain indeterminate original evidence.
    async fn submit(self) -> Result<UpdateOutcome, TransportError> {
        if !self.request.validate(self.journal) {
            return Err(TransportError::Reservation);
        }
        let response = self
            .transport
            .agent
            .update_signed(self.target, self.envelope)
            .await
            .map_err(|_| TransportError::Indeterminate)?;
        match response {
            CallResponse::Response(reply) => {
                if reply.len() > self.reply_limit {
                    return Err(TransportError::ReplyTooLarge);
                }
                Ok(UpdateOutcome::Replied {
                    request_id: self.request_id,
                    reply,
                })
            }
            CallResponse::Poll(id) if id == self.request_id => Ok(UpdateOutcome::Pending),
            CallResponse::Poll(_) => Err(TransportError::Indeterminate),
        }
    }
}

/// Passive single-update result; neither branch mutates or settles original journals.
enum UpdateOutcome {
    /// Locally certificate-verified exact request reply; method decoding/attribution remain separate.
    Replied {
        /// Exact original signed ingress identity.
        request_id: RequestId,
        /// Bounded raw method reply; use the existing method-specific decoder.
        reply: Vec<u8>,
    },
    /// Accepted/processing without a settled reply; no follow-up call was made.
    Pending,
}
