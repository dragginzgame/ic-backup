use super::*;
use crate::{
    model::{
        ic_mutation::IcMutationAcknowledgementInput, ic_request::IcManagementMethodRecord as Method,
    },
    test_support::{
        ic_mutation::{input, original},
        membership::hash,
    },
};

#[test]
fn capture_and_lifecycle_decode_with_exact_existing_evidence_without_settlement() {
    for method in [
        Method::TakeCanisterSnapshot,
        Method::LoadCanisterSnapshot,
        Method::StopCanister,
        Method::StartCanister,
    ] {
        let original = original(method);
        let before = original.journal.clone();
        let request =
            IcMutationRequest::new(&original.plan, 7, &original.journal, &original.payload)
                .unwrap();
        let acknowledgement = IcMutationAcknowledgement::new(input(&request)).unwrap();
        let view = validate_acknowledgement(&request, &original.journal, &acknowledgement).unwrap();
        assert_eq!(view.acknowledgement().input().evidence, hash("34"));
        let raw_hash = crate::model::artifacts::ArtifactChecksumRecord::from_bytes(
            &acknowledgement.input().reply,
        );
        match view.reply() {
            IcMutationReplyView::Capture(reply) => {
                assert_eq!(reply.snapshots()[0].id(), &[0, 255, 128]);
                assert_eq!(reply.snapshots()[0].taken_at_timestamp(), u64::MAX);
                assert_eq!(reply.payload_checksum(), &raw_hash);
                assert_eq!(
                    reply.digest(),
                    IcSnapshotReply::decode(&original.payload, &acknowledgement.input().reply)
                        .unwrap()
                        .digest()
                );
            }
            IcMutationReplyView::Lifecycle(reply) => {
                assert_eq!(
                    reply.kind(),
                    &crate::model::ic_lifecycle_reply::IcLifecycleReplyKind::Acknowledgement
                );
                assert_eq!(reply.payload_checksum(), &raw_hash);
                assert_eq!(
                    reply.digest(),
                    IcLifecycleReply::decode(&original.payload, &acknowledgement.input().reply)
                        .unwrap()
                        .digest()
                );
            }
        }
        assert_eq!(original.journal, before);
        assert_eq!(original.journal.view().pending_mutation, Some(1));
        assert!(!original.journal.view().applied);
    }
}

#[test]
fn different_authority_attempt_actual_context_and_target_reject() {
    let original = original(Method::StopCanister);
    let request =
        IcMutationRequest::new(&original.plan, 7, &original.journal, &original.payload).unwrap();
    for change in 0..6 {
        let mut fields = input(&request);
        match change {
            0 => fields.authority = hash("12"),
            1 => fields.mutation_attempt = 2,
            2..=4 => {
                let mut context = serde_json::to_value(&fields.context).unwrap();
                let (field, value) = match change {
                    2 => ("network", hash("12").hash().to_string()),
                    3 => ("caller", "aaaaa-aa".into()),
                    _ => ("release", hash("12").hash().to_string()),
                };
                context[field] = serde_json::json!(value);
                fields.context = serde_json::from_value(context).unwrap();
            }
            _ => fields.target = "aaaaa-aa".into(),
        }
        let acknowledgement = IcMutationAcknowledgement::new(fields).unwrap();
        let error =
            validate_acknowledgement(&request, &original.journal, &acknowledgement).unwrap_err();
        assert!(match change {
            0 => matches!(error, IcMutationAssociationError::AuthorityMismatch),
            1 => matches!(error, IcMutationAssociationError::AttemptMismatch),
            2..=4 => matches!(error, IcMutationAssociationError::ContextMismatch),
            _ => matches!(error, IcMutationAssociationError::TargetMismatch),
        });
    }
}

#[test]
fn method_specific_invalid_wire_and_stale_recovery_journal_reject() {
    for method in [Method::TakeCanisterSnapshot, Method::StopCanister] {
        let mut original = original(method);
        let request =
            IcMutationRequest::new(&original.plan, 7, &original.journal, &original.payload)
                .unwrap();
        for bytes in [
            vec![],
            b"DIDL\0\0trailing".to_vec(),
            b"unbounded invalid reply".to_vec(),
        ] {
            let acknowledgement = IcMutationAcknowledgement::new(IcMutationAcknowledgementInput {
                reply: bytes,
                ..input(&request)
            })
            .unwrap();
            let error = validate_acknowledgement(&request, &original.journal, &acknowledgement)
                .unwrap_err();
            assert!(match method {
                Method::TakeCanisterSnapshot =>
                    matches!(error, IcMutationAssociationError::Snapshot(_)),
                _ => matches!(error, IcMutationAssociationError::Lifecycle(_)),
            });
        }
        let acknowledgement = IcMutationAcknowledgement::new(input(&request)).unwrap();
        original
            .journal
            .reserve_observation(1, hash("56").hash())
            .unwrap();
        assert!(matches!(
            validate_acknowledgement(&request, &original.journal, &acknowledgement),
            Err(IcMutationAssociationError::Reservation(
                IcMutationRequestError::ObservationPending
            ))
        ));
        assert_eq!(original.journal.view().pending_observation, Some(2));
        assert_eq!(original.journal.view().mutations_used, 1);
    }
}
