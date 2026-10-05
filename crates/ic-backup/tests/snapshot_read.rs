//! Public snapshot-read binding and lost-observation recovery; no IC backend.

use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        control_authority::ControllerSet,
        ic_request::{IcManagementMethodRecord, IcManagementRequest, IcManagementRequestRecord},
        operation_plan::OperationPlanRecord,
        snapshot_read::{
            SnapshotReadObservation, SnapshotReadObservationInput, SnapshotReadRequest,
            SnapshotViewerSet, SnapshotVisibility,
        },
    },
    ops::persistence::{
        AttemptJournalGuard, BackupLayoutGuard, create_operation_plan, read_operation_plan,
    },
    policy::snapshot_read::{SnapshotReadError, SnapshotReadPath, validate},
    ports::snapshot_read::{SnapshotReadProvider, SnapshotReadProviderError},
};
use serde_json::json;
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

fn hash(pair: &str) -> ArtifactChecksumRecord {
    ArtifactChecksumRecord::from_hash(&pair.repeat(32)).unwrap()
}
fn wire(method: IcManagementMethodRecord) -> IcManagementRequestRecord {
    IcManagementRequestRecord::new(IcManagementRequest {
        method,
        target: "renrk-eyaaa-aaaaa-aaada-cai".into(),
        snapshot_id: None,
    })
    .unwrap()
}
fn plan(mutation: &IcManagementRequestRecord) -> OperationPlanRecord {
    serde_json::from_value(json!({
        "version": 1,
        "context": {"network": "ab".repeat(32), "caller": "2vxsx-fae", "release": "cd".repeat(32)},
        "inventory": {"version": 1, "targets": [{"canister_id": mutation.target(), "parent_canister_id": null, "role": null, "module_hash": null}]},
        "selected_targets": [mutation.target()],
        "graph": {"version": 1, "nodes": [{"operation_sequence": 7, "depends_on": []}]},
        "operations": [{"operation_sequence": 7, "target": mutation.target(), "request": mutation.digest().hash(), "budget": {"mutations": 1, "observations": 1}}],
        "budget": {"mutations": 1, "observations": 1}
    })).unwrap()
}
// Independent local permission values exercise the port, never actual IC visibility.
struct LocalFixtureProvider {
    plan: OperationPlanRecord,
    visibility: SnapshotVisibility,
    failure: Option<SnapshotReadProviderError>,
    requests: Vec<ArtifactChecksumRecord>,
}
impl SnapshotReadProvider for LocalFixtureProvider {
    fn observe_snapshot_read(
        &mut self,
        request: &SnapshotReadRequest<'_>,
    ) -> Result<SnapshotReadObservation, SnapshotReadProviderError> {
        self.requests.push(request.digest());
        if let Some(error) = self.failure {
            return Err(error);
        }
        SnapshotReadObservation::new(SnapshotReadObservationInput {
            request: request.digest(),
            context: self.plan.context().clone(),
            target: self.plan.selected_targets()[0].clone(),
            visibility: self.visibility.clone(),
            controllers: Some(ControllerSet::new(vec!["aaaaa-aa".into()]).unwrap()),
            evidence: hash("34"),
            remote_observations: 0,
        })
        .map_err(|_| SnapshotReadProviderError::Indeterminate)
    }
}

fn temp_root() -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().canonicalize().unwrap().join(format!(
        "ic-backup-public-snapshot-read-{}-{nonce}",
        std::process::id()
    ))
}

#[test]
fn reopens_spent_mutation_and_lost_observation_denies_stale_and_revoked_read_without_retry() {
    let mutation_wire = wire(IcManagementMethodRecord::TakeCanisterSnapshot);
    let read_wire = wire(IcManagementMethodRecord::ListCanisterSnapshots);
    let plan = plan(&mutation_wire);
    let root = temp_root();
    fs::create_dir(&root).unwrap();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    create_operation_plan(&layout, &plan).unwrap();
    let plan_bytes = fs::read(root.join("operation-plan.json")).unwrap();
    let authority = plan.attempt_authority(7).unwrap();
    let mut journal = AttemptJournalGuard::create(&layout, authority.clone()).unwrap();
    let mutation = journal.reserve_mutation().unwrap();
    let observation = journal
        .reserve_observation(mutation, read_wire.digest().hash())
        .unwrap();
    let retained_record = journal.record().unwrap().clone();
    let retained_bytes = serde_json::to_vec(&retained_record).unwrap();
    let before =
        SnapshotReadRequest::new(&plan, 7, &read_wire, &read_wire.digest(), hash("12"), 0).unwrap();
    let mut provider = LocalFixtureProvider {
        plan: plan.clone(),
        visibility: SnapshotVisibility::Public,
        failure: None,
        requests: vec![],
    };
    let retained = SnapshotReadProvider::observe_snapshot_read(&mut provider, &before).unwrap();
    assert_eq!(
        validate(&before, &retained).unwrap().path(),
        SnapshotReadPath::Public
    );
    drop(journal);
    drop(layout);

    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let reopened = read_operation_plan(&layout, &plan.digest()).unwrap();
    let mut journal = AttemptJournalGuard::open(&layout, &authority).unwrap();
    let record = journal.record().unwrap();
    assert_eq!(record, &retained_record);
    // Independent observer intent is read from its retained reservation, not the mutation digest.
    let value = serde_json::to_value(record).unwrap();
    let expected =
        ArtifactChecksumRecord::from_hash(value["events"][1]["request"].as_str().unwrap()).unwrap();
    assert_eq!(expected, read_wire.digest());
    let fresh =
        SnapshotReadRequest::new(&reopened, 7, &read_wire, &expected, hash("56"), 0).unwrap();
    assert_eq!(
        validate(&fresh, &retained).unwrap_err(),
        SnapshotReadError::RequestMismatch
    );
    assert_eq!(provider.requests.len(), 1); // Pure admission performs no port call.
    for visibility in [
        SnapshotVisibility::Controllers,
        SnapshotVisibility::AllowedViewers(
            SnapshotViewerSet::new(vec!["aaaaa-aa".into()]).unwrap(),
        ),
    ] {
        provider.visibility = visibility;
        let revoked = SnapshotReadProvider::observe_snapshot_read(&mut provider, &fresh).unwrap();
        assert_eq!(
            validate(&fresh, &revoked).unwrap_err(),
            SnapshotReadError::CallerCannotRead
        );
    }
    for failure in [
        SnapshotReadProviderError::Unavailable,
        SnapshotReadProviderError::Unsupported,
        SnapshotReadProviderError::Indeterminate,
    ] {
        provider.failure = Some(failure);
        assert_eq!(
            SnapshotReadProvider::observe_snapshot_read(&mut provider, &fresh).unwrap_err(),
            failure
        );
        let spent = journal.record().unwrap().view();
        assert_eq!(spent.pending_mutation, Some(mutation));
        assert_eq!(spent.pending_observation, Some(observation));
        assert_eq!((spent.mutations_used, spent.observations_used), (1, 1));
        assert_eq!(
            (spent.mutations_remaining, spent.observations_remaining),
            (0, 0)
        );
        assert_eq!(
            serde_json::to_vec(&journal.record().unwrap()).unwrap(),
            retained_bytes
        );
    }
    // Permission evidence does not settle the lost reply or authorize a second observation.
    assert!(
        matches!(journal.reserve_observation(mutation, expected.hash()), Err(ic_backup::ops::persistence::AttemptJournalError::Record(ic_backup::model::attempt_journal::AttemptJournalRecordError::ObservationPending { attempt })) if attempt == observation)
    );
    assert_eq!(reopened.attempt_authority(7).unwrap(), authority);
    assert_eq!(
        fs::read(root.join("operation-plan.json")).unwrap(),
        plan_bytes
    );
    drop(journal);
    drop(layout);
    fs::remove_dir_all(root).unwrap(); // Successful fixture only; failures retain local evidence.
}
