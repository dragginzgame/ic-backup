//! Read-only original fence-acquisition reconciliation; no acquire/release implementation.

use crate::model::fence_reconciliation::{
    FenceReconciliationObservation, FenceReconciliationRequest,
};
use thiserror::Error;

/// Application-owned authenticated observation of one exact uncertain acquisition.
///
/// No implementation is installed. Before invocation the integration retains the
/// original obligation/plans/requirements, durably consumes the exact observation
/// reservation, qualifies actual read authority/dependencies and excludes any earlier
/// dispatch of that reservation. The request is not a dispatch permit. This port
/// makes at most one previously accounted remote observation and no mutations.
/// Applications own immutable original request bytes/semantics and identity custody.
/// Results bind the original challenge, mutation/observation IDs and actual full
/// context/inventory/selection. Acquired evidence needs exclusive original-request
/// attribution and actual whole-unit Active fence custody; restore additionally
/// needs rewind-independent/replay-safe custody and original external revisions.
/// Matching hashes, visibility or an Active fence alone cannot supply attribution.
/// `NotAcquired` requires exact nonapplication proof excluding transient acquisition/
/// release and independent actors; current absence/inactivity is insufficient.
/// Unresolved requires a qualified settled observation, never a lost reply.
/// Opaque result evidence binds the exact request/attempts and all actual fields.
/// Failure/drop/death retain obligations and consumed allowances. A lost observation
/// reply remains pending and is not dispatched again; a retained late reply may be
/// admitted under its exact original reservation. Terminal replay invokes no provider.
/// Pure matching never automatically writes a receipt or replenishes spending.
pub trait FenceReconciliationProvider {
    /// Observe the exact reserved original acquisition without acquiring or releasing any fence.
    /// # Errors
    /// Every failure retains the pending observation; Indeterminate is not a settled Unresolved result.
    fn observe_fence_acquisition(
        &mut self,
        request: &FenceReconciliationRequest<'_, '_>,
    ) -> Result<FenceReconciliationObservation, FenceReconciliationProviderError>;
}
/// Redacted qualified-provider failure; no variant resets spending or releases obligations.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum FenceReconciliationProviderError {
    /// No qualified actual provider; no remote dispatch.
    #[error("fence reconciliation provider unavailable")]
    Unavailable,
    /// Provider cannot qualify exact attribution/coverage within this single-call contract.
    #[error("fence reconciliation contract unsupported")]
    Unsupported,
    /// Reply/authentication/accounting is indeterminate; leave the original observation pending.
    #[error("fence reconciliation observation indeterminate")]
    Indeterminate,
}
