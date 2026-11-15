//! Historical certificate authentication only; no restore dispatch or settlement.
use super::fixtures::{admission, declaration, prepared};
use crate::store::{
    RestorePlan, RestoreStatusEvidenceError, StatusCertificateError, StatusEvidence,
    capture::test_fixtures::{
        certificates::{delegated, der, reply_tree, signed, times},
        completion::{complete, settled},
        http::runtime,
        signatures::Scheme,
        source::maximum,
    },
    status_certificate_observation,
};
use ic_platform::{
    Principal,
    certification::{cbor, historical},
};
use std::fs;

#[test]
fn original_restore_certificates_authenticate_without_effects_and_refuse_changed_bindings() {
    for scheme in [Scheme::Ed25519, Scheme::Secp256k1] {
        let (mut f, server, mut provider) = settled();
        let receipt = complete(&mut f, &mut provider).unwrap();
        let ledger = f.ledger();
        let before = fs::read(&ledger).unwrap();
        let source = runtime()
            .block_on(f.store.open_capture_source(
                &mut f.op,
                &receipt,
                maximum(),
                &f.t,
                &mut provider,
            ))
            .unwrap();
        let mut request = declaration(&source);
        request.sender = Principal::self_authenticating(scheme.key());
        let plan = RestorePlan::new(&source, request).unwrap();
        let mut r = prepared(&plan, scheme);
        r.observation = status_certificate_observation(times()).unwrap();
        // Certificate validity and application status policy are separate boundaries.
        let mut e = StatusEvidence {
            rejected: false,
            response: vec![7],
            proof: vec![],
        };
        e.proof = signed(reply_tree(&r.request_id, &e.response, b"replied"), 1, None);
        let original_request = r.encode(4096).unwrap();
        let original_evidence = e.encode(4096).unwrap();
        let original_plan = plan.encoded().to_vec();
        let authenticate =
            |r: &_, e: &_, root: &[u8]| plan.authenticate_status_evidence(&admission(), r, e, root);
        authenticate(&r, &e, &der(1)).unwrap();
        assert!(plan.validate_status_reply(&e.response).is_err());
        assert!(matches!(
            authenticate(&r, &e, &der(2)),
            Err(RestoreStatusEvidenceError::Certificate(
                StatusCertificateError::TrustRoot
            ))
        ));
        for field in [
            "response",
            "outcome",
            "signature",
            "trailing",
            "request_id",
            "observation",
            "time",
            "pending",
        ] {
            let mut changed_r = crate::store::PreparedStatus::decode(&original_request).unwrap();
            let mut changed_e = StatusEvidence::decode(&original_evidence).unwrap();
            match field {
                "response" => changed_e.response[0] ^= 1,
                "outcome" => changed_e.rejected = true,
                "signature" => {
                    let mut cert = cbor::decode_certificate(&changed_e.proof, 4096).unwrap();
                    cert.signature[0] ^= 1;
                    changed_e.proof = cbor::encode_certificate(&cert, 4096).unwrap();
                }
                "trailing" => changed_e.proof.push(0),
                "request_id" => changed_r.request_id[0] ^= 1,
                "observation" => changed_r.observation.push(0),
                "time" => {
                    let mut t = times();
                    t.certificate.reference_ns += 1;
                    changed_r.observation = status_certificate_observation(t).unwrap();
                }
                _ => {
                    changed_e.proof = signed(
                        reply_tree(&r.request_id, &e.response, b"processing"),
                        1,
                        None,
                    )
                }
            }
            let error = authenticate(&changed_r, &changed_e, &der(1)).unwrap_err();
            match field {
                "response" => assert!(matches!(
                    error,
                    RestoreStatusEvidenceError::Certificate(StatusCertificateError::Response)
                )),
                "outcome" | "pending" => assert!(matches!(
                    error,
                    RestoreStatusEvidenceError::Certificate(StatusCertificateError::Outcome)
                )),
                "signature" => assert!(matches!(
                    error,
                    RestoreStatusEvidenceError::Certificate(StatusCertificateError::Certificate(
                        historical::Error::Signature
                    ))
                )),
                "trailing" => assert!(matches!(
                    error,
                    RestoreStatusEvidenceError::Certificate(StatusCertificateError::Certificate(
                        historical::Error::Decode(_)
                    ))
                )),
                "request_id" => assert!(matches!(error, RestoreStatusEvidenceError::Request(_))),
                "observation" => assert!(matches!(
                    error,
                    RestoreStatusEvidenceError::Certificate(StatusCertificateError::Observation)
                )),
                _ => assert!(matches!(
                    error,
                    RestoreStatusEvidenceError::Certificate(StatusCertificateError::Certificate(
                        historical::Error::TimeOutsideWindow
                    ))
                )),
            }
        }
        let mut a = admission();
        a.request_bytes = original_request.len() as u32;
        a.result_bytes = original_evidence.len() as u32;
        plan.authenticate_status_evidence(&a, &r, &e, &der(1))
            .unwrap();
        a.result_bytes -= 1;
        assert!(matches!(
            plan.authenticate_status_evidence(&a, &r, &e, &der(1)),
            Err(RestoreStatusEvidenceError::Certificate(
                StatusCertificateError::Admission(_)
            ))
        ));
        a.result_bytes += 1;
        a.request_bytes -= 1;
        assert!(matches!(
            plan.authenticate_status_evidence(&a, &r, &e, &der(1)),
            Err(RestoreStatusEvidenceError::Request(_))
        ));
        assert_eq!(r.encode(4096).unwrap(), original_request);
        assert_eq!(e.encode(4096).unwrap(), original_evidence);
        assert_eq!(plan.encoded(), original_plan);
        assert_eq!(fs::read(&ledger).unwrap(), before);
        assert_eq!(server.count(), 1);
        assert!(
            !f.root
                .join("operations")
                .join(&plan.request().operation_id)
                .exists()
        );
    }
}

#[test]
fn restore_delegation_binds_original_target_and_separate_historical_window() {
    let (mut f, _, mut provider) = settled();
    let receipt = complete(&mut f, &mut provider).unwrap();
    let source = runtime()
        .block_on(
            f.store
                .open_capture_source(&mut f.op, &receipt, maximum(), &f.t, &mut provider),
        )
        .unwrap();
    let mut request = declaration(&source);
    request.sender = Principal::self_authenticating(Scheme::Ed25519.key());
    let plan = RestorePlan::new(&source, request).unwrap();
    let mut r = prepared(&plan, Scheme::Ed25519);
    r.observation = status_certificate_observation(times()).unwrap();
    let mut e = StatusEvidence {
        rejected: false,
        response: vec![7],
        proof: vec![],
    };
    e.proof = delegated(plan.request().target.canister.as_slice(), &r, &e);
    plan.authenticate_status_evidence(&admission(), &r, &e, &der(1))
        .unwrap();
    let mut t = times();
    t.delegation = t.certificate;
    r.observation = status_certificate_observation(t).unwrap();
    assert!(matches!(
        plan.authenticate_status_evidence(&admission(), &r, &e, &der(1)),
        Err(RestoreStatusEvidenceError::Certificate(
            StatusCertificateError::Certificate(historical::Error::TimeOutsideWindow)
        ))
    ));
    r.observation = status_certificate_observation(times()).unwrap();
    e.proof = delegated(&[1], &r, &e);
    assert!(matches!(
        plan.authenticate_status_evidence(&admission(), &r, &e, &der(1)),
        Err(RestoreStatusEvidenceError::Certificate(
            StatusCertificateError::Certificate(historical::Error::NotAuthorized)
        ))
    ));
}
