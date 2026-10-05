use super::*;
use crate::{
    model::{
        attempt_journal::{ObservationOutcomeRecord, ObservationReceiptRequest},
        ic_request::{IcManagementMethodRecord as Method, IcManagementRequest},
    },
    test_support::{
        ic_observation::{input, original},
        membership::hash,
    },
};

#[test]
fn status_and_list_bind_exact_original_bytes_and_both_spent_attempts() {
    for method in [Method::CanisterStatus, Method::ListCanisterSnapshots] {
        let original = original(method);
        let before = original.journal.clone();
        let request = IcObservationRequest::new(
            &original.plan,
            7,
            &original.journal,
            &original.mutation,
            &original.payload,
        )
        .unwrap();
        assert_eq!(request.plan(), &original.plan);
        assert_eq!(request.mutation(), &original.mutation);
        assert_eq!(request.payload(), &original.payload);
        assert_eq!(request.authority(), original.journal.authority());
        assert_eq!(
            (request.mutation_attempt(), request.observation_attempt()),
            (1, 2)
        );
        request.validate_journal(&original.journal).unwrap();
        assert_eq!(original.journal, before);
    }
}

#[test]
fn changed_plan_mutation_target_class_and_reserved_observation_are_rejected() {
    let original = original(Method::CanisterStatus);
    assert!(matches!(
        IcObservationRequest::new(
            &original.plan,
            99,
            &original.journal,
            &original.mutation,
            &original.payload
        ),
        Err(IcObservationRequestError::Plan(_))
    ));
    let mut plan = serde_json::to_value(&original.plan).unwrap();
    plan["operations"][1]["budget"]["observations"] = serde_json::json!(1);
    let plan: OperationPlanRecord = serde_json::from_value(plan).unwrap();
    assert!(matches!(
        IcObservationRequest::new(
            &plan,
            7,
            &original.journal,
            &original.mutation,
            &original.payload
        ),
        Err(IcObservationRequestError::AuthorityMismatch)
    ));
    let changed = IcManagementRequestRecord::new(IcManagementRequest {
        method: Method::StopCanister,
        target: original.payload.target().into(),
        snapshot_id: None,
    })
    .unwrap();
    assert!(matches!(
        IcObservationRequest::new(
            &original.plan,
            7,
            &original.journal,
            &changed,
            &original.payload
        ),
        Err(IcObservationRequestError::Payload(
            IcRequestError::DigestMismatch
        ))
    ));
    assert!(matches!(
        IcObservationRequest::new(
            &original.plan,
            7,
            &original.journal,
            &original.mutation,
            &changed
        ),
        Err(IcObservationRequestError::Payload(
            IcRequestError::EffectMismatch { .. }
        ))
    ));
    assert!(matches!(
        IcObservationRequest::new(
            &original.plan,
            7,
            &original.journal,
            &original.payload,
            &original.payload
        ),
        Err(IcObservationRequestError::Payload(
            IcRequestError::EffectMismatch { .. }
        ))
    ));
    let other = IcManagementRequestRecord::new(IcManagementRequest {
        method: Method::CanisterStatus,
        target: "aaaaa-aa".into(),
        snapshot_id: None,
    })
    .unwrap();
    assert!(matches!(
        IcObservationRequest::new(
            &original.plan,
            7,
            &original.journal,
            &original.mutation,
            &other
        ),
        Err(IcObservationRequestError::Payload(
            IcRequestError::TargetMismatch
        ))
    ));
    let list = IcManagementRequestRecord::new(IcManagementRequest {
        method: Method::ListCanisterSnapshots,
        target: original.payload.target().into(),
        snapshot_id: None,
    })
    .unwrap();
    assert!(matches!(
        IcObservationRequest::new(
            &original.plan,
            7,
            &original.journal,
            &original.mutation,
            &list
        ),
        Err(IcObservationRequestError::RequestMismatch)
    ));
}

#[test]
fn missing_settled_and_replaced_reservations_keep_original_consumption() {
    let mut original = original(Method::CanisterStatus);
    let mut empty = AttemptJournalRecord::new(original.journal.authority().clone());
    assert!(matches!(
        IcObservationRequest::new(
            &original.plan,
            7,
            &empty,
            &original.mutation,
            &original.payload
        ),
        Err(IcObservationRequestError::NoPendingMutation)
    ));
    empty.reserve_mutation().unwrap();
    assert!(matches!(
        IcObservationRequest::new(
            &original.plan,
            7,
            &empty,
            &original.mutation,
            &original.payload
        ),
        Err(IcObservationRequestError::NoPendingObservation)
    ));
    let request = IcObservationRequest::new(
        &original.plan,
        7,
        &original.journal,
        &original.mutation,
        &original.payload,
    )
    .unwrap();
    original
        .journal
        .record_observation(ObservationReceiptRequest {
            attempt: 2,
            request: original.payload.digest().hash().into(),
            outcome: ObservationOutcomeRecord::Uncertain,
            evidence: hash("12").hash().into(),
        })
        .unwrap();
    assert!(matches!(
        request.validate_journal(&original.journal),
        Err(IcObservationRequestError::ObservationMismatch)
    ));
    original
        .journal
        .reserve_observation(1, original.payload.digest().hash())
        .unwrap();
    assert!(matches!(
        request.validate_journal(&original.journal),
        Err(IcObservationRequestError::ObservationMismatch)
    ));
    assert_eq!(original.journal.view().observations_remaining, 0);
    original
        .journal
        .record_observation(ObservationReceiptRequest {
            attempt: 3,
            request: original.payload.digest().hash().into(),
            outcome: ObservationOutcomeRecord::Applied,
            evidence: hash("12").hash().into(),
        })
        .unwrap();
    assert!(matches!(
        request.validate_journal(&original.journal),
        Err(IcObservationRequestError::MutationMismatch)
    ));
    assert_eq!(original.journal.view().observations_used, 2);
}

#[test]
fn passive_reply_owns_chronological_attempt_byte_and_principal_bounds() {
    let original = original(Method::CanisterStatus);
    let request = IcObservationRequest::new(
        &original.plan,
        7,
        &original.journal,
        &original.mutation,
        &original.payload,
    )
    .unwrap();
    for (mutation, observation) in [
        (0, 2),
        (1, 0),
        (2, 2),
        (3, 2),
        (1, MAX_OPERATION_ATTEMPTS + 1),
        (MAX_OPERATION_ATTEMPTS + 1, 2),
    ] {
        let mut fields = input(&request);
        fields.mutation_attempt = mutation;
        fields.observation_attempt = observation;
        assert_eq!(
            IcObservationResponse::new(fields).unwrap_err(),
            IcObservationResponseError::InvalidAttempts
        );
    }
    let mut fields = input(&request);
    fields.target = "invalid".into();
    assert_eq!(
        IcObservationResponse::new(fields).unwrap_err(),
        IcObservationResponseError::InvalidTarget
    );
    let mut fields = input(&request);
    fields.reply = vec![b'X'; MAX_IC_OBSERVATION_REPLY_BYTES + 1];
    assert_eq!(
        IcObservationResponse::new(fields).unwrap_err(),
        IcObservationResponseError::ReplyTooLarge
    );
    let mut fields = input(&request);
    fields.mutation_attempt = MAX_OPERATION_ATTEMPTS - 1;
    fields.observation_attempt = MAX_OPERATION_ATTEMPTS;
    fields.target = fields.target.to_ascii_uppercase();
    fields.reply = vec![b'X'; MAX_IC_OBSERVATION_REPLY_BYTES];
    let response = IcObservationResponse::new(fields).unwrap();
    assert_eq!(response.input().reply.len(), MAX_IC_OBSERVATION_REPLY_BYTES);
    assert_eq!(response.input().target, original.payload.target());
    assert!(!format!("{response:?}").contains("XXXX"));
}
