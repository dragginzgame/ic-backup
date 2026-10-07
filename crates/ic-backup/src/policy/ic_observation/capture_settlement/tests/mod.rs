use super::*;
use crate::{
    model::{
        attempt_journal::ObservationReceiptRequest,
        ic_observation::IcObservationResponseInput,
        ic_request::{IcManagementRequest, IcManagementRequestRecord},
    },
    test_support::{
        ic_mutation,
        ic_observation::{self, Original},
        membership::hash,
    },
};
use ic_management_canister_types::Snapshot;

fn original(mutation: Method, observation: Method) -> Original {
    let mut original = ic_mutation::original(mutation);
    let payload = IcManagementRequestRecord::new(IcManagementRequest {
        method: observation,
        target: original.payload.target().into(),
        snapshot_id: None,
    })
    .unwrap();
    original
        .journal
        .reserve_observation(1, payload.digest().hash())
        .unwrap();
    Original {
        mutation: original.payload,
        payload,
        plan: original.plan,
        journal: original.journal,
    }
}
fn request(original: &Original) -> IcObservationRequest<'_> {
    IcObservationRequest::new(
        &original.plan,
        7,
        &original.journal,
        &original.mutation,
        &original.payload,
    )
    .unwrap()
}
fn snapshot(id: &[u8]) -> Snapshot {
    Snapshot {
        id: id.into(),
        taken_at_timestamp: 42,
        total_size: 123,
    }
}
fn input(
    request: &IcObservationRequest<'_>,
    snapshots: Vec<Snapshot>,
) -> IcObservationResponseInput {
    let mut input = ic_observation::input(request);
    input.reply = candid::encode_one(snapshots).unwrap();
    input
}
fn claims(
    request: &IcObservationRequest<'_>,
    baseline: &IcSnapshotReply<'_>,
    response: &IcObservationResponse,
) -> IcCaptureSettlement {
    IcCaptureSettlement {
        authority: request.authority().digest(),
        mutation_attempt: request.mutation_attempt(),
        observation_attempt: request.observation_attempt(),
        challenge: hash("56"),
        baseline: baseline.digest(),
        inventory: IcSnapshotReply::decode(request.payload(), &response.input().reply)
            .unwrap()
            .digest(),
        observation_evidence: response.input().evidence.clone(),
        attribution: IcCaptureAttribution::Unresolved {
            uncertainty: hash("78"),
        },
        evidence: hash("90"),
    }
}

#[test]
fn explicit_new_id_attribution_preserves_exact_originals_and_spending() {
    let original = original(Method::TakeCanisterSnapshot, Method::ListCanisterSnapshots);
    let request = request(&original);
    let bytes = candid::encode_one(vec![snapshot(&[1])]).unwrap();
    let baseline = IcSnapshotReply::decode(request.payload(), &bytes).unwrap();
    // The attributed candidate need not be a singleton, first or smallest ID.
    let response = IcObservationResponse::new(input(
        &request,
        vec![snapshot(&[3]), snapshot(&[1]), snapshot(&[2])],
    ))
    .unwrap();
    let mut claim = claims(&request, &baseline, &response);
    claim.attribution = IcCaptureAttribution::Applied {
        snapshot_id: vec![3],
        attribution: hash("78"),
    };
    let before = original.journal.clone();
    let view = validate_capture_settlement(
        &request,
        &original.journal,
        &baseline,
        &response,
        &hash("56"),
        &claim,
    )
    .unwrap();
    assert_eq!(view.outcome(), ObservationOutcomeRecord::Applied);
    assert_eq!(view.captured_snapshot().unwrap().id(), &[3]);
    assert_eq!(view.captured_snapshot().unwrap().taken_at_timestamp(), 42);
    assert_eq!(view.captured_snapshot().unwrap().total_size(), 123);
    assert!(std::ptr::eq(view.baseline(), &raw const baseline));
    assert!(std::ptr::eq(
        view.observation().response(),
        &raw const response
    ));
    assert!(std::ptr::eq(view.settlement(), &raw const claim));
    assert_eq!(original.journal, before);
    assert_eq!(original.journal.view().pending_mutation, Some(1));
    assert_eq!(original.journal.view().pending_observation, Some(2));
}

#[test]
fn cardinality_never_infers_outcomes_and_only_new_bounded_ids_can_be_attributed() {
    let original = original(Method::TakeCanisterSnapshot, Method::ListCanisterSnapshots);
    let request = request(&original);
    let bytes = candid::encode_one(vec![snapshot(&[1])]).unwrap();
    let baseline = IcSnapshotReply::decode(request.payload(), &bytes).unwrap();
    for count in 0..=3_u8 {
        let response = IcObservationResponse::new(input(
            &request,
            (1..=count + 1).map(|id| snapshot(&[id])).collect(),
        ))
        .unwrap();
        let mut claim = claims(&request, &baseline, &response);
        for (attribution, outcome) in [
            (
                IcCaptureAttribution::NotApplied {
                    exclusion: hash("78"),
                },
                ObservationOutcomeRecord::NotApplied,
            ),
            (
                IcCaptureAttribution::Unresolved {
                    uncertainty: hash("78"),
                },
                ObservationOutcomeRecord::Uncertain,
            ),
        ] {
            claim.attribution = attribution;
            let before = original.journal.clone();
            let view = validate_capture_settlement(
                &request,
                &original.journal,
                &baseline,
                &response,
                &hash("56"),
                &claim,
            )
            .unwrap();
            assert_eq!(view.outcome(), outcome);
            assert!(view.captured_snapshot().is_none());
            assert_eq!(original.journal, before);
        }
        for id in [vec![], vec![1], vec![99], vec![0; 257]] {
            claim.attribution = IcCaptureAttribution::Applied {
                snapshot_id: id,
                attribution: hash("78"),
            };
            assert!(matches!(
                validate_capture_settlement(
                    &request,
                    &original.journal,
                    &baseline,
                    &response,
                    &hash("56"),
                    &claim
                ),
                Err(IcCaptureSettlementError::NotNewCandidate)
            ));
        }
    }
    let id = vec![255; 256];
    let response =
        IcObservationResponse::new(input(&request, vec![snapshot(&[1]), snapshot(&id)])).unwrap();
    let mut claim = claims(&request, &baseline, &response);
    claim.attribution = IcCaptureAttribution::Applied {
        snapshot_id: id.clone(),
        attribution: hash("78"),
    };
    let view = validate_capture_settlement(
        &request,
        &original.journal,
        &baseline,
        &response,
        &hash("56"),
        &claim,
    )
    .unwrap();
    assert_eq!(view.captured_snapshot().unwrap().id(), id);
}

#[test]
fn changed_identity_challenge_and_exact_raw_inventory_evidence_reject() {
    let original = original(Method::TakeCanisterSnapshot, Method::ListCanisterSnapshots);
    let request = request(&original);
    let bytes = candid::encode_one(vec![snapshot(&[1]), snapshot(&[2])]).unwrap();
    let baseline = IcSnapshotReply::decode(request.payload(), &bytes).unwrap();
    let response = IcObservationResponse::new(input(
        &request,
        vec![snapshot(&[1]), snapshot(&[2]), snapshot(&[3])],
    ))
    .unwrap();
    let initial = claims(&request, &baseline, &response);
    for change in 0..7 {
        let mut claim = initial.clone();
        match change {
            0 => claim.authority = hash("12"),
            1 => claim.mutation_attempt = 3,
            2 => claim.observation_attempt = 3,
            3 => claim.challenge = hash("12"),
            4 => claim.baseline = hash("12"),
            5 => claim.inventory = hash("12"),
            _ => claim.observation_evidence = hash("12"),
        }
        let error = validate_capture_settlement(
            &request,
            &original.journal,
            &baseline,
            &response,
            &hash("56"),
            &claim,
        )
        .unwrap_err();
        assert!(match change {
            0 => matches!(error, IcCaptureSettlementError::AuthorityMismatch),
            1..=2 => matches!(error, IcCaptureSettlementError::AttemptMismatch),
            3 => matches!(error, IcCaptureSettlementError::ChallengeMismatch),
            4 => matches!(error, IcCaptureSettlementError::BaselineMismatch),
            5 => matches!(error, IcCaptureSettlementError::InventoryMismatch),
            _ => matches!(error, IcCaptureSettlementError::ObservationEvidenceMismatch),
        });
    }
    // Equal canonical views cannot substitute for differently ordered raw evidence.
    let bytes = candid::encode_one(vec![snapshot(&[2]), snapshot(&[1])]).unwrap();
    let reordered = IcSnapshotReply::decode(request.payload(), &bytes).unwrap();
    assert_eq!(reordered.snapshots(), baseline.snapshots());
    assert!(matches!(
        validate_capture_settlement(
            &request,
            &original.journal,
            &reordered,
            &response,
            &hash("56"),
            &initial
        ),
        Err(IcCaptureSettlementError::BaselineMismatch)
    ));
    let changed = IcObservationResponse::new(input(
        &request,
        vec![snapshot(&[3]), snapshot(&[2]), snapshot(&[1])],
    ))
    .unwrap();
    assert!(matches!(
        validate_capture_settlement(
            &request,
            &original.journal,
            &baseline,
            &changed,
            &hash("56"),
            &initial
        ),
        Err(IcCaptureSettlementError::InventoryMismatch)
    ));
    let mut changed = response.input().clone();
    changed.evidence = hash("12");
    let changed = IcObservationResponse::new(changed).unwrap();
    assert!(matches!(
        validate_capture_settlement(
            &request,
            &original.journal,
            &baseline,
            &changed,
            &hash("56"),
            &initial
        ),
        Err(IcCaptureSettlementError::ObservationEvidenceMismatch)
    ));
}

#[test]
fn full_closed_baseline_and_original_methods_remain_mandatory() {
    let original = original(Method::TakeCanisterSnapshot, Method::ListCanisterSnapshots);
    let request = request(&original);
    let bytes = candid::encode_one(vec![snapshot(&[1])]).unwrap();
    let baseline = IcSnapshotReply::decode(request.payload(), &bytes).unwrap();
    for change in 0..3 {
        let mut row = snapshot(&[1]);
        let snapshots = match change {
            0 => vec![],
            1 => {
                row.taken_at_timestamp += 1;
                vec![row]
            }
            _ => {
                row.total_size += 1;
                vec![row]
            }
        };
        let response = IcObservationResponse::new(input(&request, snapshots)).unwrap();
        let claim = claims(&request, &baseline, &response);
        let error = validate_capture_settlement(
            &request,
            &original.journal,
            &baseline,
            &response,
            &hash("56"),
            &claim,
        )
        .unwrap_err();
        assert!(match change {
            0 => matches!(
                error,
                IcCaptureSettlementError::Baseline(SnapshotInventoryDeltaError::LostBaseline)
            ),
            _ => matches!(
                error,
                IcCaptureSettlementError::Baseline(
                    SnapshotInventoryDeltaError::ChangedBaselineMetadata
                )
            ),
        });
    }
    for method in [
        Method::StopCanister,
        Method::StartCanister,
        Method::LoadCanisterSnapshot,
    ] {
        let other = self::original(method, Method::ListCanisterSnapshots);
        let other_request = self::request(&other);
        let response =
            IcObservationResponse::new(input(&other_request, vec![snapshot(&[1])])).unwrap();
        let claim = claims(&other_request, &baseline, &response);
        assert!(matches!(
            validate_capture_settlement(
                &other_request,
                &other.journal,
                &baseline,
                &response,
                &hash("56"),
                &claim
            ),
            Err(IcCaptureSettlementError::UnsupportedMethod)
        ));
    }
}

#[test]
fn status_and_foreign_or_singleton_baselines_cannot_replace_original_list_evidence() {
    let original = original(Method::TakeCanisterSnapshot, Method::ListCanisterSnapshots);
    let request = request(&original);
    let bytes = candid::encode_one(Vec::<Snapshot>::new()).unwrap();
    let baseline = IcSnapshotReply::decode(request.payload(), &bytes).unwrap();
    let response = IcObservationResponse::new(input(&request, vec![])).unwrap();
    let claim = claims(&request, &baseline, &response);
    let other = self::original(Method::TakeCanisterSnapshot, Method::CanisterStatus);
    let other_request = self::request(&other);
    let status = IcObservationResponse::new(ic_observation::input(&other_request)).unwrap();
    assert!(matches!(
        validate_capture_settlement(
            &other_request,
            &other.journal,
            &baseline,
            &status,
            &hash("56"),
            &claim
        ),
        Err(IcCaptureSettlementError::UnsupportedMethod)
    ));
    let foreign = IcManagementRequestRecord::new(IcManagementRequest {
        method: Method::ListCanisterSnapshots,
        target: "aaaaa-aa".into(),
        snapshot_id: None,
    })
    .unwrap();
    let foreign = IcSnapshotReply::decode(&foreign, &bytes).unwrap();
    assert!(matches!(
        validate_capture_settlement(
            &request,
            &original.journal,
            &foreign,
            &response,
            &hash("56"),
            &claim
        ),
        Err(IcCaptureSettlementError::Baseline(
            SnapshotInventoryDeltaError::TargetMismatch
        ))
    ));
    let bytes = candid::encode_one(snapshot(&[1])).unwrap();
    let singleton = IcSnapshotReply::decode(request.mutation(), &bytes).unwrap();
    assert!(matches!(
        validate_capture_settlement(
            &request,
            &original.journal,
            &singleton,
            &response,
            &hash("56"),
            &claim
        ),
        Err(IcCaptureSettlementError::Baseline(
            SnapshotInventoryDeltaError::WrongInventoryMethod
        ))
    ));
}

#[test]
fn association_wire_and_current_reservations_cannot_be_bypassed() {
    let original = original(Method::TakeCanisterSnapshot, Method::ListCanisterSnapshots);
    let request = request(&original);
    let bytes = candid::encode_one(Vec::<Snapshot>::new()).unwrap();
    let baseline = IcSnapshotReply::decode(request.payload(), &bytes).unwrap();
    let response = IcObservationResponse::new(input(&request, vec![])).unwrap();
    let claim = claims(&request, &baseline, &response);
    for change in 0..6 {
        let mut fields = response.input().clone();
        match change {
            0 => fields.authority = hash("12"),
            1 => fields.request = hash("12"),
            2 => fields.target = "aaaaa-aa".into(),
            3 => {
                fields.mutation_attempt = 2;
                fields.observation_attempt = 3;
            }
            4 => {
                let mut value = serde_json::to_value(&fields.context).unwrap();
                value["network"] = serde_json::json!(hash("12").hash());
                fields.context = serde_json::from_value(value).unwrap();
            }
            _ => fields.reply = b"DIDL\0\0".to_vec(),
        }
        let changed = IcObservationResponse::new(fields).unwrap();
        let error = validate_capture_settlement(
            &request,
            &original.journal,
            &baseline,
            &changed,
            &hash("56"),
            &claim,
        )
        .unwrap_err();
        let IcCaptureSettlementError::Observation(error) = error else {
            panic!("existing association denial");
        };
        assert!(match change {
            0 => matches!(error, IcObservationAssociationError::AuthorityMismatch),
            1 => matches!(error, IcObservationAssociationError::RequestMismatch),
            2 => matches!(error, IcObservationAssociationError::TargetMismatch),
            3 => matches!(error, IcObservationAssociationError::AttemptMismatch),
            4 => matches!(error, IcObservationAssociationError::ContextMismatch),
            _ => matches!(error, IcObservationAssociationError::Inventory(_)),
        });
    }
    let mut settled = original.journal.clone();
    settled
        .record_observation(ObservationReceiptRequest {
            attempt: 2,
            request: request.payload().digest().hash().into(),
            outcome: ObservationOutcomeRecord::Uncertain,
            evidence: hash("90").hash().into(),
        })
        .unwrap();
    assert!(matches!(
        validate_capture_settlement(
            &request,
            &settled,
            &baseline,
            &response,
            &hash("56"),
            &claim
        ),
        Err(IcCaptureSettlementError::Observation(
            IcObservationAssociationError::Reservation(_)
        ))
    ));
    assert_eq!(settled.view().pending_mutation, Some(1));
    assert_eq!(
        settled.view().observations_used,
        original.journal.view().observations_used
    );
}
