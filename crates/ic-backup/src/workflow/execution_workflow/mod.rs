//! Original-workflow stage coordination without a default IC or application adapter.

use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        execution_workflow::{
            ExecutionStageBindingRecord, ExecutionStagePredecessorRecord, ExecutionWorkflowError,
            ExecutionWorkflowRecord,
        },
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{
        BackupLayoutGuard, ExecutionStageCheckpointError, ExecutionStageGuard,
        ExecutionStagePreparationError, ExecutionWorkflowPersistenceError,
        ExecutionWorkflowSettlementView, create_execution_workflow,
        read_complete_execution_workflow, read_execution_stage_occupancy, read_execution_workflow,
    },
};
use std::{collections::BTreeMap, future::Future};
use thiserror::Error;

/// Integration-owned exact child planning and one-shot execution of each original stage.
///
/// `bind_stage` must qualify learned inputs against the supplied original predecessors.
/// `execute_stage` owns fresh effect-boundary admission, provider calls, authenticated
/// request/reply retention and explicit original Applied receipts. Its returned digest
/// is opaque learned evidence; the coordinator does not authenticate it. Neither
/// callback may treat local checkpoint admission as a dispatch or release permit.
pub trait ExecutionWorkflowStageDriver {
    /// Integration-owned planning, admission or execution error.
    type Error: std::error::Error + 'static;

    /// Build one exact child plan under the original stage allocation.
    /// # Errors
    /// Returns integration-owned learned-input or child-planning rejection.
    fn bind_stage(
        &mut self,
        workflow: &ExecutionWorkflowRecord,
        stage_sequence: u64,
        predecessors: &[ExecutionStagePredecessorRecord],
    ) -> Result<OperationPlanRecord, Self::Error>;

    /// Execute only the already prepared original child; retain original effects.
    fn execute_stage(
        &mut self,
        stage: &ExecutionStageGuard<'_>,
    ) -> impl Future<Output = Result<ArtifactChecksumRecord, Self::Error>>;

    /// Qualify the learned commitment for one outgoing dependency edge.
    ///
    /// The default reuses the stage checkpoint's learned digest. An integration
    /// may derive a distinct commitment from its retained original evidence;
    /// this callback must not dispatch effects or change original spending.
    /// # Errors
    /// Refuses an unauthenticated or unavailable edge commitment before the
    /// successor is prepared or its driver callback is invoked.
    fn learned_for_successor(
        &mut self,
        predecessor: &ExecutionStagePredecessorRecord,
        _successor_sequence: u64,
    ) -> Result<ArtifactChecksumRecord, Self::Error> {
        Ok(predecessor.learned_evidence().clone())
    }
}

/// Integration-qualified identities for an already settled original stage.
///
/// Local replay derives its binding and settlement from retained records. The
/// learned digest is supplied by the integration, which must authenticate the
/// original response and its relation to the settled attempt before resuming.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetainedStageEvidence {
    binding: ArtifactChecksumRecord,
    learned_evidence: ArtifactChecksumRecord,
    successor_evidence: BTreeMap<u64, ArtifactChecksumRecord>,
}
impl RetainedStageEvidence {
    /// Supply an original binding identity and independently qualified learned input.
    #[must_use]
    pub const fn new(
        binding: ArtifactChecksumRecord,
        learned_evidence: ArtifactChecksumRecord,
    ) -> Self {
        Self {
            binding,
            learned_evidence,
            successor_evidence: BTreeMap::new(),
        }
    }

    /// Supply integration-qualified commitments for outgoing edges that differ
    /// from this stage's checkpoint digest. Keys must name original successors;
    /// occupied successor bindings are checked before any driver callback.
    #[must_use]
    pub fn with_successor_evidence(
        mut self,
        successor_evidence: BTreeMap<u64, ArtifactChecksumRecord>,
    ) -> Self {
        self.successor_evidence = successor_evidence;
        self
    }

    fn learned_for_successor(&self, sequence: u64) -> &ArtifactChecksumRecord {
        self.successor_evidence
            .get(&sequence)
            .unwrap_or(&self.learned_evidence)
    }
}

/// Coordinate one *new* fixed workflow in its declared dependency order.
///
/// Publish the original allocation once, prepare each child and all its original
/// journals before invoking the driver, then reconcile its exact all-Applied
/// checkpoint. The graph's topological order controls stage dependencies, never
/// numeric sequence order. Success reads back every retained checkpoint through
/// the canonical complete-workflow owner. No automatic outcome, retry, refund,
/// terminal receipt, fence or reference release is inferred.
///
/// This is a deliberately one-shot entry point. A failure or cancellation retains
/// the original workflow, prepared stages, pending spending, artifacts and all
/// obligations. Calling it again refuses the occupied workflow *before* invoking
/// the driver. Recovery must reopen and reconcile those exact originals through
/// their separate owners. Retained continuation requires a separately qualified
/// complete occupied-stage set; a new run cannot reconstruct authority from a prefix.
/// # Errors
/// Rejects occupied or changed originals, incomplete stages and driver failures,
/// preserving all retained evidence without reissuing an effect.
pub async fn run_new_execution_workflow<D: ExecutionWorkflowStageDriver>(
    layout: &BackupLayoutGuard,
    workflow: &ExecutionWorkflowRecord,
    driver: &mut D,
) -> Result<ExecutionWorkflowSettlementView, ExecutionWorkflowRunError<D::Error>> {
    create_execution_workflow(layout, workflow)?;
    let mut completed: BTreeMap<u64, ExecutionStagePredecessorRecord> = BTreeMap::new();
    for node in workflow.allocation().graph().ordered_nodes() {
        let sequence = node.operation_sequence();
        let predecessors = node
            .depends_on()
            .iter()
            .map(|dependency| {
                let original =
                    completed.get(dependency).ok_or(
                        ExecutionWorkflowRunError::<D::Error>::MissingPredecessor(*dependency),
                    )?;
                let learned = driver
                    .learned_for_successor(original, sequence)
                    .map_err(ExecutionWorkflowRunError::Driver)?;
                Ok(ExecutionStagePredecessorRecord::new(
                    *dependency,
                    original.binding().clone(),
                    original.settlement().clone(),
                    learned,
                ))
            })
            .collect::<Result<Vec<_>, ExecutionWorkflowRunError<D::Error>>>()?;
        let plan = driver
            .bind_stage(workflow, sequence, &predecessors)
            .map_err(ExecutionWorkflowRunError::Driver)?;
        let binding = ExecutionStageBindingRecord::new(workflow, sequence, &plan, predecessors)?;
        let stage = ExecutionStageGuard::prepare(layout, binding, plan)?;
        let learned = driver
            .execute_stage(&stage)
            .await
            .map_err(ExecutionWorkflowRunError::Driver)?;
        let checkpoint = stage.reconcile_checkpoint(learned)?;
        completed.insert(sequence, checkpoint);
    }
    Ok(read_complete_execution_workflow(
        layout,
        &workflow.digest(),
    )?)
}

/// Continue an existing allocation after replaying every occupied settled stage.
///
/// The caller supplies the complete occupied stage set and authenticates each
/// stage's learned evidence. Before any driver callback, the coordinator checks
/// that set against the held layout and replays every exact all-Applied original
/// checkpoint. Already bound successor edges must commit the qualified digest
/// supplied for that exact edge; absent overrides use the stage checkpoint's
/// learned digest. Pending, partially prepared or omitted occupied stages refuse;
/// none is reissued. Only absent stages may be prepared and executed once, with
/// fresh authority and external-effect admission remaining with the driver.
/// This is no terminal receipt, same-ID restore or reference-release permit.
/// # Errors
/// Rejects incomplete or changed retained evidence before new driver callbacks;
/// later stage failures preserve every original journal and effect obligation.
pub async fn run_retained_execution_workflow<D: ExecutionWorkflowStageDriver>(
    layout: &BackupLayoutGuard,
    expected_workflow: &ArtifactChecksumRecord,
    retained: &BTreeMap<u64, RetainedStageEvidence>,
    driver: &mut D,
) -> Result<ExecutionWorkflowSettlementView, ExecutionWorkflowRunError<D::Error>> {
    let workflow = read_execution_workflow(layout, expected_workflow)?;
    let occupied = read_execution_stage_occupancy(layout, expected_workflow)?;
    if !retained.keys().copied().eq(occupied.iter().copied()) {
        return Err(ExecutionWorkflowRunError::RetainedStageSetMismatch);
    }
    let mut completed: BTreeMap<u64, ExecutionStagePredecessorRecord> = BTreeMap::new();
    for (&sequence, evidence) in retained {
        for successor in evidence.successor_evidence.keys() {
            let Ok(node) = workflow.allocation().graph().node(*successor) else {
                return Err(ExecutionWorkflowRunError::RetainedLearnedMismatch(sequence));
            };
            if !node.depends_on().contains(&sequence) {
                return Err(ExecutionWorkflowRunError::RetainedLearnedMismatch(sequence));
            }
        }
        let stage =
            ExecutionStageGuard::open(layout, expected_workflow, sequence, &evidence.binding)?;
        for predecessor in stage.binding().predecessors() {
            let Some(original) = retained.get(&predecessor.stage_sequence()) else {
                return Err(ExecutionWorkflowRunError::RetainedStageSetMismatch);
            };
            if original.learned_for_successor(sequence) != predecessor.learned_evidence() {
                return Err(ExecutionWorkflowRunError::RetainedLearnedMismatch(sequence));
            }
        }
        let checkpoint = stage.replay_checkpoint(evidence.learned_evidence.clone())?;
        completed.insert(sequence, checkpoint);
    }
    // Recheck the namespace before invoking any integration callback. Create-only
    // stage preparation still refuses an occupied path after this preflight.
    if read_execution_stage_occupancy(layout, expected_workflow)? != occupied {
        return Err(ExecutionWorkflowRunError::RetainedStageSetMismatch);
    }
    for node in workflow.allocation().graph().ordered_nodes() {
        let sequence = node.operation_sequence();
        if completed.contains_key(&sequence) {
            continue;
        }
        let predecessors = node
            .depends_on()
            .iter()
            .map(|dependency| {
                let original =
                    completed.get(dependency).ok_or(
                        ExecutionWorkflowRunError::<D::Error>::MissingPredecessor(*dependency),
                    )?;
                let learned = if let Some(evidence) = retained.get(dependency) {
                    evidence.learned_for_successor(sequence).clone()
                } else {
                    driver
                        .learned_for_successor(original, sequence)
                        .map_err(ExecutionWorkflowRunError::Driver)?
                };
                Ok(ExecutionStagePredecessorRecord::new(
                    *dependency,
                    original.binding().clone(),
                    original.settlement().clone(),
                    learned,
                ))
            })
            .collect::<Result<Vec<_>, ExecutionWorkflowRunError<D::Error>>>()?;
        let plan = driver
            .bind_stage(&workflow, sequence, &predecessors)
            .map_err(ExecutionWorkflowRunError::Driver)?;
        let binding = ExecutionStageBindingRecord::new(&workflow, sequence, &plan, predecessors)?;
        let stage = ExecutionStageGuard::prepare(layout, binding, plan)?;
        let learned = driver
            .execute_stage(&stage)
            .await
            .map_err(ExecutionWorkflowRunError::Driver)?;
        let checkpoint = stage.reconcile_checkpoint(learned)?;
        completed.insert(sequence, checkpoint);
    }
    Ok(read_complete_execution_workflow(layout, expected_workflow)?)
}

/// Failure of new original-workflow coordination; all originals remain retained.
#[derive(Debug, Error)]
pub enum ExecutionWorkflowRunError<E: std::error::Error + 'static> {
    /// A validated predecessor was not available in the current run.
    #[error("original workflow predecessor {0} did not complete")]
    MissingPredecessor(u64),
    /// Supplied identities do not cover exactly the occupied original stage paths.
    #[error("retained stage identities do not match occupied original stages")]
    RetainedStageSetMismatch,
    /// An occupied successor froze a different learned predecessor commitment,
    /// or supplied edge evidence names a non-successor.
    #[error("retained stage {0} has a different learned predecessor commitment")]
    RetainedLearnedMismatch(u64),
    /// Original allocation could not be published or was already occupied.
    #[error(transparent)]
    Workflow(#[from] ExecutionWorkflowPersistenceError),
    /// A driver-supplied child violated the fixed original allocation.
    #[error(transparent)]
    Binding(#[from] ExecutionWorkflowError),
    /// Complete child preparation failed without replacing original evidence.
    #[error(transparent)]
    Preparation(#[from] ExecutionStagePreparationError),
    /// Driver-owned planning, qualification, provider or receipt step failed.
    #[error("original workflow stage driver failed: {0}")]
    Driver(#[source] E),
    /// Canonical all-Applied checkpoint or final local replay failed.
    #[error(transparent)]
    Checkpoint(#[from] ExecutionStageCheckpointError),
}

#[cfg(test)]
mod tests;
