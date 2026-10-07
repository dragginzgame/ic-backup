//! Pure current-reservation association using the original metadata/data decoder owners.

use crate::model::{
    attempt_journal::AttemptJournalRecord,
    ic_snapshot_data::{IcSnapshotDataError, IcSnapshotDataReply},
    ic_snapshot_metadata::{IcSnapshotMetadataError, IcSnapshotMetadataReply},
    ic_snapshot_transfer_read::{
        IcSnapshotTransferReadError, IcSnapshotTransferReadPayload, IcSnapshotTransferReadRequest,
        IcSnapshotTransferReadResponse,
    },
};
use thiserror::Error;

/// Existing bounded method-specific reply evidence; not authenticated transfer proof.
#[derive(Debug)]
pub enum IcSnapshotTransferReadReply<'request, 'metadata> {
    /// Exact original metadata under its original raw-ID request.
    Metadata(IcSnapshotMetadataReply<'request>),
    /// Exact requested range length or known hash-checked chunk bytes.
    Data(IcSnapshotDataReply<'request, 'metadata>),
}

/// Passive response together with its original method-specific decoded projection.
#[derive(Debug)]
pub struct IcSnapshotTransferReadView<'request, 'metadata> {
    response: &'request IcSnapshotTransferReadResponse,
    reply: IcSnapshotTransferReadReply<'request, 'metadata>,
}

impl<'request, 'metadata> IcSnapshotTransferReadView<'request, 'metadata> {
    /// Read exact retained claims, raw bytes and opaque provider evidence.
    #[must_use]
    pub const fn response(&self) -> &'request IcSnapshotTransferReadResponse {
        self.response
    }
    /// Read the original bounded decoder's projection without a new codec.
    #[must_use]
    pub const fn reply(&self) -> &IcSnapshotTransferReadReply<'request, 'metadata> {
        &self.reply
    }
}

/// Match original authority/reservation and actual claimed context/target, then decode.
///
/// This performs no IO or journal transition and grants no fresh read permission,
/// authenticity, retry, aggregate coverage, durable bytes or terminal/release proof.
/// Failures/lost replies stay pending. Integrations qualify actual authentication,
/// freshness, snapshot custody and never-dispatched original-call custody.
/// # Errors
/// Rejects stale reservations, mismatched claims and existing bounded decoder errors.
pub fn validate_response<'request, 'metadata>(
    request: &IcSnapshotTransferReadRequest<'request, 'metadata>,
    journal: &AttemptJournalRecord,
    response: &'request IcSnapshotTransferReadResponse,
) -> Result<IcSnapshotTransferReadView<'request, 'metadata>, IcSnapshotTransferReadAssociationError>
{
    request.validate_journal(journal)?;
    let input = response.input();
    if input.authority != request.authority().digest() {
        return Err(IcSnapshotTransferReadAssociationError::AuthorityMismatch);
    }
    if input.mutation_attempt != request.mutation_attempt() {
        return Err(IcSnapshotTransferReadAssociationError::AttemptMismatch);
    }
    if &input.context != request.plan().context() {
        return Err(IcSnapshotTransferReadAssociationError::ContextMismatch);
    }
    if input.target != request.payload().target() {
        return Err(IcSnapshotTransferReadAssociationError::TargetMismatch);
    }
    let reply = match request.payload() {
        IcSnapshotTransferReadPayload::Metadata(payload) => IcSnapshotTransferReadReply::Metadata(
            IcSnapshotMetadataReply::decode(payload, &input.reply)?,
        ),
        IcSnapshotTransferReadPayload::Data(payload) => {
            IcSnapshotTransferReadReply::Data(IcSnapshotDataReply::decode(payload, &input.reply)?)
        }
    };
    Ok(IcSnapshotTransferReadView { response, reply })
}

/// Passive association denial; no outcome or allowance refund follows.
#[derive(Debug, Error)]
pub enum IcSnapshotTransferReadAssociationError {
    /// Current original journal no longer matches the read request.
    #[error(transparent)]
    Reservation(#[from] IcSnapshotTransferReadError),
    /// Claimed full original authority differs.
    #[error("snapshot transfer read response authority differs")]
    AuthorityMismatch,
    /// Claimed original attempt differs.
    #[error("snapshot transfer read response attempt differs")]
    AttemptMismatch,
    /// Actual claimed network/caller/release differs.
    #[error("snapshot transfer read response context differs")]
    ContextMismatch,
    /// Actual claimed routing target differs.
    #[error("snapshot transfer read response target differs")]
    TargetMismatch,
    /// Original metadata decoder rejects shape or its tighter 1 MiB bound.
    #[error(transparent)]
    Metadata(#[from] IcSnapshotMetadataError),
    /// Original data decoder rejects shape, bound, range length or chunk hash.
    #[error(transparent)]
    Data(#[from] IcSnapshotDataError),
}

#[cfg(test)]
mod tests;
