//! Final status consumes its own original allowance after verified local assembly.
use super::{
    CapturePlan, CaptureStatusPhase as Phase, EventRecord, EvidenceRecord, LedgerError, State,
};
use crate::store::StatusAdmissionRecord;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::store::capture) struct StatusState {
    pub phase: Phase,
    pub admission: Option<StatusAdmissionRecord>,
    pub request: Option<EvidenceRecord>,
    pub result: Option<EvidenceRecord>,
}
impl StatusState {
    pub fn created() -> Self {
        Self {
            phase: Phase::Created,
            admission: None,
            request: None,
            result: None,
        }
    }
    pub fn evidence(&self) -> [(&'static str, Option<&EvidenceRecord>); 2] {
        [
            ("final-status-request.bin", self.request.as_ref()),
            ("final-status-result.bin", self.result.as_ref()),
        ]
    }
}
impl State {
    pub(super) fn apply_final_status(
        &mut self,
        event: &EventRecord,
        plan: &CapturePlan,
    ) -> Result<(), LedgerError> {
        if self.assembly_sha256.is_none() {
            return Err(LedgerError::Transition);
        }
        if let EventRecord::FinalStatusStarted { admission } = event {
            let original = self.admission.as_ref().ok_or(LedgerError::Transition)?;
            if self.final_status.is_some()
                || admission.provider != original.provider
                || admission.application != original.application
            {
                return Err(LedgerError::Transition);
            }
            admission
                .validate(plan)
                .map_err(|_| LedgerError::Transition)?;
            self.final_status = Some(StatusState {
                phase: Phase::PreparationReserved,
                admission: Some(admission.clone()),
                request: None,
                result: None,
            });
            return Ok(());
        }
        let state = self.final_status.as_mut().ok_or(LedgerError::Transition)?;
        let admission = state.admission.as_ref().ok_or(LedgerError::Transition)?;
        match (state.phase, event) {
            (Phase::PreparationReserved, EventRecord::FinalStatusPrepared { request })
                if (46..=admission.request_bytes).contains(&request.bytes) =>
            {
                state.request = Some(request.clone());
                state.phase = Phase::Prepared;
            }
            (Phase::Prepared, EventRecord::FinalStatusSubmissionReserved) => {
                state.phase = Phase::SubmissionReserved
            }
            (Phase::SubmissionReserved, EventRecord::FinalStatusSettled { result })
                if (7..=admission.result_bytes).contains(&result.bytes) =>
            {
                state.result = Some(result.clone());
                state.phase = Phase::Settled;
            }
            _ => return Err(LedgerError::Transition),
        }
        Ok(())
    }
}
