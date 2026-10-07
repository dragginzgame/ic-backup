//! Public local obligation, source-reference and spent-attempt recovery; no IC/fence backend.

mod support;

use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        consistency::ApplicationFenceState,
        ic_request::{IcManagementMethodRecord, IcManagementRequest, IcManagementRequestRecord},
        operation_plan::OperationPlanRecord,
        restore_safety::*,
    },
    ops::persistence::{
        AttemptJournalGuard, BackupLayoutGuard, RestoreSafetyPersistenceError, create_json_durable,
        create_operation_plan, create_restore_safety_requirement, read_operation_plan,
        read_restore_safety_requirement,
    },
    policy::restore_safety::{RestoreSafetyError, validate},
    ports::restore_safety::{RestoreSafetyProvider, RestoreSafetyProviderError},
};
use ic_management_canister_types::CanisterStatusType;
use serde_json::json;
use std::fs;
const APP: &str = "renrk-eyaaa-aaaaa-aaada-cai";
fn hash(pair: &str) -> ArtifactChecksumRecord {
    ArtifactChecksumRecord::from_hash(&pair.repeat(32)).unwrap()
}
fn wire(method: IcManagementMethodRecord) -> IcManagementRequestRecord {
    IcManagementRequestRecord::new(IcManagementRequest {
        method,
        target: APP.into(),
        snapshot_id: (method == IcManagementMethodRecord::LoadCanisterSnapshot)
            .then(|| vec![1, 2, 3]),
    })
    .unwrap()
}
fn plan(wire: &IcManagementRequestRecord) -> OperationPlanRecord {
    serde_json::from_value(json!({
    "version":1,"context":{"network":"ab".repeat(32),"caller":"2vxsx-fae","release":"cd".repeat(32)},"inventory":{"version":1,"targets":[{"canister_id":APP,"parent_canister_id":null,"role":null,"module_hash":null}]},"selected_targets":[APP],"graph":{"version":1,"nodes":[{"operation_sequence":7,"depends_on":[]}]},"operations":[{"operation_sequence":7,"target":APP,"request":wire.digest().hash(),"budget":{"mutations":1,"observations":1}}],"budget":{"mutations":1,"observations":1}
})).unwrap()
}
fn fence_binding() -> RestoreFenceBindingRecord {
    RestoreFenceBindingRecord {
        identity: hash("56"),
        membership_revision: hash("78"),
        external_obligations_revision: hash("90"),
    }
}
fn parameters(challenge: ArtifactChecksumRecord) -> RestoreSafetyRequestInput {
    RestoreSafetyRequestInput {
        operation_sequence: 7,
        challenge,
        max_remote_observations: 0,
    }
}
// Independent passive native fixture data. This never implements management or application fencing.
struct LocalEvidenceFixture {
    plan: OperationPlanRecord,
    source: OperationPlanRecord,
    source_artifacts: ArtifactChecksumRecord,
    fence: RestoreFenceEvidence,
    failure: Option<RestoreSafetyProviderError>,
    requests: Vec<ArtifactChecksumRecord>,
}
impl RestoreSafetyProvider for LocalEvidenceFixture {
    fn observe_restore_safety(
        &mut self,
        request: &RestoreSafetyRequest<'_>,
    ) -> Result<RestoreSafetyObservation, RestoreSafetyProviderError> {
        self.requests.push(request.digest());
        if let Some(error) = self.failure {
            return Err(error);
        }
        RestoreSafetyObservation::new(RestoreSafetyObservationInput {
            request: request.digest(),
            context: self.plan.context().clone(),
            inventory: self.plan.inventory().clone(),
            source_plan_intent: self.source.digest(),
            source_artifacts: self.source_artifacts.clone(),
            targets: vec![TargetRestoreEvidence {
                target: APP.into(),
                state: CanisterStatusType::Stopped,
                lifecycle_evidence: hash("23"),
                restored_acceptance: Some(hash("45")),
            }],
            safety: RestoreSafetyEvidence::ApplicationFenced(Box::new(self.fence.clone())),
            evidence: hash("67"),
            remote_observations: 0,
        })
        .map_err(|_| RestoreSafetyProviderError::Indeterminate)
    }
}
// Native retained originals captured before interruption, independently of the fresh request.
struct OriginalEvidence {
    observation: RestoreSafetyObservation,
    journal: ic_backup::model::attempt_journal::AttemptJournalRecord,
    references: ic_backup::model::restore_references::RestoreReferencesRecord,
    provider: LocalEvidenceFixture,
}
fn prepare_original_evidence(
    root: &std::path::Path,
    original: &OperationPlanRecord,
    source: &OperationPlanRecord,
    requirement: &RestoreSafetyRequirementRecord,
    load: &IcManagementRequestRecord,
) -> OriginalEvidence {
    fs::create_dir(root.join("restore")).unwrap();
    fs::create_dir(root.join("source")).unwrap();
    let layout = BackupLayoutGuard::acquire(&root.join("restore")).unwrap();
    let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
    create_operation_plan(&layout, original).unwrap();
    create_operation_plan(&source_layout, source).unwrap();
    create_restore_safety_requirement(&layout, &source_layout, original, source, requirement)
        .unwrap();
    // Outside-source integration fixture, not a product fence-record schema.
    create_json_durable(
        &root.join("outside-source-obligation.json"),
        &json!({"fence":fence_binding(),"source":source.digest()}),
    )
    .unwrap();
    let authority = original.attempt_authority(7).unwrap();
    source_layout
        .retain_restore(
            &layout.root().join("attempt-7.json"),
            authority.binding().intent(),
        )
        .unwrap();
    let references = source_layout.restore_references().unwrap();
    let mut journal = AttemptJournalGuard::create(&layout, authority).unwrap();
    let mutation = journal.reserve_mutation().unwrap();
    journal
        .reserve_observation(mutation, hash("bc").hash())
        .unwrap();
    let journal_record = journal.record().unwrap().clone();
    let request =
        RestoreSafetyRequest::new(original, source, requirement, load, parameters(hash("12")))
            .unwrap();
    let mut provider = LocalEvidenceFixture {
        plan: original.clone(),
        source: source.clone(),
        source_artifacts: hash("ab"),
        fence: RestoreFenceEvidence {
            state: ApplicationFenceState::Active,
            binding: fence_binding(),
            whole_selection: hash("bc"),
            rewind_independent_custody: hash("cd"),
            external_obligations_and_replay: hash("de"),
            controlled_execution: Some(hash("ef")),
        },
        failure: None,
        requests: vec![],
    };
    let observation = provider.observe_restore_safety(&request).unwrap();
    assert!(validate(&request, &observation).is_ok());
    OriginalEvidence {
        observation,
        journal: journal_record,
        references,
        provider,
    }
}
fn reject_weaker_requirement(
    layout: &BackupLayoutGuard,
    source_layout: &BackupLayoutGuard,
    plan: &OperationPlanRecord,
    source: &OperationPlanRecord,
) {
    let weaker = RestoreSafetyRequirementRecord::new(
        plan,
        source,
        RestoreSafetyRequirementRequest {
            source_artifacts: hash("ab"),
            safety: RestoreSafetyLaneRecord::NoIrreversibleEffects,
            expected_fence: None,
        },
    )
    .unwrap();
    assert!(
        create_restore_safety_requirement(layout, source_layout, plan, source, &weaker).is_err()
    );
    assert!(matches!(
        read_restore_safety_requirement(layout, source_layout, plan, source, &weaker.digest()),
        Err(RestoreSafetyPersistenceError::DigestMismatch)
    ));
}
#[test]
fn reopens_original_source_safety_spending_and_obligations_without_effects_or_reference_release() {
    let root = support::temp_root("ic-backup-public-restore-safety");
    let load = wire(IcManagementMethodRecord::LoadCanisterSnapshot);
    let original = plan(&load);
    let source = plan(&wire(IcManagementMethodRecord::TakeCanisterSnapshot));
    let requirement = RestoreSafetyRequirementRecord::new(
        &original,
        &source,
        RestoreSafetyRequirementRequest {
            source_artifacts: hash("ab"),
            safety: RestoreSafetyLaneRecord::ApplicationFenced,
            expected_fence: Some(fence_binding()),
        },
    )
    .unwrap();
    let mut evidence = prepare_original_evidence(&root, &original, &source, &requirement, &load);
    let obligation = root.join("outside-source-obligation.json");
    let obligation_bytes = fs::read(&obligation).unwrap();
    let requirement_path = root.join("restore/restore-safety-requirement.json");
    let requirement_bytes = fs::read(&requirement_path).unwrap();
    let journal_bytes = fs::read(root.join("restore/attempt-7.json")).unwrap();
    let authority = original.attempt_authority(7).unwrap();
    let layout = BackupLayoutGuard::acquire(&root.join("restore")).unwrap();
    let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
    let reopened = read_operation_plan(&layout, &original.digest()).unwrap();
    let reopened_source = read_operation_plan(&source_layout, &source.digest()).unwrap();
    let retained = read_restore_safety_requirement(
        &layout,
        &source_layout,
        &reopened,
        &reopened_source,
        &requirement.digest(),
    )
    .unwrap();
    let journal = AttemptJournalGuard::open(&layout, &authority).unwrap();
    let fresh = RestoreSafetyRequest::new(
        &reopened,
        &reopened_source,
        &retained,
        &load,
        parameters(hash("34")),
    )
    .unwrap();
    assert_eq!(
        validate(&fresh, &evidence.observation).unwrap_err(),
        RestoreSafetyError::RequestMismatch
    );
    assert_eq!(evidence.provider.requests.len(), 1);
    reject_weaker_requirement(&layout, &source_layout, &reopened, &reopened_source);
    reject_unsafe_fence(&mut evidence.provider, &fresh);
    for error in [
        RestoreSafetyProviderError::Unavailable,
        RestoreSafetyProviderError::Unsupported,
        RestoreSafetyProviderError::Indeterminate,
    ] {
        evidence.provider.failure = Some(error);
        assert_eq!(
            evidence
                .provider
                .observe_restore_safety(&fresh)
                .unwrap_err(),
            error
        );
        let view = journal.record().unwrap().view();
        let original_view = evidence.journal.view();
        assert_eq!(
            (view.pending_mutation, view.pending_observation),
            (
                original_view.pending_mutation,
                original_view.pending_observation
            )
        );
        assert_eq!(
            (view.mutations_remaining, view.observations_remaining),
            (0, 0)
        );
        assert_eq!(journal.record().unwrap(), &evidence.journal);
        assert_eq!(fs::read(journal.path()).unwrap(), journal_bytes);
        assert_eq!(fs::read(&requirement_path).unwrap(), requirement_bytes);
        assert_eq!(fs::read(&obligation).unwrap(), obligation_bytes);
        assert_eq!(
            source_layout.restore_references().unwrap(),
            evidence.references
        );
    }
    assert_eq!(reopened.attempt_authority(7).unwrap(), authority);
    drop(journal);
    drop(layout);
    drop(source_layout);
    assert_reopened_retention(&root, &evidence, &obligation_bytes);
    fs::remove_dir_all(root).unwrap();
}
fn reject_unsafe_fence(provider: &mut LocalEvidenceFixture, request: &RestoreSafetyRequest<'_>) {
    provider.fence.state = ApplicationFenceState::Inactive;
    let inactive = provider.observe_restore_safety(request).unwrap();
    assert_eq!(
        validate(request, &inactive).unwrap_err(),
        RestoreSafetyError::FenceNotActive
    );
    provider.fence.state = ApplicationFenceState::Active;
    provider.fence.binding.external_obligations_revision = hash("34");
    let rebound = provider.observe_restore_safety(request).unwrap();
    assert_eq!(
        validate(request, &rebound).unwrap_err(),
        RestoreSafetyError::ExternalObligationsRevisionMismatch
    );
}
fn assert_reopened_retention(
    root: &std::path::Path,
    evidence: &OriginalEvidence,
    obligation: &[u8],
) {
    let layout = BackupLayoutGuard::acquire(&root.join("restore")).unwrap();
    let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
    let authority = evidence.provider.plan.attempt_authority(7).unwrap();
    let journal = AttemptJournalGuard::open(&layout, &authority).unwrap();
    assert_eq!(journal.record().unwrap(), &evidence.journal);
    assert_eq!(
        source_layout.restore_references().unwrap(),
        evidence.references
    );
    assert_eq!(
        fs::read(root.join("outside-source-obligation.json")).unwrap(),
        obligation
    );
}
