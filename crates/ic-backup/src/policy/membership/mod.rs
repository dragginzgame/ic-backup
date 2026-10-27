//! Pure admission of a current provider result; no IO, scheduling or authority mutation.

use crate::model::{
    artifacts::ArtifactChecksumRecord,
    inventory::InventoryRecord,
    membership::{MembershipObservation, MembershipObservationRequest},
};
use thiserror::Error;

/// Read-only matching observation; not a dispatch permit, continuity proof or fresh permission.
#[derive(Clone, Debug)]
pub struct MembershipView<'a> {
    observation: &'a MembershipObservation,
    selected_targets: &'a [String],
}

impl MembershipView<'_> {
    /// Read the exact original challenge/boundary-bound request digest.
    #[must_use]
    pub const fn request(&self) -> &ArtifactChecksumRecord {
        &self.observation.request
    }
    /// Read the complete matching current inventory supplied by the provider.
    #[must_use]
    pub const fn inventory(&self) -> &InventoryRecord {
        &self.observation.inventory
    }
    /// Read original exact selection in canonical order, without dispatch order.
    #[must_use]
    pub const fn selected_targets(&self) -> &[String] {
        self.selected_targets
    }
    /// Read optional current revision; matching revisions alone prove no continuity.
    #[must_use]
    pub const fn revision(&self) -> Option<&ArtifactChecksumRecord> {
        self.observation.revision.as_ref()
    }
    /// Read the provider-owned opaque evidence identifier, not a signature or permission.
    #[must_use]
    pub const fn evidence(&self) -> &ArtifactChecksumRecord {
        &self.observation.evidence
    }
    /// Read reported calls; this does not reserve, refund or replenish any allowance.
    #[must_use]
    pub const fn remote_observations(&self) -> u32 {
        self.observation.remote_observations
    }
}

/// Validate exact current request/context/full inventory and descriptive call bound.
///
/// The caller qualifies provider authenticity, freshness and coherent custody.
/// This policy calls no provider, serializes no records, changes no plan/journal
/// and makes no application consistency, lifecycle or permission claim.
/// # Errors
/// Rejects another request, context, inventory or excessive reported observations.
pub fn validate<'a>(
    request: &'a MembershipObservationRequest<'_>,
    observation: &'a MembershipObservation,
) -> Result<MembershipView<'a>, MembershipError> {
    let digest = request.digest();
    let binding = request.binding();
    super::observed_context::admit_request_context(
        &digest,
        &observation.request,
        binding,
        &observation.context,
        MembershipError::RequestMismatch,
        MembershipError::ContextMismatch,
    )?;
    if observation.inventory != *request.inventory() {
        return Err(MembershipError::InventoryMismatch);
    }
    super::observed_context::admit_limit(
        request.max_remote_observations(),
        observation.remote_observations,
        |limit, reported| MembershipError::ObservationLimitExceeded { limit, reported },
    )?;
    Ok(MembershipView {
        observation,
        selected_targets: request.selected_targets(),
    })
}

/// Typed mismatch; no rejected result admits an effect or changes original authority.
#[derive(Debug, Eq, Error, PartialEq)]
pub enum MembershipError {
    /// Original intent, operation, challenge, boundary or descriptive ceiling changed.
    #[error("membership request identity mismatch")]
    RequestMismatch,
    /// Actually observed canonical context differs from the original declaration.
    #[error("membership observed {0} mismatch")]
    ContextMismatch(&'static str),
    /// Full current inventory differs, including unselected relationships/metadata.
    #[error("membership inventory mismatch")]
    InventoryMismatch,
    /// Provider reports more remote observations than the descriptive request permits.
    #[error("membership reports {reported} observations above ceiling {limit}")]
    ObservationLimitExceeded {
        /// Original descriptive ceiling; not a journal spending allowance.
        limit: u32,
        /// Actual provider-reported remote observations.
        reported: u32,
    },
}

#[cfg(test)]
mod tests;
