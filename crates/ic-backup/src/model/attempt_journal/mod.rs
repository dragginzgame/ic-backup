//! Durable finite local attempt accounting adapted from Canic's pending/receipt contracts.

mod authority;
mod history;
pub use authority::{
    AttemptAuthorityRecord, AttemptBudgetRecord, MAX_OPERATION_ATTEMPTS, OperationBindingRecord,
    OperationBindingRequest,
};

use crate::model::artifacts::{ArtifactChecksumRecord, ChecksumError};
use history::{AttemptEventRecord, Projection};
use serde::{Deserialize, Deserializer, Serialize, de};
use std::fmt;
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
    /// Project local evidence without IO or replenishing authority.
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
            pending_mutation: self.projection.pending_mutation,
            pending_observation: self
                .projection
                .pending_observation
                .as_ref()
                .map(|pending| pending.attempt),
            applied: self.projection.applied,
            history_len: self.events.len(),
        }
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

fn canonical_hash(hash: &str) -> Result<String, AttemptJournalRecordError> {
    Ok(ArtifactChecksumRecord::from_hash(hash)?.hash().to_owned())
}

fn bounded_events<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<AttemptEventRecord>, D::Error> {
    struct EventsVisitor;
    impl<'de> de::Visitor<'de> for EventsVisitor {
        type Value = Vec<AttemptEventRecord>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a bounded chronological attempt event list")
        }
        fn visit_seq<A: de::SeqAccess<'de>>(
            self,
            mut sequence: A,
        ) -> Result<Self::Value, A::Error> {
            let mut events = Vec::new();
            while events.len() < MAX_ATTEMPT_EVENTS {
                match sequence.next_element()? {
                    Some(event) => events.push(event),
                    None => return Ok(events),
                }
            }
            if sequence.next_element::<de::IgnoredAny>()?.is_some() {
                return Err(de::Error::custom(
                    AttemptJournalRecordError::HistoryTooLarge,
                ));
            }
            Ok(events)
        }
    }
    deserializer.deserialize_seq(EventsVisitor)
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
