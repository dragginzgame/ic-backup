//! Real BLS signatures over synthetic certificates. No network or IC effects.
use super::*;
use ic_platform::{
    bls::PrivateKey,
    canister_sig_creation::IC_ROOT_PK_DER_PREFIX,
    certification::{
        Certificate, Delegation, HashTree, cbor, fork,
        historical::{self, TimePolicy, TimeWindow},
        labeled, leaf, pruned,
    },
};

fn key(value: u8) -> PrivateKey {
    let mut bytes = [0; 32];
    bytes[31] = value;
    PrivateKey::deserialize(&bytes).unwrap()
}
fn der(value: u8) -> Vec<u8> {
    [
        IC_ROOT_PK_DER_PREFIX.as_slice(),
        &key(value).public_key().serialize(),
    ]
    .concat()
}
fn times() -> TimePolicy {
    TimePolicy {
        certificate: TimeWindow {
            reference_ns: 100,
            max_age_ns: 0,
            max_future_skew_ns: 0,
        },
        delegation: TimeWindow {
            reference_ns: 10,
            max_age_ns: 0,
            max_future_skew_ns: 0,
        },
    }
}
fn signed(tree: HashTree, signer: u8, delegation: Option<Delegation>) -> Vec<u8> {
    let message = [b"\x0dic-state-root".as_slice(), &tree.digest()].concat();
    cbor::encode_certificate(
        &Certificate {
            tree,
            signature: key(signer).sign(&message).serialize().to_vec(),
            delegation,
        },
        2 * 1024 * 1024,
    )
    .unwrap()
}
fn reply_tree(request: &[u8; 32], response: &[u8], outcome: &[u8]) -> HashTree {
    fork(
        labeled(
            b"request_status",
            labeled(
                request.to_vec(),
                fork(
                    labeled(b"reply", leaf(response.to_vec())),
                    labeled(b"status", leaf(outcome.to_vec())),
                ),
            ),
        ),
        labeled(b"time", leaf(vec![100])),
    )
}
fn fixture() -> (CapturePlan, PreparedStatus, StatusEvidence) {
    let mut request = plan().request().clone();
    request.trust_root = Sha256Digest::compute(&der(1));
    let plan = CapturePlan::new(request).unwrap();
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
    p: &CapturePlan,
    r: &PreparedStatus,
    e: &StatusEvidence,
) -> Result<(), StatusCertificateError> {
    authenticate_status_evidence(p, &admission(), r, e, &der(1))
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
            &CapturePlan::new(request).unwrap(),
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

fn delegated(canister: &[u8], r: &PreparedStatus, e: &StatusEvidence) -> Vec<u8> {
    assert!(canister.len() < 24);
    // CBOR [[h'canister', h'canister']]: an exact, single-canister certified range.
    let mut range = vec![0x81, 0x82, 0x40 + canister.len() as u8];
    range.extend_from_slice(canister);
    range.push(0x40 + canister.len() as u8);
    range.extend_from_slice(canister);
    let subnet = vec![9, 1];
    let authority = signed(
        fork(
            labeled(
                b"subnet",
                labeled(
                    subnet.clone(),
                    fork(
                        labeled(b"canister_ranges", leaf(range)),
                        labeled(b"public_key", leaf(der(2))),
                    ),
                ),
            ),
            labeled(b"time", leaf(vec![10])),
        ),
        1,
        None,
    );
    signed(
        reply_tree(&r.request_id, &e.response, b"replied"),
        2,
        Some(Delegation {
            subnet_id: subnet,
            certificate: authority,
        }),
    )
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
    let p = CapturePlan::new(request).unwrap();
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

struct CertificateProvider {
    request: Option<PreparedStatus>,
    evidence: Option<StatusEvidence>,
    root: Vec<u8>,
    authentications: Cell<u32>,
}
impl StatusProvider for CertificateProvider {
    type Error = StatusCertificateError;
    fn admit(
        &self,
        _: &CapturePlan,
        _: &StatusAdmissionRecord,
        _: &[u8],
        _: Option<&PreparedStatus>,
    ) -> Result<(), Self::Error> {
        assert!(
            self.request.is_some() || self.evidence.is_some(),
            "cold replay must not admit current authority"
        );
        Ok(())
    }
    async fn prepare(
        &mut self,
        _: &CapturePlan,
        _: &StatusAdmissionRecord,
        _: &[u8],
    ) -> Result<PreparedStatus, Self::Error> {
        Ok(self.request.take().expect("no repeated signing"))
    }
    async fn submit_once(
        &mut self,
        _: &CapturePlan,
        _: &StatusAdmissionRecord,
        _: &PreparedStatus,
    ) -> Result<StatusEvidence, Self::Error> {
        Ok(self.evidence.take().expect("no repeated submission"))
    }
    fn authenticate(
        &self,
        plan: &CapturePlan,
        admission: &StatusAdmissionRecord,
        prepared: &PreparedStatus,
        evidence: &StatusEvidence,
    ) -> Result<(), Self::Error> {
        self.authentications.set(self.authentications.get() + 1);
        authenticate_status_evidence(plan, admission, prepared, evidence, &self.root)
    }
}

#[test]
fn cold_settled_replay_reauthenticates_original_proof_without_effects_or_rewrites() {
    let (p, r, e) = fixture();
    let root = root("status-certificate-replay");
    let store = BackupStore::initialize(&root).unwrap();
    let mut operation = store
        .create_capture(CapturePlan::new(p.request().clone()).unwrap(), p.digest())
        .unwrap();
    let mut provider = CertificateProvider {
        request: Some(r),
        evidence: Some(e),
        root: der(1),
        authentications: Cell::new(0),
    };
    assert_eq!(
        ready(operation.advance_status(&admission(), &mut provider)).unwrap(),
        StatusProgress::Verified
    );
    let before = operation.inspect_status().unwrap();
    assert_eq!(before.signing_requests_reserved, 1);
    assert_eq!(before.network_requests_reserved, 2);
    let directory = root.join("operations").join(&p.request().operation_id);
    let files = ["ledger.bin", "status-request.bin", "status-result.bin"];
    let original: Vec<_> = files
        .iter()
        .map(|f| fs::read(directory.join(f)).unwrap())
        .collect();
    drop(operation);
    let mut operation = store
        .open_capture(&p.request().operation_id, p.digest())
        .unwrap();
    assert_eq!(
        ready(operation.advance_status(&admission(), &mut provider)).unwrap(),
        StatusProgress::Verified
    );
    assert_eq!(provider.authentications.get(), 3);
    provider.root = der(2);
    assert!(matches!(
        ready(operation.advance_status(&admission(), &mut provider)),
        Err(StatusAdvanceError::Provider(
            StatusCertificateError::TrustRoot
        ))
    ));
    assert_eq!(operation.inspect_status().unwrap(), before);
    for (file, bytes) in files.iter().zip(original) {
        assert_eq!(fs::read(directory.join(file)).unwrap(), bytes);
    }
}

#[test]
fn unauthenticated_result_leaves_original_submission_spent_and_cold_replay_pending() {
    let (p, r, mut e) = fixture();
    e.response[0] ^= 1;
    let root = root("status-certificate-refusal");
    let store = BackupStore::initialize(&root).unwrap();
    let mut operation = store
        .create_capture(CapturePlan::new(p.request().clone()).unwrap(), p.digest())
        .unwrap();
    let mut provider = CertificateProvider {
        request: Some(r),
        evidence: Some(e),
        root: der(1),
        authentications: Cell::new(0),
    };
    assert!(matches!(
        ready(operation.advance_status(&admission(), &mut provider)),
        Err(StatusAdvanceError::Provider(
            StatusCertificateError::Response
        ))
    ));
    drop(operation);
    let mut operation = store
        .open_capture(&p.request().operation_id, p.digest())
        .unwrap();
    assert_eq!(
        ready(operation.advance_status(&admission(), &mut provider)).unwrap(),
        StatusProgress::SubmissionPending
    );
    assert_eq!(provider.authentications.get(), 1);
    assert_eq!(
        operation
            .inspect_status()
            .unwrap()
            .network_requests_reserved,
        2
    );
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
