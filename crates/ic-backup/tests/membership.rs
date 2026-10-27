//! Public integration binding and local recovery; no current IC membership is certified.

mod support;

use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        membership::{MembershipBoundary, MembershipObservation, MembershipObservationRequest},
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{
        AttemptJournalGuard, BackupLayoutGuard, create_operation_plan, read_operation_plan,
    },
    policy::membership::{MembershipError, validate},
};
use std::fs;

fn hash(pair: &str) -> ArtifactChecksumRecord {
    ArtifactChecksumRecord::from_hash(&pair.repeat(32)).expect("bounded exact digest")
}
fn plan() -> OperationPlanRecord {
    support::single_operation_plan("renrk-eyaaa-aaaaa-aaada-cai", 7, hash("ef").hash())
}

// Fixture data qualifies the public evidence model's exact local binding, not any management backend.
struct LocalFixtureProvider {
    observed: OperationPlanRecord,
    requests: Vec<ArtifactChecksumRecord>,
}
impl LocalFixtureProvider {
    fn observe_membership(
        &mut self,
        request: &MembershipObservationRequest<'_>,
    ) -> MembershipObservation {
        self.requests.push(request.digest());
        MembershipObservation {
            request: request.digest(),
            context: self.observed.context().clone(),
            inventory: self.observed.inventory().clone(),
            revision: None,
            evidence: hash("34"),
            remote_observations: 0,
        }
    }
}

#[test]
fn binds_external_provider_to_reopened_original_plan_without_resetting_spent_allowance() {
    let root = support::temp_root("ic-backup-public-membership");
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let original = plan();
    let intent = original.digest();
    create_operation_plan(&layout, &original).unwrap();
    let original_plan_bytes = fs::read(root.join("operation-plan.json")).unwrap();
    let authority = original.attempt_authority(7).unwrap();
    let mut journal = AttemptJournalGuard::create(&layout, authority.clone()).unwrap();
    let mutation = journal.reserve_mutation().unwrap(); // Durable local consumption; no dispatch.
    let before = MembershipObservationRequest::new(
        &original,
        7,
        hash("12"),
        MembershipBoundary::BeforeEffect,
        0,
    )
    .unwrap();
    let mut provider = LocalFixtureProvider {
        observed: original.clone(),
        requests: vec![],
    };
    let retained = provider.observe_membership(&before);
    assert_eq!(
        validate(&before, &retained).unwrap().inventory(),
        original.inventory()
    );
    let used = journal.record().unwrap().view();
    drop(journal);
    drop(layout);

    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let reopened = read_operation_plan(&layout, &intent).unwrap();
    assert_eq!(reopened, original);
    let journal = AttemptJournalGuard::open(&layout, &authority).unwrap();
    let after = MembershipObservationRequest::new(
        &reopened,
        7,
        hash("56"),
        MembershipBoundary::AfterEffect,
        0,
    )
    .unwrap();
    assert_eq!(
        validate(&after, &retained).unwrap_err(),
        MembershipError::RequestMismatch
    );
    assert_eq!(provider.requests.len(), 1); // Pure rejection made no provider call.
    let current = provider.observe_membership(&after);
    let view = validate(&after, &current).unwrap();
    assert_eq!(view.request(), &after.digest());
    assert_eq!(view.selected_targets(), reopened.selected_targets());
    assert_eq!(view.revision(), None); // No invented uninterrupted membership/fence.
    assert_eq!(view.remote_observations(), 0);
    // Pure admission retains the original spending and obligations.
    let retained = journal.record().unwrap().view();
    assert_eq!(retained.mutations_used, used.mutations_used);
    assert_eq!(retained.observations_used, used.observations_used);
    assert_eq!(retained.pending_mutation, Some(mutation));
    assert_eq!(retained.mutations_remaining, 0);

    assert_eq!(
        fs::read(root.join("operation-plan.json")).unwrap(),
        original_plan_bytes
    );
    assert_eq!(reopened.attempt_authority(7).unwrap(), authority);
    drop(journal);
    drop(layout);
    fs::remove_dir_all(root).unwrap(); // Successful local fixture only.
}
