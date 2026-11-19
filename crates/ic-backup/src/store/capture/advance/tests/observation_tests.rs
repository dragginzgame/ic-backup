//! Real signatures over synthetic certificates; no remote observation is performed.

use super::{
    fixtures::{
        certificates::{delegated, der, signed},
        signatures::{Scheme, change, fixture as signed_request},
    },
    *,
};
use ic_platform::certificate::{
    TimePolicy, TimeWindow,
    construction::{HashTree, empty, fork, labeled, leaf, pruned},
};

fn window(reference_ns: u64) -> TimeWindow {
    TimeWindow {
        reference_ns,
        max_age_ns: 0,
        max_future_skew_ns: 0,
    }
}

fn policy() -> CaptureObservationPolicy {
    CaptureObservationPolicy {
        time: TimePolicy {
            certificate: window(200),
            delegation: window(10),
        },
        evidence_bytes: 4096,
    }
}

fn original(
    call: CaptureCall,
    scheme: Scheme,
) -> (CapturePolicy, StatusAdmissionRecord, PreparedStatus) {
    let (p, mut r) = signed_request(scheme);
    let mut fields = p.request().clone();
    fields.trust_root = BackupDigest::compute(&der(1));
    let p = CapturePolicy::new(fields).unwrap();
    change(&mut r, |envelope| {
        if let EnvelopeContent::Call {
            method_name,
            arg,
            ingress_expiry,
            ..
        } = &mut envelope.content
        {
            *method_name = call.method_name().into();
            *arg = call.arguments(&p).unwrap();
            *ingress_expiry = 150;
        }
        envelope.sender_sig = Some(scheme.sign(&envelope.content.signable().unwrap()));
    });
    r.expires_ns = 150;
    r.observation = status_certificate_observation(TimePolicy {
        certificate: window(100),
        delegation: window(10),
    })
    .unwrap();
    let mut a = admission();
    a.prepare_deadline_ns = 110;
    a.submit_deadline_ns = 150;
    (p, a, r)
}

fn certificate(id: &[u8; 32], branch: HashTree) -> Vec<u8> {
    signed(
        fork(
            labeled(b"request_status", labeled(id.to_vec(), branch)),
            labeled(b"time", leaf(vec![0xc8, 1])),
        ),
        1,
        None,
    )
}

fn replied(id: &[u8; 32], reply: &[u8]) -> Vec<u8> {
    certificate(
        id,
        fork(
            labeled(b"reply", leaf(reply.to_vec())),
            labeled(b"status", leaf(b"replied".to_vec())),
        ),
    )
}

fn observe(
    p: &CapturePolicy,
    a: &StatusAdmissionRecord,
    r: &PreparedStatus,
    proof: &[u8],
    limits: CaptureObservationPolicy,
) -> Result<CaptureObservation, CaptureObservationError> {
    observe_capture_certificate(p, a, r, CaptureCall::Status, proof, &der(1), limits)
}

#[test]
fn original_calls_and_direct_keys_use_separate_observation_time_without_rewriting_authority() {
    for call in [
        CaptureCall::Status,
        CaptureCall::ListSnapshots,
        CaptureCall::TakeSnapshot,
    ] {
        for scheme in [Scheme::Ed25519, Scheme::Secp256k1] {
            let (p, a, r) = original(call, scheme);
            let before = r.encode(4096).unwrap();
            let proof = replied(&r.request_id, b"opaque\0reply");
            let CaptureObservation::Replied(bytes) =
                observe_capture_certificate(&p, &a, &r, call, &proof, &der(1), policy()).unwrap()
            else {
                panic!("original reply");
            };
            assert_eq!(bytes, b"opaque\0reply");
            assert_eq!(r.encode(4096).unwrap(), before);
            assert_eq!(r.expires_ns, 150);
            assert!(matches!(
                certificate::tests::call_certificate_evidence(&p, &a, &r, &proof, &der(1), call),
                Err(StatusCertificateError::Certificate(
                    ic_platform::certificate::VerifyError::TimeOutsideWindow
                ))
            ));
        }
    }
}

#[test]
fn pending_done_absent_and_pruned_are_distinct_and_never_terminal_receipts() {
    let (p, a, r) = original(CaptureCall::Status, Scheme::Ed25519);
    for (kind, branch) in [
        (0, labeled(b"status", leaf(b"received".to_vec()))),
        (1, labeled(b"status", leaf(b"processing".to_vec()))),
        (2, labeled(b"status", leaf(b"done".to_vec()))),
        (3, empty()),
        (3, labeled(b"status", empty())),
        (4, pruned([0; 32])),
    ] {
        let result = observe(&p, &a, &r, &certificate(&r.request_id, branch), policy()).unwrap();
        assert!(matches!(
            (kind, result),
            (0, CaptureObservation::Received)
                | (1, CaptureObservation::Processing)
                | (2, CaptureObservation::Done)
                | (3, CaptureObservation::Absent)
                | (4, CaptureObservation::Unknown)
        ));
    }
    // A terminal result under another ID cannot settle the original call.
    let mut other = r.request_id;
    other[0] ^= 1;
    assert!(matches!(
        observe(&p, &a, &r, &replied(&other, b"other"), policy()).unwrap(),
        CaptureObservation::Absent
    ));
    for branch in [
        labeled(b"status", leaf(b"unknown".to_vec())),
        labeled(b"status", labeled(b"unexpected", leaf(vec![1]))),
        labeled(b"status", leaf(b"replied".to_vec())),
        fork(
            labeled(b"reply", leaf(vec![])),
            labeled(b"status", leaf(b"replied".to_vec())),
        ),
    ] {
        assert!(observe(&p, &a, &r, &certificate(&r.request_id, branch), policy()).is_err());
    }
}

#[test]
fn original_request_signature_sender_method_and_content_remain_mandatory() {
    let (p, a, r) = original(CaptureCall::Status, Scheme::Ed25519);
    let proof = replied(&r.request_id, b"reply");
    for case in 0..5 {
        let mut changed = PreparedStatus::decode(&r.encode(4096).unwrap()).unwrap();
        match case {
            0 => change(&mut changed, |e| e.sender_sig.as_mut().unwrap()[0] ^= 1),
            1 => changed.request_id[0] ^= 1,
            2 => changed.expires_ns += 1,
            3 => change(&mut changed, |e| {
                if let EnvelopeContent::Call { arg, .. } = &mut e.content {
                    arg.push(0);
                }
            }),
            _ => change(&mut changed, |e| {
                if let EnvelopeContent::Call { sender, .. } = &mut e.content {
                    *sender = [].as_slice().try_into().unwrap();
                }
            }),
        }
        assert!(matches!(
            observe(&p, &a, &changed, &proof, policy()),
            Err(CaptureObservationError::Request(_))
        ));
    }
    assert!(matches!(
        observe_capture_certificate(
            &p,
            &a,
            &r,
            CaptureCall::TakeSnapshot,
            &proof,
            &der(1),
            policy()
        ),
        Err(CaptureObservationError::Request(_))
    ));
    let mut fields = p.request().clone();
    fields.target.canister = ic_platform_types::Principal::from_slice(&[99]);
    let other = CapturePolicy::new(fields).unwrap();
    assert!(matches!(
        observe(&other, &a, &r, &proof, policy()),
        Err(CaptureObservationError::Request(_))
    ));
}

#[test]
fn roots_signatures_original_policy_and_time_windows_cannot_be_bypassed() {
    let (p, a, mut r) = original(CaptureCall::Status, Scheme::Ed25519);
    let proof = replied(&r.request_id, b"reply");
    assert!(matches!(
        observe_capture_certificate(&p, &a, &r, CaptureCall::Status, &proof, &der(2), policy()),
        Err(CaptureObservationError::Certificate(
            StatusCertificateError::TrustRoot
        ))
    ));
    let mut changed = proof.clone();
    *changed.last_mut().unwrap() ^= 1;
    assert!(observe(&p, &a, &r, &changed, policy()).is_err());
    for time in [
        TimePolicy {
            certificate: window(201),
            ..policy().time
        },
        TimePolicy {
            certificate: TimeWindow {
                reference_ns: 0,
                max_age_ns: 1,
                max_future_skew_ns: 0,
            },
            ..policy().time
        },
        TimePolicy {
            delegation: TimeWindow {
                reference_ns: u64::MAX,
                max_age_ns: 0,
                max_future_skew_ns: 1,
            },
            ..policy().time
        },
    ] {
        assert!(
            observe(
                &p,
                &a,
                &r,
                &proof,
                CaptureObservationPolicy { time, ..policy() }
            )
            .is_err()
        );
    }
    r.observation[0] ^= 1;
    assert!(matches!(
        observe(&p, &a, &r, &proof, policy()),
        Err(CaptureObservationError::Certificate(
            StatusCertificateError::Observation
        ))
    ));
}

#[test]
fn exact_combined_evidence_bound_precedes_projection_and_allocation() {
    let (p, a, r) = original(CaptureCall::Status, Scheme::Ed25519);
    let proof = replied(&r.request_id, b"reply");
    let exact = (proof.len() + 5 + b"reply".len()) as u32;
    assert!(matches!(
        observe(
            &p,
            &a,
            &r,
            &proof,
            CaptureObservationPolicy {
                evidence_bytes: exact,
                ..policy()
            }
        )
        .unwrap(),
        CaptureObservation::Replied(_)
    ));
    for bound in [
        0,
        4,
        proof.len() as u32,
        exact - 1,
        a.result_bytes + 1,
        u32::MAX,
    ] {
        assert!(
            observe(
                &p,
                &a,
                &r,
                &proof,
                CaptureObservationPolicy {
                    evidence_bytes: bound,
                    ..policy()
                }
            )
            .is_err()
        );
    }
    let mut trailing = proof;
    trailing.push(0);
    assert!(observe(&p, &a, &r, &trailing, policy()).is_err());
}

fn rejected(id: &[u8; 32], code: &[u8], message: &[u8], error: Option<HashTree>) -> Vec<u8> {
    let diagnostics = fork(
        labeled(b"reject_code", leaf(code.to_vec())),
        fork(
            labeled(b"reject_message", leaf(message.to_vec())),
            labeled(b"status", leaf(b"rejected".to_vec())),
        ),
    );
    certificate(
        id,
        error.map_or_else(
            || diagnostics.clone(),
            |error| fork(error, diagnostics.clone()),
        ),
    )
}

#[test]
fn certified_rejections_reuse_exact_diagnostics_and_refuse_unknown_optional_fields() {
    let (p, a, r) = original(CaptureCall::Status, Scheme::Ed25519);
    for present in [false, true] {
        let proof = rejected(
            &r.request_id,
            &[5],
            b"refused",
            present.then(|| labeled(b"error_code", leaf(vec![]))),
        );
        let CaptureObservation::Rejected(bytes) = observe(&p, &a, &r, &proof, policy()).unwrap()
        else {
            panic!("rejection");
        };
        let mut expected = b"ic-backup/status-rejection/1\n".to_vec();
        expected.extend_from_slice(&[5, 7, 0, 0, 0]);
        expected.extend_from_slice(b"refused");
        expected.push(u8::from(present));
        if present {
            expected.extend_from_slice(&[0; 4]);
        }
        assert_eq!(bytes, expected);
    }
    for proof in [
        rejected(&r.request_id, &[6], b"refused", None),
        rejected(&r.request_id, &[0x85, 0], b"refused", None),
        rejected(&r.request_id, &[5], &[255], None),
        rejected(&r.request_id, &[5], b"refused", Some(pruned([0; 32]))),
    ] {
        assert!(observe(&p, &a, &r, &proof, policy()).is_err());
    }
}

#[test]
fn delegation_range_and_its_separate_time_window_remain_mandatory() {
    let (p, a, r) = original(CaptureCall::Status, Scheme::Ed25519);
    let evidence = StatusEvidence {
        rejected: false,
        response: b"reply".to_vec(),
        proof: vec![],
    };
    let limits = CaptureObservationPolicy {
        time: TimePolicy {
            certificate: window(100),
            delegation: window(10),
        },
        ..policy()
    };
    let proof = delegated(p.request().target.canister.as_slice(), &r, &evidence);
    assert!(matches!(
        observe(&p, &a, &r, &proof, limits).unwrap(),
        CaptureObservation::Replied(_)
    ));
    let foreign = delegated(&[99], &r, &evidence);
    assert!(matches!(
        observe(&p, &a, &r, &foreign, limits),
        Err(CaptureObservationError::Certificate(
            StatusCertificateError::Certificate(
                ic_platform::certificate::VerifyError::NotAuthorized
            )
        ))
    ));
    let stale = CaptureObservationPolicy {
        time: TimePolicy {
            delegation: window(11),
            ..limits.time
        },
        ..limits
    };
    assert!(matches!(
        observe(&p, &a, &r, &proof, stale),
        Err(CaptureObservationError::Certificate(
            StatusCertificateError::Certificate(
                ic_platform::certificate::VerifyError::TimeOutsideWindow
            )
        ))
    ));
}
