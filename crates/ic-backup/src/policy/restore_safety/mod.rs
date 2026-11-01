//! Pure same-source and current application safety matching; no IO, effects or settlement.

use super::restore_fence::{self, RestoreFenceMismatch};
use crate::model::{
    artifacts::ArtifactChecksumRecord,
    ic_request::IcManagementMethodRecord,
    restore_safety::{
        RestoreFenceEvidence, RestoreSafetyEvidence, RestoreSafetyLaneRecord,
        RestoreSafetyObservation, RestoreSafetyRequest, RestoreSafetyRequirementRecord,
        TargetRestoreEvidence,
    },
};
use ic_platform::management_canister_types::CanisterStatusType;
use thiserror::Error;

/// Read-only matched current safety; never a load/start, spending or release permit.
#[derive(Clone, Debug)]
pub struct RestoreSafetyView<'a> {
    request: ArtifactChecksumRecord,
    requirement: ArtifactChecksumRecord,
    observation: &'a RestoreSafetyObservation,
}
impl RestoreSafetyView<'_> {
    /// Read exact current source/operation/challenge-bound request identity.
    #[must_use]
    pub const fn request(&self) -> &ArtifactChecksumRecord {
        &self.request
    }
    /// Read original retained source/safety requirement identity.
    #[must_use]
    pub const fn requirement(&self) -> &ArtifactChecksumRecord {
        &self.requirement
    }
    /// Read canonical actual selected lifecycle and restored-acceptance rows.
    #[must_use]
    pub fn targets(&self) -> &[TargetRestoreEvidence] {
        self.observation.targets()
    }
    /// Read current matched fence, when required; never a release token.
    #[must_use]
    pub fn fence(&self) -> Option<&RestoreFenceEvidence> {
        match self.observation.safety() {
            RestoreSafetyEvidence::ApplicationFenced(fence) => Some(fence),
            RestoreSafetyEvidence::NoIrreversibleEffects(_)
            | RestoreSafetyEvidence::Unresolved(_) => None,
        }
    }
    /// Read opaque current evidence identity.
    #[must_use]
    pub const fn evidence(&self) -> &ArtifactChecksumRecord {
        self.observation.evidence()
    }
    /// Read descriptive actual calls; not prior accounting or fresh allowance.
    #[must_use]
    pub const fn remote_observations(&self) -> u32 {
        self.observation.remote_observations()
    }
}
/// Match original source/lane/fence, actual full context/inventory/selection and lifecycle.
///
/// Before load all selected targets must be stopped. Before start its exact
/// target must be stopped and every selected target must have source-specific
/// application acceptance. Other targets may already be running under the
/// integration's controlled-execution contract; a still-stopping member rejects.
/// Canonical order supplies no
/// dispatch order. The original graph and actual receipts are separate admission.
///
/// Integrations qualify actual freshness/source custody/drain, irreversible-work
/// absence or outside-snapshot fence custody/replay safety. Hash equality cannot
/// prove those properties or load completion. Policy invokes no provider, performs
/// no IO/serialization and changes no journal, fence, allowance or reference.
/// # Errors
/// Rejects changed request/context/inventory/source/selection/lane/revisions,
/// excessive calls, unresolved obligations, inactive fences or absent start evidence.
pub fn validate<'a>(
    request: &RestoreSafetyRequest<'_>,
    observation: &'a RestoreSafetyObservation,
) -> Result<RestoreSafetyView<'a>, RestoreSafetyError> {
    let digest = request.digest();
    let binding = request.binding();
    super::observed_context::admit_request_context(
        &digest,
        observation.request(),
        binding,
        observation.context(),
        RestoreSafetyError::RequestMismatch,
        RestoreSafetyError::ContextMismatch,
    )?;
    if observation.inventory() != request.inventory() {
        return Err(RestoreSafetyError::InventoryMismatch);
    }
    if !observation
        .targets()
        .iter()
        .map(|target| &target.target)
        .eq(request.selected_targets())
    {
        return Err(RestoreSafetyError::SelectionMismatch);
    }
    let requirement = request.requirement();
    if observation.source_plan_intent().hash() != requirement.source_plan_intent() {
        return Err(RestoreSafetyError::SourcePlanMismatch);
    }
    if observation.source_artifacts() != requirement.source_artifacts() {
        return Err(RestoreSafetyError::SourceArtifactsMismatch);
    }
    super::observed_context::admit_limit(
        request.max_remote_observations(),
        observation.remote_observations(),
        |limit, reported| RestoreSafetyError::ObservationLimitExceeded { limit, reported },
    )?;
    let method = request.wire().method();
    validate_targets(method, binding.target(), observation.targets())?;
    validate_safety(requirement, observation.safety(), method)?;
    Ok(RestoreSafetyView {
        request: digest,
        requirement: requirement.digest(),
        observation,
    })
}
// Load requires the complete selected set stopped. Start allows previously
// restored peers to run, but requires acceptance for the complete selected set.
fn validate_targets(
    method: IcManagementMethodRecord,
    current_target: &str,
    targets: &[TargetRestoreEvidence],
) -> Result<(), RestoreSafetyError> {
    if !targets.iter().all(|target| match method {
        IcManagementMethodRecord::StartCanister if target.target != current_target => {
            target.state != CanisterStatusType::Stopping
        }
        _ => target.state == CanisterStatusType::Stopped,
    }) {
        return Err(RestoreSafetyError::TargetNotStopped);
    }
    if method == IcManagementMethodRecord::StartCanister
        && targets
            .iter()
            .any(|target| target.restored_acceptance.is_none())
    {
        return Err(RestoreSafetyError::RestoredAcceptanceRequired);
    }
    Ok(())
}

fn validate_safety(
    requirement: &RestoreSafetyRequirementRecord,
    evidence: &RestoreSafetyEvidence,
    method: IcManagementMethodRecord,
) -> Result<(), RestoreSafetyError> {
    match (requirement.safety(), evidence) {
        (_, RestoreSafetyEvidence::Unresolved(_)) => {
            return Err(RestoreSafetyError::UnresolvedExternalObligations);
        }
        (
            RestoreSafetyLaneRecord::NoIrreversibleEffects,
            RestoreSafetyEvidence::NoIrreversibleEffects(_),
        ) => {}
        (
            RestoreSafetyLaneRecord::ApplicationFenced,
            RestoreSafetyEvidence::ApplicationFenced(fence),
        ) => {
            let original = requirement
                .expected_fence()
                .ok_or(RestoreSafetyError::FenceMismatch)?;
            restore_fence::validate(original, fence).map_err(|mismatch| match mismatch {
                RestoreFenceMismatch::Identity => RestoreSafetyError::FenceMismatch,
                RestoreFenceMismatch::MembershipRevision => {
                    RestoreSafetyError::MembershipRevisionMismatch
                }
                RestoreFenceMismatch::ExternalObligationsRevision => {
                    RestoreSafetyError::ExternalObligationsRevisionMismatch
                }
                RestoreFenceMismatch::Inactive => RestoreSafetyError::FenceNotActive,
            })?;
            if method == IcManagementMethodRecord::StartCanister
                && fence.controlled_execution.is_none()
            {
                return Err(RestoreSafetyError::ControlledExecutionRequired);
            }
        }
        _ => return Err(RestoreSafetyError::SafetyLaneMismatch),
    }
    Ok(())
}
/// Typed fresh safety rejection; retains original spending, obligations and source references.
#[derive(Debug, Eq, Error, PartialEq)]
pub enum RestoreSafetyError {
    /// Original request, source, safety, operation or challenge differs.
    #[error("restore safety request mismatch")]
    RequestMismatch,
    /// Actual current context differs from original restore context.
    #[error("restore safety observed {0} mismatch")]
    ContextMismatch(&'static str),
    /// Actual complete inventory differs, including unselected metadata.
    #[error("restore safety inventory mismatch")]
    InventoryMismatch,
    /// Actual rows do not equal the exact original selected set.
    #[error("restore safety selection mismatch")]
    SelectionMismatch,
    /// Observed source plan differs from original retention.
    #[error("restore safety source plan mismatch")]
    SourcePlanMismatch,
    /// Observed artifact binding differs from original retention.
    #[error("restore safety source artifacts mismatch")]
    SourceArtifactsMismatch,
    /// Current application evidence changes the original lane.
    #[error("restore safety evidence lane mismatch")]
    SafetyLaneMismatch,
    /// External obligations/replayed work are known unresolved or unsafe.
    #[error("restore safety external obligations unresolved")]
    UnresolvedExternalObligations,
    /// Required stopped/drained lifecycle has not been observed.
    #[error("restore safety target is not stopped")]
    TargetNotStopped,
    /// Every selected restored state needs application acceptance before start.
    #[error("restore safety restored-state acceptance required")]
    RestoredAcceptanceRequired,
    /// Exact retained fence identity differs.
    #[error("restore safety fence identity mismatch")]
    FenceMismatch,
    /// Original membership revision changed; never silently rebind a fence.
    #[error("restore safety membership revision mismatch")]
    MembershipRevisionMismatch,
    /// Original external-obligation authority revision changed.
    #[error("restore safety external-obligations revision mismatch")]
    ExternalObligationsRevisionMismatch,
    /// Retained exact fence is currently known inactive.
    #[error("restore safety fence not active")]
    FenceNotActive,
    /// Isolated execution under retained custody has not been qualified before start.
    #[error("restore safety controlled execution evidence required")]
    ControlledExecutionRequired,
    /// Call reporting exceeds the descriptive ceiling.
    #[error("restore safety reports {reported} observations above ceiling {limit}")]
    ObservationLimitExceeded {
        /// Original descriptive ceiling; no allowance.
        limit: u32,
        /// Actual reported calls.
        reported: u32,
    },
}

#[cfg(test)]
mod tests;
