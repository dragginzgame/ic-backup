//! Shared original-plan fixture builders; assertions remain beside their owners.

use crate::model::execution_workflow::*;
use crate::model::{
    effect_graph::{EffectGraphRecord, EffectNodeRecord, EffectNodeRequest},
    operation_plan::{
        OperationPlanRecord, OperationPlanRequest, PlanBudgetRecord, PlannedOperationRecord,
        PlannedOperationRequest,
    },
};
use crate::ops::persistence::{BackupLayoutGuard, ExecutionStageGuard, create_execution_workflow};
use std::path::PathBuf;

pub(crate) fn workflow() -> ExecutionWorkflowRecord {
    ExecutionWorkflowRecord::new(
        OperationPlanRecord::new(crate::test_support::operation_plan::request()).unwrap(),
    )
}
pub(crate) fn child_request(sequence: u64) -> OperationPlanRequest {
    let mut request = crate::test_support::operation_plan::request();
    let stage = workflow().stage(sequence).unwrap().clone();
    request.graph = EffectGraphRecord::new(vec![
        EffectNodeRecord::new(EffectNodeRequest {
            operation_sequence: 42,
            depends_on: vec![],
        })
        .unwrap(),
    ])
    .unwrap();
    request.operations = vec![
        PlannedOperationRecord::new(PlannedOperationRequest {
            operation_sequence: 42,
            target: stage.target().into(),
            request: "78".repeat(32),
            budget: stage.budget().clone(),
        })
        .unwrap(),
    ];
    request.budget =
        PlanBudgetRecord::new(stage.budget().mutations(), stage.budget().observations()).unwrap();
    request
}
pub(crate) fn child(sequence: u64) -> OperationPlanRecord {
    OperationPlanRecord::new(child_request(sequence)).unwrap()
}

/// Prepare one fresh fixture stage; plans, effects and assertions remain with each scenario.
pub(crate) fn prepare_stage(
    prefix: &str,
    workflow: &ExecutionWorkflowRecord,
    sequence: u64,
    plan: OperationPlanRecord,
) -> (PathBuf, BackupLayoutGuard, ExecutionStageBindingRecord) {
    let root = crate::test_support::temp_dir(prefix);
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    create_execution_workflow(&layout, workflow).unwrap();
    let binding = ExecutionStageBindingRecord::new(workflow, sequence, &plan, vec![]).unwrap();
    drop(ExecutionStageGuard::prepare(&layout, binding.clone(), plan).unwrap());
    (root, layout, binding)
}
