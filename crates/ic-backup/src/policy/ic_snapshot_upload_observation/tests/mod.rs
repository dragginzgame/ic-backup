use super::*;
use crate::test_support::response_fault::ObservationFault;
use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        ic_observation::{IcObservationResponseError, MAX_IC_OBSERVATION_REPLY_BYTES},
        ic_snapshot_reply::IcSnapshotReply,
    },
    policy::ic_observation::IcObservationReplyView,
    test_support::ic_snapshot_upload::{observation_input, with_observation},
};
use ic_platform::candid::contract;

#[test]
fn zero_one_many_inventory_entries_retain_exact_evidence_and_pending_spending() {
    with_observation(|plan, upload, list, journal| {
        let before = journal.clone();
        let request =
            IcSnapshotUploadObservationRequest::new(plan, 7, journal, upload, list).unwrap();
        for count in 0..=2 {
            let mut input = observation_input(&request);
            input.reply = contract::encode_one(
                (0..count)
                    .rev()
                    .map(|id| ic_platform::management_canister_types::Snapshot {
                        id: vec![id, 0, 255],
                        taken_at_timestamp: u64::MAX,
                        total_size: u64::MAX,
                    })
                    .collect::<Vec<_>>(),
            )
            .unwrap();
            let response = IcObservationResponse::new(input).unwrap();
            let view = validate_response(&request, journal, &response).unwrap();
            assert!(std::ptr::eq(view.response(), &raw const response));
            let IcObservationReplyView::Inventory(reply) = view.reply() else {
                panic!("exact list decoder")
            };
            assert_eq!(reply.snapshots().len(), usize::from(count));
            assert_eq!(
                reply.payload_checksum(),
                &ArtifactChecksumRecord::from_bytes(&response.input().reply)
            );
            assert_eq!(
                reply.digest(),
                IcSnapshotReply::decode(list, &response.input().reply)
                    .unwrap()
                    .digest()
            );
            assert_eq!(journal, &before);
            assert_eq!(journal.view().pending_mutation, Some(1));
            assert_eq!(journal.view().pending_observation, Some(2));
            assert!(!journal.view().applied);
        }
    });
}

#[test]
fn authority_attempt_request_context_and_actual_target_drift_are_typed_denials() {
    with_observation(|plan, upload, list, journal| {
        let request =
            IcSnapshotUploadObservationRequest::new(plan, 7, journal, upload, list).unwrap();
        for fault in ObservationFault::ALL {
            let mut input = observation_input(&request);
            fault.apply(
                &mut input,
                ArtifactChecksumRecord::from_bytes(b"other authority"),
                ArtifactChecksumRecord::from_bytes(b"other list"),
            );
            let response = IcObservationResponse::new(input).unwrap();
            let error = validate_response(&request, journal, &response).unwrap_err();
            fault.assert_error(&error);
        }
    });
}

#[test]
fn malformed_duplicate_and_excess_inventory_replies_reuse_bounded_decoder() {
    with_observation(|plan, upload, list, journal| {
        let request =
            IcSnapshotUploadObservationRequest::new(plan, 7, journal, upload, list).unwrap();
        let snapshot = ic_platform::management_canister_types::Snapshot {
            id: vec![0, 255],
            taken_at_timestamp: 0,
            total_size: 0,
        };
        let replies = [
            vec![],
            b"DIDL\0\0".to_vec(),
            b"invalid".to_vec(),
            contract::encode_one(vec![snapshot.clone(), snapshot]).unwrap(),
            contract::encode_one(
                (0u64..1025)
                    .map(|id| ic_platform::management_canister_types::Snapshot {
                        id: id.to_be_bytes().to_vec(),
                        taken_at_timestamp: 0,
                        total_size: 0,
                    })
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
            contract::encode_one(vec![ic_platform::management_canister_types::Snapshot {
                id: vec![0; 257],
                taken_at_timestamp: 0,
                total_size: 0,
            }])
            .unwrap(),
        ];
        for raw in replies {
            let mut input = observation_input(&request);
            input.reply = raw;
            let response = IcObservationResponse::new(input).unwrap();
            assert!(matches!(
                validate_response(&request, journal, &response),
                Err(IcObservationAssociationError::Inventory(_))
            ));
        }
        let mut input = observation_input(&request);
        input.reply = vec![0; MAX_IC_OBSERVATION_REPLY_BYTES + 1];
        assert_eq!(
            IcObservationResponse::new(input).unwrap_err(),
            IcObservationResponseError::ReplyTooLarge
        );
        let response = IcObservationResponse::new(observation_input(&request)).unwrap();
        let empty = AttemptJournalRecord::new(journal.authority().clone());
        assert!(matches!(
            validate_response(&request, &empty, &response),
            Err(IcObservationAssociationError::Reservation(_))
        ));
    });
}
