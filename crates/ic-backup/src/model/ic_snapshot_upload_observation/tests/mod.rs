use super::*;
use crate::{
    model::{
        attempt_journal::{ObservationOutcomeRecord, ObservationReceiptRequest},
        ic_request::{IcManagementRequest, IcRequestError},
    },
    test_support::ic_snapshot_upload::{DESTINATION_ID, TARGET, with_observation},
};

#[test]
fn original_metadata_and_list_bind_both_spent_attempts_without_mutation() {
    with_observation(|plan, upload, list, journal| {
        let before = journal.clone();
        let request =
            IcSnapshotUploadObservationRequest::new(plan, 7, journal, upload, list).unwrap();
        assert_eq!(request.plan(), plan);
        assert!(std::ptr::eq(request.mutation(), upload));
        assert_eq!(request.payload(), list);
        assert_eq!(request.authority(), journal.authority());
        assert_eq!(
            (request.mutation_attempt(), request.observation_attempt()),
            (1, 2)
        );
        request.validate_journal(journal).unwrap();
        assert_eq!(journal, &before);
        assert_eq!(journal.view().mutations_remaining, 0);
        assert_eq!(journal.view().observations_remaining, 0);
    });
}

#[test]
fn unsupported_data_status_and_changed_list_target_are_typed_denials() {
    with_observation(|plan, upload, list, journal| {
        let data = IcSnapshotUploadRequest::data(
            upload,
            DESTINATION_ID,
            ic_management_canister_types::SnapshotDataKind::WasmModule { offset: 0, size: 1 },
            &[42],
        )
        .unwrap();
        assert!(matches!(
            IcSnapshotUploadObservationRequest::new(plan, 7, journal, &data, list),
            Err(IcSnapshotUploadObservationError::UnsupportedUpload)
        ));
        for method in [
            IcManagementMethodRecord::CanisterStatus,
            IcManagementMethodRecord::StopCanister,
        ] {
            let other = IcManagementRequestRecord::new(IcManagementRequest {
                method,
                target: TARGET.into(),
                snapshot_id: None,
            })
            .unwrap();
            assert!(matches!(
                IcSnapshotUploadObservationRequest::new(plan, 7, journal, upload, &other),
                Err(IcSnapshotUploadObservationError::UnsupportedObservation)
            ));
        }
        let other = IcManagementRequestRecord::new(IcManagementRequest {
            method: IcManagementMethodRecord::ListCanisterSnapshots,
            target: "aaaaa-aa".into(),
            snapshot_id: None,
        })
        .unwrap();
        assert!(matches!(
            IcSnapshotUploadObservationRequest::new(plan, 7, journal, upload, &other),
            Err(IcSnapshotUploadObservationError::Observation(
                IcObservationRequestError::Payload(IcRequestError::TargetMismatch)
            ))
        ));
    });
}

#[test]
fn original_plan_source_payload_and_allowance_drift_reject() {
    with_observation(|plan, upload, list, journal| {
        assert!(matches!(
            IcSnapshotUploadObservationRequest::new(plan, 99, journal, upload, list),
            Err(IcSnapshotUploadObservationError::Upload(
                IcSnapshotUploadAttemptError::Plan(_)
            ))
        ));
        for change in ["request", "network", "release", "budget"] {
            let mut value = serde_json::to_value(plan).unwrap();
            match change {
                "request" => value["operations"][0]["request"] = serde_json::json!("12".repeat(32)),
                "budget" => {
                    value["operations"][0]["budget"]["observations"] = serde_json::json!(2);
                    value["budget"]["observations"] = serde_json::json!(2);
                }
                field => value["context"][field] = serde_json::json!("12".repeat(32)),
            }
            let changed: OperationPlanRecord = serde_json::from_value(value).unwrap();
            assert!(matches!(
                IcSnapshotUploadObservationRequest::new(&changed, 7, journal, upload, list),
                Err(IcSnapshotUploadObservationError::Upload(
                    IcSnapshotUploadAttemptError::AuthorityMismatch
                ))
            ));
            let mut changed_journal =
                AttemptJournalRecord::new(changed.attempt_authority(7).unwrap());
            changed_journal.reserve_mutation().unwrap();
            changed_journal
                .reserve_observation(1, list.digest().hash())
                .unwrap();
            let result = IcSnapshotUploadObservationRequest::new(
                &changed,
                7,
                &changed_journal,
                upload,
                list,
            );
            match change {
                "request" => assert!(matches!(
                    result,
                    Err(IcSnapshotUploadObservationError::Upload(
                        IcSnapshotUploadAttemptError::PayloadMismatch
                    ))
                )),
                "network" | "release" => assert!(matches!(
                    result,
                    Err(IcSnapshotUploadObservationError::Upload(
                        IcSnapshotUploadAttemptError::SourceContextMismatch
                    ))
                )),
                _ => {
                    result.unwrap();
                }
            }
        }
    });
}

#[test]
fn missing_wrong_and_settled_reservations_retain_original_consumption() {
    with_observation(|plan, upload, list, journal| {
        let mut empty = AttemptJournalRecord::new(journal.authority().clone());
        assert!(matches!(
            IcSnapshotUploadObservationRequest::new(plan, 7, &empty, upload, list),
            Err(IcSnapshotUploadObservationError::Observation(
                IcObservationRequestError::NoPendingMutation
            ))
        ));
        empty.reserve_mutation().unwrap();
        assert!(matches!(
            IcSnapshotUploadObservationRequest::new(plan, 7, &empty, upload, list),
            Err(IcSnapshotUploadObservationError::Observation(
                IcObservationRequestError::NoPendingObservation
            ))
        ));
        empty.reserve_observation(1, &"12".repeat(32)).unwrap();
        assert!(matches!(
            IcSnapshotUploadObservationRequest::new(plan, 7, &empty, upload, list),
            Err(IcSnapshotUploadObservationError::Observation(
                IcObservationRequestError::RequestMismatch
            ))
        ));
        let request =
            IcSnapshotUploadObservationRequest::new(plan, 7, journal, upload, list).unwrap();
        let mut settled = journal.clone();
        settled
            .record_observation(ObservationReceiptRequest {
                attempt: 2,
                request: list.digest().hash().into(),
                outcome: ObservationOutcomeRecord::Uncertain,
                evidence: "12".repeat(32),
            })
            .unwrap();
        assert!(matches!(
            request.validate_journal(&settled),
            Err(IcObservationRequestError::ObservationMismatch)
        ));
        assert_eq!(settled.view().pending_mutation, Some(1));
        assert_eq!(settled.view().observations_remaining, 0);
        assert!(
            settled
                .reserve_observation(1, list.digest().hash())
                .is_err()
        );
        assert!(matches!(
            request.validate_journal(&AttemptJournalRecord::new(journal.authority().clone())),
            Err(IcObservationRequestError::MutationMismatch)
        ));
    });
}
