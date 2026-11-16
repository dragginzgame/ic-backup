//! Real BLS signatures over synthetic certificates. No network or IC effects.
use super::fixtures::certificates::{delegated, der, reply_tree, signed, times};
use super::*;
use ic_platform::certification::{
    cbor, fork,
    historical::{self, TimePolicy, TimeWindow},
    labeled, leaf, pruned,
};

pub(in crate::store) fn fixture() -> (CapturePolicy, PreparedStatus, StatusEvidence) {
    let mut request = plan().request().clone();
    request.trust_root = Sha256Digest::compute(&der(1));
    let plan = CapturePolicy::new(request).unwrap();
    let mut prepared = prepared();
    prepared.observation = status_certificate_observation(times()).unwrap();
    let mut evidence = evidence();
    evidence.proof = signed(
        reply_tree(&prepared.request_id, &evidence.response, b"replied"),
        1,
        None,
    );
    (plan, prepared, evidence)
}
fn authenticate(
    p: &CapturePolicy,
    r: &PreparedStatus,
    e: &StatusEvidence,
) -> Result<(), StatusCertificateError> {
    authenticate_status_evidence(p, &admission(), r, e, &der(1))
}

#[test]
fn full_extent_certificate_fits_explicit_new_profile_but_never_enlarges_old_allowance() {
    let (p, r, _) = fixture();
    let reply = ic_platform::candid_codec::encode_one(
        &ic_platform::management_canister_types::ReadCanisterSnapshotDataResult {
            chunk: vec![0xa5; 1024 * 1024],
        },
        2 * 1024 * 1024,
    )
    .unwrap();
    let certificate = signed(reply_tree(&r.request_id, &reply, b"replied"), 1, None);
    assert!(certificate.len() > 1024 * 1024);
    let requirements = super::super::CertificateRequirements {
        canister: p.request().target.canister,
        trust_root: p.request().trust_root,
    };
    let mut limits = admission();
    limits.result_bytes = crate::store::limits::RESULT_BYTES;
    assert_eq!(
        requirements
            .response(&limits, &r, &certificate, &der(1))
            .unwrap()
            .1,
        reply
    );
    limits.result_bytes = 2 * 1024 * 1024;
    assert!(
        requirements
            .response(&limits, &r, &certificate, &der(1))
            .is_err()
    );
}

#[test]
fn original_old_certificate_and_exact_reply_authenticate_after_record_round_trip() {
    let (p, r, e) = fixture();
    let request = r.encode(4096).unwrap();
    let result = e.encode(4096).unwrap();
    let retained = PreparedStatus::decode(&request).unwrap();
    let evidence = StatusEvidence::decode(&result).unwrap();
    authenticate(&p, &retained, &evidence).unwrap();
    p.validate_status_reply(&evidence.response).unwrap();
    assert_eq!(retained.encode(4096).unwrap(), request);
    assert_eq!(evidence.encode(4096).unwrap(), result);
    assert_eq!(
        &retained.observation[..31],
        b"ic-backup/status-certificate/1\n"
    );
    assert_eq!(&retained.observation[31..39], &100_u64.to_le_bytes());
}

#[test]
fn exact_original_request_id_call_and_root_are_required() {
    let (p, mut r, e) = fixture();
    r.request_id[0] ^= 1;
    assert!(matches!(
        authenticate(&p, &r, &e),
        Err(StatusCertificateError::Admission(_))
    ));
    let mut content = call_content();
    if let EnvelopeContent::Call { nonce, .. } = &mut content {
        *nonce = Some(vec![42]);
    }
    r = prepared_content(content);
    r.observation = status_certificate_observation(times()).unwrap();
    assert!(matches!(
        authenticate(&p, &r, &e),
        Err(StatusCertificateError::Outcome)
    ));
    let (p, r, e) = fixture();
    assert!(matches!(
        authenticate_status_evidence(&p, &admission(), &r, &e, &der(2)),
        Err(StatusCertificateError::TrustRoot)
    ));
    let mut malformed = der(1);
    malformed[0] ^= 1;
    let mut request = p.request().clone();
    request.trust_root = Sha256Digest::compute(&malformed);
    assert!(matches!(
        authenticate_status_evidence(
            &CapturePolicy::new(request).unwrap(),
            &admission(),
            &r,
            &e,
            &malformed
        ),
        Err(StatusCertificateError::Certificate(
            historical::Error::InvalidKey
        ))
    ));
}

#[test]
fn changed_response_signature_and_trailing_proof_refuse() {
    let (p, r, mut e) = fixture();
    e.response[0] ^= 1;
    assert!(matches!(
        authenticate(&p, &r, &e),
        Err(StatusCertificateError::Response)
    ));
    let (p, r, mut e) = fixture();
    let mut certificate = cbor::decode_certificate(&e.proof, 4096).unwrap();
    certificate.signature[0] ^= 1;
    e.proof = cbor::encode_certificate(&certificate, 4096).unwrap();
    assert!(matches!(
        authenticate(&p, &r, &e),
        Err(StatusCertificateError::Certificate(
            historical::Error::Signature
        ))
    ));
    let (p, r, mut e) = fixture();
    e.proof.push(0);
    assert!(matches!(
        authenticate(&p, &r, &e),
        Err(StatusCertificateError::Certificate(
            historical::Error::Decode(_)
        ))
    ));
}

#[test]
fn non_success_and_unavailable_paths_never_settle_as_reply() {
    let (p, r, mut e) = fixture();
    for outcome in [b"received".as_slice(), b"processing", b"done", b"unknown"] {
        e.proof = signed(reply_tree(&r.request_id, &e.response, outcome), 1, None);
        assert!(matches!(
            authenticate(&p, &r, &e),
            Err(StatusCertificateError::Outcome)
        ));
    }
    for subtree in [pruned([7; 32]), labeled(b"reply", leaf(e.response.clone()))] {
        e.proof = signed(
            fork(
                labeled(b"request_status", labeled(r.request_id.to_vec(), subtree)),
                labeled(b"time", leaf(vec![100])),
            ),
            1,
            None,
        );
        assert!(matches!(
            authenticate(&p, &r, &e),
            Err(StatusCertificateError::Outcome)
        ));
    }
    e.proof = signed(
        fork(
            labeled(
                b"request_status",
                labeled(
                    r.request_id.to_vec(),
                    labeled(b"status", leaf(b"replied".to_vec())),
                ),
            ),
            labeled(b"time", leaf(vec![100])),
        ),
        1,
        None,
    );
    assert!(matches!(
        authenticate(&p, &r, &e),
        Err(StatusCertificateError::Response)
    ));
    let (p, r, mut e) = fixture();
    e.rejected = true;
    assert!(matches!(
        authenticate(&p, &r, &e),
        Err(StatusCertificateError::Outcome)
    ));
}

#[test]
fn original_observation_is_strict_and_time_arithmetic_never_wraps() {
    let (p, mut r, e) = fixture();
    let original = r.observation.clone();
    for bytes in [
        vec![],
        original[..original.len() - 1].to_vec(),
        [original.as_slice(), &[0]].concat(),
        vec![0; original.len()],
    ] {
        r.observation = bytes;
        assert!(authenticate(&p, &r, &e).is_err());
    }
    let mut time = times();
    time.certificate.reference_ns = 101;
    r.observation = status_certificate_observation(time).unwrap();
    assert!(matches!(
        authenticate(&p, &r, &e),
        Err(StatusCertificateError::Certificate(
            historical::Error::TimeOutsideWindow
        ))
    ));
    for window in [
        TimeWindow {
            reference_ns: 0,
            max_age_ns: 1,
            max_future_skew_ns: 0,
        },
        TimeWindow {
            reference_ns: u64::MAX,
            max_age_ns: 0,
            max_future_skew_ns: 1,
        },
    ] {
        for policy in [
            TimePolicy {
                certificate: window,
                ..times()
            },
            TimePolicy {
                delegation: window,
                ..times()
            },
        ] {
            assert!(matches!(
                status_certificate_observation(policy),
                Err(StatusCertificateError::Observation)
            ));
        }
    }
    r.observation = original;
    r.observation[39..47].copy_from_slice(&101_u64.to_le_bytes());
    assert!(matches!(
        authenticate(&p, &r, &e),
        Err(StatusCertificateError::Observation)
    ));
}

#[test]
fn delegated_authority_uses_plan_canister_and_separate_retained_time() {
    let (p, mut r, mut e) = fixture();
    e.proof = delegated(p.request().target.canister.as_slice(), &r, &e);
    authenticate(&p, &r, &e).unwrap();
    let mut time = times();
    time.delegation = time.certificate;
    r.observation = status_certificate_observation(time).unwrap();
    assert!(matches!(
        authenticate(&p, &r, &e),
        Err(StatusCertificateError::Certificate(
            historical::Error::TimeOutsideWindow
        ))
    ));
    r.observation = status_certificate_observation(times()).unwrap();
    e.proof = delegated(&[1], &r, &e);
    assert!(matches!(
        authenticate(&p, &r, &e),
        Err(StatusCertificateError::Certificate(
            historical::Error::NotAuthorized
        ))
    ));
}

#[test]
fn original_complete_frame_caps_apply_before_certificate_decoding() {
    let (p, r, e) = fixture();
    let mut a = admission();
    a.request_bytes = r.encode(4096).unwrap().len() as u32;
    a.result_bytes = e.encode(4096).unwrap().len() as u32;
    authenticate_status_evidence(&p, &a, &r, &e, &der(1)).unwrap();
    a.request_bytes -= 1;
    assert!(matches!(
        authenticate_status_evidence(&p, &a, &r, &e, &der(1)),
        Err(StatusCertificateError::Admission(_))
    ));
    a.request_bytes += 1;
    a.result_bytes -= 1;
    assert!(matches!(
        authenticate_status_evidence(&p, &a, &r, &e, &der(1)),
        Err(StatusCertificateError::Admission(_))
    ));
    let mut request = p.request().clone();
    request.limits.evidence_bytes = 8 * 1024 * 1024;
    let p = CapturePolicy::new(request).unwrap();
    let mut a = admission();
    a.result_bytes = 2 * 1024 * 1024;
    let oversized = StatusEvidence {
        proof: vec![0; 1024 * 1024 + 1],
        ..e
    };
    assert!(matches!(
        authenticate_status_evidence(&p, &a, &r, &oversized, &der(1)),
        Err(StatusCertificateError::Certificate(
            historical::Error::Decode(_)
        ))
    ));
}

#[test]
fn certificate_combined_node_limit_is_enforced_by_selected_platform_boundary() {
    let (p, r, mut e) = fixture();
    let mut level: Vec<_> = (0_u32..4096)
        .map(|i| labeled(i.to_be_bytes().to_vec(), leaf(vec![0])))
        .collect();
    while level.len() > 1 {
        let mut next = Vec::new();
        let mut nodes = level.into_iter();
        while let Some(left) = nodes.next() {
            next.push(fork(left, nodes.next().unwrap()));
        }
        level = next;
    }
    let tree = fork(
        labeled(b"other", level.pop().unwrap()),
        reply_tree(&r.request_id, &e.response, b"replied"),
    );
    e.proof = signed(tree, 1, None);
    let mut a = admission();
    a.result_bytes = 128 * 1024;
    assert!(matches!(
        authenticate_status_evidence(&p, &a, &r, &e, &der(1)),
        Err(StatusCertificateError::Certificate(
            historical::Error::TreeLimit
        ))
    ));
}

mod rejection_tests;
