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
    request: ArtifactChecksumRecord,
    inventory: &'a InventoryRecord,
    selected_targets: &'a [String],
    revision: Option<&'a ArtifactChecksumRecord>,
    evidence: &'a ArtifactChecksumRecord,
    remote_observations: u32,
}

impl MembershipView<'_> {
    /// Read the exact original challenge/boundary-bound request digest.
    #[must_use]
    pub const fn request(&self) -> &ArtifactChecksumRecord {
        &self.request
    }
    /// Read the complete matching current inventory supplied by the provider.
    #[must_use]
    pub const fn inventory(&self) -> &InventoryRecord {
        self.inventory
    }
    /// Read original exact selection in canonical order, without dispatch order.
    #[must_use]
    pub const fn selected_targets(&self) -> &[String] {
        self.selected_targets
    }
    /// Read optional current revision; matching revisions alone prove no continuity.
    #[must_use]
    pub const fn revision(&self) -> Option<&ArtifactChecksumRecord> {
        self.revision
    }
    /// Read the provider-owned opaque evidence identifier, not a signature or permission.
    #[must_use]
    pub const fn evidence(&self) -> &ArtifactChecksumRecord {
        self.evidence
    }
    /// Read reported calls; this does not reserve, refund or replenish any allowance.
    #[must_use]
    pub const fn remote_observations(&self) -> u32 {
        self.remote_observations
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
    if observation.request != digest {
        return Err(MembershipError::RequestMismatch);
    }
    let binding = request.binding();
    if observation.context.network() != binding.network() {
        return Err(MembershipError::ContextMismatch("network"));
    }
    if observation.context.caller() != binding.caller() {
        return Err(MembershipError::ContextMismatch("caller"));
    }
    if observation.context.release() != binding.release() {
        return Err(MembershipError::ContextMismatch("release"));
    }
    if observation.inventory != *request.inventory() {
        return Err(MembershipError::InventoryMismatch);
    }
    if observation.remote_observations > request.max_remote_observations() {
        return Err(MembershipError::ObservationLimitExceeded {
            limit: request.max_remote_observations(),
            reported: observation.remote_observations,
        });
    }
    Ok(MembershipView {
        request: digest,
        inventory: &observation.inventory,
        selected_targets: request.selected_targets(),
        revision: observation.revision.as_ref(),
        evidence: &observation.evidence,
        remote_observations: observation.remote_observations,
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
