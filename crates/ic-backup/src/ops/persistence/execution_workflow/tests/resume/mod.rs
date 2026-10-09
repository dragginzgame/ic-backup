use super::preparation::{catalog, plan, retained_catalog};
use super::*;

fn binding() -> ExecutionStageBindingRecord {
    ExecutionStageBindingRecord::new(&catalog(), 0, &plan(), vec![]).unwrap()
}

#[test]
fn complete_resume_preserves_pending_bytes_and_unassigned_headroom_without_artifact_reads() {
    let (root, layout) = retained_catalog();
    let stage = ExecutionStageGuard::prepare(&layout, binding(), plan()).unwrap();
    let current = stage.layout().unwrap();
    // Ordinary resume does not inspect artifact bytes or unsafe artifact entries.
    symlink("missing-artifact-source", current.root().join("artifacts")).unwrap();
    let mut journal =
        AttemptJournalGuard::open(current, &plan().attempt_authority(42).unwrap()).unwrap();
    let attempt = journal.reserve_planned_mutation(&plan().digest()).unwrap();
    let original = fs::read(journal.path()).unwrap();
    drop(journal);
    drop(stage);
    let (stage, view) =
        ExecutionStageGuard::resume(&layout, &catalog().digest(), 0, &binding().digest()).unwrap();
    assert_eq!(view.attempts.mutations_used, 1);
    assert_eq!(view.attempts.mutations_remaining, 1); // Original spare headroom remains unassigned.
    let current = stage.layout().unwrap();
    assert_eq!(
        fs::read(current.root().join("attempt-42.json")).unwrap(),
        original
    );
    let mut journal =
        AttemptJournalGuard::open(current, &plan().attempt_authority(42).unwrap()).unwrap();
    assert_eq!(
        journal.record().unwrap().view().pending_mutation,
        Some(attempt)
    );
    assert!(journal.reserve_planned_mutation(&plan().digest()).is_err());
    assert_eq!(fs::read(journal.path()).unwrap(), original);
    assert!(root.join("execution-stage-0/artifacts").is_symlink());
}

#[test]
fn missing_original_child_refuses_complete_resume_but_leaves_inspection_available() {
    let (root, layout) = retained_catalog();
    drop(ExecutionStageGuard::prepare(&layout, binding(), plan()).unwrap());
    let missing = root.join("execution-stage-0/attempt-99.json");
    fs::remove_file(&missing).unwrap();
    let original = fs::read(root.join("execution-stage-0/attempt-42.json")).unwrap();
    assert!(matches!(
        ExecutionStageGuard::resume(&layout, &catalog().digest(), 0, &binding().digest()),
        Err(ExecutionStageResumeError::Progress(
            ExecutionProgressPersistenceError::Journal(_)
        ))
    ));
    assert!(!missing.exists());
    assert_eq!(
        fs::read(root.join("execution-stage-0/attempt-42.json")).unwrap(),
        original
    );
    // Record-only inspection retains its distinct recovery-evidence purpose.
    ExecutionStageGuard::open(&layout, &catalog().digest(), 0, &binding().digest()).unwrap();
}

#[test]
fn held_or_changed_original_child_refuses_complete_resume_without_replacement() {
    let (root, layout) = retained_catalog();
    drop(ExecutionStageGuard::prepare(&layout, binding(), plan()).unwrap());
    let path = root.join("execution-stage-0/attempt-99.json");
    let original = fs::read(&path).unwrap();
    let held = JournalLock::acquire(&path).unwrap();
    assert!(matches!(
        ExecutionStageGuard::resume(&layout, &catalog().digest(), 0, &binding().digest()),
        Err(ExecutionStageResumeError::Progress(
            ExecutionProgressPersistenceError::Journal(_)
        ))
    ));
    assert_eq!(fs::read(&path).unwrap(), original);
    drop(held);
    let mut value: serde_json::Value = serde_json::from_slice(&original).unwrap();
    value["authority"]["budget"]["mutations"] = serde_json::json!(2);
    fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    let changed = fs::read(&path).unwrap();
    assert!(matches!(
        ExecutionStageGuard::resume(&layout, &catalog().digest(), 0, &binding().digest()),
        Err(ExecutionStageResumeError::Progress(
            ExecutionProgressPersistenceError::Journal(_)
        ))
    ));
    assert_eq!(fs::read(&path).unwrap(), changed);
}

#[test]
fn complete_resume_retains_stage_error_for_changed_ancestor_history() {
    let (root, layout, workflow) = prepare();
    let first = first_stage(&layout, &workflow);
    let row = settle_first(&layout, &workflow, &first);
    let binding = ExecutionStageBindingRecord::new(&workflow, 7, &child(7), vec![row]).unwrap();
    drop(ExecutionStageGuard::prepare(&layout, binding.clone(), child(7)).unwrap());
    let path = root.join("execution-stage-0/attempt-42.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["events"][1]["evidence"] = serde_json::json!("34".repeat(32));
    fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    let changed = fs::read(&path).unwrap();
    assert!(matches!(
        ExecutionStageGuard::resume(&layout, &workflow.digest(), 7, &binding.digest()),
        Err(ExecutionStageResumeError::Stage(
            ExecutionWorkflowPersistenceError::Settlement(_)
        ))
    ));
    assert_eq!(fs::read(path).unwrap(), changed);
}
