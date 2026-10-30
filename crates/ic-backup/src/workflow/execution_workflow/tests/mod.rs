use super::*;
mod operator;
use crate::test_support::execution_workflow::{child, workflow};
use crate::{
    model::{
        attempt_journal::{AttemptBudgetRecord, MutationOutcomeRecord, MutationReceiptRequest},
        effect_graph::{EffectGraphRecord, EffectNodeRecord, EffectNodeRequest},
        operation_plan::{
            OperationPlanRecord, PlanBudgetRecord, PlannedOperationRecord, PlannedOperationRequest,
        },
    },
    ops::persistence::AttemptJournalGuard,
    test_support::{ready::ready, temp_dir},
};
use std::{fs, io, path::PathBuf};

struct Driver {
    calls: Vec<u64>,
    fail_at: Option<u64>,
    stop_before: Option<u64>,
    publish_at: Option<u64>,
    edge_overrides: BTreeMap<(u64, u64), ArtifactChecksumRecord>,
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

    fn learned_for_successor(
        &mut self,
        predecessor: &ExecutionStagePredecessorRecord,
        successor_sequence: u64,
    ) -> Result<ArtifactChecksumRecord, Self::Error> {
        Ok(self
            .edge_overrides
            .get(&(predecessor.stage_sequence(), successor_sequence))
            .cloned()
            .unwrap_or_else(|| predecessor.learned_evidence().clone()))
    }
}

fn layout(prefix: &str) -> (PathBuf, BackupLayoutGuard) {
    let root = temp_dir(prefix);
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    (root, layout)
}

fn branched_workflow() -> ExecutionWorkflowRecord {
    let mut request = crate::test_support::operation_plan::request();
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
    let mut request = crate::test_support::operation_plan::request();
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
        edge_overrides: BTreeMap::new(),
    };
    let view = ready(run_new_execution_workflow(&layout, &workflow, &mut driver)).unwrap();
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
        edge_overrides: BTreeMap::new(),
    };
    assert!(matches!(
        ready(run_new_execution_workflow(&layout, &workflow, &mut driver)),
        Err(ExecutionWorkflowRunError::Driver(_))
    ));
    assert_eq!(driver.calls, [0]);
    let journal = root.join("execution-stage-0/attempt-42.json");
    let original = fs::read(&journal).unwrap();
    assert!(matches!(
        ready(run_new_execution_workflow(&layout, &workflow, &mut driver)),
        Err(ExecutionWorkflowRunError::Workflow(_))
    ));
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
        edge_overrides: BTreeMap::new(),
    };
    assert!(matches!(
        ready(run_new_execution_workflow(&layout, &workflow, &mut driver)),
        Err(ExecutionWorkflowRunError::Driver(_))
    ));
    assert_eq!(driver.calls, [0]);
    let journal = root.join("execution-stage-0/attempt-42.json");
    let original = fs::read(&journal).unwrap();
    let binding = ExecutionStageBindingRecord::new(&workflow, 0, &child(0), vec![]).unwrap();
    let edge = ArtifactChecksumRecord::from_bytes(b"retained outgoing commitment");
    let retained = BTreeMap::from([(
        0,
        RetainedStageEvidence::new(
            binding.digest(),
            ArtifactChecksumRecord::from_bytes(&0_u64.to_be_bytes()),
        )
        .with_successor_evidence(BTreeMap::from([(7, edge.clone())])),
    )]);
    driver.stop_before = None;
    // Recovery uses the independently retained commitment, never a fresh driver
    // projection of an already completed predecessor.
    driver.edge_overrides.insert(
        (0, 7),
        ArtifactChecksumRecord::from_bytes(b"changed driver commitment"),
    );
    let view = ready(run_retained_execution_workflow(
        &layout,
        &workflow.digest(),
        &retained,
        &mut driver,
    ))
    .unwrap();
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
        edge_overrides: BTreeMap::new(),
    };
    assert!(matches!(
        ready(run_new_execution_workflow(&layout, &workflow, &mut driver)),
        Err(ExecutionWorkflowRunError::Driver(_))
    ));
    let journal = root.join("execution-stage-0/attempt-42.json");
    let original = fs::read(&journal).unwrap();
    assert!(matches!(
        ready(run_retained_execution_workflow(
            &layout,
            &workflow.digest(),
            &BTreeMap::new(),
            &mut driver,
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
        ready(run_retained_execution_workflow(
            &layout,
            &workflow.digest(),
            &retained,
            &mut driver,
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
        edge_overrides: BTreeMap::new(),
    };
    let view = ready(run_new_execution_workflow(&layout, &workflow, &mut driver)).unwrap();
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
        ready(run_retained_execution_workflow(
            &layout,
            &workflow.digest(),
            &retained,
            &mut driver,
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
        ready(run_retained_execution_workflow(
            &layout,
            &workflow.digest(),
            &retained,
            &mut driver,
        ))
        .unwrap(),
        view
    );
    assert_eq!(driver.calls, [0, 7]);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn retained_run_admits_exact_edge_commitment_distinct_from_stage_checkpoint() {
    let (root, layout) = layout("ic-backup-retained-edge-evidence");
    let workflow = workflow();
    let edge = ArtifactChecksumRecord::from_bytes(b"qualified metadata edge");
    let checkpoint_learned = ArtifactChecksumRecord::from_bytes(&0_u64.to_be_bytes());
    assert_ne!(edge, checkpoint_learned);
    let mut driver = Driver {
        calls: Vec::new(),
        fail_at: None,
        stop_before: None,
        publish_at: None,
        edge_overrides: BTreeMap::from([((0, 7), edge.clone())]),
    };
    let view = ready(run_new_execution_workflow(&layout, &workflow, &mut driver)).unwrap();
    assert_eq!(driver.calls, [0, 7]);
    let binding = |sequence| {
        view.stages()
            .iter()
            .find(|stage| stage.stage_sequence() == sequence)
            .unwrap()
            .binding()
            .clone()
    };
    let stage = ExecutionStageGuard::open(&layout, &workflow.digest(), 7, &binding(7)).unwrap();
    assert_eq!(stage.binding().predecessors()[0].learned_evidence(), &edge);
    drop(stage);

    let retained = BTreeMap::from([
        (
            0,
            RetainedStageEvidence::new(binding(0), checkpoint_learned)
                .with_successor_evidence(BTreeMap::from([(7, edge.clone())])),
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
    assert_eq!(
        ready(run_retained_execution_workflow(
            &layout,
            &workflow.digest(),
            &retained,
            &mut driver,
        ))
        .unwrap(),
        view
    );
    let mut forged = retained.clone();
    forged.insert(
        0,
        forged[&0].clone().with_successor_evidence(BTreeMap::from([(
            7,
            ArtifactChecksumRecord::from_bytes(b"forged edge"),
        )])),
    );
    assert!(matches!(
        ready(run_retained_execution_workflow(
            &layout,
            &workflow.digest(),
            &forged,
            &mut driver,
        )),
        Err(ExecutionWorkflowRunError::RetainedLearnedMismatch(7))
    ));
    forged.insert(
        0,
        retained[&0]
            .clone()
            .with_successor_evidence(BTreeMap::from([(99, edge)])),
    );
    assert!(matches!(
        ready(run_retained_execution_workflow(
            &layout,
            &workflow.digest(),
            &forged,
            &mut driver,
        )),
        Err(ExecutionWorkflowRunError::RetainedLearnedMismatch(0))
    ));
    assert_eq!(driver.calls, [0, 7]);
    assert_eq!(fs::read(journal).unwrap(), original);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn one_original_stage_can_bind_distinct_learned_commitments_to_two_successors() {
    let (root, layout) = layout("ic-backup-branched-edge-evidence");
    let workflow = branched_workflow();
    let edge_7 = ArtifactChecksumRecord::from_bytes(b"metadata identity");
    let edge_9 = ArtifactChecksumRecord::from_bytes(b"application revision");
    let mut driver = Driver {
        calls: Vec::new(),
        fail_at: None,
        stop_before: None,
        publish_at: None,
        edge_overrides: BTreeMap::from([((0, 7), edge_7.clone()), ((0, 9), edge_9.clone())]),
    };
    let view = ready(run_new_execution_workflow(&layout, &workflow, &mut driver)).unwrap();
    assert_eq!(driver.calls, [0, 7, 9]);
    let binding = |sequence| {
        view.stages()
            .iter()
            .find(|stage| stage.stage_sequence() == sequence)
            .unwrap()
            .binding()
            .clone()
    };
    for (sequence, expected) in [(7, &edge_7), (9, &edge_9)] {
        let stage =
            ExecutionStageGuard::open(&layout, &workflow.digest(), sequence, &binding(sequence))
                .unwrap();
        assert_eq!(
            stage.binding().predecessors()[0].learned_evidence(),
            expected
        );
    }
    let retained = BTreeMap::from([
        (
            0,
            RetainedStageEvidence::new(
                binding(0),
                ArtifactChecksumRecord::from_bytes(&0_u64.to_be_bytes()),
            )
            .with_successor_evidence(BTreeMap::from([(7, edge_7), (9, edge_9)])),
        ),
        (
            7,
            RetainedStageEvidence::new(
                binding(7),
                ArtifactChecksumRecord::from_bytes(&7_u64.to_be_bytes()),
            ),
        ),
        (
            9,
            RetainedStageEvidence::new(
                binding(9),
                ArtifactChecksumRecord::from_bytes(&9_u64.to_be_bytes()),
            ),
        ),
    ]);
    assert_eq!(
        ready(run_retained_execution_workflow(
            &layout,
            &workflow.digest(),
            &retained,
            &mut driver,
        ))
        .unwrap(),
        view
    );
    assert_eq!(driver.calls, [0, 7, 9]);
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
    let mut future = Box::pin(run_new_execution_workflow(&layout, &workflow, &mut driver));
    crate::test_support::ready::assert_pending(future.as_mut());
    drop(future);
    assert_eq!(driver.calls, 1);
    let path = root.join("execution-stage-0/attempt-42.json");
    let original = fs::read(&path).unwrap();
    assert!(matches!(
        ready(run_new_execution_workflow(&layout, &workflow, &mut driver)),
        Err(ExecutionWorkflowRunError::Workflow(_))
    ));
    assert_eq!(driver.calls, 1);
    assert_eq!(fs::read(&path).unwrap(), original);
    assert!(!root.join("execution-stage-7").exists());
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}
