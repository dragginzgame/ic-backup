//! Immutable original application fence obligations; spending belongs to attempt journals.

use crate::model::{
    artifacts::ArtifactChecksumRecord,
    consistency::{
        ApplicationFenceBinding, ConsistencyGuaranteeRecord, ConsistencyRequirementError,
        ConsistencyRequirementRecord,
    },
    operation_plan::{OperationPlanError, OperationPlanRecord},
    restore_safety::{
        RestoreFenceBindingRecord, RestoreSafetyLaneRecord, RestoreSafetyRequirementError,
        RestoreSafetyRequirementRecord,
    },
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Maximum raw input and canonical output bytes for one original obligation.
pub const MAX_FENCE_OBLIGATION_BYTES: u64 = 1024;

/// Exact original requirement and fence revisions, without current lifecycle state.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "purpose", rename_all = "snake_case", deny_unknown_fields)]
pub enum FenceObligationScopeRecord {
    /// Whole-selection application-coordinated capture.
    Capture {
        /// Exact original consistency requirement digest.
        requirement: ArtifactChecksumRecord,
        /// Original integration-chosen fence identity, retained before dispatch.
        identity: ArtifactChecksumRecord,
        /// Original membership authority revision.
        membership_revision: ArtifactChecksumRecord,
    },
    /// Same-release restoration under an outside-snapshot application fence.
    Restore {
        /// Exact original restore safety requirement digest, including source identity.
        requirement: ArtifactChecksumRecord,
        /// Original fence and membership/external-obligation authority revisions.
        fence: RestoreFenceBindingRecord,
    },
}

/// Strict immutable v1 obligation for one whole original selected unit.
///
/// An acquisition operation is an explicit opaque application request in the
/// original plan. Its physical target identifies routing only; it does not narrow
/// fence coverage to that target. Integrations qualify request semantics, original
/// identity custody and scope. This declaration proves no acquisition, Active
/// state, absence of external obligations, release or spending authority.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "ObligationFields")]
pub struct FenceObligationRecord {
    version: u16,
    plan_intent: ArtifactChecksumRecord,
    acquisition_operation: u64,
    scope: FenceObligationScopeRecord,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObligationFields {
    version: u16,
    plan_intent: ArtifactChecksumRecord,
    acquisition_operation: u64,
    scope: FenceObligationScopeRecord,
}
impl TryFrom<ObligationFields> for FenceObligationRecord {
    type Error = FenceObligationError;
    fn try_from(fields: ObligationFields) -> Result<Self, Self::Error> {
        if fields.version != 1 {
            return Err(FenceObligationError::UnsupportedVersion(fields.version));
        }
        Ok(Self {
            version: 1,
            plan_intent: fields.plan_intent,
            acquisition_operation: fields.acquisition_operation,
            scope: fields.scope,
        })
    }
}
impl FenceObligationRecord {
    /// Bind a coordinated capture obligation before attempting its explicit acquisition operation.
    /// # Errors
    /// Rejects another plan, a weaker guarantee, absent operation or zero mutation allowance.
    pub fn for_capture(
        plan: &OperationPlanRecord,
        requirement: &ConsistencyRequirementRecord,
        acquisition_operation: u64,
        fence: &ApplicationFenceBinding,
    ) -> Result<Self, FenceObligationError> {
        requirement.validate_plan(plan)?;
        if requirement.guarantee() != ConsistencyGuaranteeRecord::ApplicationCoordinated {
            return Err(FenceObligationError::FenceNotRequired);
        }
        Self::new(
            plan,
            acquisition_operation,
            FenceObligationScopeRecord::Capture {
                requirement: requirement.digest(),
                identity: fence.identity.clone(),
                membership_revision: fence.membership_revision.clone(),
            },
        )
    }
    /// Bind restoration to the exact original source, fence and revisions before acquisition.
    /// # Errors
    /// Rejects changed plans, a non-fenced safety lane, absent operation or zero mutation allowance.
    pub fn for_restore(
        plan: &OperationPlanRecord,
        source: &OperationPlanRecord,
        requirement: &RestoreSafetyRequirementRecord,
        acquisition_operation: u64,
    ) -> Result<Self, FenceObligationError> {
        requirement.validate_plans(plan, source)?;
        if requirement.safety() != RestoreSafetyLaneRecord::ApplicationFenced {
            return Err(FenceObligationError::FenceNotRequired);
        }
        let fence = requirement
            .expected_fence()
            .ok_or(FenceObligationError::FenceNotRequired)?;
        Self::new(
            plan,
            acquisition_operation,
            FenceObligationScopeRecord::Restore {
                requirement: requirement.digest(),
                fence: fence.clone(),
            },
        )
    }
    fn new(
        plan: &OperationPlanRecord,
        acquisition_operation: u64,
        scope: FenceObligationScopeRecord,
    ) -> Result<Self, FenceObligationError> {
        let record = Self {
            version: 1,
            plan_intent: plan.digest(),
            acquisition_operation,
            scope,
        };
        record.validate_plan(plan)?;
        Ok(record)
    }
    /// Read the full original plan digest, including exact selection, requests and allowances.
    #[must_use]
    pub const fn plan_intent(&self) -> &ArtifactChecksumRecord {
        &self.plan_intent
    }
    /// Read the declared explicit acquisition operation; not a dispatch permit.
    #[must_use]
    pub const fn acquisition_operation(&self) -> u64 {
        self.acquisition_operation
    }
    /// Read immutable requirement and fence bindings; not an Active/released projection.
    #[must_use]
    pub const fn scope(&self) -> &FenceObligationScopeRecord {
        &self.scope
    }
    /// Match the full original plan and its finite acquisition allowance.
    /// # Errors
    /// Rejects changed plan identity, missing operation or zero mutation allowance.
    pub fn validate_plan(&self, plan: &OperationPlanRecord) -> Result<(), FenceObligationError> {
        if self.plan_intent != plan.digest() {
            return Err(FenceObligationError::PlanMismatch);
        }
        if plan
            .attempt_authority(self.acquisition_operation)?
            .budget()
            .mutations()
            == 0
        {
            return Err(FenceObligationError::NoAcquisitionAllowance);
        }
        Ok(())
    }
    /// Match every original capture binding without weakening scope on recovery.
    /// # Errors
    /// Rejects another original requirement, fence, revision, plan or purpose.
    pub fn validate_capture(
        &self,
        plan: &OperationPlanRecord,
        requirement: &ConsistencyRequirementRecord,
        fence: &ApplicationFenceBinding,
    ) -> Result<(), FenceObligationError> {
        let original = Self::for_capture(plan, requirement, self.acquisition_operation, fence)?;
        self.validate_original(&original)
    }
    /// Match original restore/source/fence bindings without treating their hashes as custody.
    /// # Errors
    /// Rejects changed requirements, source, revisions, plans or purpose.
    pub fn validate_restore(
        &self,
        plan: &OperationPlanRecord,
        source: &OperationPlanRecord,
        requirement: &RestoreSafetyRequirementRecord,
    ) -> Result<(), FenceObligationError> {
        let original = Self::for_restore(plan, source, requirement, self.acquisition_operation)?;
        self.validate_original(&original)
    }
    fn validate_original(&self, original: &Self) -> Result<(), FenceObligationError> {
        if self != original {
            return Err(FenceObligationError::BindingMismatch);
        }
        Ok(())
    }
    /// Hash NUL-terminated v1 domain, original intent, big-endian u64 operation,
    /// purpose tag (capture=0, restore=1), requirement and original fence revisions.
    ///
    /// All hashes are 64 lowercase ASCII bytes. Restore additionally includes its
    /// external-obligations revision. No mutable status, receipts or release flag exists.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        let mut bytes = b"ic-backup/fence-obligation/v1\0".to_vec();
        bytes.extend_from_slice(self.plan_intent.hash().as_bytes());
        bytes.extend_from_slice(&self.acquisition_operation.to_be_bytes());
        let (tag, hashes) = match &self.scope {
            FenceObligationScopeRecord::Capture {
                requirement,
                identity,
                membership_revision,
            } => (0, vec![requirement, identity, membership_revision]),
            FenceObligationScopeRecord::Restore { requirement, fence } => (
                1,
                vec![
                    requirement,
                    &fence.identity,
                    &fence.membership_revision,
                    &fence.external_obligations_revision,
                ],
            ),
        };
        bytes.push(tag);
        for hash in hashes {
            bytes.extend_from_slice(hash.hash().as_bytes());
        }
        ArtifactChecksumRecord::from_bytes(&bytes)
    }
}
/// Typed original obligation rejection; never releases a fence or resets attempts.
#[derive(Debug, Error)]
pub enum FenceObligationError {
    /// Only the maintained v1 schema is admitted.
    #[error("unsupported fence obligation version {0}")]
    UnsupportedVersion(u16),
    /// Full original plan differs.
    #[error("fence obligation original plan mismatch")]
    PlanMismatch,
    /// Exact original purpose, requirement or fence revisions differ.
    #[error("fence obligation original binding mismatch")]
    BindingMismatch,
    /// The original guarantee/safety lane requires no fence.
    #[error("original requirement does not require an application fence")]
    FenceNotRequired,
    /// The selected original acquisition operation has no mutation allowance.
    #[error("original fence acquisition operation has no mutation allowance")]
    NoAcquisitionAllowance,
    /// Original operation is absent or cannot derive its authority.
    #[error(transparent)]
    Plan(#[from] OperationPlanError),
    /// Capture declaration mismatch.
    #[error(transparent)]
    Consistency(#[from] ConsistencyRequirementError),
    /// Restore/source declaration mismatch.
    #[error(transparent)]
    Restore(#[from] RestoreSafetyRequirementError),
}

#[cfg(test)]
mod tests;
