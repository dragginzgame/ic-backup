use super::*;
use crate::model::{
    attempt_journal::AttemptBudgetRecord,
    effect_graph::{EffectGraphRecord, EffectNodeRecord, EffectNodeRequest},
    execution_workflow::tests::child_request,
    operation_plan::{PlanBudgetRecord, PlannedOperationRecord, PlannedOperationRequest},
};
use serde_json::json;

pub(super) fn catalog() -> ExecutionWorkflowRecord {
    let mut value = serde_json::to_value(workflow()).unwrap();
    value["allocation"]["operations"][0]["budget"] = json!({"mutations":3,"observations":0});
    value["allocation"]["budget"] = json!({"mutations":5,"observations":1});
    serde_json::from_value(value).unwrap()
}
pub(super) fn plan() -> OperationPlanRecord {
    let mut request = child_request(0);
    request.budget = PlanBudgetRecord::new(3, 0).unwrap();
    request.graph = EffectGraphRecord::new(
        [(42, vec![]), (99, vec![42])]
            .into_iter()
            .map(|(operation_sequence, depends_on)| {
                EffectNodeRecord::new(EffectNodeRequest {
                    operation_sequence,
                    depends_on,
                })
                .unwrap()
            })
            .collect(),
    )
    .unwrap();
    request.operations = [42, 99]
        .into_iter()
        .map(|operation_sequence| {
            PlannedOperationRecord::new(PlannedOperationRequest {
                operation_sequence,
                target: request.selected_targets[0].clone(),
                request: ArtifactChecksumRecord::from_bytes(&operation_sequence.to_be_bytes())
                    .hash()
                    .into(),
                budget: AttemptBudgetRecord::new(1, 0).unwrap(),
            })
            .unwrap()
        })
        .collect();
    OperationPlanRecord::new(request).unwrap()
}
pub(super) fn retained_catalog() -> (PathBuf, BackupLayoutGuard) {
    let root = temp_dir("ic-backup-prepared-stage");
    fs::create_dir(&root).unwrap();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    create_execution_workflow(&layout, &catalog()).unwrap();
    eprintln!("Prepared-stage evidence retained: {}", root.display());
    (root, layout)
}
fn binding() -> ExecutionStageBindingRecord {
    ExecutionStageBindingRecord::new(&catalog(), 0, &plan(), vec![]).unwrap()
}

#[test]
fn complete_preparation_preserves_original_limits_and_pending_reopen() {
    let (root, layout) = retained_catalog();
    let stage = ExecutionStageGuard::prepare(&layout, binding(), plan()).unwrap();
    let current = stage.layout().unwrap();
    let view = read_execution_progress(current, &plan().digest()).unwrap();
    assert_eq!(view.attempts.mutations_used, 0);
    assert_eq!(view.attempts.mutations_remaining, 2); // Spare stage headroom is unassigned.
    for authority in plan().attempt_authorities().unwrap() {
        let journal = AttemptJournalGuard::open(current, &authority).unwrap();
        assert_eq!(journal.record().unwrap().authority(), &authority);
        assert_eq!(
            fs::metadata(journal.path()).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    let mut dependent =
        AttemptJournalGuard::open(current, &plan().attempt_authority(99).unwrap()).unwrap();
    assert!(
        dependent
            .reserve_planned_mutation(&plan().digest())
            .is_err()
    );
    assert_eq!(dependent.record().unwrap().view().mutations_used, 0);
    drop(dependent);
    let mut first =
        AttemptJournalGuard::open(current, &plan().attempt_authority(42).unwrap()).unwrap();
    first.reserve_planned_mutation(&plan().digest()).unwrap();
    let bytes = fs::read(first.path()).unwrap();
    drop(first);
    let pending = read_execution_progress(current, &plan().digest()).unwrap();
    drop(stage);
    assert!(ExecutionStageGuard::prepare(&layout, binding(), plan()).is_err());
    let reopened =
        ExecutionStageGuard::open(&layout, &catalog().digest(), 0, &binding().digest()).unwrap();
    let current = reopened.layout().unwrap();
    assert_eq!(
        read_execution_progress(current, &plan().digest()).unwrap(),
        pending
    );
    assert_eq!(
        fs::read(current.root().join("attempt-42.json")).unwrap(),
        bytes
    );
    drop(reopened);
    // Missing evidence cannot be repaired by another preparation or by open.
    fs::remove_file(root.join("execution-stage-0/attempt-99.json")).unwrap();
    let reopened =
        ExecutionStageGuard::open(&layout, &catalog().digest(), 0, &binding().digest()).unwrap();
    assert!(read_execution_progress(reopened.layout().unwrap(), &plan().digest()).is_err());
    assert!(!root.join("execution-stage-0/attempt-99.json").exists());
}

#[test]
fn changed_original_binding_refuses_before_stage_or_journal_allocation() {
    let (root, layout) = retained_catalog();
    let mut value = serde_json::to_value(binding()).unwrap();
    value["plan"] = serde_json::to_value(ArtifactChecksumRecord::from_bytes(b"changed")).unwrap();
    let altered = serde_json::from_value(value).unwrap();
    assert!(matches!(
        ExecutionStageGuard::prepare(&layout, altered, plan()),
        Err(ExecutionStagePreparationError::Stage(
            ExecutionWorkflowPersistenceError::Model(ExecutionWorkflowError::BindingMismatch)
        ))
    ));
    assert!(!root.join("execution-stage-0").exists());
}

#[test]
fn occupied_later_journal_retains_earlier_publication_without_repair() {
    let (root, layout) = retained_catalog();
    let result = ExecutionStageGuard::prepare_with(&layout, binding(), plan(), |barrier| {
        if barrier == StagePreparationBarrier::Journal(42) {
            fs::write(
                root.join("execution-stage-0/attempt-99.json"),
                b"occupied original evidence",
            )
            .unwrap();
        }
    });
    assert!(matches!(
        result,
        Err(ExecutionStagePreparationError::Journal(_))
    ));
    let first = fs::read(root.join("execution-stage-0/attempt-42.json")).unwrap();
    assert_eq!(
        fs::read(root.join("execution-stage-0/attempt-99.json")).unwrap(),
        b"occupied original evidence"
    );
    assert!(ExecutionStageGuard::prepare(&layout, binding(), plan()).is_err());
    let stage =
        ExecutionStageGuard::open(&layout, &catalog().digest(), 0, &binding().digest()).unwrap();
    assert!(read_execution_progress(stage.layout().unwrap(), &plan().digest()).is_err());
    assert_eq!(
        fs::read(root.join("execution-stage-0/attempt-42.json")).unwrap(),
        first
    );
}

#[test]
fn acknowledged_death_retains_partial_journals_or_complete_original_set() {
    const ROOT: &str = "IC_BACKUP_PREPARATION_CRASH_ROOT";
    const HANDSHAKE: &str = "IC_BACKUP_PREPARATION_CRASH_HANDSHAKE";
    const BARRIER: &str = "IC_BACKUP_PREPARATION_CRASH_SEQUENCE";
    if let Some(root) = std::env::var_os(ROOT) {
        let layout = BackupLayoutGuard::acquire(&PathBuf::from(root)).unwrap();
        let handshake = PathBuf::from(std::env::var_os(HANDSHAKE).unwrap());
        let sequence = std::env::var(BARRIER).unwrap().parse().unwrap();
        ExecutionStageGuard::prepare_with(&layout, binding(), plan(), |barrier| {
            if barrier == StagePreparationBarrier::Journal(sequence) {
                hold_at_acknowledged_barrier(&handshake);
            }
        })
        .unwrap();
        panic!("crash child passed acknowledged journal publication");
    }
    for sequence in [42, 99] {
        let (root, layout) = retained_catalog();
        drop(layout);
        let handshake = temp_dir("ic-backup-preparation-handshake");
        fs::create_dir(&handshake).unwrap();
        let mut process = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "ops::persistence::execution_workflow::tests::preparation::acknowledged_death_retains_partial_journals_or_complete_original_set", "--nocapture"])
            .env(ROOT, &root).env(HANDSHAKE, &handshake)
            .env(BARRIER, sequence.to_string()).spawn().unwrap();
        kill_child_at_acknowledged_barrier(&mut process, &handshake);
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        let stage = ExecutionStageGuard::open(&layout, &catalog().digest(), 0, &binding().digest())
            .unwrap();
        let current = stage.layout().unwrap();
        let original = fs::read(current.root().join("attempt-42.json")).unwrap();
        assert_eq!(
            current.root().join("attempt-99.json").exists(),
            sequence == 99
        );
        let view = read_execution_progress(current, &plan().digest());
        assert_eq!(view.is_ok(), sequence == 99);
        if let Ok(view) = view {
            assert_eq!(view.attempts.mutations_used, 0);
            assert_eq!(view.attempts.mutations_remaining, 2);
        }
        drop(stage);
        assert!(ExecutionStageGuard::prepare(&layout, binding(), plan()).is_err());
        assert_eq!(
            fs::read(root.join("execution-stage-0/attempt-42.json")).unwrap(),
            original
        );
        assert_eq!(
            root.join("execution-stage-0/attempt-99.json").exists(),
            sequence == 99
        );
    }
}
