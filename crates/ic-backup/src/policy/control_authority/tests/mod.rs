//! Pure controller-membership evidence admission; no fresh permissions or IC effects.

use super::*;
use crate::{
    model::control_authority::ControllerSet,
    test_support::{
        control_authority::{input, plan, stop, wires},
        membership::hash,
    },
};

#[test]
fn admits_exact_registered_mutation_results_without_modifying_original_authority() {
    for wire in wires() {
        let plan = plan(&wire);
        let bytes = serde_json::to_vec(&plan).unwrap();
        let authority = plan.attempt_authority(7).unwrap();
        let request = ControlObservationRequest::new(&plan, 7, &wire, hash("12"), 1).unwrap();
        let observation = ControlObservation::new(input(&request)).unwrap();
        let view = validate(&request, &observation).unwrap();
        assert_eq!(view.request(), &request.digest());
        assert_eq!(view.target(), wire.target());
        assert_eq!(view.controllers(), observation.controllers());
        assert_eq!(view.evidence(), &hash("34"));
        assert_eq!(view.remote_observations(), 1);
        assert_eq!(serde_json::to_vec(&plan).unwrap(), bytes);
        assert_eq!(plan.attempt_authority(7).unwrap(), authority);
    }
}

#[test]
fn denies_empty_or_root_only_sets_and_revoked_caller_without_read_or_proxy_fallback() {
    let wire = stop();
    let plan = plan(&wire);
    let request = ControlObservationRequest::new(&plan, 7, &wire, hash("12"), 1).unwrap();
    let original = ControlObservation::new(input(&request)).unwrap();
    validate(&request, &original).unwrap();
    for controllers in [vec![], vec!["aaaaa-aa".into()], vec![wire.target().into()]] {
        let mut revoked = input(&request);
        revoked.controllers = ControllerSet::new(controllers).unwrap();
        assert_eq!(
            validate(&request, &ControlObservation::new(revoked).unwrap()).unwrap_err(),
            ControlAuthorityError::CallerNotController
        );
    }
}

#[test]
fn rejects_observed_context_target_and_reported_call_mismatches() {
    let wire = stop();
    let plan = plan(&wire);
    let request = ControlObservationRequest::new(&plan, 7, &wire, hash("12"), 1).unwrap();
    for field in ["network", "caller", "release"] {
        let mut data = input(&request);
        let mut value = serde_json::to_value(&data.context).unwrap();
        value[field] = serde_json::json!(if field == "caller" {
            "aaaaa-aa".into()
        } else {
            "56".repeat(32)
        });
        data.context = serde_json::from_value(value).unwrap();
        assert_eq!(
            validate(&request, &ControlObservation::new(data).unwrap()).unwrap_err(),
            ControlAuthorityError::ContextMismatch(field)
        );
    }
    let mut data = input(&request);
    data.target = "aaaaa-aa".into();
    assert_eq!(
        validate(&request, &ControlObservation::new(data).unwrap()).unwrap_err(),
        ControlAuthorityError::TargetMismatch
    );
    let mut data = input(&request);
    data.remote_observations = u32::MAX;
    assert_eq!(
        validate(&request, &ControlObservation::new(data).unwrap()).unwrap_err(),
        ControlAuthorityError::ObservationLimitExceeded {
            limit: 1,
            reported: u32::MAX
        }
    );
}

#[test]
fn rejects_stale_challenge_and_other_intent_and_admits_zero_or_max_reported_calls() {
    let wire = stop();
    let plan = plan(&wire);
    let request = ControlObservationRequest::new(&plan, 7, &wire, hash("12"), 1).unwrap();
    let observation = ControlObservation::new(input(&request)).unwrap();
    let fresh = ControlObservationRequest::new(&plan, 7, &wire, hash("56"), 1).unwrap();
    assert_eq!(
        validate(&fresh, &observation).unwrap_err(),
        ControlAuthorityError::RequestMismatch
    );
    let mut value = serde_json::to_value(&plan).unwrap();
    value["context"]["network"] = serde_json::json!("78".repeat(32));
    let changed = serde_json::from_value(value).unwrap();
    let other = ControlObservationRequest::new(&changed, 7, &wire, hash("12"), 1).unwrap();
    assert_eq!(
        validate(&other, &observation).unwrap_err(),
        ControlAuthorityError::RequestMismatch
    );
    for limit in [0, 1024] {
        let bounded = ControlObservationRequest::new(&plan, 7, &wire, hash("12"), limit).unwrap();
        let mut data = input(&bounded);
        data.remote_observations = limit;
        assert_eq!(
            validate(&bounded, &ControlObservation::new(data).unwrap())
                .unwrap()
                .remote_observations(),
            limit
        );
    }
}
