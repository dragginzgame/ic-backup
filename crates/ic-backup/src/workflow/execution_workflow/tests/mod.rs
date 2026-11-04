use super::*;
use crate::ops::persistence::create_execution_workflow;
use std::num::NonZeroUsize;
mod operator;
use crate::tests::support::execution_workflow::{child, workflow};
use crate::{
    model::{
        attempt_journal::{AttemptBudgetRecord, MutationOutcomeRecord, MutationReceiptRequest},
        effect_graph::{EffectGraphRecord, EffectNodeRecord, EffectNodeRequest},
        operation_plan::{
            OperationPlanRecord, PlanBudgetRecord, PlannedOperationRecord, PlannedOperationRequest,
        },
    },
    ops::persistence::AttemptJournalGuard,
    tests::support::{ready::ready, temp_dir},
};
use std::{fs, io, path::PathBuf};

struct Driver {
    calls: Vec<u64>,
    fail_at: Option<u64>,
    stop_before: Option<u64>,
    publish_at: Option<u64>,
}
impl ExecutionWorkflowStageDriver for Driver {
    type Error = io::Error;

    fn bind_stage(
        &mut self,
        workflow: &ExecutionWorkflowRecord,
        sequence: u64,
        predecessors: &[ExecutionStagePredecessorRecord],
    ) -> Result<OperationPlanRecord, Self::Error> {
        if self.stop_before == Some(sequence) {
            return Err(io::Error::other("stop before stage preparation"));
        }
        let expected = workflow
            .allocation()
            .graph()
            .node(sequence)
            .unwrap()
            .depends_on();
        assert!(
            predecessors
                .iter()
                .map(ExecutionStagePredecessorRecord::stage_sequence)
                .eq(expected.iter().copied())
        );
        Ok(child(if sequence == 9 { 7 } else { sequence }))
    }

    fn execute_stage(
        &mut self,
        stage: &ExecutionStageGuard<'_>,
    ) -> impl Future<Output = Result<ArtifactChecksumRecord, Self::Error>> {
        let result = (|| {
            let sequence = stage.binding().stage_sequence();
            self.calls.push(sequence);
            let plan = stage.plan();
            let mut journal = AttemptJournalGuard::open(
                stage.layout().unwrap(),
                &plan.attempt_authority(42).unwrap(),
            )
            .unwrap();
            let attempt = journal.reserve_planned_mutation(&plan.digest()).unwrap();
            if self.fail_at == Some(sequence) {
                return Err(io::Error::other("original reply lost"));
            }
            journal
                .record_mutation(MutationReceiptRequest {
                    attempt,
                    request: plan.operation(42).unwrap().request().into(),
                    outcome: MutationOutcomeRecord::Applied,
                    evidence: "12".repeat(32),
                })
                .unwrap();
            drop(journal);
            let learned = ArtifactChecksumRecord::from_bytes(&sequence.to_be_bytes());
            if self.publish_at == Some(sequence) {
                stage.reconcile_checkpoint(learned.clone()).unwrap();
            }
            Ok(learned)
        })();
        std::future::ready(result)
    }
}

fn layout(prefix: &str) -> (PathBuf, BackupLayoutGuard) {
    let root = temp_dir(prefix);
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    (root, layout)
}

fn branched_workflow() -> ExecutionWorkflowRecord {
    let mut request = crate::tests::support::operation_plan::request();
    request.graph = EffectGraphRecord::new(
        [(0, vec![]), (7, vec![0]), (9, vec![0])]
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
    request.operations.push(
        PlannedOperationRecord::new(PlannedOperationRequest {
            operation_sequence: 9,
            target: request.operations[1].target().into(),
            request: "01".repeat(32),
            budget: AttemptBudgetRecord::new(2, 1).unwrap(),
        })
        .unwrap(),
    );
    request.budget = PlanBudgetRecord::new(5, 3).unwrap();
    ExecutionWorkflowRecord::new(OperationPlanRecord::new(request).unwrap())
}

#[test]
fn dependencies_run_in_topological_order_and_complete_originals_replay() {
    let (root, layout) = layout("ic-backup-run-workflow");
    let mut request = crate::tests::support::operation_plan::request();
    request.graph = EffectGraphRecord::new(
        [(0, vec![7]), (7, vec![])]
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
    let workflow = ExecutionWorkflowRecord::new(OperationPlanRecord::new(request).unwrap());
    let mut driver = Driver {
        calls: Vec::new(),
        fail_at: None,
        stop_before: None,
        publish_at: Some(7),
    };
    create_execution_workflow(&layout, &workflow).unwrap();
    let ExecutionWorkflowAdvance::Complete(view) = ready(advance_execution_workflow(
        &layout,
        &workflow.digest(),
        &BTreeMap::new(),
        NonZeroUsize::new(workflow.stages().len()).unwrap(),
        &mut driver,
    ))
    .unwrap() else {
        panic!("expected complete workflow")
    };
    assert_eq!(driver.calls, [7, 0]);
    assert_eq!(view.workflow(), &workflow.digest());
    assert_eq!(
        view.stages()
            .iter()
            .map(crate::ops::persistence::ExecutionStageSettlementView::stage_sequence)
            .collect::<Vec<_>>(),
        [0, 7]
    );
    assert_eq!(
        view,
        read_complete_execution_workflow(&layout, &workflow.digest()).unwrap()
    );
    assert!(root.join("execution-stage-0/attempt-42.json").exists());
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn lost_reply_stops_and_reentry_cannot_invoke_the_driver_again() {
    let (root, layout) = layout("ic-backup-run-workflow-lost");
    let workflow = workflow();
    let mut driver = Driver {
        calls: Vec::new(),
        fail_at: Some(0),
        stop_before: None,
        publish_at: None,
    };
    create_execution_workflow(&layout, &workflow).unwrap();
    assert!(matches!(
        ready(advance_execution_workflow(
            &layout,
            &workflow.digest(),
            &BTreeMap::new(),
            NonZeroUsize::new(workflow.stages().len()).unwrap(),
            &mut driver,
        )),
        Err(ExecutionWorkflowRunError::Driver(_))
    ));
    assert_eq!(driver.calls, [0]);
    let journal = root.join("execution-stage-0/attempt-42.json");
    let original = fs::read(&journal).unwrap();
    assert!(create_execution_workflow(&layout, &workflow).is_err());
    assert_eq!(driver.calls, [0]);
    assert_eq!(fs::read(&journal).unwrap(), original);
    assert!(!root.join("execution-stage-7").exists());
    assert!(read_complete_execution_workflow(&layout, &workflow.digest()).is_err());
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn retained_run_replays_settled_stage_before_executing_absent_successor() {
    let (root, layout) = layout("ic-backup-retained-workflow");
    let workflow = workflow();
    let mut driver = Driver {
        calls: Vec::new(),
        fail_at: None,
        stop_before: Some(7),
        publish_at: None,
    };
    create_execution_workflow(&layout, &workflow).unwrap();
    assert!(matches!(
        ready(advance_execution_workflow(
            &layout,
            &workflow.digest(),
            &BTreeMap::new(),
            NonZeroUsize::new(workflow.stages().len()).unwrap(),
            &mut driver,
        )),
        Err(ExecutionWorkflowRunError::Driver(_))
    ));
    assert_eq!(driver.calls, [0]);
    let journal = root.join("execution-stage-0/attempt-42.json");
    let original = fs::read(&journal).unwrap();
    let binding = ExecutionStageBindingRecord::new(&workflow, 0, &child(0), vec![]).unwrap();
    let edge = ArtifactChecksumRecord::from_bytes(&0_u64.to_be_bytes());
    let retained = BTreeMap::from([(
        0,
        RetainedStageEvidence::new(
            binding.digest(),
            ArtifactChecksumRecord::from_bytes(&0_u64.to_be_bytes()),
        ),
    )]);
    driver.stop_before = None;
    let ExecutionWorkflowAdvance::Complete(view) = ready(advance_execution_workflow(
        &layout,
        &workflow.digest(),
        &retained,
        NonZeroUsize::new(workflow.stages().len()).unwrap(),
        &mut driver,
    ))
    .unwrap() else {
        panic!("expected complete workflow")
    };
    assert_eq!(driver.calls, [0, 7]);
    assert_eq!(fs::read(journal).unwrap(), original);
    assert_eq!(view.stages().len(), workflow.stages().len());
    {
        let successor = view
            .stages()
            .iter()
            .find(|s| s.stage_sequence() == 7)
            .unwrap();
        let stage =
            ExecutionStageGuard::open(&layout, &workflow.digest(), 7, successor.binding()).unwrap();
        assert_eq!(stage.binding().predecessors()[0].learned_evidence(), &edge);
    }
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn retained_run_refuses_missing_or_pending_stage_before_driver_callback() {
    let (root, layout) = layout("ic-backup-retained-pending");
    let workflow = workflow();
    let mut driver = Driver {
        calls: Vec::new(),
        fail_at: Some(0),
        stop_before: None,
        publish_at: None,
    };
    create_execution_workflow(&layout, &workflow).unwrap();
    assert!(matches!(
        ready(advance_execution_workflow(
            &layout,
            &workflow.digest(),
            &BTreeMap::new(),
            NonZeroUsize::new(workflow.stages().len()).unwrap(),
            &mut driver,
        )),
        Err(ExecutionWorkflowRunError::Driver(_))
    ));
    let journal = root.join("execution-stage-0/attempt-42.json");
    let original = fs::read(&journal).unwrap();
    assert!(matches!(
        ready(advance_execution_workflow(
            &layout,
            &workflow.digest(),
            &BTreeMap::new(),
            NonZeroUsize::new(workflow.stages().len()).unwrap(),
            &mut driver
        )),
        Err(ExecutionWorkflowRunError::RetainedStageSetMismatch)
    ));
    let binding = ExecutionStageBindingRecord::new(&workflow, 0, &child(0), vec![]).unwrap();
    let retained = BTreeMap::from([(
        0,
        RetainedStageEvidence::new(
            binding.digest(),
            ArtifactChecksumRecord::from_bytes(&0_u64.to_be_bytes()),
        ),
    )]);
    assert!(matches!(
        ready(advance_execution_workflow(
            &layout,
            &workflow.digest(),
            &retained,
            NonZeroUsize::new(workflow.stages().len()).unwrap(),
            &mut driver
        )),
        Err(ExecutionWorkflowRunError::Checkpoint(_))
    ));
    assert_eq!(driver.calls, [0]);
    assert_eq!(fs::read(journal).unwrap(), original);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn occupied_successor_refuses_changed_learned_predecessor_before_callbacks() {
    let (root, layout) = layout("ic-backup-retained-learned-edge");
    let workflow = workflow();
    let mut driver = Driver {
        calls: Vec::new(),
        fail_at: None,
        stop_before: None,
        publish_at: None,
    };
    create_execution_workflow(&layout, &workflow).unwrap();
    let ExecutionWorkflowAdvance::Complete(view) = ready(advance_execution_workflow(
        &layout,
        &workflow.digest(),
        &BTreeMap::new(),
        NonZeroUsize::new(workflow.stages().len()).unwrap(),
        &mut driver,
    ))
    .unwrap() else {
        panic!("expected complete workflow")
    };
    assert_eq!(driver.calls, [0, 7]);
    let binding = |sequence| {
        view.stages()
            .iter()
            .find(|stage| stage.stage_sequence() == sequence)
            .unwrap()
            .binding()
            .clone()
    };
    let mut retained = BTreeMap::from([
        (
            0,
            RetainedStageEvidence::new(
                binding(0),
                ArtifactChecksumRecord::from_bytes(&0_u64.to_be_bytes()),
            ),
        ),
        (
            7,
            RetainedStageEvidence::new(
                binding(7),
                ArtifactChecksumRecord::from_bytes(&7_u64.to_be_bytes()),
            ),
        ),
    ]);
    let journal = root.join("execution-stage-0/attempt-42.json");
    let original = fs::read(&journal).unwrap();
    retained.insert(
        0,
        RetainedStageEvidence::new(binding(0), ArtifactChecksumRecord::from_bytes(b"forged")),
    );
    assert!(matches!(
        ready(advance_execution_workflow(
            &layout,
            &workflow.digest(),
            &retained,
            NonZeroUsize::new(workflow.stages().len()).unwrap(),
            &mut driver
        )),
        Err(ExecutionWorkflowRunError::RetainedLearnedMismatch(7))
    ));
    assert_eq!(driver.calls, [0, 7]);
    assert_eq!(fs::read(&journal).unwrap(), original);
    retained.insert(
        0,
        RetainedStageEvidence::new(
            binding(0),
            ArtifactChecksumRecord::from_bytes(&0_u64.to_be_bytes()),
        ),
    );
    assert_eq!(
        ready(advance_execution_workflow(
            &layout,
            &workflow.digest(),
            &retained,
            NonZeroUsize::new(workflow.stages().len()).unwrap(),
            &mut driver
        ))
        .unwrap(),
        ExecutionWorkflowAdvance::Complete(view)
    );
    assert_eq!(driver.calls, [0, 7]);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

struct SuspendedDriver {
    calls: usize,
}
impl ExecutionWorkflowStageDriver for SuspendedDriver {
    type Error = io::Error;

    fn bind_stage(
        &mut self,
        _workflow: &ExecutionWorkflowRecord,
        sequence: u64,
        _predecessors: &[ExecutionStagePredecessorRecord],
    ) -> Result<OperationPlanRecord, Self::Error> {
        assert_eq!(sequence, 0);
        Ok(child(0))
    }

    fn execute_stage(
        &mut self,
        stage: &ExecutionStageGuard<'_>,
    ) -> impl Future<Output = Result<ArtifactChecksumRecord, Self::Error>> {
        self.calls += 1;
        let plan = stage.plan();
        let mut journal = AttemptJournalGuard::open(
            stage.layout().unwrap(),
            &plan.attempt_authority(42).unwrap(),
        )
        .unwrap();
        journal.reserve_planned_mutation(&plan.digest()).unwrap();
        drop(journal);
        std::future::pending()
    }
}

#[test]
fn cancellation_after_reservation_retains_original_and_refuses_reentry() {
    let (root, layout) = layout("ic-backup-run-workflow-cancelled");
    let workflow = workflow();
    let mut driver = SuspendedDriver { calls: 0 };
    create_execution_workflow(&layout, &workflow).unwrap();
    let digest = workflow.digest();
    let retained = BTreeMap::new();
    let mut future = Box::pin(advance_execution_workflow(
        &layout,
        &digest,
        &retained,
        NonZeroUsize::new(workflow.stages().len()).unwrap(),
        &mut driver,
    ));
    crate::tests::support::ready::assert_pending(future.as_mut());
    drop(future);
    assert_eq!(driver.calls, 1);
    let path = root.join("execution-stage-0/attempt-42.json");
    let original = fs::read(&path).unwrap();
    assert!(create_execution_workflow(&layout, &workflow).is_err());
    assert_eq!(driver.calls, 1);
    assert_eq!(fs::read(&path).unwrap(), original);
    assert!(!root.join("execution-stage-7").exists());
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn fanout_carries_one_checkpoint_commitment_and_replays_without_callbacks() {
    let (root, layout) = layout("ic-backup-common-learned-fanout");
    let workflow = branched_workflow();
    let mut driver = Driver {
        calls: vec![],
        fail_at: None,
        stop_before: None,
        publish_at: None,
    };
    create_execution_workflow(&layout, &workflow).unwrap();
    let ExecutionWorkflowAdvance::Complete(view) = ready(advance_execution_workflow(
        &layout,
        &workflow.digest(),
        &BTreeMap::new(),
        NonZeroUsize::new(3).unwrap(),
        &mut driver,
    ))
    .unwrap() else {
        panic!("expected complete fanout")
    };
    assert_eq!(driver.calls, [0, 7, 9]);
    let retained = view
        .stages()
        .iter()
        .map(|row| {
            (
                row.stage_sequence(),
                RetainedStageEvidence::new(
                    row.binding().clone(),
                    ArtifactChecksumRecord::from_bytes(&row.stage_sequence().to_be_bytes()),
                ),
            )
        })
        .collect();
    let learned = ArtifactChecksumRecord::from_bytes(&0_u64.to_be_bytes());
    for row in view.stages().iter().filter(|row| row.stage_sequence() != 0) {
        let stage = ExecutionStageGuard::open(
            &layout,
            &workflow.digest(),
            row.stage_sequence(),
            row.binding(),
        )
        .unwrap();
        assert_eq!(
            stage.binding().predecessors()[0].learned_evidence(),
            &learned
        );
    }
    assert_eq!(
        ready(advance_execution_workflow(
            &layout,
            &workflow.digest(),
            &retained,
            NonZeroUsize::new(1).unwrap(),
            &mut driver
        ))
        .unwrap(),
        ExecutionWorkflowAdvance::Complete(view)
    );
    assert_eq!(driver.calls, [0, 7, 9]);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn retained_distinct_edge_refuses_without_rebinding_or_driver_callbacks() {
    let (root, layout) = layout("ic-backup-original-distinct-edge");
    let workflow = workflow();
    let mut driver = Driver {
        calls: vec![],
        fail_at: None,
        stop_before: None,
        publish_at: None,
    };
    create_execution_workflow(&layout, &workflow).unwrap();
    assert_eq!(
        ready(advance_execution_workflow(
            &layout,
            &workflow.digest(),
            &BTreeMap::new(),
            NonZeroUsize::new(1).unwrap(),
            &mut driver
        ))
        .unwrap(),
        ExecutionWorkflowAdvance::Paused
    );
    let parent_binding = ExecutionStageBindingRecord::new(&workflow, 0, &child(0), vec![]).unwrap();
    let parent =
        ExecutionStageGuard::open(&layout, &workflow.digest(), 0, &parent_binding.digest())
            .unwrap();
    let learned = ArtifactChecksumRecord::from_bytes(&0_u64.to_be_bytes());
    let original = parent.replay_checkpoint(learned.clone()).unwrap();
    drop(parent);
    let distinct = ExecutionStagePredecessorRecord::new(
        0,
        original.binding().clone(),
        original.settlement().clone(),
        ArtifactChecksumRecord::from_bytes(b"original distinct edge"),
    );
    let binding =
        ExecutionStageBindingRecord::new(&workflow, 7, &child(7), vec![distinct]).unwrap();
    drop(ExecutionStageGuard::prepare(&layout, binding.clone(), child(7)).unwrap());
    let path = root.join("execution-stage-7/stage-binding.json");
    let bytes = fs::read(&path).unwrap();
    let journal = root.join("execution-stage-7/attempt-42.json");
    let journal_bytes = fs::read(&journal).unwrap();
    let retained = BTreeMap::from([
        (
            0,
            RetainedStageEvidence::new(parent_binding.digest(), learned),
        ),
        (
            7,
            RetainedStageEvidence::new(
                binding.digest(),
                ArtifactChecksumRecord::from_bytes(b"unsettled"),
            ),
        ),
    ]);
    assert!(matches!(
        ready(advance_execution_workflow(
            &layout,
            &workflow.digest(),
            &retained,
            NonZeroUsize::new(1).unwrap(),
            &mut driver
        )),
        Err(ExecutionWorkflowRunError::RetainedLearnedMismatch(7))
    ));
    assert_eq!(driver.calls, [0]);
    assert_eq!(fs::read(path).unwrap(), bytes);
    assert_eq!(fs::read(journal).unwrap(), journal_bytes);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}
