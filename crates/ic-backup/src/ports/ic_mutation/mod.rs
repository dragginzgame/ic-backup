//! Async single reserved IC update contract; no default transport or dispatch implementation.

use crate::model::attempt_journal::AttemptJournalRecord;
use crate::model::ic_mutation::{IcMutationAcknowledgement, IcMutationRequest};
use std::future::Future;
use thiserror::Error;

/// Integration-owned authenticated single-update submission for an original reservation.
///
/// Before invocation retain the exact original plan and IC bytes, persist the
/// mutation reservation, qualify full prerequisite evidence and current exact
/// network/caller/target/controller authority. Capture requires original consistency/
/// fence/quiescence admission. Load additionally requires exact snapshot-origin
/// permissions; target control/read visibility cannot substitute. Load/start require
/// same-release source/upload identity,
/// complete transfer and application-owned outside-snapshot restoration/restart safety.
/// Retain obligations/source references and qualify exclusive dispatch/command and
/// byte custody. The structural request itself proves none of these conditions.
///
/// Invoke only if qualified custody proves this reservation was never dispatched.
/// Pending journal state after interruption is insufficient: reconcile the exact
/// uncertain effect rather than invoke again. Send the exact management receiver,
/// effective routing target, method and argument bytes as one host replicated update.
/// No proxy/query substitution, hidden retries, observations, funding or extra calls
/// occur within this invocation. Additional calls need their own prior approved
/// durable accounting and independently qualified admission.
///
/// Return exact retained raw reply bytes and actual context/target association;
/// integrations authenticate and qualify opaque evidence independently. A capture
/// ID or canonical empty acknowledgement never produces an Applied/NotApplied
/// receipt automatically. Failures/drop/death retain the consumed pending mutation,
/// fences, source references and recovery evidence. No failure proves nonapplication
/// or refunds allowance. Terminal replay invokes no provider. No default exists.
pub trait IcMutationProvider {
    /// Submit one exact originally reserved update without retry or follow-up observation.
    ///
    /// Recheck `request` against the exact currently guarded `journal` before signing
    /// or dispatch. The caller retains journal exclusion across the returned future.
    /// No runtime or `Send` requirement is imposed by this core port. Dropping a
    /// pending future retains consumption and grants no reentry or repeat submission.
    /// # Errors
    /// Every failure preserves pending original evidence; reconcile indeterminate replies.
    fn submit_mutation(
        &mut self,
        request: &IcMutationRequest<'_>,
        journal: &AttemptJournalRecord,
    ) -> impl Future<Output = Result<IcMutationAcknowledgement, IcMutationProviderError>>;
}

/// Redacted single-call failure; no variant grants retry, nonapplication or a refund.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum IcMutationProviderError {
    /// No qualified provider; no dispatch.
    #[error("IC mutation provider unavailable")]
    Unavailable,
    /// Provider cannot qualify this exact bounded host update contract.
    #[error("IC mutation contract unsupported")]
    Unsupported,
    /// Submission/reply/authentication outcome is indeterminate; retain pending evidence.
    #[error("IC mutation update indeterminate")]
    Indeterminate,
}
