//! Pure current-reservation and passive upload-reply association; no IO or settlement.

#[cfg(test)]
mod tests;

use super::response_identity::{ResponseIdentity, ResponseIdentityMismatch};
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
    ResponseIdentity::from(input).compare(&ResponseIdentity {
        authority: &request.authority().digest(),
        mutation_attempt: request.mutation_attempt(),
        context: request.plan().context(),
        target: request.payload().target(),
    })?;
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
    /// Passive claims differ from the exact original update identity.
    #[error(transparent)]
    Identity(#[from] ResponseIdentityMismatch),
    /// Canonical bounded upload decoder rejected the raw reply.
    #[error(transparent)]
    Reply(#[from] IcSnapshotUploadError),
}
