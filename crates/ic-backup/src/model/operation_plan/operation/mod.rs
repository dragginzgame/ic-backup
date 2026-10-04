//! Exact declared target/request and original per-operation finite allowance.

use super::{OperationPlanError, canonical_hash};
use crate::model::attempt_journal::AttemptBudgetRecord;
use serde::{Deserialize, Serialize};

/// Passive exact operation binding supplied by its qualified request codec owner.
#[derive(Clone, Debug)]
pub struct PlannedOperationRequest {
    /// Exact graph operation identity, including zero/nonconsecutive sequences.
    pub operation_sequence: u64,
    /// Exact physical principal, admitted against explicit selection by the plan.
    pub target: String,
    /// Canonical exact mutating request digest from its owner; not rendered text.
    pub request: String,
    /// Original finite mutation/observation limits, never replenished on derivation.
    pub budget: AttemptBudgetRecord,
}
/// Immutable canonical declared target/request/allowance for one exact graph node.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "OperationFields")]
pub struct PlannedOperationRecord {
    operation_sequence: u64,
    target: String,
    request: String,
    budget: AttemptBudgetRecord,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OperationFields {
    operation_sequence: u64,
    target: String,
    request: String,
    budget: AttemptBudgetRecord,
}
impl TryFrom<OperationFields> for PlannedOperationRecord {
    type Error = OperationPlanError;
    fn try_from(fields: OperationFields) -> Result<Self, Self::Error> {
        Self::new(PlannedOperationRequest {
            operation_sequence: fields.operation_sequence,
            target: fields.target,
            request: fields.request,
            budget: fields.budget,
        })
    }
}
impl PlannedOperationRecord {
    /// Admit canonical declared operation identity; the plan validates selection/graph binding.
    ///
    /// # Errors
    /// Rejects malformed principal and request SHA-256 digest text.
    pub fn new(request: PlannedOperationRequest) -> Result<Self, OperationPlanError> {
        Ok(Self {
            operation_sequence: request.operation_sequence,
            target: crate::model::principal::canonical_text(&request.target)
                .ok_or(OperationPlanError::InvalidPrincipal("target"))?,
            request: canonical_hash(&request.request)?,
            budget: request.budget,
        })
    }
    /// Read the exact opaque graph operation sequence.
    #[must_use]
    pub const fn operation_sequence(&self) -> u64 {
        self.operation_sequence
    }
    /// Read the exact canonical physical target principal.
    #[must_use]
    pub fn target(&self) -> &str {
        &self.target
    }
    /// Read the exact canonical integration-owned request digest.
    #[must_use]
    pub fn request(&self) -> &str {
        &self.request
    }
    /// Read the original finite operation attempt limits.
    #[must_use]
    pub const fn budget(&self) -> &AttemptBudgetRecord {
        &self.budget
    }
}
