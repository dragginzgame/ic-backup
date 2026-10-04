//! Public integration binding and local recovery; no current IC membership is certified.

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
    ports::membership::{MembershipProvider, MembershipProviderError},
};
use serde_json::json;
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

fn hash(pair: &str) -> ArtifactChecksumRecord {
    ArtifactChecksumRecord::from_hash(&pair.repeat(32)).expect("bounded exact digest")
}
fn plan() -> OperationPlanRecord {
    serde_json::from_value(json!({
        "version": 1,
        "context": {"network": "ab".repeat(32), "caller": "2vxsx-fae", "release": "cd".repeat(32)},
        "inventory": {"version": 1, "targets": [{"canister_id": "renrk-eyaaa-aaaaa-aaada-cai", "parent_canister_id": null, "role": null, "module_hash": null}]},
        "selected_targets": ["renrk-eyaaa-aaaaa-aaada-cai"],
        "graph": {"version": 1, "nodes": [{"operation_sequence": 7, "depends_on": []}]},
        "operations": [{"operation_sequence": 7, "target": "renrk-eyaaa-aaaaa-aaada-cai", "request": "ef".repeat(32), "budget": {"mutations": 1, "observations": 1}}],
        "budget": {"mutations": 1, "observations": 1}
    })).expect("original plan")
}

// Fixture data qualifies the public port's exact local binding, not any management backend.
struct LocalFixtureProvider {
    observed: OperationPlanRecord,
    failure: Option<MembershipProviderError>,
    requests: Vec<ArtifactChecksumRecord>,
}
impl MembershipProvider for LocalFixtureProvider {
    fn observe_membership(
        &mut self,
        request: &MembershipObservationRequest<'_>,
    ) -> Result<MembershipObservation, MembershipProviderError> {
        self.requests.push(request.digest());
        if let Some(failure) = self.failure {
            return Err(failure);
        }
        Ok(MembershipObservation {
            request: request.digest(),
            context: self.observed.context().clone(),
            inventory: self.observed.inventory().clone(),
            revision: None,
            evidence: hash("34"),
            remote_observations: 0,
        })
    }
}

#[test]
fn binds_external_provider_to_reopened_original_plan_without_resetting_spent_allowance() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().canonicalize().unwrap().join(format!(
        "ic-backup-public-membership-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&root).unwrap();
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
        failure: None,
        requests: vec![],
    };
    let retained = MembershipProvider::observe_membership(&mut provider, &before).unwrap();
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
    let current = MembershipProvider::observe_membership(&mut provider, &after).unwrap();
    let view = validate(&after, &current).unwrap();
    assert_eq!(view.request(), &after.digest());
    assert_eq!(view.selected_targets(), reopened.selected_targets());
    assert_eq!(view.revision(), None); // No invented uninterrupted membership/fence.
    assert_eq!(view.remote_observations(), 0);
    for failure in [
        MembershipProviderError::Unavailable,
        MembershipProviderError::Unsupported,
        MembershipProviderError::Indeterminate,
    ] {
        provider.failure = Some(failure);
        assert_eq!(
            MembershipProvider::observe_membership(&mut provider, &after).unwrap_err(),
            failure
        );
        let retained = journal.record().unwrap().view();
        assert_eq!(retained.mutations_used, used.mutations_used);
        assert_eq!(retained.observations_used, used.observations_used);
        assert_eq!(retained.pending_mutation, Some(mutation));
        assert_eq!(retained.mutations_remaining, 0);
    }
    assert_eq!(
        fs::read(root.join("operation-plan.json")).unwrap(),
        original_plan_bytes
    );
    assert_eq!(reopened.attempt_authority(7).unwrap(), authority);
    drop(journal);
    drop(layout);
    fs::remove_dir_all(root).unwrap(); // Successful local fixture only.
}
