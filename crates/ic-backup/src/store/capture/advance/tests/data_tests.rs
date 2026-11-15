//! Real signatures and synthetic certified replies; no remote payload effects.
use super::{
    certificate_tests::{der, reply_tree, signed},
    fixtures::signatures::{Scheme, change},
    *,
};
use crate::store::{CaptureTransfer, DataReadError, TransferDataError};
use ic_platform::{
    certification::{fork, labeled, leaf},
    management_canister_types::{
        ChunkHash, ReadCanisterSnapshotDataResult, ReadCanisterSnapshotMetadataResult,
        SnapshotDataKind,
    },
};

fn fixture(
    scheme: Scheme,
) -> (
    CapturePlan,
    Vec<u8>,
    ReadCanisterSnapshotMetadataResult,
    PreparedStatus,
) {
    let (p, s, r, e) = metadata_tests::fixture(scheme);
    let mut m = p.decode_metadata(&s.id, &e.response).unwrap();
    let mut request = p.request().clone();
    request.limits.extent_bytes = 4;
    request.limits.network_requests = 8192;
    request.limits.signing_requests = 8192;
    m.wasm_memory_size = 5;
    m.stable_memory_size = 3;
    m.wasm_chunk_store = [b"".as_slice(), b"xyz"]
        .into_iter()
        .map(|bytes| ChunkHash {
            hash: Sha256Digest::compute(bytes).as_bytes().to_vec(),
        })
        .collect();
    (CapturePlan::new(request).unwrap(), s.id, m, r)
}
fn request(
    transfer: &CaptureTransfer<'_>,
    ordinal: u32,
    base: &PreparedStatus,
    scheme: Scheme,
) -> PreparedStatus {
    let mut r = PreparedStatus::decode(&base.encode(4096).unwrap()).unwrap();
    change(&mut r, |e| {
        if let EnvelopeContent::Call {
            method_name, arg, ..
        } = e.content.to_mut()
        {
            *method_name = "read_canister_snapshot_data".into();
            *arg = transfer.data_arguments(ordinal).unwrap();
        }
        e.sender_sig = Some(scheme.sign(&e.content.to_request_id().signable()));
    });
    r
}
fn evidence(r: &PreparedStatus, bytes: &[u8]) -> StatusEvidence {
    raw_evidence(
        r,
        candid_codec::encode_one(
            &ReadCanisterSnapshotDataResult {
                chunk: bytes.to_vec(),
            },
            2 * 1024 * 1024,
        )
        .unwrap(),
    )
}
fn raw_evidence(r: &PreparedStatus, response: Vec<u8>) -> StatusEvidence {
    StatusEvidence {
        proof: signed(reply_tree(&r.request_id, &response, b"replied"), 1, None),
        response,
        rejected: false,
    }
}
fn check(
    t: &CaptureTransfer<'_>,
    n: u32,
    r: &PreparedStatus,
    e: &StatusEvidence,
) -> Result<Vec<u8>, DataReadError> {
    t.authenticate_data_reply(n, &admission(), r, e, &der(1))
}
#[test]
fn exact_scheduled_regions_and_chunks_authenticate_under_both_keys_without_mutation() {
    for scheme in [Scheme::Ed25519, Scheme::Secp256k1] {
        let (p, id, m, base) = fixture(scheme);
        let t = p.plan_transfer(&id, &m).unwrap();
        for (n, dto) in t.requests().enumerate() {
            let n = n as u32;
            assert_eq!(
                t.data_arguments(n).unwrap(),
                candid_codec::encode_one(&dto, 1024).unwrap()
            );
            let bytes = match dto.kind {
                SnapshotDataKind::WasmModule { size, .. }
                | SnapshotDataKind::WasmMemory { size, .. }
                | SnapshotDataKind::StableMemory { size, .. } => vec![n as u8; size as usize],
                SnapshotDataKind::WasmChunk { hash } => {
                    if hash == Sha256Digest::compute(b"").as_bytes().as_slice() {
                        vec![]
                    } else {
                        b"xyz".to_vec()
                    }
                }
            };
            let r = request(&t, n, &base, scheme);
            let e = evidence(&r, &bytes);
            let original = (r.encode(4096).unwrap(), e.encode(4096).unwrap());
            assert_eq!(check(&t, n, &r, &e).unwrap(), bytes);
            assert_eq!((r.encode(4096).unwrap(), e.encode(4096).unwrap()), original);
        }
        for n in [t.data_calls(), u32::MAX] {
            assert!(matches!(
                t.data_arguments(n),
                Err(DataReadError::Data(TransferDataError::Ordinal))
            ));
        }
        let id = vec![255; 256];
        assert!(
            p.plan_transfer(&id, &m)
                .unwrap()
                .data_arguments(0)
                .unwrap()
                .len()
                <= 1024
        );
    }
}
#[test]
fn signed_substitution_of_target_snapshot_range_hash_or_method_refuses() {
    for fault in 0..9 {
        let (p, id, m, base) = fixture(Scheme::Ed25519);
        let t = p.plan_transfer(&id, &m).unwrap();
        let mut r = request(&t, 0, &base, Scheme::Ed25519);
        let mut dto = t.requests().next().unwrap();
        match fault {
            0 => dto.canister_id = ic_platform::Principal::anonymous(),
            1 => dto.snapshot_id.push(1),
            2 => dto.kind = SnapshotDataKind::WasmModule { offset: 4, size: 4 },
            3 => dto.kind = SnapshotDataKind::WasmModule { offset: 0, size: 3 },
            4 => dto.kind = SnapshotDataKind::WasmMemory { offset: 0, size: 4 },
            5 => dto.kind = SnapshotDataKind::WasmChunk { hash: vec![0; 32] },
            _ => {}
        }
        change(&mut r, |e| {
            if let EnvelopeContent::Call {
                method_name,
                arg,
                canister_id,
                sender,
                ..
            } = e.content.to_mut()
            {
                *arg = candid_codec::encode_one(&dto, 1024).unwrap();
                match fault {
                    6 => *method_name = "read_canister_snapshot_metadata".into(),
                    7 => *canister_id = p.request().target.canister.as_slice().try_into().unwrap(),
                    8 => {
                        *sender = ic_platform::Principal::anonymous()
                            .as_slice()
                            .try_into()
                            .unwrap()
                    }
                    _ => {}
                }
            }
            e.sender_sig = Some(Scheme::Ed25519.sign(&e.content.to_request_id().signable()));
        });
        // Even a valid certificate for the substituted signed call cannot bind it to ordinal zero.
        assert!(matches!(
            check(&t, 0, &r, &evidence(&r, b"abcd")),
            Err(DataReadError::Request(_))
        ));
    }
}
#[test]
fn altered_authentication_and_wrong_same_length_ordinal_refuse() {
    for fault in 0..7 {
        let (p, id, m, base) = fixture(Scheme::Ed25519);
        let t = p.plan_transfer(&id, &m).unwrap();
        let mut r = request(&t, 0, &base, Scheme::Ed25519);
        let mut e = evidence(&r, b"abcd");
        match fault {
            0 => change(&mut r, |e| e.sender_sig.as_mut().unwrap()[0] ^= 1),
            1 => r.request_id[0] ^= 1,
            2 => r.expires_ns += 1,
            3 => e.response[0] ^= 1,
            4 => e.proof[10] ^= 1,
            5 => e.rejected = true,
            6 => {
                let mut time = super::super::certificate::observation(&r.observation).unwrap();
                time.certificate.reference_ns = 101;
                r.observation = status_certificate_observation(time).unwrap();
            }
            _ => unreachable!(),
        }
        let result = check(&t, 0, &r, &e);
        if fault < 3 {
            assert!(matches!(result, Err(DataReadError::Request(_))));
        } else {
            assert!(matches!(result, Err(DataReadError::Certificate(_))));
        }
    }
    let (p, id, m, base) = fixture(Scheme::Ed25519);
    let t = p.plan_transfer(&id, &m).unwrap();
    let r = request(&t, 0, &base, Scheme::Ed25519);
    let e = evidence(&r, b"abcd");
    assert!(matches!(
        check(&t, 1, &r, &e),
        Err(DataReadError::Request(_))
    ));
    assert!(matches!(
        t.authenticate_data_reply(0, &admission(), &r, &e, &der(2)),
        Err(DataReadError::Certificate(
            StatusCertificateError::TrustRoot
        ))
    ));
    let mut a = admission();
    a.request_bytes = 100;
    assert!(matches!(
        t.authenticate_data_reply(0, &a, &r, &e, &der(1)),
        Err(DataReadError::Request(_))
    ));
    a = admission();
    a.result_bytes = 100;
    assert!(matches!(
        t.authenticate_data_reply(0, &a, &r, &e, &der(1)),
        Err(DataReadError::Admission(_))
    ));
}
#[test]
fn certified_bad_lengths_hashes_and_wire_never_become_payload() {
    let (p, id, m, base) = fixture(Scheme::Ed25519);
    let t = p.plan_transfer(&id, &m).unwrap();
    let r = request(&t, 0, &base, Scheme::Ed25519);
    for bytes in [b"".as_slice(), b"abc", b"abcde"] {
        assert!(matches!(
            check(&t, 0, &r, &evidence(&r, bytes)),
            Err(DataReadError::Data(TransferDataError::Length { .. }))
        ));
    }
    assert!(matches!(
        check(&t, 0, &r, &raw_evidence(&r, b"invalid".to_vec())),
        Err(DataReadError::Data(TransferDataError::Decode(_)))
    ));
    for (n, dto) in t.requests().enumerate() {
        if matches!(dto.kind, SnapshotDataKind::WasmChunk { .. }) {
            let r = request(&t, n as u32, &base, Scheme::Ed25519);
            assert!(matches!(
                check(&t, n as u32, &r, &evidence(&r, b"bad")),
                Err(DataReadError::Data(TransferDataError::ChunkHash))
            ));
            assert!(matches!(
                check(&t, n as u32, &r, &evidence(&r, b"12345")),
                Err(DataReadError::Data(TransferDataError::ChunkLimit { .. }))
            ));
        }
    }
}
#[test]
fn certified_rejection_and_nonterminal_proof_never_become_payload() {
    let (p, id, m, base) = fixture(Scheme::Ed25519);
    let t = p.plan_transfer(&id, &m).unwrap();
    let r = request(&t, 0, &base, Scheme::Ed25519);
    let mut e = evidence(&r, b"abcd");
    let tree = fork(
        labeled(
            b"request_status",
            labeled(
                r.request_id.to_vec(),
                fork(
                    labeled(b"reject_code", leaf(vec![5])),
                    fork(
                        labeled(b"reject_message", leaf(b"denied".to_vec())),
                        labeled(b"status", leaf(b"rejected".to_vec())),
                    ),
                ),
            ),
        ),
        labeled(b"time", leaf(vec![100])),
    );
    e.proof = signed(tree, 1, None);
    (e.rejected, e.response) = super::super::certificate::authenticated_expected_response(
        &p,
        &admission(),
        &r,
        &e.proof,
        &der(1),
        "read_canister_snapshot_data",
        &t.data_arguments(0).unwrap(),
    )
    .unwrap();
    assert!(matches!(check(&t, 0, &r, &e), Err(DataReadError::Rejected)));
    for status in [b"received".as_slice(), b"processing", b"done"] {
        e.proof = signed(reply_tree(&r.request_id, b"", status), 1, None);
        assert!(matches!(
            check(&t, 0, &r, &e),
            Err(DataReadError::Certificate(StatusCertificateError::Outcome))
        ));
    }
}

#[test]
fn complete_certified_evidence_must_fit_independently_of_the_payload_limit() {
    for size in [512 * 1024, 1024 * 1024] {
        let (p, id, mut m, base) = fixture(Scheme::Ed25519);
        let mut fields = p.request().clone();
        fields.limits.extent_bytes = size;
        fields.limits.payload_bytes = 4 * 1024 * 1024;
        fields.limits.evidence_bytes = 8 * 1024 * 1024;
        let p = CapturePlan::new(fields).unwrap();
        m.wasm_module_size = u64::from(size);
        let t = p.plan_transfer(&id, &m).unwrap();
        let r = request(&t, 0, &base, Scheme::Ed25519);
        let payload = vec![42; size as usize];
        let e = evidence(&r, &payload);
        let mut a = admission();
        a.result_bytes = 2 * 1024 * 1024;
        let result = t.authenticate_data_reply(0, &a, &r, &e, &der(1));
        if size == 512 * 1024 {
            assert_eq!(result.unwrap(), payload);
        } else {
            assert!(matches!(result, Err(DataReadError::Admission(_))));
        }
    }
}
