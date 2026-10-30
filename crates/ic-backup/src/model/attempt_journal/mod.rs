//! Durable finite local attempt accounting adapted from Canic's pending/receipt contracts.

use crate::model::evidence_digest::EvidenceDigest;
mod authority;
mod history;
mod mutation;
pub(crate) use mutation::{MutationReservation, MutationReservationError};
mod observation;
pub use authority::{
    AttemptAuthorityRecord, AttemptBudgetRecord, MAX_OPERATION_ATTEMPTS, OperationBindingRecord,
    OperationBindingRequest,
};
pub(crate) use observation::{ObservationReservation, ObservationReservationError};

use crate::model::{
    artifacts::{ArtifactChecksumRecord, ChecksumError, canonical_hash},
    bounded_sequence,
};
use history::{AttemptEventRecord, Projection};
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

/// Maximum retained reservation/receipt events in one operation journal.
pub const MAX_ATTEMPT_EVENTS: usize = 2048;
/// Maximum encoded input and canonical output bytes admitted by persistence ops.
pub const MAX_ATTEMPT_JOURNAL_BYTES: u64 = 1024 * 1024;

/// Integration-qualified direct mutation outcome; an invocation failure is insufficient.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationOutcomeRecord {
    /// Exact requested effect is qualified as applied.
    Applied,
    /// Exact requested effect is qualified as not applied; consumed allowance remains spent.
    NotApplied,
}

/// Integration-qualified observation outcome for its exact reserved mutation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationOutcomeRecord {
    /// Observation qualifies exact requested effect as applied.
    Applied,
    /// Observation qualifies exact requested effect as not applied.
    NotApplied,
    /// Settled observation cannot resolve the mutation; no allowance is restored.
    Uncertain,
}

/// Passive exact direct-reply evidence supplied by its qualified owner.
#[derive(Clone, Debug)]
pub struct MutationReceiptRequest {
    /// Previously reserved mutation attempt number.
    pub attempt: u32,
    /// Exact mutating request digest, checked against immutable authority.
    pub request: String,
    /// Qualified effect outcome; local process exit alone cannot produce it.
    pub outcome: MutationOutcomeRecord,
    /// Canonical digest of retained owner evidence.
    pub evidence: String,
}

/// Passive exact reconciliation-reply evidence supplied by its qualified owner.
///
/// The owner qualifies observation completion, custody and paid-effect settlement.
/// A lost observation reply stays pending; it cannot be recorded as `Uncertain`
/// merely because the local invocation failed or its response was lost.
#[derive(Clone, Debug)]
pub struct ObservationReceiptRequest {
    /// Previously reserved observation attempt number.
    pub attempt: u32,
    /// Exact observation request digest, checked against its reservation.
    pub request: String,
    /// Qualified outcome, including explicit unresolved evidence.
    pub outcome: ObservationOutcomeRecord,
    /// Canonical digest of retained owner evidence.
    pub evidence: String,
}

/// Read-only derived local accounting; does not schedule or authorize effects.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AttemptJournalView {
    /// Exact immutable authority digest including original budgets.
    pub authority: ArtifactChecksumRecord,
    /// Mutation attempts already consumed, without refunds.
    pub mutations_used: u32,
    /// Reconciliation observations already consumed, without refunds.
    pub observations_used: u32,
    /// Original mutation allowance still unconsumed.
    pub mutations_remaining: u32,
    /// Original observation allowance still unconsumed.
    pub observations_remaining: u32,
    /// Exact unresolved mutation attempt, if any.
    pub pending_mutation: Option<u32>,
    /// Exact unresolved observation attempt, if any.
    pub pending_observation: Option<u32>,
    /// Retained qualified application of this operation, not full run completion.
    pub applied: bool,
    /// Number of retained reservation and receipt events.
    pub history_len: usize,
}

/// Exact locally retained evidence for an Applied operation.
///
/// The integration owns the evidence's meaning and qualification. In particular,
/// observation evidence is not the original mutation reply or retry authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppliedReceiptEvidence<'a> {
    /// The original mutation was settled by its direct qualified receipt.
    Mutation {
        /// Original mutation attempt number.
        attempt: u32,
        /// Digest of the retained integration-qualified direct evidence.
        evidence: &'a str,
    },
    /// A separately budgeted observation settled the original mutation.
    Observation {
        /// Separately reserved observation attempt number.
        attempt: u32,
        /// Exact observation request digest from its reservation.
        request: &'a str,
        /// Digest of retained integration-qualified observation evidence.
        evidence: &'a str,
    },
}

/// Maintained v1 append-only local reservations and receipts for one exact operation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "JournalFields")]
pub struct AttemptJournalRecord {
    version: u16,
    authority: AttemptAuthorityRecord,
    events: Vec<AttemptEventRecord>,
    #[serde(skip)]
    projection: Projection,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct JournalFields {
    version: u16,
    authority: AttemptAuthorityRecord,
    #[serde(deserialize_with = "bounded_events")]
    events: Vec<AttemptEventRecord>,
}

impl TryFrom<JournalFields> for AttemptJournalRecord {
    type Error = AttemptJournalRecordError;
    fn try_from(mut fields: JournalFields) -> Result<Self, Self::Error> {
        if fields.version != 1 {
            return Err(AttemptJournalRecordError::UnsupportedVersion(
                fields.version,
            ));
        }
        let mut projection = Projection::empty();
        for event in &mut fields.events {
            event.normalize()?;
            projection.apply(event, fields.authority.budget())?;
        }
        Ok(Self {
            version: 1,
            authority: fields.authority,
            events: fields.events,
            projection,
        })
    }
}

impl AttemptJournalRecord {
    /// Retain an exact validated authority with no consumed attempts.
    #[must_use]
    pub const fn new(authority: AttemptAuthorityRecord) -> Self {
        Self {
            version: 1,
            authority,
            events: Vec::new(),
            projection: Projection::empty(),
        }
    }
    /// Read exact declared identity and immutable ceilings.
    #[must_use]
    pub const fn authority(&self) -> &AttemptAuthorityRecord {
        &self.authority
    }
    // Keep passive request admission ordered: authority, mutation, then recovery.
    // Callers retain their own public error types; this grants no dispatch authority.
    pub(crate) fn validate_pending_mutation(
        &self,
        authority: &AttemptAuthorityRecord,
        mutation_attempt: u32,
    ) -> Result<(), MutationReservationError> {
        if self.authority() != authority {
            return Err(MutationReservationError::AuthorityMismatch);
        }
        if self.pending_mutation() != Some(mutation_attempt) {
            return Err(MutationReservationError::MutationMismatch);
        }
        if self.pending_observation().is_some() {
            return Err(MutationReservationError::ObservationPending);
        }
        Ok(())
    }
    // Reservation admission reads the validated projection without building the
    // public reporting view or hashing authority unrelated to these attempt IDs.
    pub(crate) const fn pending_mutation(&self) -> Option<u32> {
        self.projection.pending_mutation()
    }
    pub(crate) const fn pending_observation(&self) -> Option<u32> {
        match self.projection.pending_observation() {
            Some(pending) => Some(pending.attempt),
            None => None,
        }
    }
    /// Read the exact canonical request of the unresolved observation, if any.
    ///
    /// This projects existing validated history. It creates no reservation or
    /// repeat-dispatch permission and returns None after a settled observation.
    #[must_use]
    pub fn pending_observation_request(&self) -> Option<&str> {
        self.projection
            .pending_observation()
            .map(|pending| pending.request.as_str())
    }
    /// Read the terminal Applied receipt from validated original history.
    ///
    /// This does not authenticate evidence or grant dispatch, retry or release.
    #[must_use]
    pub fn applied_receipt(&self) -> Option<AppliedReceiptEvidence<'_>> {
        match self.events.last()? {
            AttemptEventRecord::MutationResolved {
                mutation,
                outcome: MutationOutcomeRecord::Applied,
                evidence,
            } => Some(AppliedReceiptEvidence::Mutation {
                attempt: *mutation,
                evidence,
            }),
            AttemptEventRecord::ObservationRecorded {
                observation,
                request,
                outcome: ObservationOutcomeRecord::Applied,
                evidence,
            } => Some(AppliedReceiptEvidence::Observation {
                attempt: *observation,
                request,
                evidence,
            }),
            _ => None,
        }
    }
    /// Project local evidence without IO or replenishing authority.
    ///
    /// Use `Self::applied_receipt` for the terminal receipt or `Self::digest`
    /// for exact full-history identity.
    #[must_use]
    pub fn view(&self) -> AttemptJournalView {
        AttemptJournalView {
            authority: self.authority.digest(),
            mutations_used: self.projection.mutations_used,
            observations_used: self.projection.observations_used,
            mutations_remaining: self.authority.budget().mutations()
                - self.projection.mutations_used,
            observations_remaining: self.authority.budget().observations()
                - self.projection.observations_used,
            pending_mutation: self.projection.pending_mutation(),
            pending_observation: self
                .projection
                .pending_observation()
                .map(|pending| pending.attempt),
            applied: self.projection.applied(),
            history_len: self.events.len(),
        }
    }
    /// Hash full original authority and every canonical chronological reservation/receipt.
    ///
    /// The NUL-terminated v1 domain precedes 64 ASCII authority hash bytes, a
    /// big-endian u64 event count and closed tagged event bytes. Derived projections,
    /// JSON formatting and filesystem paths are excluded. This authenticates no receipt.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        let mut bytes = EvidenceDigest::new(b"ic-backup/attempt-journal-history/v1\0");
        bytes.checksum(&self.authority.digest());
        bytes.u64_be(self.events.len() as u64);
        for event in &self.events {
            event.append_digest_bytes(&mut bytes);
        }
        bytes.finish()
    }
    pub(crate) fn reserve_mutation(&mut self) -> Result<u32, AttemptJournalRecordError> {
        let attempt = self.projection.next_attempt;
        self.append(AttemptEventRecord::MutationReserved { attempt })?;
        Ok(attempt)
    }
    pub(crate) fn reserve_observation(
        &mut self,
        mutation: u32,
        request: &str,
    ) -> Result<u32, AttemptJournalRecordError> {
        let attempt = self.projection.next_attempt;
        self.append(AttemptEventRecord::ObservationReserved {
            attempt,
            mutation,
            request: request.to_owned(),
        })?;
        Ok(attempt)
    }
    pub(crate) fn record_mutation(
        &mut self,
        receipt: MutationReceiptRequest,
    ) -> Result<(), AttemptJournalRecordError> {
        if canonical_hash(&receipt.request)? != self.authority.binding().request() {
            return Err(AttemptJournalRecordError::RequestMismatch);
        }
        self.append(AttemptEventRecord::MutationResolved {
            mutation: receipt.attempt,
            outcome: receipt.outcome,
            evidence: receipt.evidence,
        })
    }
    pub(crate) fn record_observation(
        &mut self,
        receipt: ObservationReceiptRequest,
    ) -> Result<(), AttemptJournalRecordError> {
        self.append(AttemptEventRecord::ObservationRecorded {
            observation: receipt.attempt,
            request: receipt.request,
            outcome: receipt.outcome,
            evidence: receipt.evidence,
        })
    }
    fn append(&mut self, mut event: AttemptEventRecord) -> Result<(), AttemptJournalRecordError> {
        if self.events.len() == MAX_ATTEMPT_EVENTS {
            return Err(AttemptJournalRecordError::HistoryTooLarge);
        }
        event.normalize()?;
        let mut projection = self.projection.clone();
        projection.apply(&event, self.authority.budget())?;
        self.events.push(event);
        self.projection = projection;
        Ok(())
    }
}

fn bounded_events<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<AttemptEventRecord>, D::Error> {
    bounded_sequence::deserialize::<_, _, MAX_ATTEMPT_EVENTS>(
        deserializer,
        "a bounded chronological attempt event list",
        AttemptJournalRecordError::HistoryTooLarge,
    )
}

/// Typed exact identity, finite accounting or chronological state rejection.
#[derive(Debug, Error)]
pub enum AttemptJournalRecordError {
    /// Other product generations are not maintained.
    #[error("unsupported attempt journal version {0}")]
    UnsupportedVersion(u16),
    /// Original limits overflow or exceed the bounded total.
    #[error("attempt allowance exceeds {MAX_OPERATION_ATTEMPTS}")]
    BudgetTooLarge,
    /// The chronological metadata count exceeds its finite bound.
    #[error("attempt history exceeds {MAX_ATTEMPT_EVENTS} events")]
    HistoryTooLarge,
    /// A selected principal is not admitted by its owning boundary.
    #[error("invalid attempt operation principal")]
    InvalidPrincipal,
    /// A digest fails exact canonical admission.
    #[error(transparent)]
    Checksum(#[from] ChecksumError),
    /// An unresolved paid mutation blocks another mutation.
    #[error("mutation attempt {attempt} remains unresolved")]
    MutationPending {
        /// Retained unresolved attempt number.
        attempt: u32,
    },
    /// An observation is still unsettled locally.
    #[error("observation attempt {attempt} remains unresolved")]
    ObservationPending {
        /// Retained unresolved attempt number.
        attempt: u32,
    },
    /// A receipt or reconciliation has no corresponding unresolved mutation.
    #[error("no pending mutation attempt")]
    NoPendingMutation,
    /// A reply has no corresponding unresolved observation.
    #[error("no pending observation attempt")]
    NoPendingObservation,
    /// A receipt or sequence targets another attempt.
    #[error("attempt identity mismatch: expected {expected}, actual {actual}")]
    AttemptMismatch {
        /// Expected exact attempt number.
        expected: u32,
        /// Rejected attempt number.
        actual: u32,
    },
    /// Reply/request identity differs from its retained exact reservation.
    #[error("attempt request digest mismatch")]
    RequestMismatch,
    /// Original mutation authority has been consumed.
    #[error("mutation attempt allowance exhausted")]
    MutationBudgetExhausted,
    /// Original observation authority has been consumed.
    #[error("observation attempt allowance exhausted")]
    ObservationBudgetExhausted,
    /// This exact operation is already qualified as applied.
    #[error("operation application already retained")]
    AlreadyApplied,
}

#[cfg(test)]
mod tests;
