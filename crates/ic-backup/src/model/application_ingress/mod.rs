//! Opaque application update declarations under original mutation or observation reservations.
//!
//! Applications own codecs, fresh authority, whole-unit safety, original byte/command
//! custody and qualified outcomes. These passive types grant no dispatch or resend.

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
    /// Original reservation denial, retaining the canonical typed accounting cause.
    #[error("original mutation reservation rejected: {0}")]
    Reservation(#[from] crate::model::attempt_journal::MutationReservationError),
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
    /// Effective target differs from the original operation.
    #[error("application ingress target mismatch")]
    TargetMismatch,
    /// Receiver, target, fixed update mode, method, bytes or reply ceiling changed.
    #[error("application ingress payload mismatch")]
    PayloadMismatch,
    /// Exact separately reserved recovery identity failed admission.
    #[error(transparent)]
    Observation(#[from] IcObservationRequestError),
    /// Original operation cannot be derived.
    #[error(transparent)]
    Plan(#[from] OperationPlanError),
}

#[cfg(test)]
mod tests;
