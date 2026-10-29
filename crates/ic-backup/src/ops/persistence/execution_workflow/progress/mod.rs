//! Read-only original workflow inspection through existing plan and journal admission.

use super::{
    ArtifactChecksumRecord, BackupLayoutGuard, ExecutionProgressPersistenceError,
    ExecutionWorkflowPersistenceError,
    ancestry::{acquire_stage, read_admitted_binding, read_current_binding, validate_predecessors},
    read_execution_progress, read_execution_stage_occupancy, read_execution_workflow,
};
use crate::view::execution_workflow::{
    ExecutionStageProgressState, ExecutionStageProgressView, ExecutionWorkflowProgressView,
};
use thiserror::Error;

/// Inspect all original stage journals without calling a provider or changing evidence.
///
/// Hold the workflow layout, but no stage/attempt guards. Each occupied stage must
/// admit its exact binding, ancestors and complete original journal set. Missing
/// journals or partial stage preparation are typed failures, never unused allowance.
/// Pending mutation/observation identities remain visible. Applied rows are not
/// checkpoint, transfer, application or terminal proof. No remote call, write,
/// repair, retry, artifact read or learned-evidence inference occurs.
/// # Errors
/// Rejects unsafe/changed originals, missing journals, contention and invalid accounting.
pub fn read_execution_workflow_progress(
    layout: &BackupLayoutGuard,
    expected: &ArtifactChecksumRecord,
) -> Result<ExecutionWorkflowProgressView, ExecutionWorkflowProgressError> {
    let workflow = read_execution_workflow(layout, expected)?;
    let occupied = read_execution_stage_occupancy(layout, expected)?;
    let mut stages = Vec::with_capacity(workflow.stages().len());
    for node in workflow.allocation().graph().ordered_nodes() {
        let sequence = node.operation_sequence();
        let state = if occupied.contains(&sequence) {
            let child = acquire_stage(layout, sequence)?;
            let (binding, plan) = read_current_binding(&child, &workflow, sequence)?;
            validate_predecessors(layout, &workflow, &binding)?;
            let progress = read_execution_progress(&child, &plan.digest())?;
            read_admitted_binding(layout, &child, &workflow, sequence, &binding.digest())?;
            ExecutionStageProgressState::Prepared {
                binding: binding.digest(),
                progress: Box::new(progress),
            }
        } else {
            ExecutionStageProgressState::Unprepared
        };
        stages.push(ExecutionStageProgressView {
            stage_sequence: sequence,
            state,
        });
    }
    if read_execution_stage_occupancy(layout, expected)? != occupied {
        return Err(ExecutionWorkflowPersistenceError::DigestMismatch.into());
    }
    Ok(ExecutionWorkflowProgressView {
        workflow: expected.clone(),
        stages,
    })
}

/// Local workflow inspection refusal; no original evidence or spending is changed.
#[derive(Debug, Error)]
pub enum ExecutionWorkflowProgressError {
    /// Original workflow, stage, ancestors or filesystem custody failed admission.
    #[error(transparent)]
    Stage(#[from] ExecutionWorkflowPersistenceError),
    /// Complete original journals, accounting or causal progress failed admission.
    #[error(transparent)]
    Journals(#[from] ExecutionProgressPersistenceError),
}
