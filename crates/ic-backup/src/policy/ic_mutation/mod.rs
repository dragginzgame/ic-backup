//! Pure original IC update reply association using the existing bounded codecs.

use crate::model::{
    attempt_journal::AttemptJournalRecord,
    ic_lifecycle_reply::{IcLifecycleReply, IcLifecycleReplyError},
    ic_mutation::{IcMutationAcknowledgement, IcMutationRequest, IcMutationRequestError},
    ic_request::IcManagementMethodRecord,
    ic_snapshot_reply::{IcSnapshotReply, IcSnapshotReplyError},
};
use thiserror::Error;

/// Method-specific local wire evidence; neither branch proves effect settlement.
#[derive(Debug)]
pub enum IcMutationReplyView<'a> {
    /// Exact raw snapshot identity/timestamp/size under the original capture declaration.
    Capture(IcSnapshotReply<'a>),
    /// Exact empty stop/start/load tuple under the original lifecycle declaration.
    Lifecycle(IcLifecycleReply<'a>),
}

/// Borrowed original acknowledgement with decoded method-specific local evidence.
///
/// No receipt, spending transition, fresh permission, attribution, restored-state
/// safety or terminal/reference/fence release follows from successful association.
#[derive(Debug)]
pub struct IcMutationAcknowledgementView<'a> {
    acknowledgement: &'a IcMutationAcknowledgement,
    reply: IcMutationReplyView<'a>,
}
impl<'a> IcMutationAcknowledgementView<'a> {
    /// Read exact retained passive context/authority/attempt/raw reply/evidence fields.
    #[must_use]
    pub const fn acknowledgement(&self) -> &'a IcMutationAcknowledgement {
        self.acknowledgement
    }
    /// Read the existing codec's exact method-specific wire projection.
    #[must_use]
    pub const fn reply(&self) -> &IcMutationReplyView<'a> {
        &self.reply
    }
}

/// Recheck original reservation, actual association claims and bounded reply shape.
///
/// Network/caller/target authentication, freshness, exclusive original-request
/// attribution and current lifecycle/fence/restore safety remain integration-owned.
/// This pure function invokes no provider and changes no journal or allowance.
/// # Errors
/// Rejects changed reservation/authority/attempt/context/target and invalid Candid bytes.
pub fn validate_acknowledgement<'a>(
    request: &IcMutationRequest<'a>,
    journal: &AttemptJournalRecord,
    acknowledgement: &'a IcMutationAcknowledgement,
) -> Result<IcMutationAcknowledgementView<'a>, IcMutationAssociationError> {
    request.validate_journal(journal)?;
    let input = acknowledgement.input();
    if input.authority != request.authority().digest() {
        return Err(IcMutationAssociationError::AuthorityMismatch);
    }
    if input.mutation_attempt != request.mutation_attempt() {
        return Err(IcMutationAssociationError::AttemptMismatch);
    }
    if &input.context != request.plan().context() {
        return Err(IcMutationAssociationError::ContextMismatch);
    }
    if input.target != request.payload().target() {
        return Err(IcMutationAssociationError::TargetMismatch);
    }
    let reply = match request.payload().method() {
        IcManagementMethodRecord::TakeCanisterSnapshot => {
            IcMutationReplyView::Capture(IcSnapshotReply::decode(request.payload(), &input.reply)?)
        }
        // The request owner already excludes observation methods. The existing
        // lifecycle codec remains the only owner of the other mutation reply shapes.
        _ => IcMutationReplyView::Lifecycle(IcLifecycleReply::decode(
            request.payload(),
            &input.reply,
        )?),
    };
    Ok(IcMutationAcknowledgementView {
        acknowledgement,
        reply,
    })
}

/// Typed passive association rejection, preserving every original pending obligation.
#[derive(Debug, Error)]
pub enum IcMutationAssociationError {
    /// Current original reservation does not match the request.
    #[error(transparent)]
    Reservation(#[from] IcMutationRequestError),
    /// Reply claims another full original operation authority.
    #[error("IC mutation acknowledgement authority mismatch")]
    AuthorityMismatch,
    /// Reply claims another already allocated mutation attempt.
    #[error("IC mutation acknowledgement attempt mismatch")]
    AttemptMismatch,
    /// Claimed actual network/caller/release differs from original context.
    #[error("IC mutation acknowledgement context mismatch")]
    ContextMismatch,
    /// Claimed actual target differs from the exact original routing target.
    #[error("IC mutation acknowledgement target mismatch")]
    TargetMismatch,
    /// Existing snapshot decoder rejected the capture reply.
    #[error(transparent)]
    Snapshot(#[from] IcSnapshotReplyError),
    /// Existing lifecycle decoder rejected the exact empty tuple.
    #[error(transparent)]
    Lifecycle(#[from] IcLifecycleReplyError),
}

#[cfg(test)]
mod tests;
