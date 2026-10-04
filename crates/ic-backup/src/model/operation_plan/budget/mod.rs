//! Immutable declared plan ceilings over original assigned per-operation allowances.

use super::OperationPlanError;
use serde::{Deserialize, Serialize};

/// Maximum combined mutation/observation attempts assigned by one declared plan.
pub const MAX_PLAN_ATTEMPTS: u32 = 65536;
/// Immutable declared aggregate attempt ceilings; this is not cycle accounting.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "BudgetFields")]
pub struct PlanBudgetRecord {
    mutations: u32,
    observations: u32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BudgetFields {
    mutations: u32,
    observations: u32,
}
impl TryFrom<BudgetFields> for PlanBudgetRecord {
    type Error = OperationPlanError;
    fn try_from(fields: BudgetFields) -> Result<Self, Self::Error> {
        Self::new(fields.mutations, fields.observations)
    }
}
impl PlanBudgetRecord {
    /// Admit original finite aggregate ceilings, including zero, using checked arithmetic.
    ///
    /// # Errors
    /// Rejects overflow or combined allowance above [`MAX_PLAN_ATTEMPTS`].
    pub fn new(mutations: u32, observations: u32) -> Result<Self, OperationPlanError> {
        if mutations
            .checked_add(observations)
            .is_none_or(|total| total > MAX_PLAN_ATTEMPTS)
        {
            return Err(OperationPlanError::BudgetTooLarge);
        }
        Ok(Self {
            mutations,
            observations,
        })
    }
    /// Read original declared aggregate mutation ceiling.
    #[must_use]
    pub const fn mutations(&self) -> u32 {
        self.mutations
    }
    /// Read original declared aggregate observation ceiling.
    #[must_use]
    pub const fn observations(&self) -> u32 {
        self.observations
    }
    pub(super) const fn admits(&self, allocated: AllocatedAttemptsView) -> bool {
        allocated.mutations <= self.mutations && allocated.observations <= self.observations
    }
}
/// Read-only assigned attempt totals, not consumed allowance or observed IC debits.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct AllocatedAttemptsView {
    /// Sum of immutable per-operation mutation allowances.
    pub mutations: u32,
    /// Sum of immutable per-operation observation allowances.
    pub observations: u32,
}
