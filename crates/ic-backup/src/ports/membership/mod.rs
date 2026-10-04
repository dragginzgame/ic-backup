//! Current membership integration contract; no default provider or backend implementation.

use crate::model::membership::{MembershipObservation, MembershipObservationRequest};
use thiserror::Error;

/// Integration-owned authoritative membership observation, separate from control/read authority.
///
/// Implementations resolve the locked authenticated context, perform fresh bounded
/// observations and report their exact context and full inventory. They own challenge
/// freshness, revision/evidence meaning, per-call prior accounting, lost-response
/// retention and coherent custody. A request ceiling never grants spending authority.
/// Returning an echoed declaration or decoded old receipt is not an observation.
/// Matching before/after observations cannot certify continuity or an application fence.
/// No provider is installed by the library; an absent provider must fail unavailable.
pub trait MembershipProvider {
    /// Obtain a current result under the exact original request and descriptive call bound.
    ///
    /// # Errors
    /// Unavailable/unsupported reject before remote effects. An indeterminate outcome
    /// retains any consumed allowance and does not authorize retry or mutation.
    fn observe_membership(
        &mut self,
        request: &MembershipObservationRequest<'_>,
    ) -> Result<MembershipObservation, MembershipProviderError>;
}

/// Redacted typed provider failure; none implies an accepted membership result.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum MembershipProviderError {
    /// A required qualified provider is absent; no remote effects occurred.
    #[error("membership provider unavailable")]
    Unavailable,
    /// Provider cannot meet the requested contract; reject before remote effects.
    #[error("membership observation contract unsupported")]
    Unsupported,
    /// Observation failed or reply/accounting is uncertain; retain evidence and stop.
    #[error("membership observation indeterminate")]
    Indeterminate,
}
