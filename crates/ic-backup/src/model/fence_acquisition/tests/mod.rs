use super::*;
use crate::{
    model::attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest},
    test_support::{
        fence_acquisition::original,
        membership::{APP, hash},
    },
};
use serde_json::json;

#[test]
fn independent_goldens_bind_raw_receiver_mode_method_lengths_and_exact_bytes() {
    let contract: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../../docs/contracts/fence-acquisition-port.json"
    ))
    .unwrap();
    for golden in contract["digest_goldens"].as_array().unwrap() {
        let hex = golden["arguments_hex"].as_str().unwrap();
        let arguments: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|offset| u8::from_str_radix(&hex[offset..offset + 2], 16).unwrap())
            .collect();
        let payload = FenceAcquisitionPayload::new(
            golden["target"].as_str().unwrap(),
            golden["method"].as_str().unwrap(),
            &arguments,
        )
        .unwrap();
        let hex = golden["principal_hex"].as_str().unwrap();
        let principal: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|offset| u8::from_str_radix(&hex[offset..offset + 2], 16).unwrap())
            .collect();
        assert_eq!(payload.target_bytes, principal);
        assert_eq!(payload.digest().hash(), golden["sha256"].as_str().unwrap());
    }
    let original = FenceAcquisitionPayload::new(APP, "acquire", b"a").unwrap();
    for changed in [
        FenceAcquisitionPayload::new(APP, "Acquire", b"a").unwrap(),
        FenceAcquisitionPayload::new(APP, "acquire", b"a\0").unwrap(),
        FenceAcquisitionPayload::new("rrkah-fqaaa-aaaaa-aaaaq-cai", "acquire", b"a").unwrap(),
        FenceAcquisitionPayload::new(APP, "acquirea", b"").unwrap(),
    ] {
        assert_ne!(original.digest(), changed.digest());
    }
    let uppercase = FenceAcquisitionPayload::new(&APP.to_uppercase(), "acquire", b"a").unwrap();
    assert_eq!(original.digest(), uppercase.digest());
    assert_eq!(uppercase.target(), APP);
}

#[test]
fn payload_bounds_are_exact_and_debug_retains_no_argument_content() {
    assert!(matches!(
        FenceAcquisitionPayload::new("invalid", "acquire", b""),
        Err(FenceAcquisitionError::InvalidTarget)
    ));
    assert!(matches!(
        FenceAcquisitionPayload::new("aaaaa-aa", "acquire", b""),
        Err(FenceAcquisitionError::ManagementReceiver)
    ));
    for method in ["", "has space", "line\n", "méthod", "\x7f"] {
        assert!(matches!(
            FenceAcquisitionPayload::new(APP, method, b""),
            Err(FenceAcquisitionError::InvalidMethod)
        ));
    }
    assert!(matches!(
        FenceAcquisitionPayload::new(
            APP,
            &"a".repeat(MAX_FENCE_ACQUISITION_METHOD_BYTES + 1),
            b""
        ),
        Err(FenceAcquisitionError::InvalidMethod)
    ));
    let bytes = vec![0xff; MAX_FENCE_ACQUISITION_ARGUMENT_BYTES];
    let payload =
        FenceAcquisitionPayload::new(APP, &"a".repeat(MAX_FENCE_ACQUISITION_METHOD_BYTES), &bytes)
            .unwrap();
    assert_eq!(payload.arguments(), bytes);
    assert!(matches!(
        FenceAcquisitionPayload::new(
            APP,
            "acquire",
            &vec![0; MAX_FENCE_ACQUISITION_ARGUMENT_BYTES + 1]
        ),
        Err(FenceAcquisitionError::ArgumentsTooLarge)
    ));
    let private =
        FenceAcquisitionPayload::new(APP, "acquire", b"retained private request").unwrap();
    let debug = format!("{private:?}");
    assert!(!debug.contains("retained private request"));
    assert!(debug.contains("argument_bytes: 24"));
}

#[test]
fn both_original_purposes_require_exact_original_payload_and_pending_accounting() {
    for restore in [false, true] {
        let original = original(restore);
        let request = FenceAcquisitionRequest::new(
            &original.plan,
            &original.obligation,
            &original.journal,
            &original.payload,
        )
        .unwrap();
        assert_eq!(request.mutation_attempt(), 1);
        assert_eq!(request.plan(), &original.plan);
        assert_eq!(request.obligation(), &original.obligation);
        assert_eq!(request.payload().arguments(), original.payload.arguments());
        assert_eq!(request.authority(), original.journal.authority());
        assert_eq!(original.journal.view().mutations_remaining, 0);
        let empty = AttemptJournalRecord::new(original.journal.authority().clone());
        assert!(matches!(
            FenceAcquisitionRequest::new(
                &original.plan,
                &original.obligation,
                &empty,
                &original.payload
            ),
            Err(FenceAcquisitionError::NoPendingMutation)
        ));
        let target = FenceAcquisitionPayload::new(
            "rrkah-fqaaa-aaaaa-aaaaq-cai",
            "acquire_fence",
            original.payload.arguments(),
        )
        .unwrap();
        assert!(matches!(
            FenceAcquisitionRequest::new(
                &original.plan,
                &original.obligation,
                &original.journal,
                &target
            ),
            Err(FenceAcquisitionError::TargetMismatch)
        ));
        for changed in [
            FenceAcquisitionPayload::new(APP, "different", original.payload.arguments()).unwrap(),
            FenceAcquisitionPayload::new(APP, "acquire_fence", b"changed").unwrap(),
        ] {
            assert!(matches!(
                FenceAcquisitionRequest::new(
                    &original.plan,
                    &original.obligation,
                    &original.journal,
                    &changed
                ),
                Err(FenceAcquisitionError::PayloadMismatch)
            ));
        }
    }
}

#[test]
fn changed_original_plan_and_allowances_never_rebind_acquisition() {
    let original = original(false);
    for (pointer, replacement) in [
        ("/authority/binding/network", json!(hash("90").hash())),
        ("/authority/binding/request", json!(hash("90").hash())),
        ("/authority/budget/mutations", json!(2)),
        ("/authority/budget/observations", json!(2)),
    ] {
        let mut value = serde_json::to_value(&original.journal).unwrap();
        *value.pointer_mut(pointer).unwrap() = replacement;
        let journal = serde_json::from_value(value).unwrap();
        assert!(matches!(
            FenceAcquisitionRequest::new(
                &original.plan,
                &original.obligation,
                &journal,
                &original.payload
            ),
            Err(FenceAcquisitionError::AuthorityMismatch)
        ));
    }
    let mut value = serde_json::to_value(&original.plan).unwrap();
    value["context"]["release"] = json!(hash("90").hash());
    let changed = serde_json::from_value(value).unwrap();
    assert!(matches!(
        FenceAcquisitionRequest::new(
            &changed,
            &original.obligation,
            &original.journal,
            &original.payload
        ),
        Err(FenceAcquisitionError::Obligation(_))
    ));
}

#[test]
fn settled_or_observation_recovery_cannot_validate_original_mutation_request() {
    let mut original = original(false);
    let request = FenceAcquisitionRequest::new(
        &original.plan,
        &original.obligation,
        &original.journal,
        &original.payload,
    )
    .unwrap();
    let mut settled = original.journal.clone();
    settled
        .record_mutation(MutationReceiptRequest {
            attempt: 1,
            request: original.payload.digest().hash().into(),
            outcome: MutationOutcomeRecord::NotApplied,
            evidence: hash("12").hash().into(),
        })
        .unwrap();
    assert!(matches!(
        request.validate_journal(&settled),
        Err(FenceAcquisitionError::MutationMismatch)
    ));
    original
        .journal
        .reserve_observation(1, hash("12").hash())
        .unwrap();
    assert!(matches!(
        request.validate_journal(&original.journal),
        Err(FenceAcquisitionError::ObservationPending)
    ));
    assert!(matches!(
        FenceAcquisitionRequest::new(
            &original.plan,
            &original.obligation,
            &original.journal,
            &original.payload
        ),
        Err(FenceAcquisitionError::ObservationPending)
    ));
}

#[test]
fn acknowledgement_attempts_use_existing_finite_journal_range() {
    for attempt in [0, MAX_OPERATION_ATTEMPTS + 1] {
        assert!(matches!(
            FenceAcquisitionAcknowledgement::new(hash("12"), attempt, hash("34")),
            Err(FenceAcquisitionError::InvalidAttempt)
        ));
    }
    assert_eq!(
        FenceAcquisitionAcknowledgement::new(hash("12"), MAX_OPERATION_ATTEMPTS, hash("34"))
            .unwrap()
            .mutation_attempt,
        MAX_OPERATION_ATTEMPTS
    );
}
