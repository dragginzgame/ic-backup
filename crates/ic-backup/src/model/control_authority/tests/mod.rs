//! Exact mutation and controller-set admission; no current IC authority is claimed.

use super::*;
use crate::{
    model::ic_request::{IcManagementMethodRecord, IcManagementRequest, IcRequestEffect},
    test_support::{
        control_authority::{input, plan, stop, wires},
        membership::{APP, hash},
    },
};

#[test]
fn normalizes_known_sets_and_rejects_duplicates_malformed_and_excessive_entries() {
    let set = ControllerSet::new(vec!["2VXSX-FAE".into(), "aaaaa-aa".into()]).unwrap();
    assert_eq!(
        set.principals(),
        &["2vxsx-fae".to_owned(), "aaaaa-aa".to_owned()]
    );
    assert_eq!(
        ControllerSet::new(vec![]).unwrap().principals(),
        &[] as &[String]
    );
    assert_eq!(
        ControllerSet::new(vec!["2vxsx-fae".into(), "2VXSX-FAE".into()]).unwrap_err(),
        ControlObservationError::DuplicateController
    );
    for malformed in ["not-a-principal".to_owned(), "x".repeat(64)] {
        assert_eq!(
            ControllerSet::new(vec![malformed]).unwrap_err(),
            ControlObservationError::InvalidPrincipal
        );
    }
    let principals: Vec<_> = (0_u32..10)
        .map(|id| ic_principal::Principal::from_slice(&id.to_be_bytes()).to_text())
        .collect();
    assert_eq!(
        ControllerSet::new(principals.clone())
            .unwrap()
            .principals()
            .len(),
        MAX_CONTROLLERS
    );
    let mut excessive = principals;
    excessive.push("2vxsx-fae".into());
    assert_eq!(
        ControllerSet::new(excessive).unwrap_err(),
        ControlObservationError::TooManyControllers
    );
}

#[test]
fn binds_all_registered_mutations_and_matches_independent_request_golden() {
    for wire in wires() {
        let plan = plan(&wire);
        let request = ControlObservationRequest::new(&plan, 7, &wire, hash("12"), 1).unwrap();
        assert_eq!(
            request.binding(),
            plan.attempt_authority(7).unwrap().binding()
        );
        assert_eq!(request.wire(), &wire);
        assert_eq!(request.challenge(), &hash("12"));
        assert_eq!(request.max_remote_observations(), 1);
        if wire.method() == IcManagementMethodRecord::StopCanister {
            // Independent canonical owner/wire/request binary construction and Python SHA-256.
            assert_eq!(
                plan.digest().hash(),
                "0bc4d5875ff8cffc4420b7acf34288a96ba487199e7c114892839e69effa5830"
            );
            assert_eq!(
                request.digest().hash(),
                "6b10d4d75fb9b8aa9636c8f19fce614b2f8f2976213a295bcf0dadbd819efc84"
            );
        }
    }
}

#[test]
fn rejects_nonmutation_changed_target_payload_unknown_operation_and_excessive_ceiling() {
    let wire = stop();
    let plan = plan(&wire);
    for method in [
        IcManagementMethodRecord::CanisterStatus,
        IcManagementMethodRecord::ListCanisterSnapshots,
    ] {
        let observation = IcManagementRequestRecord::new(IcManagementRequest {
            method,
            target: APP.into(),
            snapshot_id: None,
        })
        .unwrap();
        assert!(matches!(
            ControlObservationRequest::new(&plan, 7, &observation, hash("12"), 1),
            Err(ControlRequestError::Payload(
                IcRequestError::EffectMismatch {
                    expected: IcRequestEffect::Mutation
                }
            ))
        ));
    }
    let other = IcManagementRequestRecord::new(IcManagementRequest {
        method: wire.method(),
        target: "aaaaa-aa".into(),
        snapshot_id: None,
    })
    .unwrap();
    assert!(matches!(
        ControlObservationRequest::new(&plan, 7, &other, hash("12"), 1),
        Err(ControlRequestError::Payload(IcRequestError::TargetMismatch))
    ));
    let changed = wires()
        .into_iter()
        .find(|w| w.method() == IcManagementMethodRecord::StartCanister)
        .unwrap();
    assert!(matches!(
        ControlObservationRequest::new(&plan, 7, &changed, hash("12"), 1),
        Err(ControlRequestError::Payload(IcRequestError::DigestMismatch))
    ));
    assert!(matches!(
        ControlObservationRequest::new(&plan, 99, &wire, hash("12"), 1),
        Err(ControlRequestError::Plan(
            OperationPlanError::UnknownOperation(99)
        ))
    ));
    assert!(matches!(
        ControlObservationRequest::new(&plan, 7, &wire, hash("12"), 1025),
        Err(ControlRequestError::ObservationLimitTooLarge)
    ));
    for limit in [0, MAX_CONTROL_REMOTE_OBSERVATIONS] {
        assert_eq!(
            ControlObservationRequest::new(&plan, 7, &wire, hash("12"), limit)
                .unwrap()
                .max_remote_observations(),
            limit
        );
    }
}

#[test]
fn separates_challenges_ceilings_and_original_allowances_and_canonicalizes_observed_target() {
    let wire = stop();
    let plan = plan(&wire);
    let request = ControlObservationRequest::new(&plan, 7, &wire, hash("12"), 1).unwrap();
    for (challenge, limit) in [(hash("56"), 1), (hash("12"), 2)] {
        assert_ne!(
            request.digest(),
            ControlObservationRequest::new(&plan, 7, &wire, challenge, limit)
                .unwrap()
                .digest()
        );
    }
    let mut value = serde_json::to_value(&plan).unwrap();
    value["budget"]["observations"] = serde_json::json!(3);
    let changed = serde_json::from_value(value).unwrap();
    assert_ne!(
        request.digest(),
        ControlObservationRequest::new(&changed, 7, &wire, hash("12"), 1)
            .unwrap()
            .digest()
    );
    let mut data = input(&request);
    data.target = APP.to_uppercase();
    let observation = ControlObservation::new(data).unwrap();
    assert_eq!(observation.target(), APP);
    let mut bad = input(&request);
    bad.target = "invalid".into();
    assert_eq!(
        ControlObservation::new(bad).unwrap_err(),
        ControlObservationError::InvalidPrincipal
    );
}
