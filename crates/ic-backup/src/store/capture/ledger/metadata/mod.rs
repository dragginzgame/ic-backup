//! Metadata events retain the original settled snapshot association in the existing chain.
use super::{CapturePlan, CaptureSnapshotPhase, EventRecord, EvidenceRecord, LedgerError, State};
use crate::store::{CaptureMetadataAdmissionRecord, CaptureMetadataPhase};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::store::capture) struct MetadataState {
    pub admission: CaptureMetadataAdmissionRecord,
    pub snapshot_id: Vec<u8>,
    pub timestamp: u64,
    pub total_size: u64,
    pub phase: CaptureMetadataPhase,
    pub request: Option<EvidenceRecord>,
    pub result: Option<EvidenceRecord>,
}
impl MetadataState {
    pub fn evidence(&self) -> [(&'static str, Option<&EvidenceRecord>); 2] {
        [
            ("metadata-request.bin", self.request.as_ref()),
            ("metadata-result.bin", self.result.as_ref()),
        ]
    }
}
impl State {
    pub(super) fn apply_metadata(
        &mut self,
        event: &EventRecord,
        plan: &CapturePlan,
    ) -> Result<(), LedgerError> {
        use CaptureMetadataPhase as Phase;
        if self.snapshot.phase != CaptureSnapshotPhase::Settled {
            return Err(LedgerError::Transition);
        }
        if let EventRecord::MetadataStarted {
            admission,
            snapshot_id,
            timestamp,
            total_size,
        } = event
        {
            if self.metadata.is_some() || !(1..=256).contains(&snapshot_id.len()) {
                return Err(LedgerError::Transition);
            }
            admission
                .validate(plan)
                .map_err(|_| LedgerError::Transition)?;
            let original = self.admission.as_ref().ok_or(LedgerError::Transition)?;
            if admission.call.provider != original.provider
                || admission.call.application != original.application
            {
                return Err(LedgerError::Transition);
            }
            self.metadata = Some(MetadataState {
                admission: admission.clone(),
                snapshot_id: snapshot_id.clone(),
                timestamp: *timestamp,
                total_size: *total_size,
                phase: Phase::SigningReserved,
                request: None,
                result: None,
            });
            return Ok(());
        }
        let m = self.metadata.as_mut().ok_or(LedgerError::Transition)?;
        match (m.phase, event) {
            (Phase::SigningReserved, EventRecord::MetadataPrepared { request })
                if (46..=m.admission.call.request_bytes).contains(&request.bytes) =>
            {
                m.request = Some(request.clone());
                m.phase = Phase::Prepared;
            }
            (Phase::Prepared, EventRecord::MetadataSubmissionReserved) => {
                m.phase = Phase::SubmissionReserved
            }
            (Phase::SubmissionReserved, EventRecord::MetadataSettled { result })
                if (7..=m.admission.call.result_bytes).contains(&result.bytes) =>
            {
                m.result = Some(result.clone());
                m.phase = Phase::Settled;
            }
            _ => return Err(LedgerError::Transition),
        }
        Ok(())
    }
}
