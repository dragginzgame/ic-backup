use super::super::*;
use crate::store::{
    BackupStore, CaptureMetadataAdmissionRecord, CaptureMetadataSigner,
    capture::{
        advance::tests::{
            certificate_tests::{reply_tree, signed},
            fixtures::{
                durable_lookup::reserve_original,
                http::{Response, http, now, set_expiry},
                signatures::{Scheme, change},
            },
        },
        ledger::{EventRecord, EvidenceRecord},
        status::tests::status,
    },
};
use ic_host_artifacts::artifact::Sha256Digest;
use ic_platform::management_canister_types::Snapshot;
use std::{cell::Cell, future::pending};

pub(in crate::store) fn captured(
    store: &BackupStore,
    p: CapturePlan,
    s: &Snapshot,
    r: &PreparedStatus,
    a: &StatusAdmissionRecord,
) -> CaptureOperation {
    captured_with_lookup(store, p, s, r, a, false)
}

pub(in crate::store) fn captured_with_lookup(
    store: &BackupStore,
    p: CapturePlan,
    s: &Snapshot,
    r: &PreparedStatus,
    a: &StatusAdmissionRecord,
    lookup: bool,
) -> CaptureOperation {
    let original = |call: CaptureCall| {
        let mut r = PreparedStatus::decode(&r.encode(4096).unwrap()).unwrap();
        change(&mut r, |e| {
            if let EnvelopeContent::Call {
                method_name, arg, ..
            } = e.content.to_mut()
            {
                *method_name = call.method_name().into();
                *arg = call.arguments(&p).unwrap();
            }
            e.sender_sig = Some(Scheme::Ed25519.sign(&e.content.to_request_id().signable()));
        });
        r
    };
    let status_r = original(CaptureCall::Status);
    let list_r = original(CaptureCall::ListSnapshots);
    let take_r = original(CaptureCall::TakeSnapshot);
    let response = candid_codec::encode_one(&status(&p), 4096).unwrap();
    let proof = signed(
        reply_tree(&status_r.request_id, &response, b"replied"),
        1,
        None,
    );
    let bytes = StatusEvidence {
        response,
        proof,
        rejected: false,
    }
    .encode(4096)
    .unwrap();
    let mut op = reserve_original(store, p, &status_r, a);
    if lookup {
        op.commit(
            &EventRecord::LookupStarted {
                admission: crate::store::CaptureLookupAdmissionRecord {
                    call: CaptureCall::Status,
                    transport: [1; 32],
                    signer: [1; 32],
                    read_authority: [1; 32],
                    prepare_deadline_ns: a.prepare_deadline_ns,
                    lookup_deadline_ns: a.submit_deadline_ns,
                    expires_ns: r.expires_ns,
                    prepare_requests: 0,
                    request_bytes: a.request_bytes,
                    result_bytes: a.result_bytes,
                    observation: r.observation.clone(),
                },
            },
            None,
        )
        .unwrap();
    }
    op.commit(
        &EventRecord::Settled {
            sequence: 4,
            result: EvidenceRecord::new(&bytes),
        },
        Some(("status-result.bin", &bytes)),
    )
    .unwrap();
    let snaps = SnapshotAdmissionRecord {
        capacity: 2,
        list: a.clone(),
        take: a.clone(),
    };
    op.commit(
        &EventRecord::SnapshotStarted {
            sequence: 5,
            admission: snaps,
            arguments_sha256: *Sha256Digest::compute(
                &CaptureCall::ListSnapshots.arguments(&op.plan).unwrap(),
            )
            .as_bytes(),
        },
        None,
    )
    .unwrap();
    for (call, r, base) in [
        (CaptureCall::ListSnapshots, list_r, 5),
        (CaptureCall::TakeSnapshot, take_r, 9),
    ] {
        if call == CaptureCall::TakeSnapshot {
            op.commit(
                &EventRecord::SnapshotTakeReserved {
                    sequence: 9,
                    arguments_sha256: *Sha256Digest::compute(&call.arguments(&op.plan).unwrap())
                        .as_bytes(),
                },
                None,
            )
            .unwrap();
        }
        let bytes = r.encode(4096).unwrap();
        let names = crate::store::capture::advance::snapshots::names(call).unwrap();
        op.commit(
            &EventRecord::SnapshotPrepared {
                sequence: base + 1,
                call,
                request: EvidenceRecord::new(&bytes),
            },
            Some((names.0, &bytes)),
        )
        .unwrap();
        op.commit(
            &EventRecord::SnapshotSubmissionReserved {
                sequence: base + 2,
                call,
            },
            None,
        )
        .unwrap();
        let response = if call == CaptureCall::ListSnapshots {
            candid_codec::encode_one(&Vec::<Snapshot>::new(), 4096).unwrap()
        } else {
            candid_codec::encode_one(s, 4096).unwrap()
        };
        let proof = signed(reply_tree(&r.request_id, &response, b"replied"), 1, None);
        let bytes = StatusEvidence {
            response,
            proof,
            rejected: false,
        }
        .encode(4096)
        .unwrap();
        op.commit(
            &EventRecord::SnapshotSettled {
                sequence: base + 3,
                call,
                result: EvidenceRecord::new(&bytes),
            },
            Some((names.1, &bytes)),
        )
        .unwrap();
    }
    op
}

pub(in crate::store) struct MetadataSigner {
    pub(in crate::store) bytes: Vec<u8>,
    pub(in crate::store) calls: usize,
    pub(in crate::store) admitted: Cell<usize>,
    pub(in crate::store) allow: bool,
    pub(in crate::store) fail: bool,
    pub(in crate::store) hang: bool,
    pub(in crate::store) stop_prepared: bool,
}

impl CaptureMetadataSigner for MetadataSigner {
    type Error = ();
    fn protocol(&self) -> [u8; 32] {
        [91; 32]
    }
    fn admit_read(
        &self,
        _: &CapturePlan,
        _: &CaptureMetadataAdmissionRecord,
        _: &Snapshot,
    ) -> Result<(), ()> {
        let n = self.admitted.get();
        self.admitted.set(n + 1);
        if self.allow && (!self.stop_prepared || n == 0) {
            Ok(())
        } else {
            Err(())
        }
    }
    async fn sign_metadata(
        &mut self,
        _: &CapturePlan,
        _: &CaptureMetadataAdmissionRecord,
        _: &Snapshot,
    ) -> Result<Vec<u8>, ()> {
        self.calls += 1;
        if self.hang {
            pending::<()>().await;
        }
        if self.fail {
            Err(())
        } else {
            Ok(self.bytes.clone())
        }
    }
}

pub(in crate::store) fn signer(r: &PreparedStatus) -> MetadataSigner {
    MetadataSigner {
        bytes: r.envelope.clone(),
        calls: 0,
        admitted: Cell::new(0),
        allow: true,
        fail: false,
        hang: false,
        stop_prepared: false,
    }
}

pub(in crate::store) fn input() -> (
    CapturePlan,
    Snapshot,
    PreparedStatus,
    StatusEvidence,
    StatusAdmissionRecord,
) {
    let (p, s, mut r, _) =
        crate::store::capture::advance::tests::metadata_tests::fixture(Scheme::Ed25519);
    set_expiry(&mut r, now() + 30_000_000_000, Scheme::Ed25519);
    let mut a = crate::store::capture::advance::tests::fixtures::http::input().2;
    a.prepare_requests = 0;
    let old = crate::store::capture::advance::tests::metadata_tests::fixture(Scheme::Ed25519).3;
    let proof = signed(
        reply_tree(&r.request_id, &old.response, b"replied"),
        1,
        None,
    );
    (p, s, r, StatusEvidence { proof, ..old }, a)
}

pub(in crate::store) fn authority(
    t: &StatusHttpTransport,
    a: &StatusAdmissionRecord,
    r: &PreparedStatus,
) -> CaptureMetadataAdmissionRecord {
    CaptureMetadataAdmissionRecord {
        call: a.clone(),
        transport: t.lookup_commitment(),
        signer: [91; 32],
        read_authority: [92; 32],
        expires_ns: r.expires_ns,
        observation: r.observation.clone(),
    }
}

pub(in crate::store) fn response(e: &StatusEvidence) -> Response {
    let mut body = b"\xa2\x66status\x67replied\x6bcertificate\x59".to_vec();
    body.extend_from_slice(&(e.proof.len() as u16).to_be_bytes());
    body.extend_from_slice(&e.proof);
    Response::Bytes(http(200, "", &body))
}
