//! Single reserved IC status/list observation contract; no installed provider.

use crate::model::ic_observation::{IcObservationRequest, IcObservationResponse};
use thiserror::Error;

/// Integration-owned authenticated observation of an original pending mutation.
///
/// Retain original plan/mutation/observation bytes and durably reserve this exact
/// observation before invocation. Qualify actual context, method-specific current
/// read permission, original recovery chronology and exclusive command/dispatch
/// custody proving this observation reservation was never dispatched. A reconstructed
/// pending request proves none of these; a lost observation cannot be repeated.
///
/// Submit exactly one host replicated update using the original management receiver,
/// effective target, method and Candid bytes. Status/list are semantic observations,
/// still potentially paid. No query/proxy substitution, hidden retry, extra observations,
/// funding or mutation is permitted. Additional calls need independent prior accounting.
///
/// Return exact retained raw reply and actual original authority/attempt/payload/context/
/// target association. Integrations independently authenticate and qualify evidence,
/// timing and exclusive attribution. Wire projections and inventory cardinality never
/// automatically produce Applied/NotApplied/Uncertain receipts or fresh control/read
/// authority. Failure/drop/death retain pending spent observation and mutation,
/// obligations and source references; an absent reply is not settled uncertainty.
/// Terminal replay invokes no provider. No default or implementation is installed.
pub trait IcObservationProvider {
    /// Observe once under the exact already reserved original observation.
    /// # Errors
    /// All failures retain pending original accounting; no implicit reissue or outcome.
    fn observe(
        &mut self,
        request: &IcObservationRequest<'_>,
    ) -> Result<IcObservationResponse, IcObservationProviderError>;
}

/// Redacted observation failure; no automatic receipt, refund or permission follows.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum IcObservationProviderError {
    /// No qualified provider; no dispatch.
    #[error("IC observation provider unavailable")]
    Unavailable,
    /// This exact bounded original observation contract cannot be qualified.
    #[error("IC observation contract unsupported")]
    Unsupported,
    /// Observation dispatch/reply/authentication is indeterminate, not settled Uncertain.
    #[error("IC observation response indeterminate")]
    Indeterminate,
}
