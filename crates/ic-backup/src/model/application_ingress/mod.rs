//! Opaque application update declarations under original mutation or observation reservations.
//!
//! Applications own codecs, fresh authority, whole-unit safety, original byte/command
//! custody and qualified outcomes. These passive types grant no dispatch or resend.

use crate::model::application_payload::ApplicationPayloadError as Payload;
mod payload;
mod request;
pub use payload::{
    ApplicationIngressPayload, MAX_APPLICATION_INGRESS_ARGUMENT_BYTES,
    MAX_APPLICATION_INGRESS_METHOD_BYTES, MAX_APPLICATION_INGRESS_REPLY_BYTES,
};
pub use request::{ApplicationMutationRequest, ApplicationObservationRequest};

use crate::model::{ic_observation::IcObservationRequestError, operation_plan::OperationPlanError};
use thiserror::Error;

/// Exact opaque update or original reservation admission failed; no spending changes.
#[derive(Debug, Error)]
pub enum ApplicationIngressError {
    /// The receiver or effective routing target is not an admitted application principal.
    #[error("invalid application ingress principal")]
    InvalidPrincipal,
    /// Management ingress must use the closed management request owners.
    #[error("application ingress cannot target the management canister")]
    ManagementPrincipal,
    /// Method must have 1..=128 exact visible ASCII bytes.
    #[error("invalid application ingress method")]
    InvalidMethod,
    /// Opaque arguments exceed the finite owner bound.
    #[error("application ingress arguments too large")]
    ArgumentsTooLarge,
    /// Caller-selected reply ceiling must be 1..=1 MiB and is bound into the original digest.
    #[error("invalid application ingress reply limit")]
    InvalidReplyLimit,
    /// Original journal differs from the complete plan-derived authority.
    #[error("application ingress authority mismatch")]
    AuthorityMismatch,
    /// Effective target differs from the original operation.
    #[error("application ingress target mismatch")]
    TargetMismatch,
    /// Receiver, target, fixed update mode, method, bytes or reply ceiling changed.
    #[error("application ingress payload mismatch")]
    PayloadMismatch,
    /// No original mutation reservation is pending.
    #[error("application ingress requires a pending mutation")]
    NoPendingMutation,
    /// The original mutation was replaced or settled.
    #[error("application ingress mutation mismatch")]
    MutationMismatch,
    /// Recovery is pending; the original mutation cannot be submitted through this lane.
    #[error("application ingress observation is pending")]
    ObservationPending,
    /// Exact separately reserved recovery identity failed admission.
    #[error(transparent)]
    Observation(#[from] IcObservationRequestError),
    /// Original operation cannot be derived.
    #[error(transparent)]
    Plan(#[from] OperationPlanError),
}

#[cfg(test)]
mod tests;

impl From<crate::model::application_payload::ApplicationPayloadError> for ApplicationIngressError {
    fn from(error: crate::model::application_payload::ApplicationPayloadError) -> Self {
        match error {
            Payload::ArgumentsTooLarge => Self::ArgumentsTooLarge,
            Payload::InvalidMethod => Self::InvalidMethod,
            Payload::InvalidPrincipal => Self::InvalidPrincipal,
            Payload::ManagementPrincipal => Self::ManagementPrincipal,
        }
    }
}
