//! Original pending reservations over exact already retained upload payloads.

use super::IcSnapshotUploadRequest;
use crate::model::attempt_journal::MutationReservation;
use crate::model::{
    attempt_journal::{AttemptAuthorityRecord, AttemptJournalRecord},
    operation_plan::{OperationPlanError, OperationPlanRecord},
};
use thiserror::Error;

/// Structural original pending upload identity; reconstruction is no redispatch permit.
#[derive(Debug)]
pub struct IcSnapshotUploadAttempt<'request, 'source> {
    plan: &'request OperationPlanRecord,
    payload: &'request IcSnapshotUploadRequest<'source>,
    reservation: MutationReservation,
}

impl<'request, 'source> IcSnapshotUploadAttempt<'request, 'source> {
    /// Bind the full original plan, exact target/bytes and pending mutation reservation.
    ///
    /// This performs no provider call, record creation, reservation or settlement.
    /// A data payload includes its qualified new ID before its own plan is retained;
    /// the prior metadata plan/allowance cannot be rebound into new data authority.
    /// # Errors
    /// Rejects unknown operations, different authority/payload, no pending mutation
    /// or observation recovery already in progress.
    pub fn new(
        plan: &'request OperationPlanRecord,
        operation_sequence: u64,
        journal: &AttemptJournalRecord,
        payload: &'request IcSnapshotUploadRequest<'source>,
    ) -> Result<Self, IcSnapshotUploadAttemptError> {
        let authority = original_authority(plan, operation_sequence, journal, payload)?;
        let reservation = MutationReservation::new(authority, journal)?;
        Ok(Self {
            plan,
            payload,
            reservation,
        })
    }
    /// Read the exact retained full original plan, context and allowances.
    #[must_use]
    pub const fn plan(&self) -> &'request OperationPlanRecord {
        self.plan
    }
    /// Read the exact originally planned upload wire/source declaration.
    #[must_use]
    pub const fn payload(&self) -> &'request IcSnapshotUploadRequest<'source> {
        self.payload
    }
    /// Read original plan-derived authority; it grants no fresh effect.
    #[must_use]
    pub const fn authority(&self) -> &AttemptAuthorityRecord {
        &self.reservation.authority
    }
    /// Read the already consumed original mutation number.
    #[must_use]
    pub const fn mutation_attempt(&self) -> u32 {
        self.reservation.mutation_attempt
    }
    /// Recheck exact current authority and reservation before passive association.
    /// # Errors
    /// Rejects changed/settled mutation, authority mismatch or pending observation.
    pub fn validate_journal(
        &self,
        journal: &AttemptJournalRecord,
    ) -> Result<(), IcSnapshotUploadAttemptError> {
        self.reservation.validate(journal).map_err(Into::into)
    }
}

pub(crate) fn original_authority(
    plan: &OperationPlanRecord,
    operation_sequence: u64,
    journal: &AttemptJournalRecord,
    payload: &IcSnapshotUploadRequest<'_>,
) -> Result<AttemptAuthorityRecord, IcSnapshotUploadAttemptError> {
    let authority = plan.attempt_authority_for_journal(
        operation_sequence,
        journal,
        IcSnapshotUploadAttemptError::Reservation(
            crate::model::attempt_journal::MutationReservationError::AuthorityMismatch,
        ),
    )?;
    if payload.target() != authority.binding().target()
        || payload.binding_digest().hash() != authority.binding().request()
    {
        return Err(IcSnapshotUploadAttemptError::PayloadMismatch);
    }
    if !source_context_matches(plan, payload) {
        return Err(IcSnapshotUploadAttemptError::SourceContextMismatch);
    }
    Ok(authority)
}

pub(crate) fn source_context_matches(
    plan: &OperationPlanRecord,
    payload: &IcSnapshotUploadRequest<'_>,
) -> bool {
    plan.context().network() == payload.source_plan().context().network()
        && plan.context().release() == payload.source_plan().context().release()
}

/// Original upload reservation rejection; no variant changes spending or recovery.
#[derive(Debug, Error)]
pub enum IcSnapshotUploadAttemptError {
    /// Original reservation denial, retaining the canonical typed accounting cause.
    #[error("original mutation reservation rejected: {0}")]
    Reservation(#[from] crate::model::attempt_journal::MutationReservationError),
    /// Upload intent changes the original source network or release identity.
    #[error("snapshot upload source network or release differs")]
    SourceContextMismatch,
    /// Exact canonical target or original source/wire binding digest differs.
    #[error("snapshot upload original payload mismatch")]
    PayloadMismatch,
    /// Original full plan has no such operation or cannot derive its authority.
    #[error(transparent)]
    Plan(#[from] OperationPlanError),
}
