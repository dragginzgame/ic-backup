//! Async source-bound snapshot upload port over the existing exact single-update transport.

use crate::{
    AgentTransport, PreparedUpdate, ReservedUpdate,
    acknowledgement::{self, AcknowledgementIdentity},
};
use ic_backup::{
    model::{
        attempt_journal::AttemptJournalRecord, ic_mutation::IcMutationAcknowledgement,
        ic_snapshot_upload::IcSnapshotUploadAttempt,
    },
    ports::{ic_mutation::IcMutationProviderError, ic_snapshot_upload::IcSnapshotUploadProvider},
};

/// Source-bound metadata/data single-update Agent provider with mandatory durable signed-ingress retention.
///
/// `retain` must durably retain the exact envelope and request ID under the original
/// plan/attempt and exclusive never-dispatched command custody before returning.
/// Failure stops before network submission. It must perform no remote calls or
/// retries; no default retention callback exists. The original selected journal
/// remains borrowed under its guard across signing, retention and async submission.
///
/// Authentic complete source, exclusive destination attribution, fresh controller
/// permissions and stable byte custody remain integration-owned.
/// Fresh authority, application safety and prior accounting belong to the calling
/// coordinator's mandatory admission. Certificate verification uses the configured
/// trusted root; it does not authenticate declared network/release or prove whole
/// application safety. Returned acknowledgements remain passive and pending until
/// independently qualified. Accepted, cancelled or failed calls grant no reissue.
pub struct AgentSnapshotUploadProvider<'a, R> {
    transport: &'a AgentTransport,
    retain: R,
}

impl<'a, R> AgentSnapshotUploadProvider<'a, R> {
    /// Bind the fixed transport and integration-owned durable retention callback.
    /// Construction performs no signing, IO, spending or fresh admission.
    pub const fn new(transport: &'a AgentTransport, retain: R) -> Self {
        Self { transport, retain }
    }
}

impl<R> IcSnapshotUploadProvider for AgentSnapshotUploadProvider<'_, R>
where
    R: FnMut(
        &IcSnapshotUploadAttempt<'_, '_>,
        &PreparedUpdate<'_>,
    ) -> Result<(), IcMutationProviderError>,
{
    async fn submit_upload(
        &mut self,
        request: &IcSnapshotUploadAttempt<'_, '_>,
        journal: &AttemptJournalRecord,
    ) -> Result<IcMutationAcknowledgement, IcMutationProviderError> {
        acknowledgement::submit_acknowledged(
            self.transport,
            ReservedUpdate::Upload(request),
            journal,
            |prepared| (self.retain)(request, prepared),
            AcknowledgementIdentity {
                domain: b"ic-backup-agent.upload-reply.v1\0",
                authority: &request.authority().digest(),
                mutation_attempt: request.mutation_attempt(),
                target: request.payload().target(),
            },
        )
        .await
    }
}
