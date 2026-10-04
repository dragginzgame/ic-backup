//! Current read paths, exact context binding and refusal without IO.

use super::*;
use crate::{
    model::{
        control_authority::ControllerSet,
        operation_plan::{PlanContextRecord, PlanContextRequest},
        snapshot_read::SnapshotViewerSet,
    },
    test_support::{
        control_authority::{plan, stop},
        membership::{APP, hash},
        snapshot_read::{input, list},
    },
};

#[test]
fn admits_public_and_exact_viewers_without_controllers_and_never_infers_root_proxy() {
    let plan = plan(&stop());
    let wire = list();
    let request = SnapshotReadRequest::new(&plan, 7, &wire, &wire.digest(), hash("12"), 1).unwrap();
    for (visibility, path) in [
        (SnapshotVisibility::Public, SnapshotReadPath::Public),
        (
            SnapshotVisibility::AllowedViewers(
                SnapshotViewerSet::new(vec!["2VXSX-FAE".into()]).unwrap(),
            ),
            SnapshotReadPath::AllowedViewer,
        ),
    ] {
        let mut data = input(&request);
        data.visibility = visibility.clone();
        let observation = SnapshotReadObservation::new(data).unwrap();
        let view = validate(&request, &observation).unwrap();
        assert_eq!(view.path(), path);
        assert_eq!(view.target(), APP);
        assert_eq!(view.request(), &request.digest());
        assert_eq!(view.visibility(), &visibility);
        assert_eq!(view.evidence(), &hash("34"));
        assert_eq!(view.remote_observations(), 1);
    }
    for visibility in [
        SnapshotVisibility::Controllers,
        SnapshotVisibility::AllowedViewers(
            SnapshotViewerSet::new(vec!["aaaaa-aa".into()]).unwrap(),
        ),
    ] {
        let mut data = input(&request);
        data.visibility = visibility;
        let unknown = SnapshotReadObservation::new(data.clone()).unwrap();
        assert_eq!(
            validate(&request, &unknown).unwrap_err(),
            SnapshotReadError::ControllersUnobserved
        );
        for controllers in [vec![], vec!["aaaaa-aa".into()]] {
            data.controllers = Some(ControllerSet::new(controllers).unwrap());
            assert_eq!(
                validate(
                    &request,
                    &SnapshotReadObservation::new(data.clone()).unwrap()
                )
                .unwrap_err(),
                SnapshotReadError::CallerCannotRead
            );
        }
    }
}

#[test]
fn actual_caller_controller_can_read_every_visibility_including_empty_viewer_list() {
    let plan = plan(&stop());
    let wire = list();
    let request = SnapshotReadRequest::new(&plan, 7, &wire, &wire.digest(), hash("12"), 1).unwrap();
    for visibility in [
        SnapshotVisibility::Controllers,
        SnapshotVisibility::Public,
        SnapshotVisibility::AllowedViewers(SnapshotViewerSet::new(vec![]).unwrap()),
    ] {
        let mut data = input(&request);
        data.visibility = visibility;
        data.controllers = Some(ControllerSet::new(vec!["2VXSX-FAE".into()]).unwrap());
        let observation = SnapshotReadObservation::new(data).unwrap();
        assert_eq!(
            validate(&request, &observation).unwrap().path(),
            SnapshotReadPath::Controller
        );
    }
}

#[test]
fn rejects_stale_request_actual_context_target_and_excess_call_reporting() {
    let plan = plan(&stop());
    let wire = list();
    let request = SnapshotReadRequest::new(&plan, 7, &wire, &wire.digest(), hash("12"), 1).unwrap();
    let observation = SnapshotReadObservation::new(input(&request)).unwrap();
    let fresh = SnapshotReadRequest::new(&plan, 7, &wire, &wire.digest(), hash("56"), 1).unwrap();
    assert_eq!(
        validate(&fresh, &observation).unwrap_err(),
        SnapshotReadError::RequestMismatch
    );
    for field in ["network", "caller", "release"] {
        let mut data = input(&request);
        data.context = PlanContextRecord::new(&PlanContextRequest {
            network: if field == "network" {
                "56".repeat(32)
            } else {
                plan.context().network().into()
            },
            caller: if field == "caller" {
                "aaaaa-aa".into()
            } else {
                plan.context().caller().into()
            },
            release: if field == "release" {
                "56".repeat(32)
            } else {
                plan.context().release().into()
            },
        })
        .unwrap();
        assert_eq!(
            validate(&request, &SnapshotReadObservation::new(data).unwrap()).unwrap_err(),
            SnapshotReadError::ContextMismatch(field)
        );
    }
    let mut data = input(&request);
    data.target = "aaaaa-aa".into();
    assert_eq!(
        validate(&request, &SnapshotReadObservation::new(data).unwrap()).unwrap_err(),
        SnapshotReadError::TargetMismatch
    );
    let mut data = input(&request);
    data.remote_observations = 2;
    assert_eq!(
        validate(&request, &SnapshotReadObservation::new(data).unwrap()).unwrap_err(),
        SnapshotReadError::ObservationLimitExceeded {
            limit: 1,
            reported: 2
        }
    );
}

#[test]
fn zero_and_max_call_ceilings_do_not_supply_or_replenish_journal_allowance() {
    let plan = plan(&stop());
    let wire = list();
    for limit in [0, 1024] {
        let request =
            SnapshotReadRequest::new(&plan, 7, &wire, &wire.digest(), hash("12"), limit).unwrap();
        let mut data = input(&request);
        data.remote_observations = limit;
        let observation = SnapshotReadObservation::new(data.clone()).unwrap();
        assert_eq!(
            validate(&request, &observation)
                .unwrap()
                .remote_observations(),
            limit
        );
        data.remote_observations = limit + 1;
        assert_eq!(
            validate(&request, &SnapshotReadObservation::new(data).unwrap()).unwrap_err(),
            SnapshotReadError::ObservationLimitExceeded {
                limit,
                reported: limit + 1
            }
        );
        assert_eq!(
            plan.attempt_authority(7).unwrap().budget().observations(),
            1
        );
    }
}
