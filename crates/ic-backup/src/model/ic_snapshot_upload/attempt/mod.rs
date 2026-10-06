//! Original pending reservations over exact already retained upload payloads.

use super::IcSnapshotUploadRequest;
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
    authority: AttemptAuthorityRecord,
    mutation_attempt: u32,
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
        let authority = plan.attempt_authority(operation_sequence)?;
        if journal.authority() != &authority {
            return Err(IcSnapshotUploadAttemptError::AuthorityMismatch);
        }
        if payload.target() != authority.binding().target()
            || payload.binding_digest().hash() != authority.binding().request()
        {
            return Err(IcSnapshotUploadAttemptError::PayloadMismatch);
        }
        if plan.context().network() != payload.source_plan().context().network()
            || plan.context().release() != payload.source_plan().context().release()
        {
            return Err(IcSnapshotUploadAttemptError::SourceContextMismatch);
        }
        let mutation_attempt = journal
            .view()
            .pending_mutation
            .ok_or(IcSnapshotUploadAttemptError::NoPendingMutation)?;
        let attempt = Self {
            plan,
            payload,
            authority,
            mutation_attempt,
        };
        attempt.validate_journal(journal)?;
        Ok(attempt)
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
        &self.authority
    }
    /// Read the already consumed original mutation number.
    #[must_use]
    pub const fn mutation_attempt(&self) -> u32 {
        self.mutation_attempt
    }
    /// Recheck exact current authority and reservation before passive association.
    /// # Errors
    /// Rejects changed/settled mutation, authority mismatch or pending observation.
    pub fn validate_journal(
        &self,
        journal: &AttemptJournalRecord,
    ) -> Result<(), IcSnapshotUploadAttemptError> {
        if journal.authority() != &self.authority {
            return Err(IcSnapshotUploadAttemptError::AuthorityMismatch);
        }
        let view = journal.view();
        if view.pending_mutation != Some(self.mutation_attempt) {
            return Err(IcSnapshotUploadAttemptError::MutationMismatch);
        }
        if view.pending_observation.is_some() {
            return Err(IcSnapshotUploadAttemptError::ObservationPending);
        }
        Ok(())
    }
}

/// Original upload reservation rejection; no variant changes spending or recovery.
#[derive(Debug, Error)]
pub enum IcSnapshotUploadAttemptError {
    /// Upload intent changes the original source network or release identity.
    #[error("snapshot upload source network or release differs")]
    SourceContextMismatch,
    /// Original full-plan authority/context/operation/allowances differ.
    #[error("snapshot upload original authority mismatch")]
    AuthorityMismatch,
    /// Exact canonical target or original source/wire binding digest differs.
    #[error("snapshot upload original payload mismatch")]
    PayloadMismatch,
    /// No original unresolved mutation was reserved.
    #[error("snapshot upload requires an original pending mutation")]
    NoPendingMutation,
    /// Original mutation has changed or already settled.
    #[error("snapshot upload original mutation differs")]
    MutationMismatch,
    /// An already reserved recovery observation prevents original submission association.
    #[error("snapshot upload recovery observation is pending")]
    ObservationPending,
    /// Original full plan has no such operation or cannot derive its authority.
    #[error(transparent)]
    Plan(#[from] OperationPlanError),
}
