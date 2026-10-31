//! Exact originally reserved metadata/data reads; no dispatch or transfer attestation.

use crate::model::attempt_journal::MutationReservation;
use crate::model::attempt_journal::MutationReservationError as Reservation;
use crate::model::ic_update_response::IcUpdateResponseInput;
use crate::model::{
    artifacts::ArtifactChecksumRecord,
    attempt_journal::{AttemptAuthorityRecord, AttemptJournalRecord},
    ic_snapshot_data::{IcSnapshotDataRequest, MAX_IC_SNAPSHOT_DATA_REPLY_BYTES},
    ic_snapshot_metadata::IcSnapshotMetadataRequest,
    operation_plan::{OperationPlanError, OperationPlanRecord},
};
use std::fmt;
use thiserror::Error;

/// The existing two bounded replicated-update read payloads; no new encoder.
#[derive(Clone, Copy, Debug)]
pub enum IcSnapshotTransferReadPayload<'request, 'metadata> {
    /// Read the exact original raw snapshot ID's metadata.
    Metadata(&'request IcSnapshotMetadataRequest),
    /// Read one metadata-bound range or known chunk hash.
    Data(&'request IcSnapshotDataRequest<'metadata>),
}

impl<'request> IcSnapshotTransferReadPayload<'request, '_> {
    pub(crate) fn validate_binding(
        &self,
        authority: &AttemptAuthorityRecord,
    ) -> Result<(), IcSnapshotTransferReadError> {
        if self.target() != authority.binding().target()
            || self.digest().hash() != authority.binding().request()
        {
            return Err(IcSnapshotTransferReadError::PayloadMismatch);
        }
        Ok(())
    }
    /// Read the canonical effective routing target.
    #[must_use]
    pub fn target(&self) -> &'request str {
        match self {
            Self::Metadata(payload) => payload.target(),
            Self::Data(payload) => payload.target(),
        }
    }
    /// Read the management receiver; both methods use host replicated updates.
    #[must_use]
    pub const fn receiver(&self) -> &'static str {
        match self {
            Self::Metadata(payload) => payload.receiver(),
            Self::Data(payload) => payload.receiver(),
        }
    }
    /// Read the exact original method name.
    #[must_use]
    pub const fn method(&self) -> &'static str {
        match self {
            Self::Metadata(payload) => payload.method(),
            Self::Data(payload) => payload.method(),
        }
    }
    /// Read existing canonical bounded Candid bytes.
    #[must_use]
    pub fn arguments(&self) -> &'request [u8] {
        match self {
            Self::Metadata(payload) => payload.arguments(),
            Self::Data(payload) => payload.arguments(),
        }
    }
    /// Reuse the original nonrecursive wire digest; metadata evidence stays separate.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        match self {
            Self::Metadata(payload) => payload.digest(),
            Self::Data(payload) => payload.digest(),
        }
    }
}

/// Full original plan and already consumed update reservation for one transfer read.
///
/// A semantic read still uses replicated update ingress and the existing mutation
/// reservation lane. It grants no fresh permission or proof of never-dispatched
/// custody; reconstruction never permits repeating a lost read.
#[derive(Debug)]
pub struct IcSnapshotTransferReadRequest<'request, 'metadata> {
    plan: &'request OperationPlanRecord,
    payload: IcSnapshotTransferReadPayload<'request, 'metadata>,
    reservation: MutationReservation,
}

impl<'request, 'metadata> IcSnapshotTransferReadRequest<'request, 'metadata> {
    /// Bind exact original target/wire bytes and immutable original allowance.
    /// # Errors
    /// Rejects another plan/operation/payload, absent or changed update reservation,
    /// or already pending recovery. Creates no journal and spends nothing.
    pub fn new(
        plan: &'request OperationPlanRecord,
        operation_sequence: u64,
        journal: &AttemptJournalRecord,
        payload: IcSnapshotTransferReadPayload<'request, 'metadata>,
    ) -> Result<Self, IcSnapshotTransferReadError> {
        let authority = plan.attempt_authority(operation_sequence)?;
        if journal.authority() != &authority {
            return Err(IcSnapshotTransferReadError::AuthorityMismatch);
        }
        payload.validate_binding(&authority)?;
        let reservation = MutationReservation::new(authority, journal)?;
        Ok(Self {
            plan,
            payload,
            reservation,
        })
    }
    /// Read full original context, inventory, graph and allowances.
    #[must_use]
    pub const fn plan(&self) -> &'request OperationPlanRecord {
        self.plan
    }
    /// Read the exact metadata/data payload without reconstructing its bytes.
    #[must_use]
    pub const fn payload(&self) -> IcSnapshotTransferReadPayload<'request, 'metadata> {
        self.payload
    }
    /// Read the full original operation authority.
    #[must_use]
    pub const fn authority(&self) -> &AttemptAuthorityRecord {
        &self.reservation.authority
    }
    /// Read the already consumed replicated-update attempt.
    #[must_use]
    pub const fn mutation_attempt(&self) -> u32 {
        self.reservation.mutation_attempt
    }
    /// Recheck original reservation before passive response association.
    /// # Errors
    /// Rejects changed authority, settled/replaced update or pending recovery.
    pub fn validate_journal(
        &self,
        journal: &AttemptJournalRecord,
    ) -> Result<(), IcSnapshotTransferReadError> {
        self.reservation.validate(journal).map_err(Into::into)
    }
}

/// Bounded immutable raw reply, with no receipt, persisted authority or default.
#[derive(Clone)]
pub struct IcSnapshotTransferReadResponse {
    input: IcUpdateResponseInput,
}

impl IcSnapshotTransferReadResponse {
    /// Admit the existing finite raw-data ceiling and original attempt range.
    /// # Errors
    /// Rejects invalid attempt, excessive bytes and invalid target principals.
    pub fn new(mut input: IcUpdateResponseInput) -> Result<Self, IcSnapshotTransferReadError> {
        input.admit(MAX_IC_SNAPSHOT_DATA_REPLY_BYTES)?;
        Ok(Self { input })
    }
    /// Read exact retained claims and raw bytes; no mutable access is exposed.
    #[must_use]
    pub const fn input(&self) -> &IcUpdateResponseInput {
        &self.input
    }
}

impl fmt::Debug for IcSnapshotTransferReadResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.input
            .fmt_redacted(formatter, "IcSnapshotTransferReadResponse")
    }
}

/// Structural read rejection; every denial retains original accounting and evidence.
#[derive(Debug, Error)]
pub enum IcSnapshotTransferReadError {
    /// Current journal differs from full original authority.
    #[error("snapshot transfer read authority differs")]
    AuthorityMismatch,
    /// Original target or exact wire digest differs.
    #[error("snapshot transfer read payload differs")]
    PayloadMismatch,
    /// No original replicated-update reservation is pending.
    #[error("snapshot transfer read requires a pending original update")]
    NoPendingMutation,
    /// Original update was changed or settled.
    #[error("snapshot transfer read original attempt differs")]
    MutationMismatch,
    /// Existing recovery observation remains pending.
    #[error("snapshot transfer read recovery observation is pending")]
    ObservationPending,
    /// Attempt is outside the existing 1..=1,024 bound.
    #[error("invalid snapshot transfer read attempt")]
    InvalidAttempt,
    /// Raw reply exceeds the existing 2 MiB data wire ceiling.
    #[error("snapshot transfer read reply too large")]
    ReplyTooLarge,
    /// Claimed actual target is not a principal.
    #[error("invalid snapshot transfer read target")]
    InvalidTarget,
    /// Original plan cannot derive this operation authority.
    #[error(transparent)]
    Plan(#[from] OperationPlanError),
}

impl From<crate::model::attempt_journal::MutationReservationError> for IcSnapshotTransferReadError {
    fn from(error: crate::model::attempt_journal::MutationReservationError) -> Self {
        match error {
            Reservation::NoPendingMutation => Self::NoPendingMutation,
            Reservation::AuthorityMismatch => Self::AuthorityMismatch,
            Reservation::MutationMismatch => Self::MutationMismatch,
            Reservation::ObservationPending => Self::ObservationPending,
        }
    }
}

impl From<crate::model::ic_update_response::UpdateResponseAdmissionError>
    for IcSnapshotTransferReadError
{
    fn from(error: crate::model::ic_update_response::UpdateResponseAdmissionError) -> Self {
        match error {
            crate::model::ic_update_response::UpdateResponseAdmissionError::InvalidAttempt => {
                Self::InvalidAttempt
            }
            crate::model::ic_update_response::UpdateResponseAdmissionError::ReplyTooLarge => {
                Self::ReplyTooLarge
            }
            crate::model::ic_update_response::UpdateResponseAdmissionError::InvalidTarget => {
                Self::InvalidTarget
            }
        }
    }
}
