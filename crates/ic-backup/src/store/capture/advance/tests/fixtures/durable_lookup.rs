use super::super::*;
use crate::store::{
    BackupStore,
    capture::{
        advance::tests::{certificate_tests::der, fixtures::http::HttpProvider},
        ledger::{EventRecord, EvidenceRecord},
    },
};
use ic_host_artifacts::artifact::Sha256Digest;

pub(in crate::store) fn reserve_original(
    store: &BackupStore,
    p: CapturePlan,
    r: &PreparedStatus,
    a: &StatusAdmissionRecord,
) -> CaptureOperation {
    let digest = p.digest();
    let mut op = store.create_capture(p, digest).unwrap();
    op.commit(
        &EventRecord::PreparationReserved {
            sequence: 1,
            admission: a.clone(),
            arguments_sha256: *Sha256Digest::compute(&arguments(&op.plan).unwrap()).as_bytes(),
        },
        None,
    )
    .unwrap();
    let bytes = r.encode(a.request_bytes).unwrap();
    op.commit(
        &EventRecord::Prepared {
            sequence: 2,
            request: EvidenceRecord::new(&bytes),
        },
        Some(("status-request.bin", &bytes)),
    )
    .unwrap();
    op.commit(&EventRecord::SubmissionReserved { sequence: 3 }, None)
        .unwrap();
    op
}

pub(in crate::store) fn reopen(store: &BackupStore, p: &CapturePlan) -> CaptureOperation {
    store
        .open_capture(&p.request().operation_id, p.digest())
        .unwrap()
}

impl CaptureProvider for HttpProvider {
    fn admit_call(
        &self,
        _: &CapturePlan,
        _: &StatusAdmissionRecord,
        _: CaptureCall,
        _: Option<&PreparedStatus>,
    ) -> Result<(), Self::Error> {
        Err(StatusHttpError::Binding) // Explicitly revoked live authority in this fixture.
    }
    async fn prepare_call(
        &mut self,
        _: &CapturePlan,
        _: &StatusAdmissionRecord,
        _: CaptureCall,
    ) -> Result<PreparedStatus, Self::Error> {
        panic!("no fresh signing")
    }
    async fn submit_call(
        &mut self,
        _: &CapturePlan,
        _: &StatusAdmissionRecord,
        _: CaptureCall,
        _: &PreparedStatus,
    ) -> Result<StatusEvidence, Self::Error> {
        panic!("no fresh submission")
    }
    fn authenticate_call(
        &self,
        p: &CapturePlan,
        a: &StatusAdmissionRecord,
        call: CaptureCall,
        r: &PreparedStatus,
        e: &StatusEvidence,
    ) -> Result<(), Self::Error> {
        crate::store::capture::advance::certificate::authenticate_call_evidence(
            p,
            a,
            r,
            e,
            &der(1),
            call,
        )
        .map_err(StatusHttpError::Authentication)
    }
}
