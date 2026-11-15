//! Initial and final status share one workflow, while retaining distinct evidence.
use super::super::ledger::{EventRecord, EvidenceRecord, State, StatusState};
use super::{
    CaptureDataAdvanceError, CaptureError, CaptureOperation, CaptureProvider, CaptureStatusView,
    PreparedStatus, StatusAdmissionRecord, StatusAdvanceError, StatusHttpTransport, StatusProgress,
};
use ic_host_artifacts::artifact::Sha256Digest;
use std::convert::Infallible;
use thiserror::Error;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum StatusSlot {
    Initial,
    Final,
}
impl StatusSlot {
    pub fn state(self, state: &State) -> StatusState {
        match self {
            Self::Initial => StatusState {
                phase: state.phase,
                admission: state.admission.clone(),
                request: state.request.clone(),
                result: state.result.clone(),
            },
            Self::Final => state
                .final_status
                .clone()
                .unwrap_or_else(StatusState::created),
        }
    }
    pub fn names(self) -> (&'static str, &'static str) {
        match self {
            Self::Initial => ("status-request.bin", "status-result.bin"),
            Self::Final => ("final-status-request.bin", "final-status-result.bin"),
        }
    }
    pub fn started(self, admission: &StatusAdmissionRecord, args: &[u8]) -> EventRecord {
        match self {
            Self::Initial => EventRecord::PreparationReserved {
                sequence: 1,
                admission: admission.clone(),
                arguments_sha256: *Sha256Digest::compute(args).as_bytes(),
            },
            Self::Final => EventRecord::FinalStatusStarted {
                admission: admission.clone(),
            },
        }
    }
    pub fn prepared(self, request: EvidenceRecord) -> EventRecord {
        match self {
            Self::Initial => EventRecord::Prepared {
                sequence: 2,
                request,
            },
            Self::Final => EventRecord::FinalStatusPrepared { request },
        }
    }
    pub fn submitted(self) -> EventRecord {
        match self {
            Self::Initial => EventRecord::SubmissionReserved { sequence: 3 },
            Self::Final => EventRecord::FinalStatusSubmissionReserved,
        }
    }
    pub fn settled(self, result: EvidenceRecord) -> EventRecord {
        match self {
            Self::Initial => EventRecord::Settled {
                sequence: 4,
                result,
            },
            Self::Final => EventRecord::FinalStatusSettled { result },
        }
    }
    pub fn distinct_request(
        self,
        op: &CaptureOperation,
        prepared: &PreparedStatus,
    ) -> Result<(), CaptureError> {
        if self == Self::Final {
            let state = op.check()?;
            let original = PreparedStatus::decode(&op.evidence(
                "status-request.bin",
                state.request.as_ref().ok_or(CaptureError::Incomplete)?,
            )?)?;
            if prepared.request_id == original.request_id {
                return Err(CaptureError::StatusAdmission);
            }
        }
        Ok(())
    }
}

/// Final status refusal retains original spending and payload custody.
#[derive(Debug, Error)]
pub enum CaptureFinalStatusError<E> {
    /// Local custody or missing verified assembly.
    #[error(transparent)]
    Capture(#[from] CaptureError),
    /// Original capture and complete payload authentication refused.
    #[error("original capture payload verification refused")]
    Payload(Box<CaptureDataAdvanceError<E, Infallible>>),
    /// Reserved status continuation or stopped/module/controller policy refused.
    #[error(transparent)]
    Status(#[from] StatusAdvanceError<E>),
}
impl CaptureOperation {
    /// Recheck stopped state, original module and controller after verified assembly.
    /// Uses a distinct original request and finite reservation; pending calls never
    /// resign or resubmit. Settled replay authenticates locally without live authority.
    /// This does not publish a manifest or release the target. Final-call lookup and
    /// surviving-reply settlement are not yet implemented.
    /// # Errors
    /// Refuses absent assembly, invalid payload/evidence, changed original authority,
    /// reuse of the initial status request ID, failed policy and uncertain effects.
    pub async fn advance_final_status<P: CaptureProvider>(
        &mut self,
        admission: &StatusAdmissionRecord,
        transport: &StatusHttpTransport,
        provider: &mut P,
    ) -> Result<StatusProgress, CaptureFinalStatusError<P::Error>> {
        if self.check()?.assembly_sha256.is_none() {
            return Err(CaptureError::Incomplete.into());
        }
        self.verify_data(transport, provider)
            .await
            .map_err(|e| CaptureFinalStatusError::Payload(Box::new(e)))?;
        Ok(self
            .advance_status_slot(admission, provider, StatusSlot::Final)
            .await?)
    }
    /// Inspect original final-status progress and allowance without authentication or writes.
    /// Before reservation the phase is `Created` and allowance is zero.
    /// # Errors
    /// Refuses changed custody, malformed original requests or damaged history.
    pub fn inspect_final_status(&self) -> Result<CaptureStatusView, CaptureError> {
        self.inspect_status_slot(StatusSlot::Final)
    }
}
