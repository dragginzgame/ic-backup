//! Exact async metadata/data reads through the existing single-update transport.

use crate::{AgentTransport, PreparedUpdate, ReservedUpdate};
use ic_backup::model::ic_update_response::IcUpdateResponseInput;
use ic_backup::{
    model::{
        attempt_journal::AttemptJournalRecord,
        ic_snapshot_transfer_read::{
            IcSnapshotTransferReadRequest, IcSnapshotTransferReadResponse,
        },
    },
    ports::{
        ic_observation::IcObservationProviderError,
        ic_snapshot_transfer_read::IcSnapshotTransferReadProvider,
    },
};

/// One accounted replicated-update read with mandatory durable signed-ingress retention.
///
/// The caller holds the current original journal guard across this future. `retain`
/// must durably retain the exact signed envelope/request ID under the original plan
/// and attempt, with exclusive never-dispatched custody, before returning success.
/// It performs no remote calls; refusal prevents submission. Fresh permissions,
/// authentic metadata/raw-ID association and application custody remain explicit
/// integration responsibilities. There is no default retainer or admission lane.
///
/// Replied bytes are passive evidence; accepted/lost/cancelled reads stay pending.
/// No hidden calls, retry, receipt, completed transfer or release authority follows.
pub struct AgentSnapshotTransferReadProvider<'a, R> {
    transport: &'a AgentTransport,
    retain: R,
}

impl<'a, R> AgentSnapshotTransferReadProvider<'a, R> {
    /// Bind a fixed configured transport and integration-owned durable retainer.
    /// Construction performs no signing, IO, spending or fresh admission.
    pub const fn new(transport: &'a AgentTransport, retain: R) -> Self {
        Self { transport, retain }
    }
}

impl<R> IcSnapshotTransferReadProvider for AgentSnapshotTransferReadProvider<'_, R>
where
    R: FnMut(
        &IcSnapshotTransferReadRequest<'_, '_>,
        &PreparedUpdate<'_>,
    ) -> Result<(), IcObservationProviderError>,
{
    async fn read_snapshot(
        &mut self,
        request: &IcSnapshotTransferReadRequest<'_, '_>,
        journal: &AttemptJournalRecord,
    ) -> Result<IcSnapshotTransferReadResponse, IcObservationProviderError> {
        let (request_id, reply) = crate::submission::submit_once(
            self.transport,
            ReservedUpdate::TransferRead(request),
            journal,
            |prepared| (self.retain)(request, prepared),
            |_, _, _| Ok(()),
            || IcObservationProviderError::Unsupported,
            || IcObservationProviderError::Indeterminate,
        )
        .await?;
        let evidence = crate::reply_evidence::reply_evidence(
            b"ic-backup-agent.transfer-read-reply.v1\0",
            &request.authority().digest(),
            &[request.mutation_attempt()],
            &request_id,
            &reply,
        );
        IcSnapshotTransferReadResponse::new(IcUpdateResponseInput {
            authority: request.authority().digest(),
            mutation_attempt: request.mutation_attempt(),
            context: self.transport.context.clone(),
            target: request.payload().target().into(),
            reply,
            evidence,
        })
        .map_err(|_| IcObservationProviderError::Indeterminate)
    }
}
