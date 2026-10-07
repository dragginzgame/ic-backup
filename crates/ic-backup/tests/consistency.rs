//! Public original-guarantee and retained-obligation recovery; no application/IC backend.

mod support;

use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord, consistency::*, operation_plan::OperationPlanRecord,
    },
    ops::persistence::{
        AttemptJournalGuard, BackupLayoutGuard, ConsistencyPersistenceError,
        create_consistency_requirement, create_json_durable, create_operation_plan,
        read_consistency_requirement, read_operation_plan,
    },
    policy::consistency::{ConsistencyError, validate},
    ports::consistency::{ConsistencyProvider, ConsistencyProviderError},
};
use serde_json::json;
use std::fs;

fn hash(pair: &str) -> ArtifactChecksumRecord {
    ArtifactChecksumRecord::from_hash(&pair.repeat(32)).unwrap()
}
fn plan() -> OperationPlanRecord {
    serde_json::from_value(json!({
        "version":1,"context":{"network":"ab".repeat(32),"caller":"2vxsx-fae","release":"cd".repeat(32)},
        "inventory":{"version":1,"targets":[{"canister_id":"renrk-eyaaa-aaaaa-aaada-cai","parent_canister_id":null,"role":null,"module_hash":null}]},
        "selected_targets":["renrk-eyaaa-aaaaa-aaada-cai"],"graph":{"version":1,"nodes":[{"operation_sequence":7,"depends_on":[]}]},
        "operations":[{"operation_sequence":7,"target":"renrk-eyaaa-aaaaa-aaada-cai","request":"01".repeat(32),"budget":{"mutations":1,"observations":1}}],"budget":{"mutations":1,"observations":1}
    })).unwrap()
}
fn parameters(challenge: ArtifactChecksumRecord) -> ConsistencyRequestInput {
    ConsistencyRequestInput {
        operation_sequence: 7,
        challenge,
        boundary: ConsistencyBoundary::BeforeCapture,
        expected_fence: Some(ApplicationFenceBinding {
            identity: hash("56"),
            membership_revision: hash("78"),
        }),
        max_remote_observations: 0,
    }
}
fn fence() -> ApplicationFenceEvidence {
    ApplicationFenceEvidence {
        state: ApplicationFenceState::Active,
        identity: hash("56"),
        membership_revision: hash("78"),
        writes: hash("90"),
        membership: hash("ab"),
        timers: hash("bc"),
        external_work: hash("cd"),
        drained_work: hash("de"),
    }
}
// Independent native input exercises the port; it does not implement management/fencing.
struct LocalFixtureProvider {
    plan: OperationPlanRecord,
    fence: ApplicationFenceEvidence,
    failure: Option<ConsistencyProviderError>,
    requests: Vec<ArtifactChecksumRecord>,
}
impl ConsistencyProvider for LocalFixtureProvider {
    fn observe_consistency(
        &mut self,
        request: &ConsistencyRequest<'_>,
    ) -> Result<ConsistencyObservation, ConsistencyProviderError> {
        self.requests.push(request.digest());
        if let Some(error) = self.failure {
            return Err(error);
        }
        ConsistencyObservation::new(ConsistencyObservationInput {
            request: request.digest(),
            context: self.plan.context().clone(),
            inventory: self.plan.inventory().clone(),
            targets: vec![TargetCaptureEvidence {
                target: self.plan.selected_targets()[0].clone(),
                state: CaptureState::Stopped,
                stopped_and_drained: hash("ef"),
            }],
            membership_revision: Some(hash("78")),
            consistency: ConsistencyEvidence::ApplicationCoordinated(Box::new(self.fence.clone())),
            evidence: hash("34"),
            remote_observations: 0,
        })
        .map_err(|_| ConsistencyProviderError::Indeterminate)
    }
}
fn assert_spent(journal: &AttemptJournalGuard<'_>, mutation: u32, observation: u32) {
    let view = journal.record().unwrap().view();
    assert_eq!(view.pending_mutation, Some(mutation));
    assert_eq!(view.pending_observation, Some(observation));
    assert_eq!((view.mutations_used, view.observations_used), (1, 1));
    assert_eq!(
        (view.mutations_remaining, view.observations_remaining),
        (0, 0)
    );
}
#[test]
fn retains_original_guarantee_and_fence_obligation_across_stale_results_failures_and_reopen() {
    let plan = plan();
    let root = support::temp_root("ic-backup-public-consistency");
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    create_operation_plan(&layout, &plan).unwrap();
    let requirement = ConsistencyRequirementRecord::new(
        &plan,
        ConsistencyGuaranteeRecord::ApplicationCoordinated,
    );
    create_consistency_requirement(&layout, &plan, &requirement).unwrap();
    // Native integration-owned obligation fixture, not a product fence-record schema or receipt.
    let obligation_path = root.join("operator-fence-fixture.json");
    create_json_durable(&obligation_path, &json!({"fence":hash("56").hash()})).unwrap();
    let obligation_bytes = fs::read(&obligation_path).unwrap();
    let requirement_bytes = fs::read(root.join("consistency-requirement.json")).unwrap();
    let authority = plan.attempt_authority(7).unwrap();
    let mut journal = AttemptJournalGuard::create(&layout, authority.clone()).unwrap();
    let mutation = journal.reserve_mutation().unwrap();
    let observation = journal
        .reserve_observation(mutation, hash("ab").hash())
        .unwrap();
    let retained_journal = journal.record().unwrap().clone();
    let before = ConsistencyRequest::new(&plan, &requirement, parameters(hash("12"))).unwrap();
    let mut provider = LocalFixtureProvider {
        plan: plan.clone(),
        fence: fence(),
        failure: None,
        requests: vec![],
    };
    let old = ConsistencyProvider::observe_consistency(&mut provider, &before).unwrap();
    assert_eq!(
        validate(&before, &old).unwrap().fence().unwrap().identity,
        hash("56")
    );
    drop(journal);
    drop(layout);

    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let reopened = read_operation_plan(&layout, &plan.digest()).unwrap();
    let retained = read_consistency_requirement(&layout, &reopened, &requirement.digest()).unwrap();
    let journal = AttemptJournalGuard::open(&layout, &authority).unwrap();
    let fresh = ConsistencyRequest::new(&reopened, &retained, parameters(hash("90"))).unwrap();
    assert_eq!(
        validate(&fresh, &old).unwrap_err(),
        ConsistencyError::RequestMismatch
    );
    assert_eq!(provider.requests.len(), 1); // Pure rejection invokes no provider or release.
    let weaker =
        ConsistencyRequirementRecord::new(&reopened, ConsistencyGuaranteeRecord::PerCanister);
    assert!(create_consistency_requirement(&layout, &reopened, &weaker).is_err());
    assert!(matches!(
        read_consistency_requirement(&layout, &reopened, &weaker.digest()),
        Err(ConsistencyPersistenceError::DigestMismatch)
    ));
    provider.fence.identity = hash("90");
    let wrong = ConsistencyProvider::observe_consistency(&mut provider, &fresh).unwrap();
    assert_eq!(
        validate(&fresh, &wrong).unwrap_err(),
        ConsistencyError::FenceMismatch
    );
    for failure in [
        ConsistencyProviderError::Unavailable,
        ConsistencyProviderError::Unsupported,
        ConsistencyProviderError::Indeterminate,
    ] {
        provider.failure = Some(failure);
        assert_eq!(
            ConsistencyProvider::observe_consistency(&mut provider, &fresh).unwrap_err(),
            failure
        );
        assert_spent(&journal, mutation, observation);
        assert_eq!(journal.record().unwrap(), &retained_journal);
        assert_eq!(fs::read(&obligation_path).unwrap(), obligation_bytes);
        assert_eq!(
            fs::read(root.join("consistency-requirement.json")).unwrap(),
            requirement_bytes
        );
    }
    assert_eq!(reopened.attempt_authority(7).unwrap(), authority);
    drop(journal);
    drop(layout);
    assert_eq!(fs::read(&obligation_path).unwrap(), obligation_bytes); // Dropping custody releases no fixture obligation.
    fs::remove_dir_all(root).unwrap(); // Successful native fixture only; failures retain evidence.
}
