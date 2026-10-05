//! Fresh application-owned restore safety; no installed provider or fence lifecycle effects.

use crate::model::restore_safety::{RestoreSafetyObservation, RestoreSafetyRequest};
use thiserror::Error;

/// Application-owned actual same-release source and irreversible-work safety qualification.
///
/// Providers qualify source authenticity/completeness/stable custody, actual
/// target-local load snapshot ID and complete metadata/data association to the
/// original source, current context/inventory/lifecycle, a fresh unique challenge and prior per-call
/// accounting. The no-irreversible-effects lane requires application knowledge
/// about restored intent and timers, never a generic standalone default. The
/// fenced lane requires continuous exact whole-selection custody outside the
/// rewindable source, original membership/external-obligation revisions and
/// qualified settlement/replay prevention. Before start, qualify source-specific
/// restored-state acceptance and isolated execution under the retained fence.
///
/// No provider is installed. This observes existing obligations, never acquires/
/// releases fences, signs/dispatches loads or starts, settles lost load replies,
/// refunds spending or releases source references. Failure, timeout, death or
/// dropping values retains original obligations. Terminal replay never invokes
/// this provider; fresh verification is a distinct operation.
pub trait RestoreSafetyProvider {
    /// Observe current safety for the exact original load/start, source and retained requirement.
    /// # Errors
    /// Unavailable/unsupported deny before effects; indeterminate retains consumed
    /// authority/evidence/obligations and stops without retry or release.
    fn observe_restore_safety(
        &mut self,
        request: &RestoreSafetyRequest<'_>,
    ) -> Result<RestoreSafetyObservation, RestoreSafetyProviderError>;
}
/// Redacted provider failure; never changes original allowance or obligation custody.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum RestoreSafetyProviderError {
    /// Required qualified provider is absent.
    #[error("restore safety provider unavailable")]
    Unavailable,
    /// Integration cannot qualify this original safety requirement.
    #[error("restore safety contract unsupported")]
    Unsupported,
    /// Actual state, evidence, custody, reply or prior accounting is unresolved.
    #[error("restore safety observation indeterminate")]
    Indeterminate,
}
