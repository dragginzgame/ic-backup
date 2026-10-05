//! Application-owned single-update fence acquisition port; no installed implementation.

use crate::model::fence_acquisition::{FenceAcquisitionAcknowledgement, FenceAcquisitionRequest};
use thiserror::Error;

/// One previously accounted application update for the exact original selected unit.
///
/// Before invocation the integration retains exact payload bytes, original plans,
/// requirements and obligation, plus source references for unfinished restore work.
/// It durably reserves the original mutation and qualifies fresh exact actual network,
/// caller, receiver/whole-unit membership, acquisition permissions, application method/
/// argument semantics, original fence/revisions and outside-snapshot restore custody.
/// Complete dependency evidence and exclusive dispatch/command custody are required.
/// The structural request proves none of these facts and is not a dispatch permit.
///
/// Invoke only when qualified custody proves this reservation has never been
/// dispatched. A pending reservation after interruption is not that proof; reconcile
/// instead. Send exactly its receiver/method/arguments as one host replicated update,
/// with no proxy/query substitution, implicit funding, mutation batching or hidden
/// retries or extra remote observations. Provider reissues require distinct prior approved accounting;
/// they cannot occur within this invocation. Local locks do not exclude remote actors.
///
/// An acknowledgement associates retained actual reply bytes and authenticated
/// context with the exact original authority/attempt. It is not an acquisition
/// receipt, current Active fence or negative effect proof. Independently qualify
/// whole-unit original-request attribution before any direct journal settlement.
/// Provider errors/drop/death retain the consumed pending mutation, obligations and
/// references; never synthesize nonapplication or a refund. Lost replies require
/// the reserved reconciliation port, not another invocation of this mutation.
/// Terminal replay is local and invokes no provider. No automatic release exists.
pub trait FenceAcquisitionProvider {
    /// Submit one originally reserved exact application acquisition update.
    /// # Errors
    /// Every failure retains consumed pending evidence; no variant proves nonapplication.
    fn acquire_fence(
        &mut self,
        request: &FenceAcquisitionRequest<'_>,
    ) -> Result<FenceAcquisitionAcknowledgement, FenceAcquisitionProviderError>;
}

/// Redacted application update failure; no automatic receipt, refund or retry.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum FenceAcquisitionProviderError {
    /// No qualified provider, without dispatch.
    #[error("fence acquisition provider unavailable")]
    Unavailable,
    /// Application cannot qualify this exact whole-unit acquisition contract.
    #[error("fence acquisition contract unsupported")]
    Unsupported,
    /// Dispatch/reply/authority outcome is indeterminate; reconcile the pending acquisition.
    #[error("fence acquisition update indeterminate")]
    Indeterminate,
}
