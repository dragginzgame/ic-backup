use super::*;
use crate::{
    model::ic_request::IcManagementMethodRecord as Method,
    test_support::{
        ic_observation::{input, original},
        membership::hash,
    },
};

#[test]
fn existing_status_and_inventory_wire_evidence_never_settles_attempts() {
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
        let response = IcObservationResponse::new(input(&request)).unwrap();
        let view = validate_response(&request, &original.journal, &response).unwrap();
        assert_eq!(view.response().input().evidence, hash("34"));
        let checksum =
            crate::model::artifacts::ArtifactChecksumRecord::from_bytes(&response.input().reply);
        match view.reply() {
            IcObservationReplyView::Status(reply) => {
                let crate::model::ic_lifecycle_reply::IcLifecycleReplyKind::Status(info) =
                    reply.kind()
                else {
                    panic!("status shape")
                };
                assert_eq!(
                    info.status(),
                    ic_management_canister_types::CanisterStatusType::Stopped
                );
                assert_eq!(reply.payload_checksum(), &checksum);
            }
            IcObservationReplyView::Inventory(reply) => {
                assert_eq!(reply.payload_checksum(), &checksum);
                assert_eq!(
                    reply.digest(),
                    IcSnapshotReply::decode(&original.payload, &response.input().reply)
                        .unwrap()
                        .digest()
                );
                // Cardinality is wire evidence, never exclusive capture attribution.
                for count in 0..=2 {
                    let snapshots = (0..count)
                        .map(|id| ic_management_canister_types::Snapshot {
                            id: vec![id],
                            taken_at_timestamp: 42,
                            total_size: 128,
                        })
                        .collect::<Vec<_>>();
                    let mut fields = input(&request);
                    fields.reply = candid::encode_one(snapshots).unwrap();
                    let response = IcObservationResponse::new(fields).unwrap();
                    let associated =
                        validate_response(&request, &original.journal, &response).unwrap();
                    let IcObservationReplyView::Inventory(inventory) = associated.reply() else {
                        panic!("inventory shape")
                    };
                    assert_eq!(inventory.snapshots().len(), usize::from(count));
                    assert_eq!(original.journal, before);
                }
            }
        }
        assert_eq!(original.journal, before);
        assert_eq!(original.journal.view().pending_mutation, Some(1));
        assert_eq!(original.journal.view().pending_observation, Some(2));
        assert!(!original.journal.view().applied);
    }
}

#[test]
fn actual_authority_attempt_payload_context_and_target_drift_are_typed_denials() {
    let original = original(Method::CanisterStatus);
    let request = IcObservationRequest::new(
        &original.plan,
        7,
        &original.journal,
        &original.mutation,
        &original.payload,
    )
    .unwrap();
    for change in 0..8 {
        let mut fields = input(&request);
        match change {
            0 => fields.authority = hash("12"),
            1 => {
                fields.mutation_attempt = 2;
                fields.observation_attempt = 3;
            }
            2 => fields.observation_attempt = 3,
            3 => fields.request = hash("12"),
            4..=6 => {
                let mut context = serde_json::to_value(&fields.context).unwrap();
                let (field, value) = match change {
                    4 => ("network", hash("12").hash().to_string()),
                    5 => ("caller", "aaaaa-aa".into()),
                    _ => ("release", hash("12").hash().to_string()),
                };
                context[field] = serde_json::json!(value);
                fields.context = serde_json::from_value(context).unwrap();
            }
            _ => fields.target = "aaaaa-aa".into(),
        }
        let response = IcObservationResponse::new(fields).unwrap();
        let error = validate_response(&request, &original.journal, &response).unwrap_err();
        assert!(match change {
            0 => matches!(error, IcObservationAssociationError::AuthorityMismatch),
            1..=2 => matches!(error, IcObservationAssociationError::AttemptMismatch),
            3 => matches!(error, IcObservationAssociationError::RequestMismatch),
            4..=6 => matches!(error, IcObservationAssociationError::ContextMismatch),
            _ => matches!(error, IcObservationAssociationError::TargetMismatch),
        });
    }
}

#[test]
fn wrong_wire_and_current_reservation_cannot_bypass_existing_owners() {
    for method in [Method::CanisterStatus, Method::ListCanisterSnapshots] {
        let original = original(method);
        let request = IcObservationRequest::new(
            &original.plan,
            7,
            &original.journal,
            &original.mutation,
            &original.payload,
        )
        .unwrap();
        for raw in [vec![], b"DIDL\0\0".to_vec(), b"invalid".to_vec()] {
            let mut fields = input(&request);
            fields.reply = raw;
            let response = IcObservationResponse::new(fields).unwrap();
            let error = validate_response(&request, &original.journal, &response).unwrap_err();
            assert!(match method {
                Method::CanisterStatus => matches!(error, IcObservationAssociationError::Status(_)),
                _ => matches!(error, IcObservationAssociationError::Inventory(_)),
            });
        }
        let response = IcObservationResponse::new(input(&request)).unwrap();
        let empty = crate::model::attempt_journal::AttemptJournalRecord::new(
            original.journal.authority().clone(),
        );
        assert!(matches!(
            validate_response(&request, &empty, &response),
            Err(IcObservationAssociationError::Reservation(_))
        ));
    }
}
