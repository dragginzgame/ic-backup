//! Shared exact child-plan topology for metadata-derived snapshot data stages.

use crate::model::{
    artifacts::ArtifactChecksumRecord,
    attempt_journal::{AttemptBudgetRecord, AttemptJournalRecordError},
    effect_graph::{EffectGraphError, EffectGraphRecord, EffectNodeRecord, EffectNodeRequest},
    execution_workflow::{
        ExecutionStageBindingRecord, ExecutionStagePredecessorRecord, ExecutionWorkflowError,
        ExecutionWorkflowRecord,
    },
    ic_snapshot_data::IcSnapshotDataError,
    ic_snapshot_upload::IcSnapshotUploadError,
    operation_plan::{
        OperationPlanError, OperationPlanRecord, OperationPlanRequest, PlanBudgetRecord,
        PlannedOperationRecord, PlannedOperationRequest,
    },
};
use thiserror::Error;

/// Original workflow edge and exact learned evidence shared by data-stage plans.
pub(crate) struct LearnedDataStage<'a> {
    pub workflow: &'a ExecutionWorkflowRecord,
    pub sequence: u64,
    pub evidence: ArtifactChecksumRecord,
}

impl LearnedDataStage<'_> {
    pub fn validate(
        &self,
        binding: &ExecutionStageBindingRecord,
        plan: &OperationPlanRecord,
    ) -> Result<(), SnapshotDataPlanningError> {
        binding.validate(self.workflow, plan)?;
        if !binding.has_learned_evidence(self.sequence, &self.evidence) {
            return Err(SnapshotDataPlanningError::OriginalMismatch);
        }
        Ok(())
    }

    pub fn bind(
        &self,
        plan: &OperationPlanRecord,
        source_binding: &ExecutionStageBindingRecord,
        source_plan: &OperationPlanRecord,
        source_request: &str,
        predecessors: Vec<ExecutionStagePredecessorRecord>,
    ) -> Result<ExecutionStageBindingRecord, SnapshotDataPlanningError> {
        source_binding.validate(self.workflow, source_plan)?;
        if !source_binding.matches_learned_predecessor(
            source_plan,
            source_request,
            &self.evidence,
            &predecessors,
        ) {
            return Err(SnapshotDataPlanningError::OriginalMismatch);
        }
        Ok(ExecutionStageBindingRecord::new(
            self.workflow,
            self.sequence,
            plan,
            predecessors,
        )?)
    }
}

/// Build one sequential, single-update operation per original ordered request digest.
pub(crate) fn build(
    original: &OperationPlanRecord,
    stage: &PlannedOperationRecord,
    requests: impl ExactSizeIterator<Item = ArtifactChecksumRecord>,
) -> Result<Option<OperationPlanRecord>, SnapshotDataPlanningError> {
    if requests.len() == 0 {
        return Ok(None);
    }
    let mut nodes = Vec::with_capacity(requests.len());
    let mut operations = Vec::with_capacity(requests.len());
    for (index, request) in requests.enumerate() {
        let operation_sequence =
            u64::try_from(index).map_err(|_| SnapshotDataPlanningError::CountOverflow)?;
        nodes.push(EffectNodeRecord::new(EffectNodeRequest {
            operation_sequence,
            depends_on: operation_sequence.checked_sub(1).into_iter().collect(),
        })?);
        operations.push(PlannedOperationRecord::new(PlannedOperationRequest {
            operation_sequence,
            target: stage.target().into(),
            request: request.hash().into(),
            budget: AttemptBudgetRecord::new(1, 0)?,
        })?);
    }
    Ok(Some(OperationPlanRecord::new(OperationPlanRequest {
        context: original.context().clone(),
        inventory: original.inventory().clone(),
        selected_targets: vec![stage.target().into()],
        graph: EffectGraphRecord::new(nodes)?,
        operations,
        budget: PlanBudgetRecord::new(stage.budget().mutations(), stage.budget().observations())?,
    })?))
}

/// Pure rejection of an original metadata-derived download or upload data stage.
/// No error resets allocation, creates a stage or grants transfer/retry authority.
#[derive(Debug, Error)]
pub enum SnapshotDataPlanningError {
    /// Per-call data chunk size must be in 1..=1 MiB.
    #[error("invalid snapshot data chunk size")]
    InvalidChunkSize,
    /// Declared extent count overflows nat64 or host indexing.
    #[error("snapshot data request count overflow")]
    CountOverflow,
    /// Complete data calls exceed the stage's original mutation allocation.
    #[error("snapshot data needs {required} updates; original allocation is {original}")]
    InsufficientAllowance {
        /// Complete required data-call count.
        required: u64,
        /// Original stage mutation ceiling, excluding workflow headroom.
        original: u32,
    },
    /// Target, original source request, allocation or learned evidence differs.
    #[error("snapshot data originals differ")]
    OriginalMismatch,
    /// Empty data has no placeholder child plan or binding.
    #[error("snapshot data has no operations to bind")]
    NoDataOperations,
    /// Original operation-plan admission failed.
    #[error(transparent)]
    Plan(#[from] OperationPlanError),
    /// Explicit dependency-graph admission failed.
    #[error(transparent)]
    Graph(#[from] EffectGraphError),
    /// Original workflow or learned stage binding failed.
    #[error(transparent)]
    Workflow(#[from] ExecutionWorkflowError),
    /// Metadata-bound read request admission failed.
    #[error(transparent)]
    Data(#[from] IcSnapshotDataError),
    /// Source/destination upload request admission failed.
    #[error(transparent)]
    Upload(#[from] IcSnapshotUploadError),
    /// Original per-operation attempt admission failed.
    #[error(transparent)]
    Budget(#[from] AttemptJournalRecordError),
}
