//! Direct-controller observation port; no backend or success default.

use crate::model::control_authority::{ControlObservation, ControlObservationRequest};
use thiserror::Error;

/// Integration-owned fresh authenticated controller observation for exact IC mutation bytes.
///
/// Implementations qualify complete actual controller data and locked actual context,
/// target, challenge freshness, evidence and prior per-call spending/custody. Missing
/// or unknown controllers must fail; status/read success is not controller evidence.
/// No provider is installed. Results grant no signing, dispatch, same-release safety,
/// snapshot-origin permission, stopped-state proof, application fence or terminal release.
pub trait ControlAuthorityProvider {
    /// Observe current controllers under exact original request and bounded descriptive ceiling.
    /// # Errors
    /// Unavailable/unsupported reject before effects; indeterminate retains spending/evidence
    /// and stops without retry. An absent required provider must fail unavailable.
    fn observe_control(
        &mut self,
        request: &ControlObservationRequest<'_>,
    ) -> Result<ControlObservation, ControlProviderError>;
}
/// Redacted typed provider failure; no failure grants admission or replenishes allowances.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ControlProviderError {
    /// No qualified provider is available; reject before remote effects.
    #[error("control authority provider unavailable")]
    Unavailable,
    /// Requested contract unsupported; reject before remote effects.
    #[error("control authority contract unsupported")]
    Unsupported,
    /// Actual observation/reply/accounting is uncertain; retain evidence and stop.
    #[error("control authority observation indeterminate")]
    Indeterminate,
}
