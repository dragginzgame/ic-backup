use super::*;
use crate::model::execution_workflow::tests::child_request;
use serde_json::json;

fn catalog() -> ExecutionWorkflowRecord {
    let mut value = serde_json::to_value(workflow()).unwrap();
    let allocation = &mut value["allocation"];
    allocation["graph"]["nodes"] = json!([
        {"operation_sequence":0,"depends_on":[]},
        {"operation_sequence":7,"depends_on":[0]},
        {"operation_sequence":9,"depends_on":[7]}
    ]);
    let mut operation = allocation["operations"][1].clone();
    operation["operation_sequence"] = json!(9);
    allocation["operations"]
        .as_array_mut()
        .unwrap()
        .push(operation);
    allocation["budget"] = json!({"mutations":5,"observations":3});
    serde_json::from_value(value).unwrap()
}
fn plan(sequence: u64) -> OperationPlanRecord {
    // Every later fixture stage keeps stage 7's exact original allocation.
    OperationPlanRecord::new(child_request(if sequence == 0 { 0 } else { 7 })).unwrap()
}
fn settled_stage(
    layout: &BackupLayoutGuard,
    workflow: &ExecutionWorkflowRecord,
    sequence: u64,
    predecessors: Vec<ExecutionStagePredecessorRecord>,
) -> ExecutionStagePredecessorRecord {
    let plan = plan(sequence);
    let binding =
        ExecutionStageBindingRecord::new(workflow, sequence, &plan, predecessors).unwrap();
    let stage = ExecutionStageGuard::prepare(layout, binding.clone(), plan).unwrap();
    let current = stage.layout().unwrap();
    let mut journal =
        AttemptJournalGuard::open(current, &stage.plan().attempt_authority(42).unwrap()).unwrap();
    let attempt = journal
        .reserve_planned_mutation(&stage.plan().digest())
        .unwrap();
    journal
        .record_mutation(MutationReceiptRequest {
            attempt,
            request: stage.plan().operation(42).unwrap().request().into(),
            outcome: MutationOutcomeRecord::Applied,
            evidence: ArtifactChecksumRecord::from_bytes(&sequence.to_be_bytes())
                .hash()
                .into(),
        })
        .unwrap();
    let settlement = ExecutionSettlementRecord::new(
        stage.plan().digest(),
        vec![ExecutionSettlementJournalRecord::from_journal(
            journal.record().unwrap(),
        )],
    )
    .unwrap();
    drop(journal);
    create_execution_settlement(current, &settlement).unwrap();
    ExecutionStagePredecessorRecord::new(
        sequence,
        binding.digest(),
        settlement.digest(),
        ArtifactChecksumRecord::from_bytes(b"fixture learned evidence"),
    )
}
fn retained_chain() -> (
    PathBuf,
    BackupLayoutGuard,
    ExecutionWorkflowRecord,
    ExecutionStageBindingRecord,
) {
    let root = temp_dir("ic-backup-stage-ancestry");
    fs::create_dir(&root).unwrap();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let workflow = catalog();
    create_execution_workflow(&layout, &workflow).unwrap();
    let first = settled_stage(&layout, &workflow, 0, vec![]);
    let second = settled_stage(&layout, &workflow, 7, vec![first]);
    let third = ExecutionStageBindingRecord::new(&workflow, 9, &plan(9), vec![second]).unwrap();
    eprintln!("Stage ancestry evidence retained: {}", root.display());
    (root, layout, workflow, third)
}

#[test]
fn changed_ancestor_history_refuses_descendant_before_allocation() {
    let (root, layout, _, binding) = retained_chain();
    let path = root.join("execution-stage-0/attempt-42.json");
    let mut journal: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    journal["events"][1]["evidence"] = json!("34".repeat(32));
    fs::write(&path, serde_json::to_vec_pretty(&journal).unwrap()).unwrap();
    let changed = fs::read(&path).unwrap();
    assert!(matches!(
        ExecutionStageGuard::create(&layout, binding.clone(), plan(9)),
        Err(ExecutionWorkflowPersistenceError::Settlement(_))
    ));
    assert!(matches!(
        ExecutionStageGuard::prepare(&layout, binding, plan(9)),
        Err(ExecutionStagePreparationError::Stage(
            ExecutionWorkflowPersistenceError::Settlement(_)
        ))
    ));
    assert!(!root.join("execution-stage-9").exists());
    assert_eq!(fs::read(path).unwrap(), changed);
}

#[test]
fn missing_ancestor_journal_refuses_reopen_and_retained_layout_without_repair() {
    let (root, layout, workflow, binding) = retained_chain();
    let stage = ExecutionStageGuard::prepare(&layout, binding.clone(), plan(9)).unwrap();
    let own_path = root.join("execution-stage-9/attempt-42.json");
    let own_original = fs::read(&own_path).unwrap();
    let missing = root.join("execution-stage-0/attempt-42.json");
    fs::remove_file(&missing).unwrap();
    assert!(matches!(
        stage.layout(),
        Err(ExecutionWorkflowPersistenceError::Settlement(_))
    ));
    drop(stage);
    assert!(matches!(
        ExecutionStageGuard::open(&layout, &workflow.digest(), 9, &binding.digest()),
        Err(ExecutionWorkflowPersistenceError::Settlement(_))
    ));
    assert!(!missing.exists());
    assert_eq!(fs::read(own_path).unwrap(), own_original);
}

#[test]
fn shared_ancestor_admits_both_branches_with_distinct_learned_evidence() {
    let mut value = serde_json::to_value(catalog()).unwrap();
    let allocation = &mut value["allocation"];
    allocation["graph"]["nodes"][2]["depends_on"] = json!([0]);
    allocation["graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(json!({"operation_sequence":13,"depends_on":[7,9]}));
    let mut operation = allocation["operations"][1].clone();
    operation["operation_sequence"] = json!(13);
    allocation["operations"]
        .as_array_mut()
        .unwrap()
        .push(operation);
    allocation["budget"] = json!({"mutations":7,"observations":4});
    let workflow: ExecutionWorkflowRecord = serde_json::from_value(value).unwrap();
    let root = temp_dir("ic-backup-stage-shared-ancestor");
    fs::create_dir(&root).unwrap();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    create_execution_workflow(&layout, &workflow).unwrap();
    let first = settled_stage(&layout, &workflow, 0, vec![]);
    let left = settled_stage(&layout, &workflow, 7, vec![first.clone()]);
    // The two edges learn different inputs from the same exact original history.
    let right_input = ExecutionStagePredecessorRecord::new(
        0,
        first.binding().clone(),
        first.settlement().clone(),
        ArtifactChecksumRecord::from_bytes(b"other learned input"),
    );
    let right = settled_stage(&layout, &workflow, 9, vec![right_input]);
    let binding =
        ExecutionStageBindingRecord::new(&workflow, 13, &plan(13), vec![left, right]).unwrap();
    let stage = ExecutionStageGuard::prepare(&layout, binding.clone(), plan(13)).unwrap();
    assert_eq!(
        read_execution_progress(stage.layout().unwrap(), &plan(13).digest())
            .unwrap()
            .attempts
            .mutations_used,
        0
    );
    drop(stage);
    let stage =
        ExecutionStageGuard::open(&layout, &workflow.digest(), 13, &binding.digest()).unwrap();
    let ancestor = root.join("execution-stage-0/attempt-42.json");
    let held = JournalLock::acquire(&ancestor).unwrap();
    assert!(matches!(
        stage.layout(),
        Err(ExecutionWorkflowPersistenceError::Settlement(_))
    ));
    drop(held);
    stage.layout().unwrap();
    eprintln!("Shared-ancestor evidence retained: {}", root.display());
}
