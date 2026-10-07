//! Native exact obligation and acquisition recovery; no application fence or IC backend.

mod support;

use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{ObservationOutcomeRecord, ObservationReceiptRequest},
        fence_obligation::FenceObligationRecord,
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
    policy::fence_obligation::acquisition_progress,
};
use serde_json::json;
use std::fs;

fn hash(pair: &str) -> ArtifactChecksumRecord {
    ArtifactChecksumRecord::from_hash(&pair.repeat(32)).unwrap()
}
fn plan() -> OperationPlanRecord {
    // An opaque application acquisition request, not an IC management payload.
    let request = ArtifactChecksumRecord::from_bytes(b"native application acquire fixture");
    serde_json::from_value(json!({
        "version":1,"context":{"network":"ab".repeat(32),"caller":"2vxsx-fae","release":"cd".repeat(32)},
        "inventory":{"version":1,"targets":[{"canister_id":"renrk-eyaaa-aaaaa-aaada-cai","parent_canister_id":null,"role":null,"module_hash":null}]},
        "selected_targets":["renrk-eyaaa-aaaaa-aaada-cai"],"graph":{"version":1,"nodes":[{"operation_sequence":0,"depends_on":[]}]},
        "operations":[{"operation_sequence":0,"target":"renrk-eyaaa-aaaaa-aaada-cai","request":request.hash(),"budget":{"mutations":1,"observations":1}}],"budget":{"mutations":1,"observations":1}
    })).unwrap()
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
fn prepare(root: &std::path::Path, original: &Originals) {
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
    let authority = original.plan.attempt_authority(0).unwrap();
    assert!(AttemptJournalGuard::open(&layout, &authority).is_err());
    assert!(!root.join("restore/attempt-0.json").exists());
    let mut journal = AttemptJournalGuard::create(&layout, authority).unwrap();
    let mutation = journal.reserve_mutation().unwrap();
    journal
        .reserve_observation(mutation, hash("34").hash())
        .unwrap();
    // Neither reservation invokes an adapter. The lost reply remains pending.
}
#[test]
fn interrupted_acquisition_and_every_settled_outcome_retain_original_obligation_and_source() {
    let original = Originals::new();
    for outcome in [
        ObservationOutcomeRecord::Applied,
        ObservationOutcomeRecord::NotApplied,
        ObservationOutcomeRecord::Uncertain,
    ] {
        let root = support::temp_root("ic-backup-public-fence");
        prepare(&root, &original);
        let path = root.join("restore/fence-obligation.json");
        let bytes = fs::read(&path).unwrap();
        let layout = BackupLayoutGuard::acquire(&root.join("restore")).unwrap();
        let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
        let reference = source_layout.restore_references().unwrap();
        let retained = read_fence_obligation(
            &layout,
            &original.plan,
            original.input(&source_layout),
            &original.obligation.digest(),
        )
        .unwrap();
        let authority = original.plan.attempt_authority(0).unwrap();
        let mut journal = AttemptJournalGuard::open(&layout, &authority).unwrap();
        let pending =
            acquisition_progress(&original.plan, &retained, journal.record().unwrap()).unwrap();
        assert_eq!(
            (pending.mutations_remaining, pending.observations_remaining),
            (0, 0)
        );
        assert!(pending.pending_mutation.is_some());
        assert!(pending.pending_observation.is_some());
        assert!(journal.reserve_mutation().is_err());
        let before = fs::read(journal.path()).unwrap();
        assert!(
            create_fence_obligation(
                &layout,
                &original.plan,
                original.input(&source_layout),
                &retained
            )
            .is_err()
        );
        assert_eq!(fs::read(journal.path()).unwrap(), before);
        // Passive native evidence only; an actual adapter must independently qualify attribution.
        journal
            .record_observation(ObservationReceiptRequest {
                attempt: pending.pending_observation.unwrap(),
                request: hash("34").hash().into(),
                outcome,
                evidence: hash("ab").hash().into(),
            })
            .unwrap();
        let settled =
            acquisition_progress(&original.plan, &retained, journal.record().unwrap()).unwrap();
        assert_eq!(
            settled.applied,
            outcome == ObservationOutcomeRecord::Applied
        );
        assert_eq!(
            (settled.mutations_remaining, settled.observations_remaining),
            (0, 0)
        );
        assert_eq!(source_layout.restore_references().unwrap(), reference);
        assert_eq!(fs::read(&path).unwrap(), bytes);
        let journal_bytes = fs::read(journal.path()).unwrap();
        drop(journal);
        drop(layout);
        drop(source_layout);
        verify_retention(&root, &original, &bytes, &journal_bytes);
        fs::remove_dir_all(root).unwrap(); // Successful native fixture only.
    }
}
fn verify_retention(
    root: &std::path::Path,
    original: &Originals,
    obligation_bytes: &[u8],
    journal_bytes: &[u8],
) {
    let layout = BackupLayoutGuard::acquire(&root.join("restore")).unwrap();
    let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
    assert!(source_layout.has_restore_references().unwrap());
    let retained = read_fence_obligation(
        &layout,
        &original.plan,
        original.input(&source_layout),
        &original.obligation.digest(),
    )
    .unwrap();
    assert_eq!(retained, original.obligation);
    assert_eq!(
        fs::read(root.join("restore/fence-obligation.json")).unwrap(),
        obligation_bytes
    );
    let journal =
        AttemptJournalGuard::open(&layout, &original.plan.attempt_authority(0).unwrap()).unwrap();
    assert_eq!(fs::read(journal.path()).unwrap(), journal_bytes);
    assert_eq!(journal.record().unwrap().view().mutations_remaining, 0);
}
