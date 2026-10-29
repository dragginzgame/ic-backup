use super::*;
use crate::model::{
    artifacts::ArtifactChecksumRecord,
    attempt_journal::{
        AttemptJournalRecord, MutationOutcomeRecord, MutationReceiptRequest,
        ObservationOutcomeRecord, ObservationReceiptRequest,
    },
    ic_observation::IcObservationRequestError,
    operation_plan::{OperationPlanRecord, PlannedOperationRecord, PlannedOperationRequest},
};
use crate::test_support::operation_plan::{APP, request};
use sha2::{Digest, Sha256};

fn payload(method: &str) -> ApplicationIngressPayload {
    ApplicationIngressPayload::new(APP, APP, method, b"opaque\0bytes", 128).unwrap()
}
fn plan(payload: &ApplicationIngressPayload) -> OperationPlanRecord {
    let mut input = request();
    let original = &input.operations[0];
    input.operations[0] = PlannedOperationRecord::new(PlannedOperationRequest {
        operation_sequence: 0,
        target: APP.into(),
        request: payload.digest().hash().into(),
        budget: original.budget().clone(),
    })
    .unwrap();
    OperationPlanRecord::new(input).unwrap()
}
#[test]
fn exact_opaque_envelope_has_independent_framing_and_finite_admission() {
    let payload = payload("apply");
    let raw = ic_platform::Principal::from_text(APP).unwrap();
    let mut bytes = b"ic-backup/application-ingress/v1\0".to_vec();
    for _ in 0..2 {
        bytes.push(u8::try_from(raw.as_slice().len()).unwrap());
        bytes.extend_from_slice(raw.as_slice());
    }
    bytes.push(1); // Fixed update mode; there is no query constructor.
    for value in [b"apply".as_slice(), b"opaque\0bytes".as_slice()] {
        bytes.extend_from_slice(&(value.len() as u64).to_be_bytes());
        bytes.extend_from_slice(value);
    }
    bytes.extend_from_slice(&128_u64.to_be_bytes());
    assert_eq!(
        payload.digest().hash(),
        format!("{:x}", Sha256::digest(bytes))
    );
    assert_eq!(
        payload.digest(),
        ApplicationIngressPayload::new(&APP.to_uppercase(), APP, "apply", b"opaque\0bytes", 128)
            .unwrap()
            .digest()
    );
    assert!(!format!("{payload:?}").contains(&format!("{:?}", payload.arguments())));
    for method in ["", "a b", "a\n", "é", &"x".repeat(129)] {
        assert!(matches!(
            ApplicationIngressPayload::new(APP, APP, method, b"", 1),
            Err(ApplicationIngressError::InvalidMethod)
        ));
    }
    for bound in [0, MAX_APPLICATION_INGRESS_REPLY_BYTES + 1] {
        assert!(matches!(
            ApplicationIngressPayload::new(APP, APP, "apply", b"", bound),
            Err(ApplicationIngressError::InvalidReplyLimit)
        ));
    }
    assert!(matches!(
        ApplicationIngressPayload::new("aaaaa-aa", APP, "apply", b"", 1),
        Err(ApplicationIngressError::ManagementPrincipal)
    ));
    assert!(matches!(
        ApplicationIngressPayload::new(APP, "aaaaa-aa", "apply", b"", 1),
        Err(ApplicationIngressError::ManagementPrincipal)
    ));
    assert!(matches!(
        ApplicationIngressPayload::new("invalid", APP, "apply", b"", 1),
        Err(ApplicationIngressError::InvalidPrincipal)
    ));
    assert!(matches!(
        ApplicationIngressPayload::new(
            APP,
            APP,
            "apply",
            &vec![0; MAX_APPLICATION_INGRESS_ARGUMENT_BYTES + 1],
            1
        ),
        Err(ApplicationIngressError::ArgumentsTooLarge)
    ));
    assert!(
        ApplicationIngressPayload::new(
            APP,
            APP,
            &"x".repeat(128),
            &vec![0; MAX_APPLICATION_INGRESS_ARGUMENT_BYTES],
            MAX_APPLICATION_INGRESS_REPLY_BYTES
        )
        .is_ok()
    );
}
#[test]
fn recovery_target_is_admitted_before_spending_with_distinct_payload_bytes() {
    let plan = plan(&payload("apply"));
    let authority = plan.attempt_authority(0).unwrap();
    assert!(
        payload("observe")
            .validate_observation_binding(authority.binding())
            .is_ok()
    );
    let changed = ApplicationIngressPayload::new(APP, "2vxsx-fae", "observe", b"", 128).unwrap();
    assert!(matches!(
        changed.validate_observation_binding(authority.binding()),
        Err(ApplicationIngressError::TargetMismatch)
    ));
}
#[test]
fn changed_fields_cannot_replace_original_mutation_or_recovery_bytes() {
    let payload = payload("apply");
    let plan = plan(&payload);
    let authority = plan.attempt_authority(0).unwrap();
    let mut journal = AttemptJournalRecord::new(authority.clone());
    assert!(matches!(
        ApplicationMutationRequest::new(&plan, 0, &journal, &payload),
        Err(ApplicationIngressError::NoPendingMutation)
    ));
    journal.reserve_mutation().unwrap();
    let mutation = ApplicationMutationRequest::new(&plan, 0, &journal, &payload).unwrap();
    for (receiver, target, method, arguments, limit) in [
        ("2vxsx-fae", APP, "apply", b"opaque\0bytes".as_slice(), 128),
        (APP, "2vxsx-fae", "apply", b"opaque\0bytes".as_slice(), 128),
        (APP, APP, "Apply", b"opaque\0bytes".as_slice(), 128),
        (APP, APP, "apply", b"changed".as_slice(), 128),
        (APP, APP, "apply", b"opaque\0bytes".as_slice(), 129),
    ] {
        let changed =
            ApplicationIngressPayload::new(receiver, target, method, arguments, limit).unwrap();
        assert!(ApplicationMutationRequest::new(&plan, 0, &journal, &changed).is_err());
    }
    let read = self::payload("observe");
    assert!(matches!(
        ApplicationObservationRequest::new(&plan, 0, &journal, &read),
        Err(ApplicationIngressError::Observation(
            IcObservationRequestError::NoPendingObservation
        ))
    ));
    let observation = journal
        .reserve_observation(1, read.digest().hash())
        .unwrap();
    assert!(matches!(
        mutation.validate_journal(&journal),
        Err(ApplicationIngressError::ObservationPending)
    ));
    let recovery = ApplicationObservationRequest::new(&plan, 0, &journal, &read).unwrap();
    assert_eq!(
        (recovery.mutation_attempt(), recovery.observation_attempt()),
        (1, observation)
    );
    assert!(ApplicationObservationRequest::new(&plan, 0, &journal, &payload).is_err());
    let unchanged = serde_json::to_vec(&journal).unwrap();
    let foreign = AttemptJournalRecord::new(plan.attempt_authority(7).unwrap());
    assert!(matches!(
        recovery.validate_journal(&foreign),
        Err(ApplicationIngressError::Observation(
            IcObservationRequestError::AuthorityMismatch
        ))
    ));
    assert_eq!(serde_json::to_vec(&journal).unwrap(), unchanged);
    journal
        .record_observation(ObservationReceiptRequest {
            attempt: observation,
            request: read.digest().hash().into(),
            outcome: ObservationOutcomeRecord::Uncertain,
            evidence: ArtifactChecksumRecord::from_bytes(b"qualified uncertainty")
                .hash()
                .into(),
        })
        .unwrap();
    assert!(recovery.validate_journal(&journal).is_err());
    assert_eq!(journal.view().pending_mutation, Some(1));
    assert_eq!(journal.view().pending_observation, None);
    journal
        .record_mutation(MutationReceiptRequest {
            attempt: 1,
            request: payload.digest().hash().into(),
            outcome: MutationOutcomeRecord::Applied,
            evidence: ArtifactChecksumRecord::from_bytes(b"late original reply")
                .hash()
                .into(),
        })
        .unwrap();
    assert!(mutation.validate_journal(&journal).is_err());
    let reopened: AttemptJournalRecord =
        serde_json::from_slice(&serde_json::to_vec(&journal).unwrap()).unwrap();
    assert!(reopened.view().applied);
    assert!(ApplicationMutationRequest::new(&plan, 0, &reopened, &payload).is_err());
}
