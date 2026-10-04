//! Exact original intent/list payload and bounded current model evidence.

use super::*;
use crate::test_support::{
    control_authority::{plan, stop},
    membership::{APP, hash},
    snapshot_read::{input, list},
};

#[test]
fn canonical_viewer_sets_reject_alias_duplicates_invalid_and_excessive_entries() {
    let set = SnapshotViewerSet::new(vec!["aaaaa-aa".into(), "2VXSX-FAE".into()]).unwrap();
    assert_eq!(
        set.principals(),
        &["2vxsx-fae".to_owned(), "aaaaa-aa".to_owned()]
    );
    assert_eq!(
        SnapshotViewerSet::new(vec![]).unwrap().principals(),
        &[] as &[String]
    );
    assert_eq!(
        SnapshotViewerSet::new(vec!["2vxsx-fae".into(), "2VXSX-FAE".into()]).unwrap_err(),
        SnapshotReadObservationError::DuplicateViewer
    );
    for malformed in ["invalid".into(), "x".repeat(64)] {
        assert_eq!(
            SnapshotViewerSet::new(vec![malformed]).unwrap_err(),
            SnapshotReadObservationError::InvalidPrincipal
        );
    }
    let principals: Vec<_> = (0_u32..10)
        .map(|id| ic_principal::Principal::from_slice(&id.to_be_bytes()).to_text())
        .collect();
    assert_eq!(
        SnapshotViewerSet::new(principals.clone())
            .unwrap()
            .principals()
            .len(),
        MAX_SNAPSHOT_VIEWERS
    );
    let mut excessive = principals;
    excessive.push("2vxsx-fae".into());
    assert_eq!(
        SnapshotViewerSet::new(excessive).unwrap_err(),
        SnapshotReadObservationError::TooManyViewers
    );
}

#[test]
fn binds_original_mutation_and_independent_read_bytes_with_binary_golden() {
    let mutation = stop();
    let plan = plan(&mutation);
    let wire = list();
    let request = SnapshotReadRequest::new(&plan, 7, &wire, &wire.digest(), hash("12"), 1).unwrap();
    assert_eq!(
        request.binding(),
        plan.attempt_authority(7).unwrap().binding()
    );
    assert_eq!(request.binding().request(), mutation.digest().hash());
    assert_ne!(request.binding().request(), wire.digest().hash());
    assert_eq!(request.wire(), &wire);
    assert_eq!(request.challenge(), &hash("12"));
    assert_eq!(request.max_remote_observations(), 1);
    // Independent Python binary construction from prior original-plan and wire goldens.
    assert_eq!(
        plan.digest().hash(),
        "0bc4d5875ff8cffc4420b7acf34288a96ba487199e7c114892839e69effa5830"
    );
    assert_eq!(
        request.digest().hash(),
        "f9de63c724de8c8adf277ac28470693f59e6bdbeaa0171609cf20c1268ec9c11"
    );
}

#[test]
fn rejects_other_registered_methods_wrong_read_hash_target_operation_and_ceiling() {
    let mutation = stop();
    let plan = plan(&mutation);
    let list = list();
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../ic_request/tests/golden.json")).unwrap();
    for case in cases {
        let method: IcManagementMethodRecord =
            serde_json::from_value(case["method"].clone()).unwrap();
        if method == IcManagementMethodRecord::ListCanisterSnapshots {
            continue;
        }
        let wire = IcManagementRequestRecord::new(crate::model::ic_request::IcManagementRequest {
            method,
            target: APP.into(),
            snapshot_id: serde_json::from_value(case["snapshot_id"].clone()).unwrap(),
        })
        .unwrap();
        assert!(matches!(
            SnapshotReadRequest::new(&plan, 7, &wire, &wire.digest(), hash("12"), 1),
            Err(SnapshotReadRequestError::UnsupportedMethod)
        ));
    }
    assert!(matches!(
        SnapshotReadRequest::new(&plan, 7, &list, &mutation.digest(), hash("12"), 1),
        Err(SnapshotReadRequestError::Payload(
            IcRequestError::DigestMismatch
        ))
    ));
    let other = IcManagementRequestRecord::new(crate::model::ic_request::IcManagementRequest {
        method: list.method(),
        target: "aaaaa-aa".into(),
        snapshot_id: None,
    })
    .unwrap();
    assert!(matches!(
        SnapshotReadRequest::new(&plan, 7, &other, &other.digest(), hash("12"), 1),
        Err(SnapshotReadRequestError::Payload(
            IcRequestError::TargetMismatch
        ))
    ));
    assert!(matches!(
        SnapshotReadRequest::new(&plan, 99, &list, &list.digest(), hash("12"), 1),
        Err(SnapshotReadRequestError::Plan(
            OperationPlanError::UnknownOperation(99)
        ))
    ));
    assert!(matches!(
        SnapshotReadRequest::new(&plan, 7, &list, &list.digest(), hash("12"), 1025),
        Err(SnapshotReadRequestError::ObservationLimitTooLarge)
    ));
    for limit in [0, MAX_SNAPSHOT_READ_REMOTE_OBSERVATIONS] {
        assert_eq!(
            SnapshotReadRequest::new(&plan, 7, &list, &list.digest(), hash("12"), limit)
                .unwrap()
                .max_remote_observations(),
            limit
        );
    }
}

#[test]
fn separates_challenges_ceilings_original_allowances_and_observed_target_normalization() {
    let plan = plan(&stop());
    let wire = list();
    let request = SnapshotReadRequest::new(&plan, 7, &wire, &wire.digest(), hash("12"), 1).unwrap();
    for (challenge, limit) in [(hash("56"), 1), (hash("12"), 2)] {
        assert_ne!(
            request.digest(),
            SnapshotReadRequest::new(&plan, 7, &wire, &wire.digest(), challenge, limit)
                .unwrap()
                .digest()
        );
    }
    let mut value = serde_json::to_value(&plan).unwrap();
    value["budget"]["observations"] = serde_json::json!(3);
    let changed = serde_json::from_value(value).unwrap();
    assert_ne!(
        request.digest(),
        SnapshotReadRequest::new(&changed, 7, &wire, &wire.digest(), hash("12"), 1)
            .unwrap()
            .digest()
    );
    let mut data = input(&request);
    data.target = APP.to_uppercase();
    assert_eq!(SnapshotReadObservation::new(data).unwrap().target(), APP);
    let mut data = input(&request);
    data.target = "invalid".into();
    assert_eq!(
        SnapshotReadObservation::new(data).unwrap_err(),
        SnapshotReadObservationError::InvalidPrincipal
    );
}
