//! Certified rejection projection and durable terminal replay, without IC effects.
use super::*;
use ic_platform::certificate::construction::HashTree;
fn rejected_tree(
    id: &[u8; 32],
    code: Option<&[u8]>,
    message: Option<&[u8]>,
    error: Option<HashTree>,
) -> HashTree {
    let mut fields = Vec::new();
    if let Some(error) = error {
        fields.push(error);
    }
    if let Some(code) = code {
        fields.push(labeled(b"reject_code", leaf(code.to_vec())));
    }
    if let Some(message) = message {
        fields.push(labeled(b"reject_message", leaf(message.to_vec())));
    }
    fields.push(labeled(b"status", leaf(b"rejected".to_vec())));
    fork(
        labeled(
            b"request_status",
            labeled(id.to_vec(), fields.into_iter().reduce(fork).unwrap()),
        ),
        labeled(b"time", leaf(vec![100])),
    )
}

fn rejection_proof(id: &[u8; 32], code: &[u8], message: &[u8], error: Option<&[u8]>) -> Vec<u8> {
    signed(
        rejected_tree(
            id,
            Some(code),
            Some(message),
            error.map(|bytes| labeled(b"error_code", leaf(bytes.to_vec()))),
        ),
        1,
        None,
    )
}
fn build(
    p: &CapturePolicy,
    r: &PreparedStatus,
    proof: &[u8],
) -> Result<StatusEvidence, StatusCertificateError> {
    status_certificate_evidence(p, &admission(), r, proof, &der(1))
}

#[test]
fn builder_preserves_original_successful_reply_and_certificate() {
    let (p, r, original) = fixture();
    let evidence = build(&p, &r, &original.proof).unwrap();
    assert!(!evidence.rejected);
    assert_eq!(evidence.response, original.response);
    assert_eq!(evidence.proof, original.proof);
    authenticate(&p, &r, &evidence).unwrap();
}

#[test]
fn certified_rejection_uses_frozen_diagnostic_frame_and_all_direct_ingress_codes() {
    let (p, r, _) = fixture();
    for code in 1..=5 {
        let proof = rejection_proof(&r.request_id, &[code], b"bad", Some(b"IC0503"));
        let evidence = build(&p, &r, &proof).unwrap();
        assert!(evidence.rejected);
        assert_eq!(evidence.proof, proof);
        let mut golden =
            b"ic-backup/status-rejection/1\n\x05\x03\0\0\0bad\x01\x06\0\0\0IC0503".to_vec();
        golden[29] = code;
        assert_eq!(evidence.response, golden);
        authenticate(&p, &r, &evidence).unwrap();
    }
}

#[test]
fn absent_and_present_empty_optional_error_are_distinct_and_unicode_is_exact() {
    let (p, r, _) = fixture();
    let absent = build(&p, &r, &rejection_proof(&r.request_id, &[4], b"", None)).unwrap();
    let present = build(
        &p,
        &r,
        &rejection_proof(&r.request_id, &[4], b"", Some(b"")),
    )
    .unwrap();
    assert_ne!(absent.response, present.response);
    assert_eq!(
        absent.response,
        b"ic-backup/status-rejection/1\n\x04\0\0\0\0\0"
    );
    assert_eq!(
        present.response,
        b"ic-backup/status-rejection/1\n\x04\0\0\0\0\x01\0\0\0\0"
    );
    for message in ["é\0\n", "e\u{301}\0\n"] {
        let proof = rejection_proof(
            &r.request_id,
            &[5],
            message.as_bytes(),
            Some("错误".as_bytes()),
        );
        let evidence = build(&p, &r, &proof).unwrap();
        assert!(
            evidence
                .response
                .windows(message.len())
                .any(|part| part == message.as_bytes())
        );
        authenticate(&p, &r, &evidence).unwrap();
    }
}

#[test]
fn missing_or_pruned_diagnostics_refuse_instead_of_becoming_absent() {
    let (p, r, _) = fixture();
    for tree in [
        rejected_tree(&r.request_id, None, Some(b"message"), None),
        rejected_tree(&r.request_id, Some(&[5]), None, None),
        rejected_tree(
            &r.request_id,
            Some(&[5]),
            Some(b"message"),
            Some(pruned([7; 32])),
        ),
        rejected_tree(
            &r.request_id,
            Some(&[5]),
            Some(b"message"),
            Some(labeled(b"error_code", labeled(b"child", leaf(vec![0])))),
        ),
    ] {
        assert!(matches!(
            build(&p, &r, &signed(tree, 1, None)),
            Err(StatusCertificateError::Rejection)
        ));
    }
}

#[test]
fn unsupported_codes_and_invalid_text_refuse_without_lossy_conversion() {
    let (p, r, _) = fixture();
    for code in [
        b"".as_slice(),
        &[0],
        &[6],
        &[7],
        &[0x84, 0],
        &[0x84],
        &[4, 0],
    ] {
        assert!(matches!(
            build(
                &p,
                &r,
                &rejection_proof(&r.request_id, code, b"message", None)
            ),
            Err(StatusCertificateError::Rejection)
        ));
    }
    for (message, error) in [
        ([0xff].as_slice(), None),
        (b"message".as_slice(), Some([0xff].as_slice())),
    ] {
        assert!(matches!(
            build(
                &p,
                &r,
                &rejection_proof(&r.request_id, &[5], message, error)
            ),
            Err(StatusCertificateError::Rejection)
        ));
    }
}

#[test]
fn altered_diagnostics_outcome_and_original_request_are_refused() {
    let (p, r, _) = fixture();
    let proof = rejection_proof(&r.request_id, &[5], b"private diagnostic", Some(b"IC0503"));
    let mut evidence = build(&p, &r, &proof).unwrap();
    for offset in [0, 28, 29, 34, evidence.response.len() - 1] {
        evidence.response[offset] ^= 1;
        let error = authenticate(&p, &r, &evidence).unwrap_err();
        assert!(matches!(error, StatusCertificateError::Response));
        assert!(!format!("{error:?}").contains("private diagnostic"));
        evidence.response[offset] ^= 1;
    }
    evidence.rejected = false;
    assert!(matches!(
        authenticate(&p, &r, &evidence),
        Err(StatusCertificateError::Outcome)
    ));
    evidence.rejected = true;
    evidence.proof = rejection_proof(
        &r.request_id,
        &[5],
        b"different diagnostic",
        Some(b"IC0503"),
    );
    assert!(matches!(
        authenticate(&p, &r, &evidence),
        Err(StatusCertificateError::Response)
    ));
    evidence.proof = rejection_proof(&[42; 32], &[5], b"private diagnostic", Some(b"IC0503"));
    assert!(matches!(
        authenticate(&p, &r, &evidence),
        Err(StatusCertificateError::Outcome)
    ));
}

#[test]
fn builder_checks_complete_result_size_and_rejects_http_errors_without_certificates() {
    let (p, r, _) = fixture();
    let proof = rejection_proof(&r.request_id, &[5], b"message", None);
    let evidence = build(&p, &r, &proof).unwrap();
    let mut a = admission();
    a.result_bytes = evidence.encode(4096).unwrap().len() as u32;
    status_certificate_evidence(&p, &a, &r, &proof, &der(1)).unwrap();
    a.result_bytes -= 1;
    assert!(matches!(
        status_certificate_evidence(&p, &a, &r, &proof, &der(1)),
        Err(StatusCertificateError::Admission(_))
    ));
    a.result_bytes = 7;
    assert!(matches!(
        status_certificate_evidence(&p, &a, &r, &proof, &der(1)),
        Err(StatusCertificateError::Admission(_))
    ));
    let unsigned =
        br#"{"status":"non_replicated_rejection","reject_code":4,"reject_message":"denied"}"#;
    assert!(matches!(
        build(&p, &r, unsigned),
        Err(StatusCertificateError::Decode(_))
    ));
}
