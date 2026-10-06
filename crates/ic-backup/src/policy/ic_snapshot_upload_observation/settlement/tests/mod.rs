use super::*;
use crate::{
    model::{
        attempt_journal::ObservationReceiptRequest,
        ic_request::{IcManagementMethodRecord, IcManagementRequest, IcManagementRequestRecord},
    },
    test_support::ic_snapshot_upload::{SOURCE_ID, TARGET, observation_input, with_observation},
};
use ic_management_canister_types::Snapshot;

fn snapshot(id: &[u8]) -> Snapshot {
    Snapshot {
        id: id.into(),
        taken_at_timestamp: 42,
        total_size: 123,
    }
}
fn response(
    request: &IcSnapshotUploadObservationRequest<'_, '_>,
    snapshots: Vec<Snapshot>,
) -> IcObservationResponse {
    let mut input = observation_input(request);
    input.reply = candid::encode_one(snapshots).unwrap();
    IcObservationResponse::new(input).unwrap()
}
fn claims(
    request: &IcSnapshotUploadObservationRequest<'_, '_>,
    baseline: &IcSnapshotReply<'_>,
    response: &IcObservationResponse,
    attribution: IcSnapshotUploadAttribution,
) -> IcSnapshotUploadSettlement {
    IcSnapshotUploadSettlement {
        authority: request.authority().digest(),
        mutation_attempt: 1,
        observation_attempt: 2,
        challenge: ArtifactChecksumRecord::from_bytes(b"fresh challenge"),
        baseline: baseline.digest(),
        inventory: IcSnapshotReply::decode(request.payload(), &response.input().reply)
            .unwrap()
            .digest(),
        observation_evidence: response.input().evidence.clone(),
        attribution,
        evidence: ArtifactChecksumRecord::from_bytes(b"synthetic local qualification only"),
    }
}
fn applied(id: &[u8]) -> IcSnapshotUploadAttribution {
    IcSnapshotUploadAttribution::Applied {
        snapshot_id: id.into(),
        attribution: ArtifactChecksumRecord::from_bytes(b"exclusive allocation declaration"),
    }
}
fn unresolved() -> IcSnapshotUploadAttribution {
    IcSnapshotUploadAttribution::Unresolved {
        uncertainty: ArtifactChecksumRecord::from_bytes(
            b"settled authenticated uncertainty declaration",
        ),
    }
}

#[test]
fn explicit_attribution_selects_one_of_several_new_ids_and_retains_exact_originals() {
    with_observation(|plan, upload, list, journal| {
        let request =
            IcSnapshotUploadObservationRequest::new(plan, 7, journal, upload, list).unwrap();
        let bytes = candid::encode_one(vec![snapshot(&[1])]).unwrap();
        let baseline = IcSnapshotReply::decode(list, &bytes).unwrap();
        let response = response(
            &request,
            vec![snapshot(&[3]), snapshot(&[1]), snapshot(&[2])],
        );
        let claims = claims(&request, &baseline, &response, applied(&[3]));
        let before = journal.digest();
        let view = validate_settlement(
            &request,
            journal,
            &baseline,
            &response,
            &claims.challenge,
            &claims,
        )
        .unwrap();
        assert_eq!(view.outcome(), ObservationOutcomeRecord::Applied);
        assert_eq!(view.allocated_snapshot().unwrap().id(), &[3]);
        assert_eq!(view.baseline().digest(), baseline.digest());
        assert_eq!(
            view.observation().response().input().evidence,
            claims.observation_evidence
        );
        assert!(std::ptr::eq(view.settlement(), &raw const claims));
        assert_eq!(journal.digest(), before);
        assert_eq!(journal.view().pending_mutation, Some(1));
        assert_eq!(journal.view().pending_observation, Some(2));
    });
}

#[test]
fn zero_one_many_candidates_require_claims_and_explicit_transitions_never_refund() {
    with_observation(|plan, upload, list, journal| {
        let request =
            IcSnapshotUploadObservationRequest::new(plan, 7, journal, upload, list).unwrap();
        let bytes = candid::encode_one(Vec::<Snapshot>::new()).unwrap();
        let baseline = IcSnapshotReply::decode(list, &bytes).unwrap();
        for count in 0..=3_u8 {
            let response = response(&request, (1..=count).map(|id| snapshot(&[id])).collect());
            for attribution in [
                unresolved(),
                IcSnapshotUploadAttribution::NotApplied {
                    exclusion: ArtifactChecksumRecord::from_bytes(
                        b"excludes transient allocation deletion",
                    ),
                },
            ] {
                let claims = claims(&request, &baseline, &response, attribution);
                let view = validate_settlement(
                    &request,
                    journal,
                    &baseline,
                    &response,
                    &claims.challenge,
                    &claims,
                )
                .unwrap();
                assert!(view.allocated_snapshot().is_none());
                assert!(!journal.view().applied);
                let mut retained = journal.clone();
                retained
                    .record_observation(ObservationReceiptRequest {
                        attempt: 2,
                        request: list.digest().hash().into(),
                        outcome: view.outcome(),
                        evidence: claims.evidence.hash().into(),
                    })
                    .unwrap();
                assert_eq!(retained.view().pending_observation, None);
                assert_eq!(
                    retained.view().pending_mutation,
                    if view.outcome() == ObservationOutcomeRecord::Uncertain {
                        Some(1)
                    } else {
                        None
                    }
                );
                assert_eq!(retained.view().mutations_used, 1);
                assert_eq!(retained.view().observations_used, 1);
                assert_eq!(retained.view().mutations_remaining, 0);
                assert_eq!(retained.view().observations_remaining, 0);
                assert!(retained.reserve_mutation().is_err());
                assert!(
                    retained
                        .reserve_observation(1, list.digest().hash())
                        .is_err()
                );
                assert!(matches!(
                    validate_settlement(
                        &request,
                        &retained,
                        &baseline,
                        &response,
                        &claims.challenge,
                        &claims
                    ),
                    Err(IcSnapshotUploadSettlementError::Observation(_))
                ));
            }
        }
    });
}

#[test]
fn applied_requires_new_candidate_and_canonical_destination_bounds() {
    with_observation(|plan, upload, list, journal| {
        let request =
            IcSnapshotUploadObservationRequest::new(plan, 7, journal, upload, list).unwrap();
        let bytes = candid::encode_one(vec![snapshot(&[1])]).unwrap();
        let baseline = IcSnapshotReply::decode(list, &bytes).unwrap();
        let response = response(
            &request,
            vec![
                snapshot(&[1]),
                snapshot(SOURCE_ID),
                snapshot(&[2]),
                snapshot(&[9; 256]),
            ],
        );
        for id in [vec![], vec![9; 257], SOURCE_ID.to_vec()] {
            let claims = claims(&request, &baseline, &response, applied(&id));
            assert!(matches!(
                validate_settlement(
                    &request,
                    journal,
                    &baseline,
                    &response,
                    &claims.challenge,
                    &claims
                ),
                Err(IcSnapshotUploadSettlementError::Destination(_))
            ));
        }
        for id in [&[1][..], &[8][..]] {
            let claims = claims(&request, &baseline, &response, applied(id));
            assert!(matches!(
                validate_settlement(
                    &request,
                    journal,
                    &baseline,
                    &response,
                    &claims.challenge,
                    &claims
                ),
                Err(IcSnapshotUploadSettlementError::NotNewCandidate)
            ));
        }
        let claims = claims(&request, &baseline, &response, applied(&[9; 256]));
        assert_eq!(
            validate_settlement(
                &request,
                journal,
                &baseline,
                &response,
                &claims.challenge,
                &claims
            )
            .unwrap()
            .allocated_snapshot()
            .unwrap()
            .id(),
            &[9; 256]
        );
    });
}

#[test]
fn missing_or_changed_baseline_cannot_support_any_claim() {
    with_observation(|plan, upload, list, journal| {
        let request =
            IcSnapshotUploadObservationRequest::new(plan, 7, journal, upload, list).unwrap();
        let bytes = candid::encode_one(vec![snapshot(&[1])]).unwrap();
        let baseline = IcSnapshotReply::decode(list, &bytes).unwrap();
        let mut changed = snapshot(&[1]);
        changed.total_size += 1;
        for (rows, expected) in [
            (
                vec![snapshot(&[2])],
                SnapshotInventoryDeltaError::LostBaseline,
            ),
            (
                vec![changed],
                SnapshotInventoryDeltaError::ChangedBaselineMetadata,
            ),
        ] {
            let response = response(&request, rows);
            let claims = claims(&request, &baseline, &response, unresolved());
            assert!(
                matches!(validate_settlement(&request, journal, &baseline, &response, &claims.challenge, &claims), Err(IcSnapshotUploadSettlementError::Baseline(actual)) if actual == expected)
            );
        }
        for (method, target, rows, expected) in [
            (
                IcManagementMethodRecord::ListCanisterSnapshots,
                "rrkah-fqaaa-aaaaa-aaaaq-cai",
                vec![snapshot(&[1])],
                SnapshotInventoryDeltaError::TargetMismatch,
            ),
            (
                IcManagementMethodRecord::TakeCanisterSnapshot,
                TARGET,
                vec![snapshot(&[1])],
                SnapshotInventoryDeltaError::WrongInventoryMethod,
            ),
        ] {
            let original = IcManagementRequestRecord::new(IcManagementRequest {
                method,
                target: target.into(),
                snapshot_id: None,
            })
            .unwrap();
            let raw = if method == IcManagementMethodRecord::TakeCanisterSnapshot {
                candid::encode_one(&rows[0]).unwrap()
            } else {
                candid::encode_one(rows).unwrap()
            };
            let baseline = IcSnapshotReply::decode(&original, &raw).unwrap();
            let response = response(&request, vec![snapshot(&[1])]);
            let claims = claims(&request, &baseline, &response, unresolved());
            assert!(
                matches!(validate_settlement(&request, journal, &baseline, &response, &claims.challenge, &claims), Err(IcSnapshotUploadSettlementError::Baseline(actual)) if actual == expected)
            );
        }
    });
}

#[test]
fn every_settlement_identity_and_exact_raw_evidence_is_bound() {
    with_observation(|plan, upload, list, journal| {
        let request =
            IcSnapshotUploadObservationRequest::new(plan, 7, journal, upload, list).unwrap();
        let bytes = candid::encode_one(Vec::<Snapshot>::new()).unwrap();
        let baseline = IcSnapshotReply::decode(list, &bytes).unwrap();
        let response = response(&request, vec![snapshot(&[2]), snapshot(&[1])]);
        let original = claims(&request, &baseline, &response, unresolved());
        let wrong = ArtifactChecksumRecord::from_bytes(b"different identity");
        for index in 0..7 {
            let mut claims = original.clone();
            match index {
                0 => claims.authority = wrong.clone(),
                1 => claims.mutation_attempt += 1,
                2 => claims.observation_attempt += 1,
                3 => claims.challenge = wrong.clone(),
                4 => claims.baseline = wrong.clone(),
                5 => claims.inventory = wrong.clone(),
                _ => claims.observation_evidence = wrong.clone(),
            }
            let result = validate_settlement(
                &request,
                journal,
                &baseline,
                &response,
                &original.challenge,
                &claims,
            );
            assert!(matches!(
                (index, result),
                (0, Err(IcSnapshotUploadSettlementError::AuthorityMismatch))
                    | (1 | 2, Err(IcSnapshotUploadSettlementError::AttemptMismatch))
                    | (3, Err(IcSnapshotUploadSettlementError::ChallengeMismatch))
                    | (4, Err(IcSnapshotUploadSettlementError::BaselineMismatch))
                    | (5, Err(IcSnapshotUploadSettlementError::InventoryMismatch))
                    | (
                        6,
                        Err(IcSnapshotUploadSettlementError::ObservationEvidenceMismatch)
                    )
            ));
        }
        let reordered = response_input_reordered(&request);
        assert!(matches!(
            validate_settlement(
                &request,
                journal,
                &baseline,
                &reordered,
                &original.challenge,
                &original
            ),
            Err(IcSnapshotUploadSettlementError::InventoryMismatch)
        ));
        let mut changed = response.input().clone();
        changed.evidence = wrong;
        let changed = IcObservationResponse::new(changed).unwrap();
        assert!(matches!(
            validate_settlement(
                &request,
                journal,
                &baseline,
                &changed,
                &original.challenge,
                &original
            ),
            Err(IcSnapshotUploadSettlementError::ObservationEvidenceMismatch)
        ));
    });
}
fn response_input_reordered(
    request: &IcSnapshotUploadObservationRequest<'_, '_>,
) -> IcObservationResponse {
    response(request, vec![snapshot(&[1]), snapshot(&[2])])
}

#[test]
fn malformed_or_lost_observation_cannot_become_settled_uncertainty() {
    with_observation(|plan, upload, list, journal| {
        let request =
            IcSnapshotUploadObservationRequest::new(plan, 7, journal, upload, list).unwrap();
        let bytes = candid::encode_one(Vec::<Snapshot>::new()).unwrap();
        let baseline = IcSnapshotReply::decode(list, &bytes).unwrap();
        let valid = response(&request, vec![]);
        let claims = claims(&request, &baseline, &valid, unresolved());
        let before = journal.digest();
        for raw in [
            vec![],
            b"lost reply".to_vec(),
            candid::encode_one(vec![snapshot(&[1]), snapshot(&[1])]).unwrap(),
        ] {
            let mut input = observation_input(&request);
            input.reply = raw;
            let malformed = IcObservationResponse::new(input).unwrap();
            assert!(matches!(
                validate_settlement(
                    &request,
                    journal,
                    &baseline,
                    &malformed,
                    &claims.challenge,
                    &claims
                ),
                Err(IcSnapshotUploadSettlementError::Observation(_))
            ));
            assert_eq!(journal.digest(), before);
            assert_eq!(journal.view().pending_observation, Some(2));
        }
    });
}

#[test]
fn allocation_claim_diagnostics_redact_raw_identifiers() {
    let attribution = applied(b"secret-destination-identity");
    let debug = format!("{attribution:?}");
    assert!(debug.contains("snapshot_id_bytes: 27"));
    assert!(!debug.contains("secret-destination-identity"));
    assert!(!debug.contains("115, 101, 99"));
}
