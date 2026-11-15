//! Lookup transitions share the original chain without renumbering original calls.
use super::{
    CaptureCall, CaptureError, CapturePlan, CaptureSnapshotPhase, CaptureStatusPhase, EventRecord,
    EvidenceRecord, LedgerError, State, StatusAdmissionRecord,
};
use crate::store::{CaptureLookupAdmissionRecord, CaptureLookupPhase};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::store::capture) struct LookupState {
    pub admission: CaptureLookupAdmissionRecord,
    pub phase: CaptureLookupPhase,
    pub request: Option<EvidenceRecord>,
    pub certificate: Option<EvidenceRecord>,
    pub settlement_root: Option<Vec<u8>>,
}
impl LookupState {
    pub fn evidence(&self) -> [(&'static str, Option<&EvidenceRecord>); 2] {
        [
            ("lookup-request.bin", self.request.as_ref()),
            ("lookup-certificate.bin", self.certificate.as_ref()),
        ]
    }
}
impl State {
    pub fn pending(&self, call: CaptureCall) -> bool {
        match call {
            CaptureCall::Status => self.phase == CaptureStatusPhase::SubmissionReserved,
            CaptureCall::ListSnapshots => {
                self.snapshot.phase == CaptureSnapshotPhase::ListSubmissionReserved
            }
            CaptureCall::TakeSnapshot => {
                self.snapshot.phase == CaptureSnapshotPhase::TakeSubmissionReserved
            }
        }
    }
    pub fn original(
        &self,
        call: CaptureCall,
    ) -> Result<
        (
            &StatusAdmissionRecord,
            &EvidenceRecord,
            Option<&EvidenceRecord>,
        ),
        CaptureError,
    > {
        let (a, r, e) = match call {
            CaptureCall::Status => (
                self.admission.as_ref(),
                self.request.as_ref(),
                self.result.as_ref(),
            ),
            CaptureCall::ListSnapshots => (
                self.snapshot.admission.as_ref().map(|a| &a.list),
                self.snapshot.list_request.as_ref(),
                self.snapshot.list_result.as_ref(),
            ),
            CaptureCall::TakeSnapshot => (
                self.snapshot.admission.as_ref().map(|a| &a.take),
                self.snapshot.take_request.as_ref(),
                self.snapshot.take_result.as_ref(),
            ),
        };
        Ok((
            a.ok_or(CaptureError::StatusAdmission)?,
            r.ok_or(CaptureError::StatusAdmission)?,
            e,
        ))
    }
    pub(super) fn apply_lookup(
        &mut self,
        event: &EventRecord,
        plan: &CapturePlan,
    ) -> Result<(), LedgerError> {
        use CaptureLookupPhase as Phase;
        if let EventRecord::LookupStarted { admission } = event {
            if self.lookup.is_some() || !self.pending(admission.call) {
                return Err(LedgerError::Transition);
            }
            admission.policy().map_err(|_| LedgerError::Transition)?;
            let (original, _, _) = self
                .original(admission.call)
                .map_err(|_| LedgerError::Transition)?;
            if admission.result_bytes > original.result_bytes {
                return Err(LedgerError::Transition);
            }
            self.lookup = Some(LookupState {
                admission: admission.clone(),
                phase: Phase::SigningReserved,
                request: None,
                certificate: None,
                settlement_root: None,
            });
            return Ok(());
        }
        let call = self
            .lookup
            .as_ref()
            .ok_or(LedgerError::Transition)?
            .admission
            .call;
        if !self.pending(call) {
            return Err(LedgerError::Transition);
        }
        if let EventRecord::LookupSettled {
            root_key_der,
            result,
        } = event
        {
            let lookup = self.lookup.as_ref().ok_or(LedgerError::Transition)?;
            if lookup.phase != Phase::Observed
                || lookup.settlement_root.is_some()
                || root_key_der.len() != 133
                || super::Sha256Digest::compute(root_key_der) != plan.request().trust_root
            {
                return Err(LedgerError::Transition);
            }
            let event = match call {
                CaptureCall::Status => EventRecord::Settled {
                    sequence: 4,
                    result: result.clone(),
                },
                _ => EventRecord::SnapshotSettled {
                    sequence: if call == CaptureCall::ListSnapshots {
                        8
                    } else {
                        12
                    },
                    call,
                    result: result.clone(),
                },
            };
            self.apply_original(&event, plan)?;
            self.lookup
                .as_mut()
                .ok_or(LedgerError::Transition)?
                .settlement_root = Some(root_key_der.clone());
            return Ok(());
        }
        let lookup = self.lookup.as_mut().ok_or(LedgerError::Transition)?;
        match (lookup.phase, event) {
            (Phase::SigningReserved, EventRecord::LookupPrepared { request })
                if (1..=lookup.admission.request_bytes).contains(&request.bytes) =>
            {
                lookup.request = Some(request.clone());
                lookup.phase = Phase::Prepared;
            }
            (Phase::Prepared, EventRecord::LookupSubmissionReserved) => {
                lookup.phase = Phase::SubmissionReserved
            }
            (Phase::SubmissionReserved, EventRecord::LookupObserved { certificate })
                if (1..=lookup.admission.result_bytes.saturating_sub(5))
                    .contains(&certificate.bytes) =>
            {
                lookup.certificate = Some(certificate.clone());
                lookup.phase = Phase::Observed;
            }
            _ => return Err(LedgerError::Transition),
        }
        Ok(())
    }
}
