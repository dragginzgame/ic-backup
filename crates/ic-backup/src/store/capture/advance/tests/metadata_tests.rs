//! Real signatures over synthetic metadata certificates; no remote read is simulated.
use super::{
    certificate_tests::{der, reply_tree, signed},
    fixtures::signatures::{Scheme, change},
    *,
};
use ic_platform::{
    candid_codec::Reserved,
    certification::{fork, labeled, leaf},
    management_canister_types::{
        CanisterTimer, OnLowWasmMemoryHookStatus, ReadCanisterSnapshotMetadataArgs,
        ReadCanisterSnapshotMetadataResult, Snapshot, SnapshotMetadataGlobal, SnapshotSource,
    },
};

fn metadata() -> ReadCanisterSnapshotMetadataResult {
    ReadCanisterSnapshotMetadataResult {
        source: Some(SnapshotSource::TakenFromCanister(Reserved)),
        taken_at_timestamp: 55,
        wasm_module_size: 10,
        wasm_memory_size: 0,
        stable_memory_size: 0,
        globals: vec![
            Some(SnapshotMetadataGlobal::F32(f32::from_bits(0x7fc00042))),
            Some(SnapshotMetadataGlobal::F64(-0.0)),
        ],
        wasm_chunk_store: vec![],
        canister_version: 7,
        certified_data: vec![0, 255],
        global_timer: Some(CanisterTimer::Inactive),
        on_low_wasm_memory_hook_status: Some(OnLowWasmMemoryHookStatus::ConditionNotSatisfied),
    }
}
pub(super) fn fixture(scheme: Scheme) -> (CapturePlan, Snapshot, PreparedStatus, StatusEvidence) {
    let (p, mut r) = fixtures::signatures::fixture(scheme);
    let mut fields = p.request().clone();
    fields.trust_root = Sha256Digest::compute(&der(1));
    let p = CapturePlan::new(fields).unwrap();
    let snapshot = Snapshot {
        id: vec![0, 255, 17],
        taken_at_timestamp: 55,
        total_size: u64::MAX,
    };
    let args = p.metadata_read_arguments(&snapshot).unwrap();
    change(&mut r, |envelope| {
        if let EnvelopeContent::Call {
            method_name, arg, ..
        } = envelope.content.to_mut()
        {
            *method_name = "read_canister_snapshot_metadata".into();
            *arg = args;
        }
        envelope.sender_sig = Some(scheme.sign(&envelope.content.to_request_id().signable()));
    });
    r.observation = certificate_tests::fixture().1.observation;
    let e = evidence_for(&r, &metadata());
    (p, snapshot, r, e)
}
fn evidence_for(r: &PreparedStatus, m: &ReadCanisterSnapshotMetadataResult) -> StatusEvidence {
    let response = candid_codec::encode_one(m, 4096).unwrap();
    let proof = signed(reply_tree(&r.request_id, &response, b"replied"), 1, None);
    StatusEvidence {
        response,
        proof,
        rejected: false,
    }
}
fn authenticate(
    p: &CapturePlan,
    s: &Snapshot,
    r: &PreparedStatus,
    e: &StatusEvidence,
) -> Result<ReadCanisterSnapshotMetadataResult, MetadataReadError> {
    authenticate_metadata_reply(p, s, &admission(), r, e, &der(1))
}

#[test]
fn exact_metadata_request_and_certified_reply_preserve_bits_and_original_bytes() {
    for scheme in [Scheme::Ed25519, Scheme::Secp256k1] {
        let (p, s, r, e) = fixture(scheme);
        let original = r.encode(4096).unwrap();
        let proof = e.encode(4096).unwrap();
        let args = p.metadata_read_arguments(&s).unwrap();
        let decoded: ReadCanisterSnapshotMetadataArgs = candid_codec::decoder::decode_one(
            &args,
            candid_codec::decoder::Limits {
                bytes: 512,
                header: 512,
                types: 16,
                work: 4096,
                allocation: 4096,
            },
        )
        .unwrap();
        assert_eq!(decoded.canister_id, p.request().target.canister);
        assert_eq!(decoded.snapshot_id, s.id);
        let result = authenticate(&p, &s, &r, &e).unwrap();
        assert_eq!(candid_codec::encode_one(&result, 4096).unwrap(), e.response);
        assert!(
            matches!(result.globals[0],Some(SnapshotMetadataGlobal::F32(v)) if v.to_bits()==0x7fc00042)
        );
        assert!(
            matches!(result.globals[1],Some(SnapshotMetadataGlobal::F64(v)) if v.to_bits()==(-0.0_f64).to_bits())
        );
        assert_eq!(r.encode(4096).unwrap(), original);
        assert_eq!(e.encode(4096).unwrap(), proof);
        assert_eq!(p.plan_transfer(&s.id, &result).unwrap().data_calls(), 1);
    }
}

#[test]
fn snapshot_id_bounds_and_exact_timestamp_binding_refuse_substitution() {
    let (p, mut s, r, e) = fixture(Scheme::Ed25519);
    for size in [0, 257] {
        s.id = vec![1; size];
        assert!(matches!(
            p.metadata_read_arguments(&s),
            Err(MetadataReadError::SnapshotId)
        ));
    }
    s.id = vec![1; 256];
    assert!(p.metadata_read_arguments(&s).unwrap().len() <= 512);
    s.id = vec![0, 255, 18];
    assert!(matches!(
        authenticate(&p, &s, &r, &e),
        Err(MetadataReadError::Request(_))
    ));
    s.id = vec![0, 255, 17];
    s.taken_at_timestamp += 1;
    assert!(matches!(
        authenticate(&p, &s, &r, &e),
        Err(MetadataReadError::SnapshotMismatch)
    ));
}

#[test]
fn valid_signatures_cannot_change_method_receiver_target_arguments_or_sender() {
    for fault in 0..5 {
        let (p, s, mut r, e) = fixture(Scheme::Ed25519);
        change(&mut r, |envelope| {
            if let EnvelopeContent::Call {
                method_name,
                canister_id,
                arg,
                sender,
                ..
            } = envelope.content.to_mut()
            {
                match fault {
                    0 => *method_name = "canister_status".into(),
                    1 => *canister_id = p.request().target.canister.as_slice().try_into().unwrap(),
                    2 => {
                        *arg = candid_codec::encode_one(
                            &ReadCanisterSnapshotMetadataArgs {
                                canister_id: ic_platform::Principal::anonymous(),
                                snapshot_id: s.id.clone(),
                            },
                            512,
                        )
                        .unwrap()
                    }
                    3 => arg.push(0),
                    4 => {
                        *sender = ic_platform::Principal::anonymous()
                            .as_slice()
                            .try_into()
                            .unwrap()
                    }
                    _ => unreachable!(),
                }
            }
            envelope.sender_sig =
                Some(Scheme::Ed25519.sign(&envelope.content.to_request_id().signable()));
        });
        assert!(matches!(
            authenticate(&p, &s, &r, &e),
            Err(MetadataReadError::Request(_))
        ));
    }
}

#[test]
fn metadata_authentication_refuses_changed_signature_reply_proof_root_and_window() {
    for fault in 0..7 {
        let (p, s, mut r, mut e) = fixture(Scheme::Ed25519);
        match fault {
            0 => change(&mut r, |envelope| {
                envelope.sender_sig.as_mut().unwrap()[0] ^= 1
            }),
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
        assert!(authenticate(&p, &s, &r, &e).is_err());
    }
    let (p, s, r, e) = fixture(Scheme::Ed25519);
    assert!(matches!(
        authenticate_metadata_reply(&p, &s, &admission(), &r, &e, &der(2)),
        Err(MetadataReadError::Certificate(
            StatusCertificateError::TrustRoot
        ))
    ));
    let mut small = admission();
    small.result_bytes = 100;
    assert!(authenticate_metadata_reply(&p, &s, &small, &r, &e, &der(1)).is_err());
    small = admission();
    small.request_bytes = 100;
    assert!(authenticate_metadata_reply(&p, &s, &small, &r, &e, &der(1)).is_err());
}

#[test]
fn authenticated_metadata_still_requires_supported_profile_and_payload_budget() {
    let (p, s, r, _) = fixture(Scheme::Ed25519);
    for fault in 0..3 {
        let mut m = metadata();
        match fault {
            0 => m.source = None,
            1 => m.globals.push(None),
            2 => m.wasm_module_size = p.request().limits.payload_bytes + 1,
            _ => unreachable!(),
        }
        assert!(matches!(
            authenticate(&p, &s, &r, &evidence_for(&r, &m)),
            Err(MetadataReadError::Metadata(_))
        ));
    }
}

#[test]
fn certified_metadata_rejection_and_unresolved_request_never_decode_as_metadata() {
    let (p, s, r, mut e) = fixture(Scheme::Ed25519);
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
    let (rejected, response) = super::super::certificate::authenticated_expected_response(
        &p,
        &admission(),
        &r,
        &e.proof,
        &der(1),
        "read_canister_snapshot_metadata",
        &p.metadata_read_arguments(&s).unwrap(),
    )
    .unwrap();
    e.rejected = rejected;
    e.response = response;
    assert!(matches!(
        authenticate(&p, &s, &r, &e),
        Err(MetadataReadError::Rejected)
    ));
    for status in [b"received".as_slice(), b"processing", b"done"] {
        e.proof = signed(reply_tree(&r.request_id, b"", status), 1, None);
        assert!(matches!(
            authenticate(&p, &s, &r, &e),
            Err(MetadataReadError::Certificate(
                StatusCertificateError::Outcome
            ))
        ));
    }
}
