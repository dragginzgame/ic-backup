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
use std::{collections::BTreeMap, future::Future, num::NonZeroUsize};
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
    execute_remaining_stages(layout, workflow, BTreeMap::new(), usize::MAX, driver).await?;
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
    let (workflow, completed) = admit_retained_stages(layout, expected_workflow, retained)?;
    execute_remaining_stages(layout, &workflow, completed, usize::MAX, driver).await?;
    Ok(read_complete_execution_workflow(layout, expected_workflow)?)
}

/// Bounded operator advancement, without a second progress or spending ledger.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionWorkflowAdvance {
    /// The selected bound was reached between settled stages; original absent stages remain.
    Paused,
    /// Every original stage's exact retained checkpoint passed local replay.
    /// This grants no product terminal or source-reference release authority.
    Complete(ExecutionWorkflowSettlementView),
}

/// Advance at most `stage_limit` absent stages of one prepared original workflow.
///
/// Prepare once with `create_execution_workflow`, retaining the original declarations
/// before effects. Every call uses the same complete occupied-stage evidence admission
/// as `run_retained_execution_workflow`. A pause happens only between settled stages,
/// never halfway through a transfer. The limit bounds stage callbacks, not each stage's
/// original finite effect allowance or elapsed time. All admission, custody, durable
/// reply retention and effect-specific qualification remain mandatory driver duties.
///
/// Pending observations/mutations, partial preparation or changed originals refuse
/// before any callback; the bound cannot reset them or supply resend permission.
/// A failed/cancelled callback retains original uncertainty. Inspect it through
/// `read_execution_workflow_progress` and reconcile through the original owners.
/// Complete replay invokes no driver callbacks, regardless of the selected limit.
/// # Errors
/// Retains the existing typed preparation, checkpoint, original-evidence and driver failures.
pub async fn advance_execution_workflow<D: ExecutionWorkflowStageDriver>(
    layout: &BackupLayoutGuard,
    expected_workflow: &ArtifactChecksumRecord,
    retained: &BTreeMap<u64, RetainedStageEvidence>,
    stage_limit: NonZeroUsize,
    driver: &mut D,
) -> Result<ExecutionWorkflowAdvance, ExecutionWorkflowRunError<D::Error>> {
    let (workflow, completed) = admit_retained_stages(layout, expected_workflow, retained)?;
    match execute_remaining_stages(layout, &workflow, completed, stage_limit.get(), driver).await? {
        StageExecutionOutcome::Complete => Ok(ExecutionWorkflowAdvance::Complete(
            read_complete_execution_workflow(layout, expected_workflow)?,
        )),
        StageExecutionOutcome::Paused => Ok(ExecutionWorkflowAdvance::Paused),
    }
}

// Each completed stage carries the source of its qualified outgoing edge evidence.
struct CompletedStage<'a> {
    checkpoint: ExecutionStagePredecessorRecord,
    retained: Option<&'a RetainedStageEvidence>,
}

fn admit_retained_stages<'a, E: std::error::Error + 'static>(
    layout: &BackupLayoutGuard,
    expected_workflow: &ArtifactChecksumRecord,
    retained: &'a BTreeMap<u64, RetainedStageEvidence>,
) -> Result<
    (ExecutionWorkflowRecord, BTreeMap<u64, CompletedStage<'a>>),
    ExecutionWorkflowRunError<E>,
> {
    let workflow = read_execution_workflow(layout, expected_workflow)?;
    let occupied = read_execution_stage_occupancy(layout, expected_workflow)?;
    if !retained.keys().copied().eq(occupied.iter().copied()) {
        return Err(ExecutionWorkflowRunError::RetainedStageSetMismatch);
    }
    let mut completed = BTreeMap::new();
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
        completed.insert(
            sequence,
            CompletedStage {
                checkpoint,
                retained: Some(evidence),
            },
        );
    }
    // Recheck the namespace before invoking any integration callback. Create-only
    // stage preparation still refuses an occupied path after this preflight.
    if read_execution_stage_occupancy(layout, expected_workflow)? != occupied {
        return Err(ExecutionWorkflowRunError::RetainedStageSetMismatch);
    }
    Ok((workflow, completed))
}

// Entry points finish their own create-only or retained-evidence admission before
// sharing dependency propagation and execution of only the remaining stages.
enum StageExecutionOutcome {
    Complete,
    Paused,
}

async fn execute_remaining_stages<D: ExecutionWorkflowStageDriver>(
    layout: &BackupLayoutGuard,
    workflow: &ExecutionWorkflowRecord,
    mut completed: BTreeMap<u64, CompletedStage<'_>>,
    stage_limit: usize,
    driver: &mut D,
) -> Result<StageExecutionOutcome, ExecutionWorkflowRunError<D::Error>> {
    let mut advanced = 0;
    for node in workflow.allocation().graph().ordered_nodes() {
        let sequence = node.operation_sequence();
        if completed.contains_key(&sequence) {
            continue;
        }
        if advanced == stage_limit {
            return Ok(StageExecutionOutcome::Paused);
        }
        let predecessors = node
            .depends_on()
            .iter()
            .map(|dependency| {
                let original =
                    completed.get(dependency).ok_or(
                        ExecutionWorkflowRunError::<D::Error>::MissingPredecessor(*dependency),
                    )?;
                let learned = if let Some(evidence) = original.retained {
                    evidence.learned_for_successor(sequence).clone()
                } else {
                    driver
                        .learned_for_successor(&original.checkpoint, sequence)
                        .map_err(ExecutionWorkflowRunError::Driver)?
                };
                Ok(ExecutionStagePredecessorRecord::new(
                    *dependency,
                    original.checkpoint.binding().clone(),
                    original.checkpoint.settlement().clone(),
                    learned,
                ))
            })
            .collect::<Result<Vec<_>, ExecutionWorkflowRunError<D::Error>>>()?;
        let checkpoint =
            execute_absent_stage(layout, workflow, sequence, predecessors, driver).await?;
        completed.insert(
            sequence,
            CompletedStage {
                checkpoint,
                retained: None,
            },
        );
        advanced += 1;
    }
    Ok(StageExecutionOutcome::Complete)
}

async fn execute_absent_stage<D: ExecutionWorkflowStageDriver>(
    layout: &BackupLayoutGuard,
    workflow: &ExecutionWorkflowRecord,
    sequence: u64,
    predecessors: Vec<ExecutionStagePredecessorRecord>,
    driver: &mut D,
) -> Result<ExecutionStagePredecessorRecord, ExecutionWorkflowRunError<D::Error>> {
    let plan = driver
        .bind_stage(workflow, sequence, &predecessors)
        .map_err(ExecutionWorkflowRunError::Driver)?;
    let binding = ExecutionStageBindingRecord::new(workflow, sequence, &plan, predecessors)?;
    let stage = ExecutionStageGuard::prepare(layout, binding, plan)?;
    let learned = driver
        .execute_stage(&stage)
        .await
        .map_err(ExecutionWorkflowRunError::Driver)?;
    Ok(stage.reconcile_checkpoint(learned)?)
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
