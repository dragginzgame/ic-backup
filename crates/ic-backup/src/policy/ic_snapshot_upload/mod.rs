//! Pure current-reservation and passive upload-reply association; no IO or settlement.

#[cfg(test)]
mod tests;

use crate::model::{
    attempt_journal::AttemptJournalRecord,
    ic_mutation::IcMutationAcknowledgement,
    ic_snapshot_upload::{
        IcSnapshotUploadAttempt, IcSnapshotUploadAttemptError, IcSnapshotUploadError,
        IcSnapshotUploadReply,
    },
};
use thiserror::Error;

/// Read-only retained original acknowledgement and bounded source-bound upload reply.
#[derive(Debug)]
pub struct IcSnapshotUploadAcknowledgementView<'request, 'source> {
    acknowledgement: &'request IcMutationAcknowledgement,
    reply: IcSnapshotUploadReply<'request, 'source>,
}
impl<'request, 'source> IcSnapshotUploadAcknowledgementView<'request, 'source> {
    /// Read exact claimed context/target/authority/attempt/raw bytes/evidence.
    #[must_use]
    pub const fn acknowledgement(&self) -> &'request IcMutationAcknowledgement {
        self.acknowledgement
    }
    /// Read passive bounded metadata ID or data acknowledgement; no Applied receipt.
    #[must_use]
    pub const fn reply(&self) -> &IcSnapshotUploadReply<'request, 'source> {
        &self.reply
    }
}

/// Recheck exact original pending upload and passive actual association fields.
///
/// Reuses the existing bounded acknowledgement owner, with tighter upload decoding.
/// Actual authentication, exclusive original allocation/write attribution, freshness,
/// controller/byte custody and independent per-call accounting remain integration-owned.
/// Lost replies stay pending. Zero/one/multiple inventory entries or absent data never
/// establish an outcome. Only separately qualified settled observations can reconcile;
/// no observation, receipt, retry, refund, completion or release is produced here.
/// # Errors
/// Rejects original reservation drift, mismatching actual claims and invalid reply shape.
pub fn validate_acknowledgement<'request, 'source>(
    request: &IcSnapshotUploadAttempt<'request, 'source>,
    journal: &AttemptJournalRecord,
    acknowledgement: &'request IcMutationAcknowledgement,
) -> Result<IcSnapshotUploadAcknowledgementView<'request, 'source>, IcSnapshotUploadAssociationError>
{
    request.validate_journal(journal)?;
    let input = acknowledgement.input();
    if input.authority != request.authority().digest() {
        return Err(IcSnapshotUploadAssociationError::AuthorityMismatch);
    }
    if input.mutation_attempt != request.mutation_attempt() {
        return Err(IcSnapshotUploadAssociationError::AttemptMismatch);
    }
    if &input.context != request.plan().context() {
        return Err(IcSnapshotUploadAssociationError::ContextMismatch);
    }
    if input.target != request.payload().target() {
        return Err(IcSnapshotUploadAssociationError::TargetMismatch);
    }
    let reply = IcSnapshotUploadReply::decode(request.payload(), &input.reply)?;
    Ok(IcSnapshotUploadAcknowledgementView {
        acknowledgement,
        reply,
    })
}

/// Typed passive association denial; all original obligations and counters remain retained.
#[derive(Debug, Error)]
pub enum IcSnapshotUploadAssociationError {
    /// Exact current original mutation/reservation no longer matches.
    #[error(transparent)]
    Reservation(#[from] IcSnapshotUploadAttemptError),
    /// Claimed full original authority differs.
    #[error("snapshot upload acknowledgement authority mismatch")]
    AuthorityMismatch,
    /// Claimed already allocated mutation differs.
    #[error("snapshot upload acknowledgement attempt mismatch")]
    AttemptMismatch,
    /// Claimed actual network/caller/release differs.
    #[error("snapshot upload acknowledgement context mismatch")]
    ContextMismatch,
    /// Claimed actual target differs.
    #[error("snapshot upload acknowledgement target mismatch")]
    TargetMismatch,
    /// Canonical bounded upload decoder rejected the raw reply.
    #[error(transparent)]
    Reply(#[from] IcSnapshotUploadError),
}
