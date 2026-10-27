use super::*;
use crate::{
    model::{
        attempt_journal::ObservationReceiptRequest,
        ic_lifecycle_reply::IcLifecycleReply,
        ic_observation::{IcObservationRequest, IcObservationResponseInput},
        ic_request::{IcManagementRequest, IcManagementRequestRecord},
    },
    test_support::{
        ic_mutation,
        ic_observation::{self, Original},
        membership::hash,
    },
};
use ic_platform::candid::contract;
use ic_platform::management_canister_types::CanisterStatusType as Status;

fn original(method: Method, observation: Method) -> Original {
    let mut original = ic_mutation::original(method);
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
#[derive(contract::CandidType)]
#[candid_path("ic_platform::candid::contract")]
struct Settings {
    controllers: Vec<contract::Principal>,
}
#[derive(contract::CandidType)]
#[candid_path("ic_platform::candid::contract")]
struct WireStatus {
    status: Status,
    settings: Settings,
    marker: u64,
}
fn input(
    request: &IcObservationRequest<'_>,
    status: Status,
    marker: u64,
) -> IcObservationResponseInput {
    let mut input = ic_observation::input(request);
    input.reply = contract::encode_one(WireStatus {
        status,
        settings: Settings {
            controllers: vec![contract::Principal::anonymous()],
        },
        marker,
    })
    .unwrap();
    input
}
fn settlement(
    request: &IcObservationRequest<'_>,
    response: &IcObservationResponse,
) -> IcLifecycleSettlement {
    IcLifecycleSettlement {
        authority: request.authority().digest(),
        mutation_attempt: request.mutation_attempt(),
        observation_attempt: request.observation_attempt(),
        challenge: hash("56"),
        status: IcLifecycleReply::decode(request.payload(), &response.input().reply)
            .unwrap()
            .digest(),
        observation_evidence: response.input().evidence.clone(),
        attribution: IcLifecycleAttribution::Unresolved {
            uncertainty: hash("78"),
        },
        evidence: hash("90"),
    }
}

#[test]
fn statuses_never_infer_original_outcomes_or_change_spending() {
    for method in [
        Method::StopCanister,
        Method::StartCanister,
        Method::LoadCanisterSnapshot,
    ] {
        let original = original(method, Method::CanisterStatus);
        let before = original.journal.clone();
        let request = request(&original);
        for status in [Status::Stopped, Status::Running, Status::Stopping] {
            let response = IcObservationResponse::new(input(&request, status, 42)).unwrap();
            let mut claim = settlement(&request, &response);
            for (attribution, outcome) in [
                (
                    IcLifecycleAttribution::Applied {
                        attribution: hash("78"),
                    },
                    ObservationOutcomeRecord::Applied,
                ),
                (
                    IcLifecycleAttribution::NotApplied {
                        exclusion: hash("78"),
                    },
                    ObservationOutcomeRecord::NotApplied,
                ),
                (
                    IcLifecycleAttribution::Unresolved {
                        uncertainty: hash("78"),
                    },
                    ObservationOutcomeRecord::Uncertain,
                ),
            ] {
                claim.attribution = attribution;
                let view = validate_lifecycle_settlement(
                    &request,
                    &original.journal,
                    &response,
                    &hash("56"),
                    &claim,
                )
                .unwrap();
                assert_eq!(view.outcome(), outcome);
                assert!(std::ptr::eq(view.settlement(), &raw const claim));
                assert!(std::ptr::eq(
                    view.observation().response(),
                    &raw const response
                ));
                assert_eq!(original.journal, before);
            }
        }
    }
}

#[test]
fn capture_and_inventory_have_no_lifecycle_settlement_lane() {
    for (method, observation) in [
        (Method::TakeCanisterSnapshot, Method::CanisterStatus),
        (Method::StopCanister, Method::ListCanisterSnapshots),
        (Method::StartCanister, Method::ListCanisterSnapshots),
        (Method::LoadCanisterSnapshot, Method::ListCanisterSnapshots),
    ] {
        let original = original(method, observation);
        let request = request(&original);
        let response = IcObservationResponse::new(ic_observation::input(&request)).unwrap();
        let claim = IcLifecycleSettlement {
            authority: request.authority().digest(),
            mutation_attempt: 1,
            observation_attempt: 2,
            challenge: hash("56"),
            status: hash("78"),
            observation_evidence: hash("34"),
            attribution: IcLifecycleAttribution::Unresolved {
                uncertainty: hash("78"),
            },
            evidence: hash("90"),
        };
        assert!(matches!(
            validate_lifecycle_settlement(
                &request,
                &original.journal,
                &response,
                &hash("56"),
                &claim
            ),
            Err(IcLifecycleSettlementError::UnsupportedMethod)
        ));
    }
}

#[test]
fn changed_settlement_identity_challenge_and_exact_evidence_reject() {
    let original = original(Method::LoadCanisterSnapshot, Method::CanisterStatus);
    let request = request(&original);
    let response = IcObservationResponse::new(input(&request, Status::Stopped, 42)).unwrap();
    let initial = settlement(&request, &response);
    let before = original.journal.clone();
    // Simultaneous mismatches preserve authority, attempts, challenge, then evidence order.
    let mut combined = initial.clone();
    combined.authority = hash("12");
    combined.mutation_attempt = 3;
    combined.observation_attempt = 4;
    combined.challenge = hash("12");
    combined.status = hash("12");
    for expected in 0..4 {
        let error = validate_lifecycle_settlement(
            &request,
            &original.journal,
            &response,
            &hash("56"),
            &combined,
        )
        .unwrap_err();
        match expected {
            0 => {
                assert!(matches!(
                    error,
                    IcLifecycleSettlementError::Identity(SettlementIdentityMismatch::Authority)
                ));
                combined.authority = initial.authority.clone();
            }
            1 => {
                assert!(matches!(
                    error,
                    IcLifecycleSettlementError::Identity(SettlementIdentityMismatch::Attempt)
                ));
                combined.mutation_attempt = initial.mutation_attempt;
                combined.observation_attempt = initial.observation_attempt;
            }
            2 => {
                assert!(matches!(
                    error,
                    IcLifecycleSettlementError::Identity(SettlementIdentityMismatch::Challenge)
                ));
                combined.challenge = initial.challenge.clone();
            }
            _ => assert!(matches!(error, IcLifecycleSettlementError::StatusMismatch)),
        }
        assert_eq!(original.journal, before);
    }
    for change in 0..6 {
        let mut claim = initial.clone();
        match change {
            0 => claim.authority = hash("12"),
            1 => claim.mutation_attempt = 3,
            2 => claim.observation_attempt = 3,
            3 => claim.challenge = hash("12"),
            4 => claim.status = hash("12"),
            _ => claim.observation_evidence = hash("12"),
        }
        let error = validate_lifecycle_settlement(
            &request,
            &original.journal,
            &response,
            &hash("56"),
            &claim,
        )
        .unwrap_err();
        assert!(match change {
            0 => matches!(
                error,
                IcLifecycleSettlementError::Identity(SettlementIdentityMismatch::Authority)
            ),
            1..=2 => matches!(
                error,
                IcLifecycleSettlementError::Identity(SettlementIdentityMismatch::Attempt)
            ),
            3 => matches!(
                error,
                IcLifecycleSettlementError::Identity(SettlementIdentityMismatch::Challenge)
            ),
            4 => matches!(error, IcLifecycleSettlementError::StatusMismatch),
            _ => matches!(
                error,
                IcLifecycleSettlementError::ObservationEvidenceMismatch
            ),
        });
    }
    // Equal projected status/controllers cannot substitute for different raw metadata.
    let changed = IcObservationResponse::new(input(&request, Status::Stopped, 43)).unwrap();
    assert!(matches!(
        validate_lifecycle_settlement(&request, &original.journal, &changed, &hash("56"), &initial),
        Err(IcLifecycleSettlementError::StatusMismatch)
    ));
    let mut fields = response.input().clone();
    fields.evidence = hash("12");
    let changed = IcObservationResponse::new(fields).unwrap();
    assert!(matches!(
        validate_lifecycle_settlement(&request, &original.journal, &changed, &hash("56"), &initial),
        Err(IcLifecycleSettlementError::ObservationEvidenceMismatch)
    ));
}

#[test]
fn existing_association_wire_and_current_journal_admission_remain_mandatory() {
    let original = original(Method::StopCanister, Method::CanisterStatus);
    let request = request(&original);
    let response = IcObservationResponse::new(input(&request, Status::Stopped, 42)).unwrap();
    let claim = settlement(&request, &response);
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
        let error = validate_lifecycle_settlement(
            &request,
            &original.journal,
            &changed,
            &hash("56"),
            &claim,
        )
        .unwrap_err();
        let IcLifecycleSettlementError::Observation(error) = error else {
            panic!("existing association denial");
        };
        assert!(match change {
            0 => matches!(error, IcObservationAssociationError::AuthorityMismatch),
            1 => matches!(error, IcObservationAssociationError::RequestMismatch),
            2 => matches!(error, IcObservationAssociationError::TargetMismatch),
            3 => matches!(error, IcObservationAssociationError::AttemptMismatch),
            4 => matches!(error, IcObservationAssociationError::ContextMismatch),
            _ => matches!(error, IcObservationAssociationError::Status(_)),
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
        validate_lifecycle_settlement(&request, &settled, &response, &hash("56"), &claim),
        Err(IcLifecycleSettlementError::Observation(
            IcObservationAssociationError::Reservation(_)
        ))
    ));
    assert_eq!(settled.view().pending_mutation, Some(1));
    assert_eq!(
        settled.view().observations_used,
        original.journal.view().observations_used
    );
}
