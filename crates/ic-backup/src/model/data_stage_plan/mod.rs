//! Shared exact child-plan topology for metadata-derived snapshot data stages.

use crate::model::{
    artifacts::ArtifactChecksumRecord,
    attempt_journal::{AttemptBudgetRecord, AttemptJournalRecordError},
    effect_graph::{EffectGraphError, EffectGraphRecord, EffectNodeRecord, EffectNodeRequest},
    execution_workflow::{
        ExecutionStageBindingRecord, ExecutionStagePredecessorRecord, ExecutionWorkflowError,
        ExecutionWorkflowRecord,
    },
    operation_plan::{
        OperationPlanError, OperationPlanRecord, OperationPlanRequest, PlanBudgetRecord,
        PlannedOperationRecord, PlannedOperationRequest,
    },
};

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
    ) -> Result<(), LearnedDataStageError> {
        binding.validate(self.workflow, plan)?;
        if !binding.has_learned_evidence(self.sequence, &self.evidence) {
            return Err(LearnedDataStageError::Mismatch);
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
    ) -> Result<ExecutionStageBindingRecord, LearnedDataStageError> {
        source_binding.validate(self.workflow, source_plan)?;
        if !source_binding.matches_learned_predecessor(
            source_plan,
            source_request,
            &self.evidence,
            &predecessors,
        ) {
            return Err(LearnedDataStageError::Mismatch);
        }
        Ok(ExecutionStageBindingRecord::new(
            self.workflow,
            self.sequence,
            plan,
            predecessors,
        )?)
    }
}

// Preserve structural workflow failures separately from learned-evidence mismatch.
pub(crate) enum LearnedDataStageError {
    Workflow(ExecutionWorkflowError),
    Mismatch,
}
impl From<ExecutionWorkflowError> for LearnedDataStageError {
    fn from(error: ExecutionWorkflowError) -> Self {
        Self::Workflow(error)
    }
}

/// Build one sequential, single-update operation per original ordered request digest.
pub(crate) fn build(
    original: &OperationPlanRecord,
    stage: &PlannedOperationRecord,
    requests: impl ExactSizeIterator<Item = ArtifactChecksumRecord>,
) -> Result<Option<OperationPlanRecord>, DataStagePlanError> {
    if requests.len() == 0 {
        return Ok(None);
    }
    let mut nodes = Vec::with_capacity(requests.len());
    let mut operations = Vec::with_capacity(requests.len());
    for (index, request) in requests.enumerate() {
        let operation_sequence =
            u64::try_from(index).map_err(|_| DataStagePlanError::CountOverflow)?;
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

pub(crate) enum DataStagePlanError {
    CountOverflow,
    Graph(EffectGraphError),
    Plan(OperationPlanError),
    Budget(AttemptJournalRecordError),
}

impl From<EffectGraphError> for DataStagePlanError {
    fn from(error: EffectGraphError) -> Self {
        Self::Graph(error)
    }
}

impl From<OperationPlanError> for DataStagePlanError {
    fn from(error: OperationPlanError) -> Self {
        Self::Plan(error)
    }
}

impl From<AttemptJournalRecordError> for DataStagePlanError {
    fn from(error: AttemptJournalRecordError) -> Self {
        Self::Budget(error)
    }
}
