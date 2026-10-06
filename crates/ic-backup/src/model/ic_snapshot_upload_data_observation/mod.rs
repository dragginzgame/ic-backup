//! Exact already reserved destination-data observations of unresolved uploads.

use crate::model::{
    artifacts::ArtifactChecksumRecord,
    attempt_journal::{AttemptAuthorityRecord, AttemptJournalRecord},
    ic_observation::{
        IcObservationRequestError, IcObservationResponseError, IcObservationResponseInput,
        ObservationReservation, fmt_response_input, validate_response_input,
    },
    ic_snapshot_data::{IcSnapshotDataRequest, MAX_IC_SNAPSHOT_DATA_REPLY_BYTES},
    ic_snapshot_upload::{
        IcSnapshotUploadAttemptError, IcSnapshotUploadKind, IcSnapshotUploadRequest,
        original_authority,
    },
    operation_plan::OperationPlanRecord,
};
use std::fmt;
use thiserror::Error;

mod settlement;
pub use settlement::{IcSnapshotUploadDataAttribution, IcSnapshotUploadDataSettlement};

/// Original data-upload intent and its independently spent exact readback request.
///
/// Retain destination metadata and original read bytes before reservation. Their
/// authenticity, allocation attribution, fresh access, chronology and proof of no
/// prior observation dispatch remain integration-owned. Reconstruction is no retry.
#[derive(Debug)]
pub struct IcSnapshotUploadDataObservationRequest<'request, 'source, 'metadata> {
    plan: &'request OperationPlanRecord,
    mutation: &'request IcSnapshotUploadRequest<'source>,
    payload: &'request IcSnapshotDataRequest<'metadata>,
    authority: AttemptAuthorityRecord,
    mutation_attempt: u32,
    observation_attempt: u32,
    original_chunk_checksum: &'request ArtifactChecksumRecord,
}

impl<'request, 'source, 'metadata>
    IcSnapshotUploadDataObservationRequest<'request, 'source, 'metadata>
{
    /// Bind one original data upload to a reserved read of the same destination/extent.
    ///
    /// Metadata allocation, other IDs, targets, regions, offsets, lengths or chunk
    /// hashes reject. Destination metadata must declare uploaded source and original
    /// region sizes. This neither authenticates that metadata nor spends.
    /// # Errors
    /// Rejects changed originals, mismatching readback and missing reservations.
    pub fn new(
        plan: &'request OperationPlanRecord,
        operation_sequence: u64,
        journal: &AttemptJournalRecord,
        mutation: &'request IcSnapshotUploadRequest<'source>,
        payload: &'request IcSnapshotDataRequest<'metadata>,
    ) -> Result<Self, IcSnapshotUploadDataObservationError> {
        let IcSnapshotUploadKind::Data {
            snapshot_id,
            source_kind,
            chunk_checksum,
            ..
        } = mutation.kind()
        else {
            return Err(IcSnapshotUploadDataObservationError::DataUploadRequired);
        };
        let authority = original_authority(plan, operation_sequence, journal, mutation)?;
        if payload.target() != mutation.target()
            || payload.snapshot_id() != snapshot_id
            || !payload.matches_kind(source_kind)
        {
            return Err(IcSnapshotUploadDataObservationError::ReadbackMismatch);
        }
        let actual = payload.metadata().metadata();
        let original = mutation.source().metadata();
        let actual_sizes = [
            actual.wasm_module_size,
            actual.wasm_memory_size,
            actual.stable_memory_size,
        ];
        let original_sizes = [
            original.wasm_module_size,
            original.wasm_memory_size,
            original.stable_memory_size,
        ];
        if !matches!(
            actual.source,
            Some(ic_management_canister_types::SnapshotSource::MetadataUpload(_))
        ) || actual_sizes != original_sizes
        {
            return Err(IcSnapshotUploadDataObservationError::DestinationMetadataMismatch);
        }
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
            original_chunk_checksum: chunk_checksum,
        };
        request.validate_journal(journal)?;
        Ok(request)
    }

    /// Read the retained complete original plan and immutable allowances.
    #[must_use]
    pub const fn plan(&self) -> &OperationPlanRecord {
        self.plan
    }
    /// Read exact original source-bound upload bytes, without a dispatch permit.
    #[must_use]
    pub const fn mutation(&self) -> &'request IcSnapshotUploadRequest<'source> {
        self.mutation
    }
    /// Read exact original destination metadata/read request, without fresh access.
    #[must_use]
    pub const fn payload(&self) -> &'request IcSnapshotDataRequest<'metadata> {
        self.payload
    }
    /// Read original operation authority and assigned limits.
    #[must_use]
    pub const fn authority(&self) -> &AttemptAuthorityRecord {
        &self.authority
    }
    /// Read the original pending data-upload attempt.
    #[must_use]
    pub const fn mutation_attempt(&self) -> u32 {
        self.mutation_attempt
    }
    /// Read the already consumed exact data-read observation attempt.
    #[must_use]
    pub const fn observation_attempt(&self) -> u32 {
        self.observation_attempt
    }
    /// Read original encoded chunk identity; it establishes no write attribution.
    #[must_use]
    pub const fn original_chunk_checksum(&self) -> &ArtifactChecksumRecord {
        self.original_chunk_checksum
    }
    /// Recheck original current authority, pending IDs and exact read digest.
    /// # Errors
    /// Rejects changed/settled reservations and different original read bytes.
    pub fn validate_journal(
        &self,
        journal: &AttemptJournalRecord,
    ) -> Result<(), IcObservationRequestError> {
        ObservationReservation {
            authority: &self.authority,
            mutation_attempt: self.mutation_attempt,
            observation_attempt: self.observation_attempt,
            request: self.payload.digest(),
        }
        .validate(journal)
    }
}

/// Exact original data/readback declaration denial, with no outcome transition.
#[derive(Debug, Error)]
pub enum IcSnapshotUploadDataObservationError {
    /// Metadata allocation is outside this boundary.
    #[error("data upload observation requires original data intent")]
    DataUploadRequired,
    /// Destination target, raw ID or exact original extent/hash differs.
    #[error("data upload observation readback differs")]
    ReadbackMismatch,
    /// Destination source is unknown/not uploaded, or declared region sizes differ.
    #[error("data upload observation destination metadata differs")]
    DestinationMetadataMismatch,
    /// Canonical original upload authority/source admission rejected.
    #[error(transparent)]
    Upload(#[from] IcSnapshotUploadAttemptError),
    /// Canonical current original observation reservation rejected.
    #[error(transparent)]
    Observation(#[from] IcObservationRequestError),
}

/// Passive destination-data claims using the existing 2 MiB data-reply ceiling.
///
/// The existing status/list response keeps its 1 MiB ceiling. This owner reuses
/// canonical attempt/principal admission and redaction without widening that API.
#[derive(Clone)]
pub struct IcSnapshotUploadDataObservationResponse {
    input: IcObservationResponseInput,
}

impl IcSnapshotUploadDataObservationResponse {
    /// Admit bounded raw bytes and passive chronological/canonical claims.
    /// # Errors
    /// Rejects invalid attempts/principals and replies over the existing data bound.
    pub fn new(mut input: IcObservationResponseInput) -> Result<Self, IcObservationResponseError> {
        validate_response_input(&mut input, MAX_IC_SNAPSHOT_DATA_REPLY_BYTES)?;
        Ok(Self { input })
    }
    /// Read immutable existing claim fields and exact original raw data reply.
    #[must_use]
    pub const fn input(&self) -> &IcObservationResponseInput {
        &self.input
    }
}
impl fmt::Debug for IcSnapshotUploadDataObservationResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_response_input(
            &self.input,
            formatter,
            "IcSnapshotUploadDataObservationResponse",
        )
    }
}

#[cfg(test)]
mod tests;
