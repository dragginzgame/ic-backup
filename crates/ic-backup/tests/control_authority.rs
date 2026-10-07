//! Public direct-controller contract and spent local recovery; no IC backend is modeled.

mod support;

use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        control_authority::{
            ControlObservation, ControlObservationInput, ControlObservationRequest, ControllerSet,
        },
        ic_request::{IcManagementMethodRecord, IcManagementRequest, IcManagementRequestRecord},
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{
        AttemptJournalGuard, BackupLayoutGuard, create_operation_plan, read_operation_plan,
    },
    policy::control_authority::{ControlAuthorityError, validate},
    ports::control_authority::{ControlAuthorityProvider, ControlProviderError},
};
use serde_json::json;
use std::fs;

fn hash(pair: &str) -> ArtifactChecksumRecord {
    ArtifactChecksumRecord::from_hash(&pair.repeat(32)).unwrap()
}
fn plan(wire: &IcManagementRequestRecord) -> OperationPlanRecord {
    serde_json::from_value(json!({
        "version": 1,
        "context": {"network": "ab".repeat(32), "caller": "2vxsx-fae", "release": "cd".repeat(32)},
        "inventory": {"version": 1, "targets": [{"canister_id": wire.target(), "parent_canister_id": null, "role": null, "module_hash": null}]},
        "selected_targets": [wire.target()],
        "graph": {"version": 1, "nodes": [{"operation_sequence": 7, "depends_on": []}]},
        "operations": [{"operation_sequence": 7, "target": wire.target(), "request": wire.digest().hash(), "budget": {"mutations": 1, "observations": 1}}],
        "budget": {"mutations": 1, "observations": 1}
    })).unwrap()
}

// Independent local fixture values exercise the public port, never actual controller custody.
struct LocalFixtureProvider {
    context: OperationPlanRecord,
    controllers: ControllerSet,
    failure: Option<ControlProviderError>,
    requests: Vec<ArtifactChecksumRecord>,
}
impl ControlAuthorityProvider for LocalFixtureProvider {
    fn observe_control(
        &mut self,
        request: &ControlObservationRequest<'_>,
    ) -> Result<ControlObservation, ControlProviderError> {
        self.requests.push(request.digest());
        if let Some(error) = self.failure {
            return Err(error);
        }
        ControlObservation::new(ControlObservationInput {
            request: request.digest(),
            context: self.context.context().clone(),
            target: self.context.selected_targets()[0].clone(),
            controllers: self.controllers.clone(),
            evidence: hash("34"),
            remote_observations: 0,
        })
        .map_err(|_| ControlProviderError::Indeterminate)
    }
}

#[test]
fn reopens_spent_original_intent_and_denies_replay_and_revoked_controller_without_reset() {
    let wire = IcManagementRequestRecord::new(IcManagementRequest {
        method: IcManagementMethodRecord::TakeCanisterSnapshot,
        target: "renrk-eyaaa-aaaaa-aaada-cai".into(),
        snapshot_id: None,
    })
    .unwrap();
    let plan = plan(&wire);
    let root = support::temp_root("ic-backup-public-control");
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    create_operation_plan(&layout, &plan).unwrap();
    let plan_bytes = fs::read(root.join("operation-plan.json")).unwrap();
    let authority = plan.attempt_authority(7).unwrap();
    let mut journal = AttemptJournalGuard::create(&layout, authority.clone()).unwrap();
    let mutation = journal.reserve_mutation().unwrap(); // Local durable consumption only.
    let before = ControlObservationRequest::new(&plan, 7, &wire, hash("12"), 0).unwrap();
    let mut provider = LocalFixtureProvider {
        context: plan.clone(),
        controllers: ControllerSet::new(vec!["2VXSX-FAE".into()]).unwrap(),
        failure: None,
        requests: vec![],
    };
    let retained = ControlAuthorityProvider::observe_control(&mut provider, &before).unwrap();
    assert_eq!(
        validate(&before, &retained).unwrap().target(),
        wire.target()
    );
    drop(journal);
    drop(layout);

    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let reopened = read_operation_plan(&layout, &plan.digest()).unwrap();
    let journal = AttemptJournalGuard::open(&layout, &authority).unwrap();
    let fresh = ControlObservationRequest::new(&reopened, 7, &wire, hash("56"), 0).unwrap();
    assert_eq!(
        validate(&fresh, &retained).unwrap_err(),
        ControlAuthorityError::RequestMismatch
    );
    assert_eq!(provider.requests.len(), 1); // Pure rejection invokes no provider.
    provider.controllers = ControllerSet::new(vec!["aaaaa-aa".into()]).unwrap();
    let revoked = ControlAuthorityProvider::observe_control(&mut provider, &fresh).unwrap();
    assert_eq!(
        validate(&fresh, &revoked).unwrap_err(),
        ControlAuthorityError::CallerNotController
    );
    for failure in [
        ControlProviderError::Unavailable,
        ControlProviderError::Unsupported,
        ControlProviderError::Indeterminate,
    ] {
        provider.failure = Some(failure);
        assert_eq!(
            ControlAuthorityProvider::observe_control(&mut provider, &fresh).unwrap_err(),
            failure
        );
        let spent = journal.record().unwrap().view();
        assert_eq!(spent.pending_mutation, Some(mutation));
        assert_eq!(spent.mutations_used, 1);
        assert_eq!(spent.mutations_remaining, 0);
        assert_eq!(spent.observations_used, 0);
    }
    assert_eq!(reopened.attempt_authority(7).unwrap(), authority);
    assert_eq!(
        fs::read(root.join("operation-plan.json")).unwrap(),
        plan_bytes
    );
    drop(journal);
    drop(layout);
    fs::remove_dir_all(root).unwrap(); // Successful local fixture only; failures retain evidence.
}
