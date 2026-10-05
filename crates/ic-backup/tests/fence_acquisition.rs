//! Native exact update acknowledgement/failure recovery; no application or IC effects.

use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        fence_acquisition::*,
        fence_obligation::FenceObligationRecord,
        fence_reconciliation::{FenceReconciliationIntent, FenceReconciliationRequest},
        operation_plan::OperationPlanRecord,
        restore_references::RestoreReferencesRecord,
        restore_safety::{
            RestoreFenceBindingRecord, RestoreSafetyLaneRecord, RestoreSafetyRequirementRecord,
            RestoreSafetyRequirementRequest,
        },
    },
    ops::persistence::{
        AttemptJournalGuard, BackupLayoutGuard, FenceObligationRequirement,
        create_fence_obligation, create_operation_plan, create_restore_safety_requirement,
        read_fence_obligation,
    },
    policy::fence_acquisition::validate_acknowledgement,
    ports::fence_acquisition::{FenceAcquisitionProvider, FenceAcquisitionProviderError},
};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const TARGET: &str = "renrk-eyaaa-aaaaa-aaada-cai";
fn hash(pair: &str) -> ArtifactChecksumRecord {
    ArtifactChecksumRecord::from_hash(&pair.repeat(32)).unwrap()
}
fn temp_root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().canonicalize().unwrap().join(format!(
        "ic-backup-native-acquisition-{}-{nonce}",
        std::process::id()
    ))
}
struct Originals {
    payload: FenceAcquisitionPayload,
    plan: OperationPlanRecord,
    source: OperationPlanRecord,
    requirement: RestoreSafetyRequirementRecord,
    obligation: FenceObligationRecord,
}
impl Originals {
    fn new() -> Self {
        let payload = FenceAcquisitionPayload::new(
            TARGET,
            "native_acquire",
            b"opaque original application bytes",
        )
        .unwrap();
        let plan: OperationPlanRecord = serde_json::from_value(json!({
            "version":1,"context":{"network":"ab".repeat(32),"caller":"2vxsx-fae","release":"cd".repeat(32)},
            "inventory":{"version":1,"targets":[{"canister_id":TARGET,"parent_canister_id":null,"role":null,"module_hash":null}]},
            "selected_targets":[TARGET],"graph":{"version":1,"nodes":[{"operation_sequence":0,"depends_on":[]}]},
            "operations":[{"operation_sequence":0,"target":TARGET,"request":payload.digest().hash(),"budget":{"mutations":1,"observations":1}}],"budget":{"mutations":1,"observations":1}
        })).unwrap();
        let mut source = serde_json::to_value(&plan).unwrap();
        source["operations"][0]["request"] = json!(hash("ef").hash());
        let source = serde_json::from_value(source).unwrap();
        let requirement = RestoreSafetyRequirementRecord::new(
            &plan,
            &source,
            RestoreSafetyRequirementRequest {
                source_artifacts: hash("12"),
                safety: RestoreSafetyLaneRecord::ApplicationFenced,
                expected_fence: Some(RestoreFenceBindingRecord {
                    identity: hash("56"),
                    membership_revision: hash("78"),
                    external_obligations_revision: hash("90"),
                }),
            },
        )
        .unwrap();
        let obligation =
            FenceObligationRecord::for_restore(&plan, &source, &requirement, 0).unwrap();
        Self {
            payload,
            plan,
            source,
            requirement,
            obligation,
        }
    }
    fn retained<'a>(
        &'a self,
        source_layout: &'a BackupLayoutGuard,
    ) -> FenceObligationRequirement<'a> {
        FenceObligationRequirement::Restore {
            source_layout,
            source: &self.source,
            requirement: &self.requirement,
        }
    }
}

/// Passive native port fixture logs one call and returns association/failure only.
/// It does not implement management/application fencing or qualify an outcome.
struct NativeProvider {
    failure: Option<FenceAcquisitionProviderError>,
    calls: Vec<(String, u32, String, Vec<u8>)>,
}
impl FenceAcquisitionProvider for NativeProvider {
    fn acquire_fence(
        &mut self,
        request: &FenceAcquisitionRequest<'_>,
    ) -> Result<FenceAcquisitionAcknowledgement, FenceAcquisitionProviderError> {
        self.calls.push((
            request.payload().target().into(),
            request.mutation_attempt(),
            request.payload().method().into(),
            request.payload().arguments().into(),
        ));
        if let Some(error) = self.failure {
            return Err(error);
        }
        FenceAcquisitionAcknowledgement::new(
            request.authority().digest(),
            request.mutation_attempt(),
            hash("34"),
        )
        .map_err(|_| FenceAcquisitionProviderError::Indeterminate)
    }
}

#[test]
fn every_native_acknowledgement_and_failure_preserves_pending_originals_through_reopen() {
    for failure in [
        None,
        Some(FenceAcquisitionProviderError::Unavailable),
        Some(FenceAcquisitionProviderError::Unsupported),
        Some(FenceAcquisitionProviderError::Indeterminate),
    ] {
        let originals = Originals::new();
        let root = temp_root();
        fs::create_dir_all(root.join("source")).unwrap();
        fs::create_dir(root.join("restore")).unwrap();
        // Native fixture custody only; real integrations qualify durable original byte retention.
        fs::write(
            root.join("request.arguments"),
            originals.payload.arguments(),
        )
        .unwrap();
        fs::write(root.join("request.method"), originals.payload.method()).unwrap();
        let layout = BackupLayoutGuard::acquire(&root.join("restore")).unwrap();
        let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
        create_operation_plan(&layout, &originals.plan).unwrap();
        create_operation_plan(&source_layout, &originals.source).unwrap();
        source_layout
            .retain_restore(
                &root.join("restore/attempt-0.json"),
                originals.plan.digest().hash(),
            )
            .unwrap();
        create_restore_safety_requirement(
            &layout,
            &source_layout,
            &originals.plan,
            &originals.source,
            &originals.requirement,
        )
        .unwrap();
        create_fence_obligation(
            &layout,
            &originals.plan,
            originals.retained(&source_layout),
            &originals.obligation,
        )
        .unwrap();
        let obligation_bytes = fs::read(root.join("restore/fence-obligation.json")).unwrap();
        let references = source_layout.restore_references().unwrap();
        let mut journal =
            AttemptJournalGuard::create(&layout, originals.plan.attempt_authority(0).unwrap())
                .unwrap();
        let mutation = journal.reserve_mutation().unwrap();
        let bytes = fs::read(journal.path()).unwrap();
        let request = FenceAcquisitionRequest::new(
            &originals.plan,
            &originals.obligation,
            journal.record().unwrap(),
            &originals.payload,
        )
        .unwrap();
        let mut provider = NativeProvider {
            failure,
            calls: vec![],
        };
        let result = provider.acquire_fence(&request);
        if let Some(expected) = failure {
            assert_eq!(result.unwrap_err(), expected);
        } else {
            let acknowledgement = result.unwrap();
            validate_acknowledgement(&request, journal.record().unwrap(), &acknowledgement)
                .unwrap();
        }
        assert_eq!(
            provider.calls,
            vec![(
                TARGET.into(),
                mutation,
                "native_acquire".into(),
                originals.payload.arguments().into()
            )]
        );
        assert_eq!(fs::read(journal.path()).unwrap(), bytes);
        assert_eq!(
            journal.record().unwrap().view().pending_mutation,
            Some(mutation)
        );
        assert!(!journal.record().unwrap().view().applied);
        assert_eq!(
            fs::read(root.join("restore/fence-obligation.json")).unwrap(),
            obligation_bytes
        );
        assert_eq!(source_layout.restore_references().unwrap(), references);
        assert!(journal.reserve_mutation().is_err());
        drop(journal);
        drop(source_layout);
        drop(layout);

        assert_retained_recovery(&root, &originals, mutation, &references, &obligation_bytes);
        // Reconstruction and reconciliation reservation invoke no second acquisition.
        assert_eq!(provider.calls.len(), 1);
        fs::remove_dir_all(root).unwrap();
    }
}

fn assert_retained_recovery(
    root: &Path,
    originals: &Originals,
    mutation: u32,
    references: &RestoreReferencesRecord,
    obligation_bytes: &[u8],
) {
    let layout = BackupLayoutGuard::acquire(&root.join("restore")).unwrap();
    let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
    let retained = read_fence_obligation(
        &layout,
        &originals.plan,
        originals.retained(&source_layout),
        &originals.obligation.digest(),
    )
    .unwrap();
    let mut journal =
        AttemptJournalGuard::open(&layout, &originals.plan.attempt_authority(0).unwrap()).unwrap();
    let method = fs::read_to_string(root.join("request.method")).unwrap();
    let arguments = fs::read(root.join("request.arguments")).unwrap();
    let payload = FenceAcquisitionPayload::new(TARGET, &method, &arguments).unwrap();
    let recovered = FenceAcquisitionRequest::new(
        &originals.plan,
        &retained,
        journal.record().unwrap(),
        &payload,
    )
    .unwrap();
    assert_eq!(recovered.payload().digest(), originals.payload.digest());
    assert_eq!(recovered.mutation_attempt(), mutation);
    assert_eq!(journal.record().unwrap().view().mutations_remaining, 0);
    let intent = FenceReconciliationIntent::new(
        &originals.plan,
        &retained,
        journal.record().unwrap(),
        hash("78"),
    )
    .unwrap();
    let observation = journal
        .reserve_observation(mutation, intent.digest().hash())
        .unwrap();
    let request = FenceReconciliationRequest::new(&intent, journal.record().unwrap()).unwrap();
    assert_eq!(request.observation_attempt(), observation);
    assert!(matches!(
        recovered.validate_journal(journal.record().unwrap()),
        Err(FenceAcquisitionError::ObservationPending)
    ));
    let pending_bytes = fs::read(journal.path()).unwrap();
    drop(journal);
    let journal =
        AttemptJournalGuard::open(&layout, &originals.plan.attempt_authority(0).unwrap()).unwrap();
    let pending = journal.record().unwrap().view();
    assert_eq!(
        (pending.mutations_remaining, pending.observations_remaining),
        (0, 0)
    );
    assert_eq!(pending.pending_mutation, Some(mutation));
    assert_eq!(pending.pending_observation, Some(observation));
    assert_eq!(fs::read(journal.path()).unwrap(), pending_bytes);
    assert_eq!(&source_layout.restore_references().unwrap(), references);
    assert_eq!(
        fs::read(root.join("restore/fence-obligation.json")).unwrap(),
        obligation_bytes
    );
}
