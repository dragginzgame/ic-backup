//! Current consistency validation port; no fence acquisition/release or default coordinator.

use crate::model::consistency::{ConsistencyObservation, ConsistencyRequest};
use thiserror::Error;

/// Application-owned fresh capture consistency and exact retained fence validation.
///
/// Providers qualify actual authenticated context/full inventory/selection, stopped
/// and drained targets, challenge freshness and prior approved per-call accounting.
/// Coordinated results additionally qualify currently active exact fence identity,
/// membership revision, whole-unit write/membership/timer/external-work fencing and
/// drained work, with continuous retained custody across interruption/capture.
/// Sequential stops or matching hashes/revisions alone cannot prove that guarantee.
/// No provider is installed. This port observes existing obligations; it does not
/// acquire/release fences, restart targets, sign/dispatch capture, settle lost replies
/// or qualify restore/payment safety. Failure, timeout, process death and dropping
/// requests/results never release the integration's retained fence obligations.
/// Terminal replay must not invoke this provider; fresh live verification is separate.
pub trait ConsistencyProvider {
    /// Validate current evidence for the exact original requested guarantee and capture boundary.
    /// # Errors
    /// Unavailable/unsupported reject before effects; indeterminate retains consumed
    /// spending, evidence and fence obligations and stops without retry or release.
    fn observe_consistency(
        &mut self,
        request: &ConsistencyRequest<'_>,
    ) -> Result<ConsistencyObservation, ConsistencyProviderError>;
}
/// Redacted provider denial; no failure changes allowances or releases an obligation.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ConsistencyProviderError {
    /// Required qualified provider absent; no remote effects.
    #[error("consistency provider unavailable")]
    Unavailable,
    /// Cannot satisfy original guarantee; reject before remote effects.
    #[error("consistency contract unsupported")]
    Unsupported,
    /// Actual evidence/reply/accounting uncertain; retain obligations and stop.
    #[error("consistency observation indeterminate")]
    Indeterminate,
}
