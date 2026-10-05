//! Pure reserved-observation and original-acquisition attribution matching; no settlement IO.

use crate::model::{
    artifacts::ArtifactChecksumRecord,
    attempt_journal::{AttemptJournalRecord, ObservationOutcomeRecord},
    consistency::ApplicationFenceState,
    fence_obligation::FenceObligationScopeRecord,
    fence_reconciliation::{
        FenceReconciliationEvidence, FenceReconciliationObservation, FenceReconciliationRequest,
        FenceReconciliationRequestError, MAX_FENCE_RECONCILIATION_REMOTE_OBSERVATIONS,
    },
};
use thiserror::Error;

/// Matched passive provider claim; not an authenticated receipt or current execution permit.
#[derive(Clone, Debug)]
pub struct FenceReconciliationView<'a> {
    observation: &'a FenceReconciliationObservation,
    outcome: ObservationOutcomeRecord,
}
impl FenceReconciliationView<'_> {
    /// Read the passive matched outcome. Only a qualified integration may settle its journal.
    #[must_use]
    pub const fn outcome(&self) -> ObservationOutcomeRecord {
        self.outcome
    }
    /// Read exact original reserved observation digest.
    #[must_use]
    pub const fn request(&self) -> &ArtifactChecksumRecord {
        &self.observation.input().request
    }
    /// Read exact original observation attempt, without new reservation or allowance.
    #[must_use]
    pub const fn observation_attempt(&self) -> u32 {
        self.observation.input().observation_attempt
    }
    /// Read the opaque request/attempt/result evidence qualified by the actual integration.
    #[must_use]
    pub const fn evidence(&self) -> &ArtifactChecksumRecord {
        &self.observation.input().evidence
    }
    /// Read retained application attribution/nonapplication/uncertainty evidence.
    #[must_use]
    pub const fn settlement(&self) -> &FenceReconciliationEvidence {
        &self.observation.input().settlement
    }
}
/// Match current retained reservation and exact actual original-acquisition evidence.
///
/// This performs no IO, provider invocation, serialization or journal transition.
/// An Active matching fence alone cannot construct acquired evidence: the provider
/// independently qualifies exclusive attribution, fresh actual whole-unit context
/// and custody. Absence/inactivity alone cannot qualify `NotApplied`. An indeterminate
/// or lost reply stays pending; Uncertain needs a qualified settled observation.
/// Views grant no new mutation, repetition, capture/load/start or release authority.
/// # Errors
/// Rejects stale/resolved reservations, wrong attempts/request/context/unit/purpose,
/// inactive/rebound acquired fences or excessive call reporting.
pub fn validate<'a>(
    request: &FenceReconciliationRequest<'_, '_>,
    journal: &AttemptJournalRecord,
    observation: &'a FenceReconciliationObservation,
) -> Result<FenceReconciliationView<'a>, FenceReconciliationError> {
    request.validate_journal(journal)?;
    let input = observation.input();
    let intent = request.intent();
    if input.request != request.digest() {
        return Err(FenceReconciliationError::RequestMismatch);
    }
    if input.mutation_attempt != intent.mutation_attempt()
        || input.observation_attempt != request.observation_attempt()
    {
        return Err(FenceReconciliationError::AttemptMismatch);
    }
    if &input.context != intent.plan().context() {
        return Err(FenceReconciliationError::ContextMismatch);
    }
    if &input.inventory != intent.plan().inventory() {
        return Err(FenceReconciliationError::InventoryMismatch);
    }
    if input.selected_targets != intent.plan().selected_targets() {
        return Err(FenceReconciliationError::SelectionMismatch);
    }
    if input.remote_observations > MAX_FENCE_RECONCILIATION_REMOTE_OBSERVATIONS {
        return Err(FenceReconciliationError::ObservationLimitExceeded);
    }
    let outcome = match_claim(intent.obligation().scope(), &input.settlement)?;
    Ok(FenceReconciliationView {
        observation,
        outcome,
    })
}
fn match_claim(
    scope: &FenceObligationScopeRecord,
    actual: &FenceReconciliationEvidence,
) -> Result<ObservationOutcomeRecord, FenceReconciliationError> {
    match (scope, actual) {
        (_, FenceReconciliationEvidence::NotAcquired { .. }) => {
            Ok(ObservationOutcomeRecord::NotApplied)
        }
        (_, FenceReconciliationEvidence::Unresolved { .. }) => {
            Ok(ObservationOutcomeRecord::Uncertain)
        }
        (
            FenceObligationScopeRecord::Capture {
                identity,
                membership_revision,
                ..
            },
            FenceReconciliationEvidence::AcquiredCapture { fence, .. },
        ) => {
            if &fence.identity != identity {
                return Err(FenceReconciliationError::FenceMismatch);
            }
            if &fence.membership_revision != membership_revision {
                return Err(FenceReconciliationError::MembershipRevisionMismatch);
            }
            active(fence.state)?;
            Ok(ObservationOutcomeRecord::Applied)
        }
        (
            FenceObligationScopeRecord::Restore {
                fence: original, ..
            },
            FenceReconciliationEvidence::AcquiredRestore { fence, .. },
        ) => {
            if fence.binding.identity != original.identity {
                return Err(FenceReconciliationError::FenceMismatch);
            }
            if fence.binding.membership_revision != original.membership_revision {
                return Err(FenceReconciliationError::MembershipRevisionMismatch);
            }
            if fence.binding.external_obligations_revision != original.external_obligations_revision
            {
                return Err(FenceReconciliationError::ExternalObligationsRevisionMismatch);
            }
            active(fence.state)?;
            Ok(ObservationOutcomeRecord::Applied)
        }
        _ => Err(FenceReconciliationError::PurposeMismatch),
    }
}
fn active(state: ApplicationFenceState) -> Result<(), FenceReconciliationError> {
    if state != ApplicationFenceState::Active {
        return Err(FenceReconciliationError::FenceNotActive);
    }
    Ok(())
}
/// Typed pure attribution/result denial; no error settles or refunds an attempt.
#[derive(Debug, Error)]
pub enum FenceReconciliationError {
    /// Original pending evidence no longer binds the request.
    #[error(transparent)]
    Reservation(#[from] FenceReconciliationRequestError),
    /// Actual challenge-bound observation request differs.
    #[error("fence reconciliation observation request mismatch")]
    RequestMismatch,
    /// Actual result belongs to another original mutation or observation attempt.
    #[error("fence reconciliation result attempt mismatch")]
    AttemptMismatch,
    /// Actual network/caller/release differs from the full original plan.
    #[error("fence reconciliation actual context mismatch")]
    ContextMismatch,
    /// Full actual inventory differs, including unselected metadata.
    #[error("fence reconciliation actual inventory mismatch")]
    InventoryMismatch,
    /// Actual whole-unit coverage differs from the exact original selection.
    #[error("fence reconciliation actual selection mismatch")]
    SelectionMismatch,
    /// Acquired proof is for another capture/restore purpose.
    #[error("fence reconciliation original purpose mismatch")]
    PurposeMismatch,
    /// Acquired proof names another original fence.
    #[error("fence reconciliation original fence mismatch")]
    FenceMismatch,
    /// Acquired proof rebinds the original membership revision.
    #[error("fence reconciliation membership revision mismatch")]
    MembershipRevisionMismatch,
    /// Restore acquisition rebinds original outside-snapshot obligations.
    #[error("fence reconciliation external obligations revision mismatch")]
    ExternalObligationsRevisionMismatch,
    /// Acquisition cannot be admitted under this port's known inactive fence result.
    #[error("fence reconciliation acquired fence is not active")]
    FenceNotActive,
    /// This single reserved observation cannot report multiple remote calls.
    #[error("fence reconciliation remote observation ceiling exceeded")]
    ObservationLimitExceeded,
}

#[cfg(test)]
mod tests;
