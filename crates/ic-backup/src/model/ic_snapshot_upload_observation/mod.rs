//! Exact reserved inventory observations of unresolved metadata uploads.

use crate::model::{
    attempt_journal::{AttemptAuthorityRecord, AttemptJournalRecord},
    ic_observation::{IcObservationRequestError, ObservationReservation},
    ic_request::{IcManagementMethodRecord, IcManagementRequestRecord},
    ic_snapshot_upload::{
        IcSnapshotUploadAttemptError, IcSnapshotUploadKind, IcSnapshotUploadRequest,
        original_authority,
    },
    operation_plan::OperationPlanRecord,
};
use thiserror::Error;

/// Original metadata-upload intent and its already spent exact list observation.
///
/// This declaration supplies no dispatch permission, authentication or allocation
/// attribution. Integrations retain original bytes and qualify fresh read access,
/// chronology, command custody and proof of no prior observation dispatch.
/// Lost observations stay pending; reconstruction permits no repeated call.
#[derive(Debug)]
pub struct IcSnapshotUploadObservationRequest<'request, 'source> {
    plan: &'request OperationPlanRecord,
    mutation: &'request IcSnapshotUploadRequest<'source>,
    payload: &'request IcManagementRequestRecord,
    authority: AttemptAuthorityRecord,
    mutation_attempt: u32,
    observation_attempt: u32,
}

impl<'request, 'source> IcSnapshotUploadObservationRequest<'request, 'source> {
    /// Bind exact original metadata intent and an independently reserved list payload.
    ///
    /// Data uploads and status observations have no admission through this boundary.
    /// No journal is created, reserved, reset or settled by construction.
    /// # Errors
    /// Rejects unsupported methods, changed original bindings and missing reservations.
    pub fn new(
        plan: &'request OperationPlanRecord,
        operation_sequence: u64,
        journal: &AttemptJournalRecord,
        mutation: &'request IcSnapshotUploadRequest<'source>,
        payload: &'request IcManagementRequestRecord,
    ) -> Result<Self, IcSnapshotUploadObservationError> {
        if !matches!(mutation.kind(), IcSnapshotUploadKind::Metadata) {
            return Err(IcSnapshotUploadObservationError::UnsupportedUpload);
        }
        if payload.method() != IcManagementMethodRecord::ListCanisterSnapshots {
            return Err(IcSnapshotUploadObservationError::UnsupportedObservation);
        }
        let authority = original_authority(plan, operation_sequence, journal, mutation)?;
        payload
            .validate_observation_binding(authority.binding(), &payload.digest())
            .map_err(IcObservationRequestError::from)?;
        let current = journal.view();
        let request = Self {
            plan,
            mutation,
            payload,
            authority,
            mutation_attempt: current
                .pending_mutation
                .ok_or(IcObservationRequestError::NoPendingMutation)?,
            observation_attempt: current
                .pending_observation
                .ok_or(IcObservationRequestError::NoPendingObservation)?,
        };
        request.validate_journal(journal)?;
        Ok(request)
    }

    /// Read the complete original upload plan and immutable allowances.
    #[must_use]
    pub const fn plan(&self) -> &OperationPlanRecord {
        self.plan
    }
    /// Read exact original metadata upload bytes and retained source binding.
    #[must_use]
    pub const fn mutation(&self) -> &'request IcSnapshotUploadRequest<'source> {
        self.mutation
    }
    /// Read the original accounted list receiver, target, method and Candid bytes.
    #[must_use]
    pub const fn payload(&self) -> &'request IcManagementRequestRecord {
        self.payload
    }
    /// Read original plan-derived authority, without fresh spending permission.
    #[must_use]
    pub const fn authority(&self) -> &AttemptAuthorityRecord {
        &self.authority
    }
    /// Read the original unresolved metadata-upload attempt.
    #[must_use]
    pub const fn mutation_attempt(&self) -> u32 {
        self.mutation_attempt
    }
    /// Read the already consumed inventory-observation attempt.
    #[must_use]
    pub const fn observation_attempt(&self) -> u32 {
        self.observation_attempt
    }
    /// Recheck exact current authority, both pending attempts and reserved list bytes.
    /// # Errors
    /// Rejects changed/settled reservations or different original authority/payload.
    pub fn validate_journal(
        &self,
        journal: &AttemptJournalRecord,
    ) -> Result<(), IcObservationRequestError> {
        ObservationReservation {
            authority: &self.authority,
            mutation_attempt: self.mutation_attempt,
            observation_attempt: self.observation_attempt,
            payload: self.payload,
        }
        .validate(journal)
    }
}

/// Structural upload/list admission failure; no outcome or spending transition.
#[derive(Debug, Error)]
pub enum IcSnapshotUploadObservationError {
    /// This inventory boundary admits metadata allocation only.
    #[error("snapshot upload observation requires metadata intent")]
    UnsupportedUpload,
    /// Only the exact original list payload is supported.
    #[error("snapshot upload observation requires snapshot list")]
    UnsupportedObservation,
    /// Original upload authority, bytes or source context differs.
    #[error(transparent)]
    Upload(#[from] IcSnapshotUploadAttemptError),
    /// Existing original observation reservation admission failed.
    #[error(transparent)]
    Observation(#[from] IcObservationRequestError),
}

#[cfg(test)]
mod tests;
