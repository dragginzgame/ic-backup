use super::*;
use crate::policy::ResponseIdentityMismatch;
use crate::test_support::response_fault::ResponseFault;
use crate::{
    model::{
        ic_request::IcManagementMethodRecord as Method, ic_update_response::IcUpdateResponseInput,
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
    for change in ResponseFault::IDENTITY {
        let mut fields = input(&request);
        change.apply_with_later_failures(&mut fields, hash("12"));
        let acknowledgement = IcMutationAcknowledgement::new(fields).unwrap();
        let error =
            validate_acknowledgement(&request, &original.journal, &acknowledgement).unwrap_err();
        assert!(match change {
            ResponseFault::Authority => matches!(
                error,
                IcMutationAssociationError::Identity(ResponseIdentityMismatch::Authority)
            ),
            ResponseFault::Attempt => matches!(
                error,
                IcMutationAssociationError::Identity(ResponseIdentityMismatch::Attempt)
            ),
            ResponseFault::Network | ResponseFault::Caller | ResponseFault::Release => matches!(
                error,
                IcMutationAssociationError::Identity(ResponseIdentityMismatch::Context)
            ),
            _ => matches!(
                error,
                IcMutationAssociationError::Identity(ResponseIdentityMismatch::Target)
            ),
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
            let acknowledgement = IcMutationAcknowledgement::new(IcUpdateResponseInput {
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
        let mut stale_fields = input(&request);
        stale_fields.authority = hash("12");
        stale_fields.reply = b"invalid wire".to_vec();
        let acknowledgement = IcMutationAcknowledgement::new(stale_fields).unwrap();
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
