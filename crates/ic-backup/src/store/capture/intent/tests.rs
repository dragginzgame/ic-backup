//! Real signatures and synthetic certificate trees; these tests do not qualify IC effects.
use super::*;
use crate::store::capture::{
    procedure::{Error as ProcedureError, Next, Procedure},
    status::tests::status,
    test_fixtures::{
        certificates::{der, reply_tree, signed},
        signatures::{Scheme, change, fixture},
    },
};
use crate::store::operation::Step;
use crate::store::operation::intent::Intent;
use crate::store::{PreparedStatus, StatusEvidence};
use ic_platform::ingress::Content as EnvelopeContent;

pub(in crate::store) fn profile() -> ProfileRecord {
    ProfileRecord {
        manager: None,
        codec: None,
        transport: [1; 32],
        signer: [2; 32],
        guard: [3; 32],
        application: [4; 32],
        root_der: der(1),
        snapshot_capacity: 2,
        prepare_duration_ns: 10,
        submit_duration_ns: 20,
        certificate_max_age_ns: 0,
        delegation_max_age_ns: 90,
        clock_skew_ns: 0,
        signer_network: 0,
        request_bytes: 4096,
        result_bytes: 4096,
    }
}
fn input() -> (CapturePlan, PreparedStatus, StatusEvidence) {
    let (plan, _) = fixture(Scheme::Ed25519);
    let mut request = plan.request().clone();
    request.trust_root = BackupDigest::compute(&der(1));
    request.limits.evidence_bytes = 8 * 1024 * 1024;
    let intent = CapturePlan::new(request, profile()).unwrap();
    let (method, args) = Procedure::new(&intent).unwrap().request().unwrap();
    let response = crate::store::capture::test_fixtures::wire::encode(&status(intent.plan()), 4096);
    let (prepared, evidence) = signed_reply(&intent, 0, method, args, response);
    (intent, prepared, evidence)
}

pub(in crate::store) fn signed_reply(
    intent: &CapturePlan,
    ordinal: u8,
    method: &str,
    arguments: Vec<u8>,
    response: Vec<u8>,
) -> (PreparedStatus, StatusEvidence) {
    let (_, mut prepared) = fixture(Scheme::Ed25519);
    change(&mut prepared, |envelope| {
        if let EnvelopeContent::Call {
            ingress_expiry,
            method_name,
            arg,
            nonce,
            ..
        } = &mut envelope.content
        {
            *ingress_expiry = 120;
            *method_name = method.into();
            *arg = arguments;
            *nonce = Some(vec![ordinal]);
        }
        let message = [
            b"\x0aic-request".as_slice(),
            envelope.content.request_id().unwrap().as_slice(),
        ]
        .concat();
        envelope.sender_sig = Some(Scheme::Ed25519.sign(&message));
    });
    prepared.expires_ns = 120;
    prepared.observation = intent.view().observation(100).unwrap();
    let proof = signed(
        reply_tree(&prepared.request_id, &response, b"replied"),
        1,
        None,
    );
    (
        prepared,
        StatusEvidence {
            rejected: false,
            response,
            proof,
        },
    )
}

#[test]
fn replacement_approval_binds_profile_and_refuses_old_or_changed_encodings() {
    let (intent, _, _) = input();
    let reopened = CapturePlan::decode(intent.encoded()).unwrap();
    assert_eq!(intent.digest(), reopened.digest());
    assert_eq!(intent.encoded(), reopened.encoded());
    assert!(CapturePlan::decode(include_bytes!("../../tests/fixtures/capture-plan.json")).is_err());
    let mut changed = intent.encoded().to_vec();
    changed.push(b' ');
    assert!(CapturePlan::decode(&changed).is_err());
    let mut record: serde_json::Value = serde_json::from_slice(intent.encoded()).unwrap();
    record["extra"] = true.into();
    assert!(CapturePlan::decode(&serde_json::to_vec(&record).unwrap()).is_err());
    for field in [
        "transport",
        "signer",
        "guard",
        "application",
        "snapshot_capacity",
        "prepare_duration_ns",
        "submit_duration_ns",
        "certificate_max_age_ns",
        "delegation_max_age_ns",
        "clock_skew_ns",
        "signer_network",
        "request_bytes",
        "result_bytes",
    ] {
        let mut record: serde_json::Value = serde_json::to_value(intent.profile()).unwrap();
        if record[field].is_array() {
            record[field][0] = 42.into();
        } else {
            record[field] = (record[field].as_u64().unwrap() + 1).into();
        }
        let changed = CapturePlan::new(
            intent.plan().request().clone(),
            serde_json::from_value(record).unwrap(),
        )
        .unwrap();
        assert_ne!(intent.digest(), changed.digest(), "{field}");
    }
}

#[test]
fn intent_wire_and_approval_have_an_independently_constructed_vector() {
    // Literal ordered JSON assembled outside the Rust encoder with Perl;
    // Digest::SHA binds the explicit NUL-terminated intent domain. Public root
    // bytes are the DER-wrapped BLS key-1 generator used by these fixtures.
    let (policy, _) = fixture(Scheme::Ed25519);
    let mut request = policy.request().clone();
    request.trust_root = BackupDigest::compute(&profile().root_der);
    request.module_sha256 = BackupDigest::compute(b"abcdefghij");
    request.limits.payload_bytes = 64;
    request.limits.extent_bytes = 4;
    request.limits.network_requests = 20;
    request.limits.observations = 0;
    request.limits.signing_requests = 20;
    request.limits.evidence_bytes = 8 * 1024 * 1024;
    let intent = CapturePlan::new(request, profile()).unwrap();
    assert_eq!(intent.encoded(), include_bytes!("golden.json"));
    assert_eq!(
        intent.digest().to_string(),
        "d50ea8e4f79cc9bcd32f67d4c5b22007bac97fb6fad5840ad45e69666b3f13ec"
    );
    let reopened = CapturePlan::decode(include_bytes!("golden.json")).unwrap();
    assert_eq!(reopened.digest(), intent.digest());
}

#[test]
fn original_profile_rejects_unrepresentable_windows_costs_and_roots() {
    let (intent, _, _) = input();
    for field in [
        "prepare_duration_ns",
        "submit_duration_ns",
        "certificate_max_age_ns",
        "delegation_max_age_ns",
        "clock_skew_ns",
        "signer_network",
        "request_bytes",
        "result_bytes",
    ] {
        let mut record = serde_json::to_value(intent.profile()).unwrap();
        record[field] = u64::MAX.into();
        let invalid = serde_json::from_value::<ProfileRecord>(record)
            .ok()
            .and_then(|p| CapturePlan::new(intent.plan().request().clone(), p).ok());
        assert!(invalid.is_none(), "{field}");
    }
    let mut p = profile();
    p.root_der = der(2);
    assert!(matches!(
        CapturePlan::new(intent.plan().request().clone(), p),
        Err(Error::Profile)
    ));
    assert!(intent.view().admission(0).is_err());
    assert!(intent.view().admission(u64::MAX - 19).is_err());
    assert!(intent.view().observation(89).is_err());
    assert_eq!(
        intent.view().admission(100).unwrap().submit_deadline_ns,
        120
    );
}

#[test]
fn authenticated_procedure_replays_original_evidence_without_live_dependencies() {
    let (intent, prepared, evidence) = input();
    let reopened = CapturePlan::decode(intent.encoded()).unwrap();
    for approval in [&intent, &reopened] {
        let mut procedure = Procedure::new(approval).unwrap();
        procedure
            .accept_authenticated(Step::InitialStatus, 100, &prepared, &evidence)
            .unwrap();
        assert_eq!(procedure.next().unwrap(), Next::Call(Step::ListSnapshots));
        assert_eq!(procedure.request().unwrap().0, "list_canister_snapshots");
    }
}

#[test]
fn changed_time_request_or_proof_blocks_the_same_original_procedure() {
    for mutation in 0..9 {
        let (intent, mut prepared, mut evidence) = input();
        let reference = if mutation == 0 { 101 } else { 100 };
        match mutation {
            1 => prepared.expires_ns += 1,
            2 => prepared.observation[0] ^= 1,
            3 => prepared.envelope[0] ^= 1,
            4 => evidence.response[0] ^= 1,
            5 => evidence.proof[0] ^= 1,
            6 => evidence.rejected = true,
            7 | 8 => change(&mut prepared, |envelope| {
                if let EnvelopeContent::Call {
                    method_name, arg, ..
                } = &mut envelope.content
                {
                    if mutation == 7 {
                        *method_name = "list_canister_snapshots".into();
                    } else {
                        arg.push(0);
                    }
                }
                let message = [
                    b"\x0aic-request".as_slice(),
                    envelope.content.request_id().unwrap().as_slice(),
                ]
                .concat();
                envelope.sender_sig = Some(Scheme::Ed25519.sign(&message));
            }),
            _ => (),
        }
        let mut procedure = Procedure::new(&intent).unwrap();
        assert!(
            matches!(
                procedure.accept_authenticated(
                    Step::InitialStatus,
                    reference,
                    &prepared,
                    &evidence
                ),
                Err(ProcedureError::Authentication(_))
            ),
            "{mutation}"
        );
        assert!(matches!(procedure.next(), Err(ProcedureError::Order)));
    }
}

#[test]
fn certified_rejection_is_terminal_evidence_and_never_unlocks_next_step() {
    use crate::store::capture::advance::status_certificate_evidence;
    use ic_platform::certificate::construction::{fork, labeled, leaf};
    let (intent, prepared, _) = input();
    let tree = fork(
        labeled(
            b"request_status",
            labeled(
                prepared.request_id.to_vec(),
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
    let proof = signed(tree, 1, None);
    let evidence = status_certificate_evidence(
        intent.plan(),
        &intent.view().admission(100).unwrap(),
        &prepared,
        &proof,
        &intent.profile().root_der,
    )
    .unwrap();
    assert!(evidence.rejected);
    for approval in [&intent, &CapturePlan::decode(intent.encoded()).unwrap()] {
        let mut procedure = Procedure::new(approval).unwrap();
        assert!(matches!(
            procedure.accept_authenticated(Step::InitialStatus, 100, &prepared, &evidence),
            Err(ProcedureError::Rejected)
        ));
        assert!(matches!(procedure.next(), Err(ProcedureError::Order)));
    }
}

#[test]
fn original_root_approval_survives_reopen_and_requires_sole_child_control() {
    use crate::store::operation::intent::ManagerRecord;
    use ic_platform_types::Principal;

    let (direct, prepared, _) = input();
    let request = direct.request().clone();
    let root = Principal::from_slice(&[8, 1]);
    let mut profile = direct.profile().clone();
    profile.manager = Some(ManagerRecord {
        root,
        generation: 1,
        protocol: [70; 32],
        authority_policy: [71; 32],
        cycles: 0,
        cycle_limit: 1_000_000_000_000,
        reply_bytes: 16_384,
    });
    let approved = CapturePlan::new(request.clone(), profile.clone()).unwrap();
    let directory = crate::store::tests::root("manager-approved-capture");
    let store = crate::store::BackupStore::initialize(&directory).unwrap();
    drop(
        store
            .create_capture(approved.clone(), approved.digest())
            .unwrap(),
    );
    drop(store);
    let store = crate::store::BackupStore::open(&directory).unwrap();
    assert!(
        store
            .open_capture(&request.operation_id, direct.digest())
            .is_err()
    );
    let operation = store
        .open_capture(&request.operation_id, approved.digest())
        .unwrap();
    assert_eq!(operation.inspect().unwrap().signing_requests_reserved, 0);
    let reopened = &operation.intent;
    assert_eq!(reopened.digest(), approved.digest());
    assert_ne!(approved.digest(), direct.digest());
    assert_eq!(reopened.view().ingress_recipient(), root);
    assert_eq!(reopened.view().sender(), direct.view().sender());
    let requirements = reopened.view().status_requirements();
    let mut observed = status(direct.plan());
    observed.settings.controllers = vec![root];
    requirements.validate_status(&observed).unwrap();
    observed.settings.controllers.push(request.sender);
    assert!(requirements.validate_status(&observed).is_err());
    assert!(matches!(
        reopened
            .view()
            .authenticate_request(100, &prepared, "canister_status", &[]),
        Err(Error::ManagerExchange)
    ));
    let manager = profile.manager.as_mut().unwrap();
    manager.reply_bytes -= 1;
    assert_ne!(
        CapturePlan::new(request.clone(), profile.clone())
            .unwrap()
            .digest(),
        approved.digest()
    );
    profile.manager.as_mut().unwrap().generation = 2;
    assert!(CapturePlan::new(request, profile).is_err());
}
