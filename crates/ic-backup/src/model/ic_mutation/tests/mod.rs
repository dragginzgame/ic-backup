use super::*;
use crate::{
    model::{
        attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest},
        ic_request::{IcManagementMethodRecord as Method, IcManagementRequest},
    },
    test_support::{
        ic_mutation::{input, original},
        membership::hash,
    },
};

#[test]
fn all_four_mutations_bind_original_bytes_authority_and_consumed_attempt() {
    for method in [
        Method::TakeCanisterSnapshot,
        Method::LoadCanisterSnapshot,
        Method::StartCanister,
        Method::StopCanister,
    ] {
        let original = original(method);
        let before = original.journal.clone();
        let request =
            IcMutationRequest::new(&original.plan, 7, &original.journal, &original.payload)
                .unwrap();
        assert_eq!(request.plan(), &original.plan);
        assert_eq!(request.payload(), &original.payload);
        assert_eq!(request.authority(), original.journal.authority());
        assert_eq!(request.mutation_attempt(), 1);
        request.validate_journal(&original.journal).unwrap();
        assert_eq!(original.journal, before);
    }
}

#[test]
fn unknown_operation_and_changed_original_authority_are_typed_denials() {
    let original = original(Method::StopCanister);
    assert!(matches!(
        IcMutationRequest::new(&original.plan, 9, &original.journal, &original.payload),
        Err(IcMutationRequestError::Plan(_))
    ));
    let other = AttemptJournalRecord::new(original.plan.attempt_authority(0).unwrap());
    assert!(matches!(
        IcMutationRequest::new(&original.plan, 7, &other, &original.payload),
        Err(IcMutationRequestError::AuthorityMismatch)
    ));
    let mut changed = serde_json::to_value(&original.plan).unwrap();
    changed["operations"][1]["budget"]["mutations"] = serde_json::json!(1);
    let changed: OperationPlanRecord = serde_json::from_value(changed).unwrap();
    assert!(matches!(
        IcMutationRequest::new(&changed, 7, &original.journal, &original.payload),
        Err(IcMutationRequestError::AuthorityMismatch)
    ));
}

#[test]
fn observations_wrong_target_and_changed_load_snapshot_bytes_reject() {
    for method in [Method::CanisterStatus, Method::ListCanisterSnapshots] {
        let original = original(method);
        assert!(matches!(
            IcMutationRequest::new(&original.plan, 7, &original.journal, &original.payload),
            Err(IcMutationRequestError::Payload(
                IcRequestError::EffectMismatch { .. }
            ))
        ));
    }
    let original = original(Method::LoadCanisterSnapshot);
    let changed = IcManagementRequestRecord::new(IcManagementRequest {
        method: Method::LoadCanisterSnapshot,
        target: original.payload.target().into(),
        snapshot_id: Some(vec![1, 255, 128]),
    })
    .unwrap();
    assert!(matches!(
        IcMutationRequest::new(&original.plan, 7, &original.journal, &changed),
        Err(IcMutationRequestError::Payload(
            IcRequestError::DigestMismatch
        ))
    ));
    let other = IcManagementRequestRecord::new(IcManagementRequest {
        method: Method::LoadCanisterSnapshot,
        target: "aaaaa-aa".into(),
        snapshot_id: original.payload.snapshot_id().map(<[u8]>::to_vec),
    })
    .unwrap();
    assert!(matches!(
        IcMutationRequest::new(&original.plan, 7, &original.journal, &other),
        Err(IcMutationRequestError::Payload(
            IcRequestError::TargetMismatch
        ))
    ));
}

#[test]
fn absent_replaced_settled_and_recovering_reservations_do_not_admit_dispatch() {
    let mut original = original(Method::StopCanister);
    let empty = AttemptJournalRecord::new(original.journal.authority().clone());
    assert!(matches!(
        IcMutationRequest::new(&original.plan, 7, &empty, &original.payload),
        Err(IcMutationRequestError::NoPendingMutation)
    ));
    let request =
        IcMutationRequest::new(&original.plan, 7, &original.journal, &original.payload).unwrap();
    original
        .journal
        .record_mutation(MutationReceiptRequest {
            attempt: 1,
            request: original.payload.digest().hash().into(),
            outcome: MutationOutcomeRecord::NotApplied,
            evidence: hash("12").hash().into(),
        })
        .unwrap();
    assert!(matches!(
        request.validate_journal(&original.journal),
        Err(IcMutationRequestError::MutationMismatch)
    ));
    original.journal.reserve_mutation().unwrap();
    assert!(matches!(
        request.validate_journal(&original.journal),
        Err(IcMutationRequestError::MutationMismatch)
    ));
    original
        .journal
        .reserve_observation(2, hash("56").hash())
        .unwrap();
    assert!(matches!(
        IcMutationRequest::new(&original.plan, 7, &original.journal, &original.payload),
        Err(IcMutationRequestError::ObservationPending)
    ));
    assert_eq!(original.journal.view().mutations_remaining, 0);
}

#[test]
fn acknowledgement_attempt_target_raw_bounds_and_redacted_debug_are_owned() {
    let original = original(Method::StopCanister);
    let request =
        IcMutationRequest::new(&original.plan, 7, &original.journal, &original.payload).unwrap();
    for attempt in [0, MAX_OPERATION_ATTEMPTS + 1] {
        let mut fields = input(&request);
        fields.mutation_attempt = attempt;
        assert_eq!(
            IcMutationAcknowledgement::new(fields).unwrap_err(),
            IcMutationAcknowledgementError::InvalidAttempt
        );
    }
    let mut fields = input(&request);
    fields.target = "invalid".into();
    assert_eq!(
        IcMutationAcknowledgement::new(fields).unwrap_err(),
        IcMutationAcknowledgementError::InvalidTarget
    );
    let mut fields = input(&request);
    fields.reply = vec![b'X'; MAX_IC_MUTATION_REPLY_BYTES + 1];
    assert_eq!(
        IcMutationAcknowledgement::new(fields).unwrap_err(),
        IcMutationAcknowledgementError::ReplyTooLarge
    );
    let mut fields = input(&request);
    fields.mutation_attempt = MAX_OPERATION_ATTEMPTS;
    fields.target = fields.target.to_ascii_uppercase();
    fields.reply = vec![b'X'; MAX_IC_MUTATION_REPLY_BYTES];
    let reply = IcMutationAcknowledgement::new(fields).unwrap();
    assert_eq!(reply.input().reply.len(), MAX_IC_MUTATION_REPLY_BYTES);
    assert_eq!(reply.input().target, original.payload.target());
    assert!(!format!("{reply:?}").contains("XXXX"));
}
