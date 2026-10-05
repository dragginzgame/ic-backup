//! Closed chronological attempt events and pure validated progress replay.

use super::{
    AttemptBudgetRecord, AttemptJournalRecordError, MutationOutcomeRecord, ObservationOutcomeRecord,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "event", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum AttemptEventRecord {
    MutationReserved {
        attempt: u32,
    },
    ObservationReserved {
        attempt: u32,
        mutation: u32,
        request: String,
    },
    MutationResolved {
        mutation: u32,
        outcome: MutationOutcomeRecord,
        evidence: String,
    },
    ObservationRecorded {
        observation: u32,
        request: String,
        outcome: ObservationOutcomeRecord,
        evidence: String,
    },
}

impl AttemptEventRecord {
    pub(super) fn append_digest_bytes(&self, bytes: &mut Vec<u8>) {
        match self {
            Self::MutationReserved { attempt } => {
                bytes.push(0);
                bytes.extend_from_slice(&attempt.to_be_bytes());
            }
            Self::ObservationReserved {
                attempt,
                mutation,
                request,
            } => {
                bytes.push(1);
                bytes.extend_from_slice(&attempt.to_be_bytes());
                bytes.extend_from_slice(&mutation.to_be_bytes());
                bytes.extend_from_slice(request.as_bytes());
            }
            Self::MutationResolved {
                mutation,
                outcome,
                evidence,
            } => {
                bytes.push(2);
                bytes.extend_from_slice(&mutation.to_be_bytes());
                bytes.push(match outcome {
                    MutationOutcomeRecord::Applied => 0,
                    MutationOutcomeRecord::NotApplied => 1,
                });
                bytes.extend_from_slice(evidence.as_bytes());
            }
            Self::ObservationRecorded {
                observation,
                request,
                outcome,
                evidence,
            } => {
                bytes.push(3);
                bytes.extend_from_slice(&observation.to_be_bytes());
                bytes.extend_from_slice(request.as_bytes());
                bytes.push(match outcome {
                    ObservationOutcomeRecord::Applied => 0,
                    ObservationOutcomeRecord::NotApplied => 1,
                    ObservationOutcomeRecord::Uncertain => 2,
                });
                bytes.extend_from_slice(evidence.as_bytes());
            }
        }
    }
    pub(super) fn normalize(&mut self) -> Result<(), AttemptJournalRecordError> {
        match self {
            Self::MutationReserved { .. } => {}
            Self::ObservationReserved { request, .. } => *request = super::canonical_hash(request)?,
            Self::MutationResolved { evidence, .. } => *evidence = super::canonical_hash(evidence)?,
            Self::ObservationRecorded {
                request, evidence, ..
            } => {
                *request = super::canonical_hash(request)?;
                *evidence = super::canonical_hash(evidence)?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PendingObservation {
    pub attempt: u32,
    pub request: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Projection {
    pub next_attempt: u32,
    pub mutations_used: u32,
    pub observations_used: u32,
    pub pending_mutation: Option<u32>,
    pub pending_observation: Option<PendingObservation>,
    pub applied: bool,
}

impl Projection {
    pub(super) const fn empty() -> Self {
        Self {
            next_attempt: 1,
            mutations_used: 0,
            observations_used: 0,
            pending_mutation: None,
            pending_observation: None,
            applied: false,
        }
    }

    pub(super) fn apply(
        &mut self,
        event: &AttemptEventRecord,
        budget: &AttemptBudgetRecord,
    ) -> Result<(), AttemptJournalRecordError> {
        if self.applied {
            return Err(AttemptJournalRecordError::AlreadyApplied);
        }
        match event {
            AttemptEventRecord::MutationReserved { attempt } => {
                self.check_observation_ended()?;
                if let Some(attempt) = self.pending_mutation {
                    return Err(AttemptJournalRecordError::MutationPending { attempt });
                }
                if self.mutations_used == budget.mutations() {
                    return Err(AttemptJournalRecordError::MutationBudgetExhausted);
                }
                self.check_sequence(*attempt)?;
                self.mutations_used += 1;
                self.pending_mutation = Some(*attempt);
                self.next_attempt += 1;
            }
            AttemptEventRecord::ObservationReserved {
                attempt,
                mutation,
                request,
            } => {
                self.check_observation_ended()?;
                self.check_mutation(*mutation)?;
                if self.observations_used == budget.observations() {
                    return Err(AttemptJournalRecordError::ObservationBudgetExhausted);
                }
                self.check_sequence(*attempt)?;
                self.observations_used += 1;
                self.pending_observation = Some(PendingObservation {
                    attempt: *attempt,
                    request: request.clone(),
                });
                self.next_attempt += 1;
            }
            AttemptEventRecord::MutationResolved {
                mutation, outcome, ..
            } => {
                self.check_observation_ended()?;
                self.check_mutation(*mutation)?;
                self.pending_mutation = None;
                self.applied = *outcome == MutationOutcomeRecord::Applied;
            }
            AttemptEventRecord::ObservationRecorded {
                observation,
                request,
                outcome,
                ..
            } => {
                let pending = self
                    .pending_observation
                    .as_ref()
                    .ok_or(AttemptJournalRecordError::NoPendingObservation)?;
                check_attempt(pending.attempt, *observation)?;
                if pending.request != *request {
                    return Err(AttemptJournalRecordError::RequestMismatch);
                }
                self.pending_observation = None;
                match outcome {
                    ObservationOutcomeRecord::Applied => {
                        self.pending_mutation = None;
                        self.applied = true;
                    }
                    ObservationOutcomeRecord::NotApplied => self.pending_mutation = None,
                    ObservationOutcomeRecord::Uncertain => {}
                }
            }
        }
        Ok(())
    }

    fn check_sequence(&self, actual: u32) -> Result<(), AttemptJournalRecordError> {
        check_attempt(self.next_attempt, actual)
    }
    fn check_mutation(&self, actual: u32) -> Result<(), AttemptJournalRecordError> {
        check_attempt(
            self.pending_mutation
                .ok_or(AttemptJournalRecordError::NoPendingMutation)?,
            actual,
        )
    }
    fn check_observation_ended(&self) -> Result<(), AttemptJournalRecordError> {
        if let Some(pending) = &self.pending_observation {
            return Err(AttemptJournalRecordError::ObservationPending {
                attempt: pending.attempt,
            });
        }
        Ok(())
    }
}

fn check_attempt(expected: u32, actual: u32) -> Result<(), AttemptJournalRecordError> {
    if expected != actual {
        return Err(AttemptJournalRecordError::AttemptMismatch { expected, actual });
    }
    Ok(())
}
