//! Bounded advancement and inspection use the original local journal owners.
use super::*;
use crate::{
    ops::persistence::{ExecutionWorkflowProgressError, read_execution_workflow_progress},
    policy::execution_progress::OperationProgressState,
    view::execution_workflow::ExecutionStageProgressState,
};

fn driver() -> Driver {
    Driver {
        calls: vec![],
        fail_at: None,
        stop_before: None,
        publish_at: None,
        edge_overrides: BTreeMap::new(),
    }
}
fn limit() -> NonZeroUsize {
    NonZeroUsize::new(1).unwrap()
}
fn learned(sequence: u64) -> ArtifactChecksumRecord {
    ArtifactChecksumRecord::from_bytes(&sequence.to_be_bytes())
}
fn evidence(
    layout: &BackupLayoutGuard,
    workflow: &ExecutionWorkflowRecord,
) -> BTreeMap<u64, RetainedStageEvidence> {
    read_execution_workflow_progress(layout, &workflow.digest())
        .unwrap()
        .stages
        .into_iter()
        .filter_map(|row| match row.state {
            ExecutionStageProgressState::Unprepared => None,
            ExecutionStageProgressState::Prepared { binding, .. } => Some((
                row.stage_sequence,
                RetainedStageEvidence::new(binding, learned(row.stage_sequence)),
            )),
        })
        .collect()
}

#[test]
fn bounded_operator_pauses_before_binding_and_cold_replays_without_callbacks() {
    let (root, layout) = layout("ic-backup-operator-bounded");
    let workflow = workflow();
    create_execution_workflow(&layout, &workflow).unwrap();
    let declared = fs::read(root.join("execution-workflow.json")).unwrap();
    let initial = read_execution_workflow_progress(&layout, &workflow.digest()).unwrap();
    assert_eq!(
        initial
            .stages
            .iter()
            .map(|s| s.stage_sequence)
            .collect::<Vec<_>>(),
        [0, 7]
    );
    assert!(
        initial
            .stages
            .iter()
            .all(|s| matches!(s.state, ExecutionStageProgressState::Unprepared))
    );
    let mut driver = driver();
    driver.stop_before = Some(7);
    assert_eq!(
        ready(advance_execution_workflow(
            &layout,
            &workflow.digest(),
            &BTreeMap::new(),
            limit(),
            &mut driver
        ))
        .unwrap(),
        ExecutionWorkflowAdvance::Paused
    );
    assert_eq!(driver.calls, [0]);
    assert!(!root.join("execution-stage-7").exists());
    driver.stop_before = None;
    let retained = evidence(&layout, &workflow);
    let completed = ready(advance_execution_workflow(
        &layout,
        &workflow.digest(),
        &retained,
        limit(),
        &mut driver,
    ))
    .unwrap();
    assert!(matches!(completed, ExecutionWorkflowAdvance::Complete(_)));
    assert_eq!(driver.calls, [0, 7]);
    let retained = evidence(&layout, &workflow);
    assert_eq!(
        ready(advance_execution_workflow(
            &layout,
            &workflow.digest(),
            &retained,
            limit(),
            &mut driver
        ))
        .unwrap(),
        completed
    );
    assert_eq!(driver.calls, [0, 7]);
    assert_eq!(
        fs::read(root.join("execution-workflow.json")).unwrap(),
        declared
    );
    assert!(matches!(
        ready(advance_execution_workflow(
            &layout,
            &ArtifactChecksumRecord::from_bytes(b"changed original"),
            &retained,
            limit(),
            &mut driver
        )),
        Err(ExecutionWorkflowRunError::Workflow(_))
    ));
    assert_eq!(driver.calls, [0, 7]);
}

#[test]
fn pending_observation_is_visible_and_cannot_be_resumed_by_a_new_stage_limit() {
    let (root, layout) = layout("ic-backup-operator-pending");
    let workflow = workflow();
    create_execution_workflow(&layout, &workflow).unwrap();
    let mut driver = driver();
    driver.fail_at = Some(0);
    assert!(matches!(
        ready(advance_execution_workflow(
            &layout,
            &workflow.digest(),
            &BTreeMap::new(),
            limit(),
            &mut driver
        )),
        Err(ExecutionWorkflowRunError::Driver(_))
    ));
    let inspect = || {
        let view = read_execution_workflow_progress(&layout, &workflow.digest()).unwrap();
        let ExecutionStageProgressState::Prepared { progress, .. } = &view.stages[0].state else {
            panic!("original stage retained")
        };
        progress.operations[0].clone()
    };
    let mutation = inspect();
    assert_eq!(mutation.state, OperationProgressState::MutationUnresolved);
    let binding = match &read_execution_workflow_progress(&layout, &workflow.digest())
        .unwrap()
        .stages[0]
        .state
    {
        ExecutionStageProgressState::Prepared { binding, .. } => binding.clone(),
        ExecutionStageProgressState::Unprepared => panic!("original stage retained"),
    };
    let stage = ExecutionStageGuard::open(&layout, &workflow.digest(), 0, &binding).unwrap();
    let mut journal = AttemptJournalGuard::open(
        stage.layout().unwrap(),
        &stage.plan().attempt_authority(42).unwrap(),
    )
    .unwrap();
    journal
        .reserve_planned_observation(&stage.plan().digest(), 1, &"aa".repeat(32))
        .unwrap();
    drop(journal);
    drop(stage);
    let observation = inspect();
    assert_eq!(
        observation.state,
        OperationProgressState::ObservationUnresolved
    );
    assert_eq!(observation.attempts.pending_mutation, Some(1));
    assert_eq!(observation.attempts.pending_observation, Some(2));
    assert_eq!(observation.attempts.mutations_used, 1);
    assert_eq!(observation.attempts.observations_used, 1);
    let path = root.join("execution-stage-0/attempt-42.json");
    let original = fs::read(&path).unwrap();
    let retained = evidence(&layout, &workflow);
    assert!(matches!(
        ready(advance_execution_workflow(
            &layout,
            &workflow.digest(),
            &retained,
            limit(),
            &mut driver
        )),
        Err(ExecutionWorkflowRunError::Checkpoint(_))
    ));
    assert_eq!(driver.calls, [0]);
    assert_eq!(fs::read(path).unwrap(), original);
    assert!(!root.join("execution-stage-7").exists());
}

#[test]
fn inspection_refuses_missing_journals_and_partial_stages_without_repair() {
    let (root, layout) = layout("ic-backup-operator-missing");
    let workflow = workflow();
    create_execution_workflow(&layout, &workflow).unwrap();
    let binding = ExecutionStageBindingRecord::new(&workflow, 0, &child(0), vec![]).unwrap();
    let stage = ExecutionStageGuard::prepare(&layout, binding, child(0)).unwrap();
    drop(stage);
    let journal = root.join("execution-stage-0/attempt-42.json");
    let retained = journal.with_extension("retained");
    fs::rename(&journal, &retained).unwrap();
    assert!(matches!(
        read_execution_workflow_progress(&layout, &workflow.digest()),
        Err(ExecutionWorkflowProgressError::Journals(_))
    ));
    assert!(!journal.exists());
    assert!(retained.exists());
    // Preserve the moved journal as evidence while completing this independent
    // malformed-namespace probe in another original workflow fixture.
    let (other_root, other) = super::layout("ic-backup-operator-partial");
    create_execution_workflow(&other, &workflow).unwrap();
    fs::create_dir(other_root.join("execution-stage-0")).unwrap();
    assert!(matches!(
        read_execution_workflow_progress(&other, &workflow.digest()),
        Err(ExecutionWorkflowProgressError::Stage(_))
    ));
    assert!(
        !other_root
            .join("execution-stage-0/stage-binding.json")
            .exists()
    );
}
