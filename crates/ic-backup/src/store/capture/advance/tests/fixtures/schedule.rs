use super::super::*;
use crate::store::{
    CaptureMetadataAdmissionRecord,
    capture::ledger::{EventRecord, EvidenceRecord},
};
use ic_platform::management_canister_types::Snapshot;

pub(in crate::store) fn settle(
    op: &mut CaptureOperation,
    s: &Snapshot,
    r: &PreparedStatus,
    e: &StatusEvidence,
    ma: &CaptureMetadataAdmissionRecord,
) {
    let request = r.encode(ma.call.request_bytes).unwrap();
    let result = e.encode(ma.call.result_bytes).unwrap();
    op.commit(
        &EventRecord::MetadataStarted {
            admission: ma.clone(),
            snapshot_id: s.id.clone(),
            timestamp: s.taken_at_timestamp,
            total_size: s.total_size,
        },
        None,
    )
    .unwrap();
    op.commit(
        &EventRecord::MetadataPrepared {
            request: EvidenceRecord::new(&request),
        },
        Some(("metadata-request.bin", &request)),
    )
    .unwrap();
    op.commit(&EventRecord::MetadataSubmissionReserved, None)
        .unwrap();
    op.commit(
        &EventRecord::MetadataSettled {
            result: EvidenceRecord::new(&result),
        },
        Some(("metadata-result.bin", &result)),
    )
    .unwrap();
}
