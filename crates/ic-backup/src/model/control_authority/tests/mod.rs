//! Canonical bounded controller identities; no current authority is inferred.

use super::*;

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
        .map(|id| ic_platform::Principal::from_slice(&id.to_be_bytes()).to_text())
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
fn exact_original_caller_membership_does_not_delegate_to_other_controllers() {
    let plan = crate::test_support::operation_plan::plan();
    let binding = plan.operation_binding(7).unwrap();
    assert!(
        ControllerSet::new(vec![binding.caller().to_uppercase()])
            .unwrap()
            .contains_caller(&binding)
    );
    assert!(
        !ControllerSet::new(vec!["aaaaa-aa".into()])
            .unwrap()
            .contains_caller(&binding)
    );
    assert!(
        !ControllerSet::new(vec![])
            .unwrap()
            .contains_caller(&binding)
    );
}
