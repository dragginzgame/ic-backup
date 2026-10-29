//! Closed chronological attempt events and pure validated progress replay.

use super::{
    AttemptBudgetRecord, AttemptJournalRecordError, MutationOutcomeRecord, ObservationOutcomeRecord,
};
use crate::model::artifacts::canonical_hash;
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
    pub(super) fn append_digest_bytes(
        &self,
        bytes: &mut crate::model::evidence_digest::EvidenceDigest,
    ) {
        match self {
            Self::MutationReserved { attempt } => {
                bytes.byte(0);
                bytes.u32_be(*attempt);
            }
            Self::ObservationReserved {
                attempt,
                mutation,
                request,
            } => {
                bytes.byte(1);
                bytes.u32_be(*attempt);
                bytes.u32_be(*mutation);
                bytes.append(request.as_bytes());
            }
            Self::MutationResolved {
                mutation,
                outcome,
                evidence,
            } => {
                bytes.byte(2);
                bytes.u32_be(*mutation);
                bytes.byte(match outcome {
                    MutationOutcomeRecord::Applied => 0,
                    MutationOutcomeRecord::NotApplied => 1,
                });
                bytes.append(evidence.as_bytes());
            }
            Self::ObservationRecorded {
                observation,
                request,
                outcome,
                evidence,
            } => {
                bytes.byte(3);
                bytes.u32_be(*observation);
                bytes.append(request.as_bytes());
                bytes.byte(match outcome {
                    ObservationOutcomeRecord::Applied => 0,
                    ObservationOutcomeRecord::NotApplied => 1,
                    ObservationOutcomeRecord::Uncertain => 2,
                });
                bytes.append(evidence.as_bytes());
            }
        }
    }
    pub(super) fn normalize(&mut self) -> Result<(), AttemptJournalRecordError> {
        match self {
            Self::MutationReserved { .. } => {}
            Self::ObservationReserved { request, .. } => *request = canonical_hash(request)?,
            Self::MutationResolved { evidence, .. } => *evidence = canonical_hash(evidence)?,
            Self::ObservationRecorded {
                request, evidence, ..
            } => {
                *request = canonical_hash(request)?;
                *evidence = canonical_hash(evidence)?;
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
    state: AttemptState,
}

// An observation can exist only while its original mutation remains pending.
#[derive(Clone, Debug, Eq, PartialEq)]
enum AttemptState {
    Idle,
    Mutation(u32),
    Observation {
        mutation: u32,
        pending: PendingObservation,
    },
    Applied,
}

impl Projection {
    pub(super) const fn empty() -> Self {
        Self {
            next_attempt: 1,
            mutations_used: 0,
            observations_used: 0,
            state: AttemptState::Idle,
        }
    }

    pub(super) const fn pending_mutation(&self) -> Option<u32> {
        match self.state {
            AttemptState::Mutation(attempt)
            | AttemptState::Observation {
                mutation: attempt, ..
            } => Some(attempt),
            AttemptState::Idle | AttemptState::Applied => None,
        }
    }

    pub(super) const fn pending_observation(&self) -> Option<&PendingObservation> {
        match &self.state {
            AttemptState::Observation { pending, .. } => Some(pending),
            _ => None,
        }
    }

    pub(super) const fn applied(&self) -> bool {
        matches!(self.state, AttemptState::Applied)
    }

    pub(super) fn apply(
        &mut self,
        event: &AttemptEventRecord,
        budget: &AttemptBudgetRecord,
    ) -> Result<(), AttemptJournalRecordError> {
        if self.applied() {
            return Err(AttemptJournalRecordError::AlreadyApplied);
        }
        match event {
            AttemptEventRecord::MutationReserved { attempt } => {
                self.check_observation_ended()?;
                if let Some(attempt) = self.pending_mutation() {
                    return Err(AttemptJournalRecordError::MutationPending { attempt });
                }
                if self.mutations_used == budget.mutations() {
                    return Err(AttemptJournalRecordError::MutationBudgetExhausted);
                }
                self.check_sequence(*attempt)?;
                self.mutations_used += 1;
                self.state = AttemptState::Mutation(*attempt);
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
                self.state = AttemptState::Observation {
                    mutation: *mutation,
                    pending: PendingObservation {
                        attempt: *attempt,
                        request: request.clone(),
                    },
                };
                self.next_attempt += 1;
            }
            AttemptEventRecord::MutationResolved {
                mutation, outcome, ..
            } => {
                self.check_observation_ended()?;
                self.check_mutation(*mutation)?;
                self.state = match outcome {
                    MutationOutcomeRecord::Applied => AttemptState::Applied,
                    MutationOutcomeRecord::NotApplied => AttemptState::Idle,
                };
            }
            AttemptEventRecord::ObservationRecorded {
                observation,
                request,
                outcome,
                ..
            } => {
                let AttemptState::Observation { mutation, pending } = &self.state else {
                    return Err(AttemptJournalRecordError::NoPendingObservation);
                };
                check_attempt(pending.attempt, *observation)?;
                if pending.request != *request {
                    return Err(AttemptJournalRecordError::RequestMismatch);
                }
                self.state = match outcome {
                    ObservationOutcomeRecord::Applied => AttemptState::Applied,
                    ObservationOutcomeRecord::NotApplied => AttemptState::Idle,
                    ObservationOutcomeRecord::Uncertain => AttemptState::Mutation(*mutation),
                };
            }
        }
        Ok(())
    }

    fn check_sequence(&self, actual: u32) -> Result<(), AttemptJournalRecordError> {
        check_attempt(self.next_attempt, actual)
    }
    fn check_mutation(&self, actual: u32) -> Result<(), AttemptJournalRecordError> {
        check_attempt(
            self.pending_mutation()
                .ok_or(AttemptJournalRecordError::NoPendingMutation)?,
            actual,
        )
    }
    fn check_observation_ended(&self) -> Result<(), AttemptJournalRecordError> {
        if let Some(pending) = self.pending_observation() {
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
