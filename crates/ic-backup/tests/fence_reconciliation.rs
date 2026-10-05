//! Public reserved acquisition reconciliation and late-reply recovery; no IC/fence backend.

use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{ObservationOutcomeRecord, ObservationReceiptRequest},
        fence_obligation::FenceObligationRecord,
        fence_reconciliation::*,
        operation_plan::OperationPlanRecord,
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
    policy::fence_reconciliation::{FenceReconciliationError, validate},
    ports::fence_reconciliation::{FenceReconciliationProvider, FenceReconciliationProviderError},
};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

fn hash(pair: &str) -> ArtifactChecksumRecord {
    ArtifactChecksumRecord::from_hash(&pair.repeat(32)).unwrap()
}
fn plan() -> OperationPlanRecord {
    let request = ArtifactChecksumRecord::from_bytes(b"native application acquisition fixture");
    serde_json::from_value(json!({
        "version":1,"context":{"network":"ab".repeat(32),"caller":"2vxsx-fae","release":"cd".repeat(32)},
        "inventory":{"version":1,"targets":[{"canister_id":"renrk-eyaaa-aaaaa-aaada-cai","parent_canister_id":null,"role":null,"module_hash":null}]},
        "selected_targets":["renrk-eyaaa-aaaaa-aaada-cai"],"graph":{"version":1,"nodes":[{"operation_sequence":0,"depends_on":[]}]},
        "operations":[{"operation_sequence":0,"target":"renrk-eyaaa-aaaaa-aaada-cai","request":request.hash(),"budget":{"mutations":1,"observations":1}}],"budget":{"mutations":1,"observations":1}
    })).unwrap()
}
fn temp_root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().canonicalize().unwrap().join(format!(
        "ic-backup-public-fence-reconciliation-{}-{nonce}",
        std::process::id()
    ))
}
struct Originals {
    plan: OperationPlanRecord,
    source: OperationPlanRecord,
    requirement: RestoreSafetyRequirementRecord,
    obligation: FenceObligationRecord,
}
impl Originals {
    fn new() -> Self {
        let plan = plan();
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
            plan,
            source,
            requirement,
            obligation,
        }
    }
    fn input<'a>(&'a self, source_layout: &'a BackupLayoutGuard) -> FenceObligationRequirement<'a> {
        FenceObligationRequirement::Restore {
            source_layout,
            source: &self.source,
            requirement: &self.requirement,
        }
    }
}
fn prepare(root: &Path, original: &Originals) {
    fs::create_dir_all(root.join("source")).unwrap();
    fs::create_dir(root.join("restore")).unwrap();
    let layout = BackupLayoutGuard::acquire(&root.join("restore")).unwrap();
    let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
    create_operation_plan(&layout, &original.plan).unwrap();
    create_operation_plan(&source_layout, &original.source).unwrap();
    source_layout
        .retain_restore(
            &root.join("restore/attempt-0.json"),
            original.plan.digest().hash(),
        )
        .unwrap();
    create_restore_safety_requirement(
        &layout,
        &source_layout,
        &original.plan,
        &original.source,
        &original.requirement,
    )
    .unwrap();
    create_fence_obligation(
        &layout,
        &original.plan,
        original.input(&source_layout),
        &original.obligation,
    )
    .unwrap();
    let mut journal =
        AttemptJournalGuard::create(&layout, original.plan.attempt_authority(0).unwrap()).unwrap();
    journal.reserve_mutation().unwrap();
    let intent = FenceReconciliationIntent::new(
        &original.plan,
        &original.obligation,
        journal.record().unwrap(),
        hash("34"),
    )
    .unwrap();
    journal
        .reserve_observation(intent.mutation_attempt(), intent.digest().hash())
        .unwrap();
}

// Passive native evidence only. No fixture invokes an IC call or changes an application fence.
struct LocalEvidenceProvider {
    original: Originals,
    reply: FenceReconciliationEvidence,
    failure: Option<FenceReconciliationProviderError>,
    requests: Vec<(ArtifactChecksumRecord, u32)>,
    late: Option<FenceReconciliationObservation>,
}
impl FenceReconciliationProvider for LocalEvidenceProvider {
    fn observe_fence_acquisition(
        &mut self,
        request: &FenceReconciliationRequest<'_, '_>,
    ) -> Result<FenceReconciliationObservation, FenceReconciliationProviderError> {
        self.requests
            .push((request.digest(), request.observation_attempt()));
        let observation =
            FenceReconciliationObservation::new(FenceReconciliationObservationInput {
                request: request.digest(),
                mutation_attempt: request.intent().mutation_attempt(),
                observation_attempt: request.observation_attempt(),
                context: self.original.plan.context().clone(),
                inventory: self.original.plan.inventory().clone(),
                selected_targets: self.original.plan.selected_targets().into(),
                settlement: self.reply.clone(),
                evidence: hash("ab"),
                remote_observations: 0,
            })
            .unwrap();
        if let Some(failure) = self.failure {
            self.late = Some(observation);
            Err(failure)
        } else {
            Ok(observation)
        }
    }
}
fn fixture_provider(failure: Option<FenceReconciliationProviderError>) -> LocalEvidenceProvider {
    LocalEvidenceProvider {
        original: Originals::new(),
        reply: FenceReconciliationEvidence::Unresolved {
            uncertainty: hash("cd"),
        },
        failure,
        requests: vec![],
        late: None,
    }
}
fn persist_native_claim(
    request: &FenceReconciliationRequest<'_, '_>,
    journal: &mut AttemptJournalGuard<'_>,
    observation: &FenceReconciliationObservation,
) {
    let before = journal.record().unwrap().clone();
    let matched = validate(request, journal.record().unwrap(), observation).unwrap();
    assert_eq!(journal.record().unwrap(), &before); // Pure admission never settles automatically.
    journal
        .record_observation(ObservationReceiptRequest {
            attempt: matched.observation_attempt(),
            request: matched.request().hash().into(),
            outcome: matched.outcome(),
            evidence: matched.evidence().hash().into(),
        })
        .unwrap(); // Native fixture claim only; actual adapters independently qualify receipts.
}
fn assert_retained(
    root: &Path,
    source_layout: &BackupLayoutGuard,
    obligation: &[u8],
    requirement: &[u8],
) {
    assert!(source_layout.has_restore_references().unwrap());
    assert_eq!(
        fs::read(root.join("restore/fence-obligation.json")).unwrap(),
        obligation
    );
    assert_eq!(
        fs::read(root.join("restore/restore-safety-requirement.json")).unwrap(),
        requirement
    );
}

#[test]
fn provider_failures_preserve_exact_pending_spending_and_accept_late_reply_without_redispatch() {
    for failure in [
        FenceReconciliationProviderError::Unavailable,
        FenceReconciliationProviderError::Unsupported,
        FenceReconciliationProviderError::Indeterminate,
    ] {
        let root = temp_root();
        let mut provider = fixture_provider(Some(failure));
        prepare(&root, &provider.original);
        let obligation = fs::read(root.join("restore/fence-obligation.json")).unwrap();
        let requirement = fs::read(root.join("restore/restore-safety-requirement.json")).unwrap();
        let layout = BackupLayoutGuard::acquire(&root.join("restore")).unwrap();
        let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
        let authority = provider.original.plan.attempt_authority(0).unwrap();
        let journal = AttemptJournalGuard::open(&layout, &authority).unwrap();
        let before = fs::read(journal.path()).unwrap();
        let original = Originals::new();
        let intent = FenceReconciliationIntent::new(
            &original.plan,
            &original.obligation,
            journal.record().unwrap(),
            hash("34"),
        )
        .unwrap();
        let request = FenceReconciliationRequest::new(&intent, journal.record().unwrap()).unwrap();
        assert_eq!(
            provider.observe_fence_acquisition(&request).unwrap_err(),
            failure
        );
        assert_eq!(fs::read(journal.path()).unwrap(), before);
        assert_retained(&root, &source_layout, &obligation, &requirement);
        drop(journal);
        drop(layout);
        drop(source_layout);
        recover_late_reply(
            &root,
            &original,
            &mut provider,
            &before,
            &obligation,
            &requirement,
        );
        fs::remove_dir_all(root).unwrap();
    }
}
fn recover_late_reply(
    root: &Path,
    original: &Originals,
    provider: &mut LocalEvidenceProvider,
    before: &[u8],
    obligation: &[u8],
    requirement: &[u8],
) {
    let layout = BackupLayoutGuard::acquire(&root.join("restore")).unwrap();
    let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
    let retained = read_fence_obligation(
        &layout,
        &original.plan,
        original.input(&source_layout),
        &original.obligation.digest(),
    )
    .unwrap();
    let mut journal =
        AttemptJournalGuard::open(&layout, &original.plan.attempt_authority(0).unwrap()).unwrap();
    assert_eq!(fs::read(journal.path()).unwrap(), before);
    let intent = FenceReconciliationIntent::new(
        &original.plan,
        &retained,
        journal.record().unwrap(),
        hash("34"),
    )
    .unwrap();
    let request = FenceReconciliationRequest::new(&intent, journal.record().unwrap()).unwrap();
    let stale = FenceReconciliationIntent::new(
        &original.plan,
        &retained,
        journal.record().unwrap(),
        hash("56"),
    )
    .unwrap();
    assert!(matches!(
        FenceReconciliationRequest::new(&stale, journal.record().unwrap()),
        Err(FenceReconciliationRequestError::RequestMismatch)
    ));
    let view = journal.record().unwrap().view();
    assert_eq!(
        (view.mutations_remaining, view.observations_remaining),
        (0, 0)
    );
    assert_eq!(
        journal.record().unwrap().pending_observation_request(),
        Some(request.digest().hash())
    );
    assert_eq!(provider.requests.len(), 1);
    if provider.failure == Some(FenceReconciliationProviderError::Indeterminate) {
        let late = provider.late.take().unwrap();
        persist_native_claim(&request, &mut journal, &late);
        assert_eq!(journal.record().unwrap().view().pending_observation, None);
        assert_eq!(
            journal.record().unwrap().view().pending_mutation,
            Some(intent.mutation_attempt())
        );
        assert!(matches!(
            validate(&request, journal.record().unwrap(), &late),
            Err(FenceReconciliationError::Reservation(_))
        ));
    }
    assert_eq!(provider.requests.len(), 1); // Late admission did not invoke the provider again.
    assert!(journal.reserve_mutation().is_err());
    assert_retained(root, &source_layout, obligation, requirement);
}

fn native_claim(outcome: ObservationOutcomeRecord) -> FenceReconciliationEvidence {
    match outcome {
        ObservationOutcomeRecord::Applied => FenceReconciliationEvidence::AcquiredRestore {
            fence: Box::new(ic_backup::model::restore_safety::RestoreFenceEvidence {
                state: ic_backup::model::consistency::ApplicationFenceState::Active,
                binding: RestoreFenceBindingRecord {
                    identity: hash("56"),
                    membership_revision: hash("78"),
                    external_obligations_revision: hash("90"),
                },
                whole_selection: hash("de"),
                rewind_independent_custody: hash("ef"),
                external_obligations_and_replay: hash("01"),
                controlled_execution: None,
            }),
            attribution: hash("02"),
        },
        ObservationOutcomeRecord::NotApplied => FenceReconciliationEvidence::NotAcquired {
            exclusion: hash("02"),
        },
        ObservationOutcomeRecord::Uncertain => FenceReconciliationEvidence::Unresolved {
            uncertainty: hash("02"),
        },
    }
}
#[test]
fn every_settled_native_claim_persists_exact_receipt_without_fence_or_source_release() {
    for outcome in [
        ObservationOutcomeRecord::Applied,
        ObservationOutcomeRecord::NotApplied,
        ObservationOutcomeRecord::Uncertain,
    ] {
        let root = temp_root();
        let mut provider = fixture_provider(None);
        provider.reply = native_claim(outcome);
        let original = Originals::new();
        prepare(&root, &original);
        let obligation = fs::read(root.join("restore/fence-obligation.json")).unwrap();
        let requirement = fs::read(root.join("restore/restore-safety-requirement.json")).unwrap();
        let layout = BackupLayoutGuard::acquire(&root.join("restore")).unwrap();
        let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
        let mut journal =
            AttemptJournalGuard::open(&layout, &original.plan.attempt_authority(0).unwrap())
                .unwrap();
        let intent = FenceReconciliationIntent::new(
            &original.plan,
            &original.obligation,
            journal.record().unwrap(),
            hash("34"),
        )
        .unwrap();
        let request = FenceReconciliationRequest::new(&intent, journal.record().unwrap()).unwrap();
        let observation = provider.observe_fence_acquisition(&request).unwrap();
        persist_native_claim(&request, &mut journal, &observation);
        let expected = journal.record().unwrap().clone();
        let journal_bytes = fs::read(journal.path()).unwrap();
        assert_eq!(
            expected.view().applied,
            outcome == ObservationOutcomeRecord::Applied
        );
        assert_eq!(expected.view().pending_observation, None);
        assert_eq!(
            expected.view().pending_mutation.is_some(),
            outcome == ObservationOutcomeRecord::Uncertain
        );
        assert_retained(&root, &source_layout, &obligation, &requirement);
        drop(journal);
        drop(layout);
        drop(source_layout);
        let layout = BackupLayoutGuard::acquire(&root.join("restore")).unwrap();
        let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
        let journal =
            AttemptJournalGuard::open(&layout, &original.plan.attempt_authority(0).unwrap())
                .unwrap();
        assert_eq!(journal.record().unwrap(), &expected);
        assert_eq!(fs::read(journal.path()).unwrap(), journal_bytes);
        assert_eq!(
            (
                expected.view().mutations_remaining,
                expected.view().observations_remaining
            ),
            (0, 0)
        );
        assert_eq!(provider.requests.len(), 1);
        assert_retained(&root, &source_layout, &obligation, &requirement);
        drop(journal);
        drop(layout);
        drop(source_layout);
        fs::remove_dir_all(root).unwrap();
    }
}
