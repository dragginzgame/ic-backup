//! Ordered data reservations and accepted byte identities in the original ledger.
use super::{CapturePlan, EventRecord, EvidenceRecord, LedgerError, State};
use crate::store::{CaptureDataAdmissionRecord, CaptureDataPhase};
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::store::capture) struct DataState {
    pub admission: CaptureDataAdmissionRecord,
    pub phase: CaptureDataPhase,
    pub request: Option<EvidenceRecord>,
    pub result: Option<EvidenceRecord>,
    pub payload: Option<EvidenceRecord>,
}
pub(in crate::store::capture) fn data_names(ordinal: u32) -> (String, String) {
    (
        format!("data-{ordinal:04}-request.bin"),
        format!("data-{ordinal:04}-result.bin"),
    )
}
impl State {
    pub(super) fn apply_data(
        &mut self,
        event: &EventRecord,
        plan: &CapturePlan,
    ) -> Result<(), LedgerError> {
        if let EventRecord::DataStarted { ordinal, admission } = event {
            let binding = self.transfer.as_ref().ok_or(LedgerError::Transition)?;
            let original = self.admission.as_ref().ok_or(LedgerError::Transition)?;
            if *ordinal as usize != self.data.len()
                || *ordinal >= binding.data_calls
                || self
                    .data
                    .last()
                    .is_some_and(|d| d.phase != CaptureDataPhase::Accepted)
                || admission.call.provider != original.provider
                || admission.call.application != original.application
            {
                return Err(LedgerError::Transition);
            }
            admission
                .validate(plan)
                .map_err(|_| LedgerError::Transition)?;
            self.data.push(DataState {
                admission: admission.clone(),
                phase: CaptureDataPhase::SigningReserved,
                request: None,
                result: None,
                payload: None,
            });
            return Ok(());
        }
        let ordinal = match event {
            EventRecord::DataPrepared { ordinal, .. }
            | EventRecord::DataSubmissionReserved { ordinal }
            | EventRecord::DataSettled { ordinal, .. }
            | EventRecord::DataAccepted { ordinal, .. } => *ordinal,
            _ => return Err(LedgerError::Transition),
        };
        if ordinal as usize + 1 != self.data.len() {
            return Err(LedgerError::Transition);
        }
        let d = self.data.last_mut().ok_or(LedgerError::Transition)?;
        match (d.phase, event) {
            (CaptureDataPhase::SigningReserved, EventRecord::DataPrepared { request, .. })
                if (46..=d.admission.call.request_bytes).contains(&request.bytes) =>
            {
                d.request = Some(request.clone());
                d.phase = CaptureDataPhase::Prepared;
            }
            (CaptureDataPhase::Prepared, EventRecord::DataSubmissionReserved { .. }) => {
                d.phase = CaptureDataPhase::SubmissionReserved
            }
            (CaptureDataPhase::SubmissionReserved, EventRecord::DataSettled { result, .. })
                if (7..=d.admission.call.result_bytes).contains(&result.bytes) =>
            {
                d.result = Some(result.clone());
                d.phase = CaptureDataPhase::Settled;
            }
            (CaptureDataPhase::Settled, EventRecord::DataAccepted { payload, .. })
                if payload.bytes <= plan.request().limits.extent_bytes =>
            {
                d.payload = Some(payload.clone());
                d.phase = CaptureDataPhase::Accepted;
            }
            _ => return Err(LedgerError::Transition),
        }
        Ok(())
    }
    pub(in crate::store::capture) fn data_name_admitted(&self, name: &str) -> bool {
        let Some(rest) = name.strip_prefix("data-") else {
            return false;
        };
        let Some((n, _)) = rest.split_once('-') else {
            return false;
        };
        let Ok(ordinal) = n.parse::<u32>() else {
            return false;
        };
        let Some(d) = self.data.get(ordinal as usize) else {
            return false;
        };
        let names = data_names(ordinal);
        (name == names.0 && d.request.is_some()) || (name == names.1 && d.result.is_some())
    }
}
