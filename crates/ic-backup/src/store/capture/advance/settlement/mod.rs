//! Local settlement from surviving original evidence; no new observation or dispatch.
use super::{
    CaptureCall, CaptureCallEvidence, CaptureError, CaptureOperation, CaptureProvider,
    PreparedStatus,
    ledger::{CaptureStatusPhase, EventRecord, EvidenceRecord},
    snapshots::{self, CaptureSnapshotPhase},
};
use thiserror::Error;

/// Retained-result admission failed; original requests and spending remain unchanged.
#[derive(Debug, Error)]
pub enum CaptureSettlementError<E> {
    /// Local evidence, bounds or original custody refused.
    #[error(transparent)]
    Capture(#[from] CaptureError),
    /// This call has no original submission reservation to settle.
    #[error("capture result requires an original submission reservation")]
    NotSubmitted,
    /// A result is already retained and the supplied bytes differ.
    #[error("capture result differs from the original settled evidence")]
    EvidenceMismatch,
    /// Local historical authentication refused the original request/result binding.
    #[error("capture result authentication refused")]
    Provider(E),
}

impl CaptureOperation {
    /// Authenticate and retain a surviving original result without repeating its call.
    ///
    /// Use when an integration retained the original reply/proof but submission returned
    /// uncertainty before the operation settled. This synchronous method invokes only
    /// local historical authentication. It performs no signing, live admission, network,
    /// polling or fresh observation and consumes no additional remote allowance.
    /// The caller must bound evidence loading by the original result-byte allowance.
    ///
    /// The original submitted request, provider, target, trust and certificate windows
    /// remain mandatory, including after expiry or loss of live application authority.
    /// A matching existing settlement is authenticated without writes; different bytes
    /// cannot replace it. Orphan evidence or partial ledger writes still refuse without
    /// repair. No future certificate window or inventory-difference receipt is admitted.
    ///
    /// Success means authenticated evidence is durable, including certified rejection or
    /// a reply that fails application policy. Call `advance_status` / `advance_snapshot`
    /// separately to evaluate that outcome. This method never advances dependent calls,
    /// restores maintenance, refunds spending or declares a completed backup.
    ///
    /// ```no_run
    /// use ic_backup::store::{CaptureOperation, CaptureProvider, CaptureCall,
    ///     CaptureCallEvidence, CaptureSettlementError};
    /// fn recover<P: CaptureProvider>(operation: &mut CaptureOperation, provider: &P,
    ///     original_reply: &CaptureCallEvidence) -> Result<(), CaptureSettlementError<P::Error>> {
    ///     operation.settle_capture_result(CaptureCall::TakeSnapshot, original_reply, provider)
    /// }
    /// ```
    ///
    /// # Errors
    /// Refuses absent submission, changed original evidence or custody, oversized data,
    /// failed authentication and uncertain persistence. No error grants retry authority.
    pub fn settle_capture_result<P: CaptureProvider>(
        &mut self,
        call: CaptureCall,
        evidence: &CaptureCallEvidence,
        provider: &P,
    ) -> Result<(), CaptureSettlementError<P::Error>> {
        let state = self.check()?;
        let (admission, request, result, submitted, sequence, names) = match call {
            CaptureCall::Status => (
                state.admission.as_ref(),
                state.request.as_ref(),
                state.result.as_ref(),
                state.phase == CaptureStatusPhase::SubmissionReserved,
                4,
                ("status-request.bin", "status-result.bin"),
            ),
            CaptureCall::ListSnapshots => (
                state.snapshot.admission.as_ref().map(|a| &a.list),
                state.snapshot.list_request.as_ref(),
                state.snapshot.list_result.as_ref(),
                state.snapshot.phase == CaptureSnapshotPhase::ListSubmissionReserved,
                8,
                snapshots::names(call)?,
            ),
            CaptureCall::TakeSnapshot => (
                state.snapshot.admission.as_ref().map(|a| &a.take),
                state.snapshot.take_request.as_ref(),
                state.snapshot.take_result.as_ref(),
                state.snapshot.phase == CaptureSnapshotPhase::TakeSubmissionReserved,
                12,
                snapshots::names(call)?,
            ),
        };
        if !submitted && result.is_none() {
            return Err(CaptureSettlementError::NotSubmitted);
        }
        let admission = admission.ok_or(CaptureError::Incomplete)?;
        let bytes = evidence.encode(admission.result_bytes)?;
        if let Some(result) = result
            && self.evidence(names.1, result)? != bytes
        {
            return Err(CaptureSettlementError::EvidenceMismatch);
        }
        let prepared = PreparedStatus::decode(
            &self.evidence(names.0, request.ok_or(CaptureError::Incomplete)?)?,
        )?;
        prepared.validate_call_for(&self.plan, call)?;
        match call {
            CaptureCall::Status => {
                provider.authenticate(&self.plan, admission, &prepared, evidence)
            }
            _ => provider.authenticate_call(&self.plan, admission, call, &prepared, evidence),
        }
        .map_err(CaptureSettlementError::Provider)?;
        if self.check()? != state {
            return Err(CaptureError::Changed.into());
        }
        if result.is_none() {
            let result = EvidenceRecord::new(&bytes);
            let event = match call {
                CaptureCall::Status => EventRecord::Settled { sequence, result },
                _ => EventRecord::SnapshotSettled {
                    sequence,
                    call,
                    result,
                },
            };
            self.commit(&event, Some((names.1, &bytes)))?;
        }
        Ok(())
    }
}
