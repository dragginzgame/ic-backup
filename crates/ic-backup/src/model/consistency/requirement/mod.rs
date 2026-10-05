//! Immutable declared consistency choice; never current fence evidence.

use crate::model::{
    artifacts::{ArtifactChecksumRecord, ChecksumError},
    operation_plan::OperationPlanRecord,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Maximum raw input and canonical output bytes for a retained requirement.
pub const MAX_CONSISTENCY_REQUIREMENT_BYTES: u64 = 1024;
/// Explicit maintained consistency declaration; neither choice is proven by a record.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsistencyGuaranteeRecord {
    /// Each stopped target is separately qualified; no atomic application checkpoint.
    PerCanister,
    /// Application owns a retained whole-selection fence and drained-work proof.
    ApplicationCoordinated,
}
impl ConsistencyGuaranteeRecord {
    pub(super) const fn tag(self) -> u8 {
        match self {
            Self::PerCanister => 0,
            Self::ApplicationCoordinated => 1,
        }
    }
}
/// Immutable v1 original-plan-bound requested guarantee, separate from fresh observations.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "RequirementFields")]
pub struct ConsistencyRequirementRecord {
    version: u16,
    plan_intent: String,
    guarantee: ConsistencyGuaranteeRecord,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RequirementFields {
    version: u16,
    plan_intent: String,
    guarantee: ConsistencyGuaranteeRecord,
}
impl TryFrom<RequirementFields> for ConsistencyRequirementRecord {
    type Error = ConsistencyRequirementError;
    fn try_from(fields: RequirementFields) -> Result<Self, Self::Error> {
        if fields.version != 1 {
            return Err(ConsistencyRequirementError::UnsupportedVersion(
                fields.version,
            ));
        }
        Ok(Self {
            version: 1,
            plan_intent: ArtifactChecksumRecord::from_hash(&fields.plan_intent)?
                .hash()
                .into(),
            guarantee: fields.guarantee,
        })
    }
}
impl ConsistencyRequirementRecord {
    /// Declare a guarantee for the full original plan; does not establish consistency.
    #[must_use]
    pub fn new(plan: &OperationPlanRecord, guarantee: ConsistencyGuaranteeRecord) -> Self {
        Self {
            version: 1,
            plan_intent: plan.digest().hash().into(),
            guarantee,
        }
    }
    /// Read canonical original full plan intent, including selection and original budgets.
    #[must_use]
    pub fn plan_intent(&self) -> &str {
        &self.plan_intent
    }
    /// Read the immutable requested guarantee.
    #[must_use]
    pub const fn guarantee(&self) -> ConsistencyGuaranteeRecord {
        self.guarantee
    }
    /// Match the exact original plan; declarations never upgrade a guarantee on recovery.
    /// # Errors
    /// Rejects a different full plan intent.
    pub fn validate_plan(
        &self,
        plan: &OperationPlanRecord,
    ) -> Result<(), ConsistencyRequirementError> {
        if self.plan_intent != plan.digest().hash() {
            return Err(ConsistencyRequirementError::PlanMismatch);
        }
        Ok(())
    }
    /// Hash NUL-terminated v1 ASCII domain, 64 ASCII original-intent bytes and guarantee tag.
    ///
    /// Tags are `per_canister=0` and `application_coordinated=1`. No current evidence,
    /// timestamps or editable derived hashes are included or retained in this record.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        let mut bytes = b"ic-backup/consistency-requirement/v1\0".to_vec();
        bytes.extend_from_slice(self.plan_intent.as_bytes());
        bytes.push(self.guarantee.tag());
        ArtifactChecksumRecord::from_bytes(&bytes)
    }
}
/// Typed immutable declaration rejection; no error changes original authority.
#[derive(Debug, Error)]
pub enum ConsistencyRequirementError {
    /// Only v1 is maintained.
    #[error("unsupported consistency requirement version {0}")]
    UnsupportedVersion(u16),
    /// Requirement belongs to another full original plan.
    #[error("consistency requirement original plan mismatch")]
    PlanMismatch,
    /// Original intent is not a canonicalizable SHA-256 value.
    #[error(transparent)]
    Checksum(#[from] ChecksumError),
}
