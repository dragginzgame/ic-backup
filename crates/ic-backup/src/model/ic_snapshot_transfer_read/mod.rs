//! Exact originally reserved metadata/data reads; no dispatch or transfer attestation.

use crate::model::{
    artifacts::ArtifactChecksumRecord,
    attempt_journal::{AttemptAuthorityRecord, AttemptJournalRecord, MAX_OPERATION_ATTEMPTS},
    ic_snapshot_data::{IcSnapshotDataRequest, MAX_IC_SNAPSHOT_DATA_REPLY_BYTES},
    ic_snapshot_metadata::IcSnapshotMetadataRequest,
    operation_plan::{OperationPlanError, OperationPlanRecord, PlanContextRecord},
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

impl IcSnapshotTransferReadPayload<'_, '_> {
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
    pub fn target(&self) -> &str {
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
    pub fn arguments(&self) -> &[u8] {
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
    authority: AttemptAuthorityRecord,
    mutation_attempt: u32,
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
        let mutation_attempt = journal
            .view()
            .pending_mutation
            .ok_or(IcSnapshotTransferReadError::NoPendingMutation)?;
        let request = Self {
            plan,
            payload,
            authority,
            mutation_attempt,
        };
        request.validate_journal(journal)?;
        Ok(request)
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
        &self.authority
    }
    /// Read the already consumed replicated-update attempt.
    #[must_use]
    pub const fn mutation_attempt(&self) -> u32 {
        self.mutation_attempt
    }
    /// Recheck original reservation before passive response association.
    /// # Errors
    /// Rejects changed authority, settled/replaced update or pending recovery.
    pub fn validate_journal(
        &self,
        journal: &AttemptJournalRecord,
    ) -> Result<(), IcSnapshotTransferReadError> {
        if journal.authority() != &self.authority {
            return Err(IcSnapshotTransferReadError::AuthorityMismatch);
        }
        let view = journal.view();
        if view.pending_mutation != Some(self.mutation_attempt) {
            return Err(IcSnapshotTransferReadError::MutationMismatch);
        }
        if view.pending_observation.is_some() {
            return Err(IcSnapshotTransferReadError::ObservationPending);
        }
        Ok(())
    }
}

/// Passive provider association claims; integrations authenticate these independently.
#[derive(Clone)]
pub struct IcSnapshotTransferReadResponseInput {
    /// Full original plan/context/operation/request/allowance authority digest.
    pub authority: ArtifactChecksumRecord,
    /// Already reserved replicated-update number.
    pub mutation_attempt: u32,
    /// Actual claimed network/caller/release, without credentials.
    pub context: PlanContextRecord,
    /// Actual claimed routing target; normalized on admission.
    pub target: String,
    /// Exact raw reply. Data is capped at 2 MiB; metadata decoding retains 1 MiB.
    pub reply: Vec<u8>,
    /// Opaque evidence binding the original read and actual response claims.
    pub evidence: ArtifactChecksumRecord,
}

/// Bounded immutable raw reply, with no receipt, persisted authority or default.
#[derive(Clone)]
pub struct IcSnapshotTransferReadResponse {
    input: IcSnapshotTransferReadResponseInput,
}

impl IcSnapshotTransferReadResponse {
    /// Admit the existing finite raw-data ceiling and original attempt range.
    /// # Errors
    /// Rejects invalid attempt, excessive bytes and invalid target principals.
    pub fn new(
        mut input: IcSnapshotTransferReadResponseInput,
    ) -> Result<Self, IcSnapshotTransferReadError> {
        if input.mutation_attempt == 0 || input.mutation_attempt > MAX_OPERATION_ATTEMPTS {
            return Err(IcSnapshotTransferReadError::InvalidAttempt);
        }
        if input.reply.len() > MAX_IC_SNAPSHOT_DATA_REPLY_BYTES {
            return Err(IcSnapshotTransferReadError::ReplyTooLarge);
        }
        input.target = crate::model::principal::canonical_text(&input.target)
            .ok_or(IcSnapshotTransferReadError::InvalidTarget)?;
        Ok(Self { input })
    }
    /// Read exact retained claims and raw bytes; no mutable access is exposed.
    #[must_use]
    pub const fn input(&self) -> &IcSnapshotTransferReadResponseInput {
        &self.input
    }
}

impl fmt::Debug for IcSnapshotTransferReadResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IcSnapshotTransferReadResponse")
            .field("authority", &self.input.authority)
            .field("mutation_attempt", &self.input.mutation_attempt)
            .field("target", &self.input.target)
            .field("reply_bytes", &self.input.reply.len())
            .finish_non_exhaustive()
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
