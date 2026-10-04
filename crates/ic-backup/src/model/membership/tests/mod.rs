//! Fresh identity/bound qualification; challenges remain integration-owned.

use super::*;
use crate::test_support::membership::{APP, hash, plan};

#[test]
fn derives_original_binding_and_matches_independent_request_digest() {
    let plan = plan();
    assert_eq!(
        plan.digest().hash(),
        "5385eead7d20b830b36459b1e34edfef024f319b76477d0217a370a27481017a"
    );
    let request = MembershipObservationRequest::new(
        &plan,
        7,
        hash("12"),
        MembershipBoundary::BeforeEffect,
        2,
    )
    .expect("original intent binding");
    // Independent Python SHA-256 over the documented binary fields, not this codec.
    assert_eq!(
        request.digest().hash(),
        "cf998adb6201b5b2fc398c536c73762da4cd2e48411b7d2d6aa698fb56ffa28c"
    );
    assert_eq!(
        request.binding(),
        plan.attempt_authority(7).unwrap().binding()
    );
    assert_eq!(request.inventory(), plan.inventory());
    assert_eq!(request.selected_targets(), &[APP.to_owned()]);
    assert_eq!(request.challenge(), &hash("12"));
    assert_eq!(request.boundary(), MembershipBoundary::BeforeEffect);
    assert_eq!(request.max_remote_observations(), 2);
    assert_eq!(plan.operation(7).unwrap().budget().observations(), 1);
    // A descriptive provider limit does not replace or consume journal authority.
    assert_eq!(
        plan.attempt_authority(7).unwrap().budget().observations(),
        1
    );
}

#[test]
fn separates_challenges_boundaries_operations_and_call_ceilings() {
    let plan = plan();
    let request = |sequence, challenge, boundary, limit| {
        MembershipObservationRequest::new(&plan, sequence, hash(challenge), boundary, limit)
            .expect("bounded check")
    };
    let base = request(7, "12", MembershipBoundary::BeforeEffect, 2).digest();
    for changed in [
        request(0, "12", MembershipBoundary::BeforeEffect, 2),
        request(7, "34", MembershipBoundary::BeforeEffect, 2),
        request(7, "12", MembershipBoundary::AfterEffect, 2),
        request(7, "12", MembershipBoundary::BeforeEffect, 1),
    ] {
        assert_ne!(base, changed.digest());
    }
    let alias = ArtifactChecksumRecord::from_hash(&"AB".repeat(32)).unwrap();
    assert_eq!(
        request(7, "ab", MembershipBoundary::BeforeEffect, 2).digest(),
        MembershipObservationRequest::new(&plan, 7, alias, MembershipBoundary::BeforeEffect, 2)
            .unwrap()
            .digest()
    );
    let mut value = serde_json::to_value(&plan).unwrap();
    for (pointer, replacement) in [
        ("/context/network", serde_json::json!("23".repeat(32))),
        ("/context/caller", serde_json::json!("aaaaa-aa")),
        ("/context/release", serde_json::json!("45".repeat(32))),
        (
            "/inventory/targets/0/role",
            serde_json::json!("unselected parent"),
        ),
        ("/operations/1/request", serde_json::json!("67".repeat(32))),
        ("/budget/observations", serde_json::json!(3)),
    ] {
        let old = value.pointer(pointer).unwrap().clone();
        *value.pointer_mut(pointer).unwrap() = replacement;
        let changed: OperationPlanRecord = serde_json::from_value(value.clone()).unwrap();
        assert_ne!(
            base,
            MembershipObservationRequest::new(
                &changed,
                7,
                hash("12"),
                MembershipBoundary::BeforeEffect,
                2
            )
            .unwrap()
            .digest(),
            "{pointer}"
        );
        *value.pointer_mut(pointer).unwrap() = old;
    }
}

#[test]
fn admits_zero_and_maximum_call_ceilings_and_rejects_unknown_operations() {
    let plan = plan();
    for limit in [0, MAX_MEMBERSHIP_REMOTE_OBSERVATIONS] {
        assert_eq!(
            MembershipObservationRequest::new(
                &plan,
                7,
                hash("12"),
                MembershipBoundary::BeforeEffect,
                limit
            )
            .unwrap()
            .max_remote_observations(),
            limit
        );
    }
    assert!(matches!(
        MembershipObservationRequest::new(
            &plan,
            7,
            hash("12"),
            MembershipBoundary::BeforeEffect,
            1025
        ),
        Err(MembershipRequestError::ObservationLimitTooLarge)
    ));
    assert!(matches!(
        MembershipObservationRequest::new(
            &plan,
            99,
            hash("12"),
            MembershipBoundary::BeforeEffect,
            1
        ),
        Err(MembershipRequestError::Plan(
            OperationPlanError::UnknownOperation(99)
        ))
    ));
}
