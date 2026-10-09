use super::*;
use crate::{
    model::{
        attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest},
        execution_workflow::{
            ExecutionStagePredecessorRecord,
            tests::{child, workflow},
        },
    },
    ops::persistence::{AttemptJournalGuard, read_execution_progress},
    test_support::{hold_at_acknowledged_barrier, kill_child_at_acknowledged_barrier, temp_dir},
};
use std::os::unix::fs::{PermissionsExt, symlink};
use std::process::Command;

mod ancestry;
mod checkpoint;
mod preparation;
mod resume;

fn prepare() -> (PathBuf, BackupLayoutGuard, ExecutionWorkflowRecord) {
    let root = temp_dir("ic-backup-execution-workflow");
    fs::create_dir(&root).unwrap();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let workflow = workflow();
    create_execution_workflow(&layout, &workflow).unwrap();
    (root, layout, workflow)
}
fn first_stage(
    layout: &BackupLayoutGuard,
    workflow: &ExecutionWorkflowRecord,
) -> ExecutionStageBindingRecord {
    let plan = child(0);
    let binding = ExecutionStageBindingRecord::new(workflow, 0, &plan, vec![]).unwrap();
    let stage = ExecutionStageGuard::create(layout, binding.clone(), plan).unwrap();
    assert_eq!(stage.binding(), &binding);
    binding
}
fn settle_first(
    layout: &BackupLayoutGuard,
    workflow: &ExecutionWorkflowRecord,
    binding: &ExecutionStageBindingRecord,
) -> ExecutionStagePredecessorRecord {
    let stage =
        ExecutionStageGuard::open(layout, &workflow.digest(), 0, &binding.digest()).unwrap();
    let plan = stage.plan();
    let stage_layout = stage.layout().unwrap();
    let mut journal =
        AttemptJournalGuard::create(stage_layout, plan.attempt_authority(42).unwrap()).unwrap();
    let attempt = journal.reserve_planned_mutation(&plan.digest()).unwrap();
    journal
        .record_mutation(MutationReceiptRequest {
            attempt,
            request: plan.operation(42).unwrap().request().into(),
            outcome: MutationOutcomeRecord::Applied,
            evidence: "12".repeat(32),
        })
        .unwrap();
    drop(journal);
    stage
        .checkpoint(ArtifactChecksumRecord::from_bytes(
            b"snapshot ID and metadata evidence",
        ))
        .unwrap()
}

#[test]
fn pending_attempt_survives_exact_reopen_without_new_plan_or_allowance() {
    let (root, layout, workflow) = prepare();
    let binding = first_stage(&layout, &workflow);
    let stage =
        ExecutionStageGuard::open(&layout, &workflow.digest(), 0, &binding.digest()).unwrap();
    let stage_layout = stage.layout().unwrap();
    assert_eq!(
        fs::metadata(stage_layout.root())
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    for name in ["stage-binding.json", "operation-plan.json"] {
        assert_eq!(
            fs::metadata(stage_layout.root().join(name))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    let authority = stage.plan().attempt_authority(42).unwrap();
    let mut journal = AttemptJournalGuard::create(stage_layout, authority.clone()).unwrap();
    let attempt = journal
        .reserve_planned_mutation(&stage.plan().digest())
        .unwrap();
    let original = journal.record().unwrap().clone();
    assert_eq!(attempt, 1);
    drop(journal);
    let bytes = fs::read(stage_layout.root().join("attempt-42.json")).unwrap();
    drop(stage);
    assert!(ExecutionStageGuard::create(&layout, binding.clone(), child(0)).is_err());
    let stage =
        ExecutionStageGuard::open(&layout, &workflow.digest(), 0, &binding.digest()).unwrap();
    let stage_layout = stage.layout().unwrap();
    let mut journal = AttemptJournalGuard::open(stage_layout, &authority).unwrap();
    assert_eq!(journal.record().unwrap(), &original);
    assert!(journal.reserve_mutation().is_err());
    drop(journal);
    assert_eq!(
        fs::read(stage_layout.root().join("attempt-42.json")).unwrap(),
        bytes
    );
    // An absent original journal is not zero consumption and is never recreated.
    fs::remove_file(stage_layout.root().join("attempt-42.json")).unwrap();
    assert!(read_execution_progress(stage_layout, &stage.plan().digest()).is_err());
    drop(stage);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn exact_predecessor_settlement_admits_learned_plan_but_changed_history_stops() {
    let (root, layout, workflow) = prepare();
    let first = first_stage(&layout, &workflow);
    let row = settle_first(&layout, &workflow, &first);
    let plan = child(7);
    let binding = ExecutionStageBindingRecord::new(&workflow, 7, &plan, vec![row]).unwrap();
    let stage = ExecutionStageGuard::create(&layout, binding.clone(), plan.clone()).unwrap();
    let original = fs::read(stage.layout().unwrap().root().join("stage-binding.json")).unwrap();
    assert_eq!(stage.plan(), &plan);
    assert!(ExecutionStageGuard::open(&layout, &workflow.digest(), 7, &binding.digest()).is_err());
    drop(stage);
    let journal_path = root.join("execution-stage-0/attempt-42.json");
    let mut journal: serde_json::Value =
        serde_json::from_slice(&fs::read(&journal_path).unwrap()).unwrap();
    journal["events"][1]["evidence"] = serde_json::json!("34".repeat(32));
    fs::write(&journal_path, serde_json::to_vec_pretty(&journal).unwrap()).unwrap();
    assert!(matches!(
        ExecutionStageGuard::open(&layout, &workflow.digest(), 7, &binding.digest()),
        Err(ExecutionWorkflowPersistenceError::Settlement(_))
    ));
    assert_eq!(
        fs::read(root.join("execution-stage-7/stage-binding.json")).unwrap(),
        original
    );
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn declarations_and_missing_settlement_never_allocate_a_successor() {
    let (root, layout, workflow) = prepare();
    let first = first_stage(&layout, &workflow);
    let row = ExecutionStagePredecessorRecord::new(
        0,
        first.digest(),
        ArtifactChecksumRecord::from_bytes(b"declared only"),
        ArtifactChecksumRecord::from_bytes(b"learned"),
    );
    let binding = ExecutionStageBindingRecord::new(&workflow, 7, &child(7), vec![row]).unwrap();
    assert!(ExecutionStageGuard::create(&layout, binding, child(7)).is_err());
    assert!(!root.join("execution-stage-7").exists());
    assert!(!root.join("execution-stage-0/attempt-42.json").exists());
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn lost_complete_creation_reopens_but_partial_or_unsafe_preparation_is_retained() {
    let (root, layout, workflow) = prepare();
    let binding = first_stage(&layout, &workflow);
    // Discard the successful response and recover only exact retained records.
    let stage =
        ExecutionStageGuard::open(&layout, &workflow.digest(), 0, &binding.digest()).unwrap();
    assert_eq!(stage.plan(), &child(0));
    drop(stage);
    let path = root.join("execution-stage-0/stage-binding.json");
    let original = fs::read(&path).unwrap();
    fs::remove_file(&path).unwrap();
    assert!(ExecutionStageGuard::open(&layout, &workflow.digest(), 0, &binding.digest()).is_err());
    assert!(ExecutionStageGuard::create(&layout, binding.clone(), child(0)).is_err());
    assert!(root.join("execution-stage-0/operation-plan.json").exists());
    assert!(!path.exists());
    fs::write(&path, original).unwrap();
    let actual = root.join("retained-stage");
    fs::rename(root.join("execution-stage-0"), &actual).unwrap();
    symlink(&actual, root.join("execution-stage-0")).unwrap();
    assert!(matches!(
        ExecutionStageGuard::open(&layout, &workflow.digest(), 0, &binding.digest()),
        Err(ExecutionWorkflowPersistenceError::UnsafeStage)
    ));
    assert!(actual.join("operation-plan.json").exists());
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn changed_binding_or_workflow_and_oversized_binding_reject_without_repair() {
    let (root, layout, workflow) = prepare();
    let binding = first_stage(&layout, &workflow);
    let path = root.join("execution-stage-0/stage-binding.json");
    let original = fs::read(&path).unwrap();
    let mut changed: serde_json::Value = serde_json::from_slice(&original).unwrap();
    changed["plan"] =
        serde_json::to_value(ArtifactChecksumRecord::from_bytes(b"new plan")).unwrap();
    fs::write(&path, serde_json::to_vec_pretty(&changed).unwrap()).unwrap();
    assert!(matches!(
        ExecutionStageGuard::open(&layout, &workflow.digest(), 0, &binding.digest()),
        Err(ExecutionWorkflowPersistenceError::DigestMismatch)
    ));
    fs::write(
        &path,
        vec![b' '; usize::try_from(MAX_EXECUTION_STAGE_BYTES).unwrap() + 1],
    )
    .unwrap();
    assert!(matches!(
        ExecutionStageGuard::open(&layout, &workflow.digest(), 0, &binding.digest()),
        Err(ExecutionWorkflowPersistenceError::Persistence(
            PersistenceError::RecordTooLarge { .. }
        ))
    ));
    fs::write(&path, original).unwrap();
    let workflow_path = root.join(WORKFLOW_FILE);
    let original = fs::read(&workflow_path).unwrap();
    assert!(create_execution_workflow(&layout, &workflow).is_err());
    assert_eq!(fs::read(&workflow_path).unwrap(), original);
    fs::remove_file(&workflow_path).unwrap();
    assert!(ExecutionStageGuard::open(&layout, &workflow.digest(), 0, &binding.digest()).is_err());
    assert!(!workflow_path.exists());
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn acknowledged_process_death_retains_partial_stage_or_reopens_complete_binding() {
    const ROOT: &str = "IC_BACKUP_STAGE_CRASH_ROOT";
    const HANDSHAKE: &str = "IC_BACKUP_STAGE_CRASH_HANDSHAKE";
    const BARRIER: &str = "IC_BACKUP_STAGE_CRASH_BARRIER";
    if let Some(root) = std::env::var_os(ROOT) {
        let layout = BackupLayoutGuard::acquire(&PathBuf::from(root)).unwrap();
        let handshake = PathBuf::from(std::env::var_os(HANDSHAKE).unwrap());
        let selected = if std::env::var(BARRIER).unwrap() == "complete" {
            StagePreparationBarrier::Binding
        } else {
            StagePreparationBarrier::Plan
        };
        let binding = ExecutionStageBindingRecord::new(&workflow(), 0, &child(0), vec![]).unwrap();
        ExecutionStageGuard::create_with(&layout, binding, child(0), |barrier| {
            if barrier == selected {
                hold_at_acknowledged_barrier(&handshake);
            }
        })
        .unwrap();
        panic!("crash child passed its acknowledged stage barrier");
    }
    for complete in [false, true] {
        let (root, layout, workflow) = prepare();
        drop(layout);
        let handshake = temp_dir("ic-backup-stage-handshake");
        fs::create_dir(&handshake).unwrap();
        let mut child_process = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "ops::persistence::execution_workflow::tests::acknowledged_process_death_retains_partial_stage_or_reopens_complete_binding", "--nocapture"])
            .env(ROOT, &root).env(HANDSHAKE, &handshake)
            .env(BARRIER, if complete { "complete" } else { "partial" }).spawn().unwrap();
        kill_child_at_acknowledged_barrier(&mut child_process, &handshake);
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        let binding = ExecutionStageBindingRecord::new(&workflow, 0, &child(0), vec![]).unwrap();
        let stage_path = root.join("execution-stage-0");
        let original = fs::read(stage_path.join("operation-plan.json")).unwrap();
        assert!(!stage_path.join("attempt-42.json").exists());
        assert_eq!(stage_path.join(BINDING_FILE).exists(), complete);
        let reopened = ExecutionStageGuard::open(&layout, &workflow.digest(), 0, &binding.digest());
        assert_eq!(reopened.is_ok(), complete);
        drop(reopened);
        assert!(ExecutionStageGuard::create(&layout, binding, child(0)).is_err());
        assert_eq!(
            fs::read(stage_path.join("operation-plan.json")).unwrap(),
            original
        );
        drop(layout);
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(handshake).unwrap();
    }
}
