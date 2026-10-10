//! Exact bounded data-upload declarations, retaining no aggregate payload bytes.

use super::{
    IcSnapshotUploadError, IcSnapshotUploadReply, IcSnapshotUploadReplyKind,
    IcSnapshotUploadRequest,
};
use crate::model::{
    artifacts::ArtifactChecksumRecord,
    attempt_journal::{AttemptBudgetRecord, AttemptJournalRecordError},
    effect_graph::{EffectGraphError, EffectGraphRecord, EffectNodeRecord, EffectNodeRequest},
    execution_workflow::{
        ExecutionStageBindingRecord, ExecutionStagePredecessorRecord, ExecutionWorkflowError,
        ExecutionWorkflowRecord,
    },
    ic_snapshot_data::{ExtentPlanningError, planned_extents},
    operation_plan::{
        OperationPlanError, OperationPlanRecord, OperationPlanRequest, PlanBudgetRecord,
        PlannedOperationRecord, PlannedOperationRequest,
    },
};
use ic_management_canister_types::SnapshotDataKind;
use thiserror::Error;

/// Complete ordered extents and source-bound request digests for one original data stage.
///
/// Guarded local source preparation creates this declaration. Module, heap and stable
/// regions precede known chunks in original metadata order. Every operation has one
/// update and zero observations with an explicit predecessor dependency. Original
/// stage headroom stays unassigned. No upload bytes, spending, authenticated allocation
/// or complete-transfer proof are retained here. Empty data needs no placeholder plan.
pub struct IcSnapshotDataUploadPlan<'workflow, 'request, 'source> {
    workflow: &'workflow ExecutionWorkflowRecord,
    sequence: u64,
    metadata: &'request IcSnapshotUploadRequest<'source>,
    destination: Vec<u8>,
    allocation_evidence: ArtifactChecksumRecord,
    kinds: Vec<SnapshotDataKind>,
    plan: Option<OperationPlanRecord>,
}
impl<'workflow, 'request, 'source> IcSnapshotDataUploadPlan<'workflow, 'request, 'source> {
    pub(crate) fn derive_kinds(
        workflow: &ExecutionWorkflowRecord,
        sequence: u64,
        allocation: &IcSnapshotUploadReply<'_, '_>,
        chunk_bytes: u64,
    ) -> Result<Vec<SnapshotDataKind>, IcSnapshotDataUploadPlanningError> {
        let metadata = allocation.request();
        let stage = workflow.stage(sequence)?;
        let IcSnapshotUploadReplyKind::Metadata { snapshot_id } = allocation.kind() else {
            return Err(IcSnapshotDataUploadPlanningError::OriginalMismatch);
        };
        metadata.validate_data_destination(snapshot_id)?;
        if stage.target() != metadata.target()
            || !super::attempt::source_context_matches(workflow.allocation(), metadata)
        {
            return Err(IcSnapshotDataUploadPlanningError::OriginalMismatch);
        }
        Ok(planned_extents(
            metadata.source(),
            chunk_bytes,
            stage.budget().mutations(),
        )?)
    }

    pub(crate) fn from_bindings(
        workflow: &'workflow ExecutionWorkflowRecord,
        sequence: u64,
        allocation: &IcSnapshotUploadReply<'request, 'source>,
        chunk_bytes: u64,
        bindings: &[ArtifactChecksumRecord],
    ) -> Result<Self, IcSnapshotDataUploadPlanningError> {
        let kinds = Self::derive_kinds(workflow, sequence, allocation, chunk_bytes)?;
        if bindings.len() != kinds.len() {
            return Err(IcSnapshotDataUploadPlanningError::OriginalMismatch);
        }
        let IcSnapshotUploadReplyKind::Metadata { snapshot_id } = allocation.kind() else {
            return Err(IcSnapshotDataUploadPlanningError::OriginalMismatch);
        };
        let stage = workflow.stage(sequence)?;
        let plan = if kinds.is_empty() {
            None
        } else {
            let mut nodes = Vec::with_capacity(kinds.len());
            let mut operations = Vec::with_capacity(kinds.len());
            for (index, request) in bindings.iter().enumerate() {
                let operation_sequence = u64::try_from(index)
                    .map_err(|_| IcSnapshotDataUploadPlanningError::CountOverflow)?;
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
            let original = workflow.allocation();
            Some(OperationPlanRecord::new(OperationPlanRequest {
                context: original.context().clone(),
                inventory: original.inventory().clone(),
                selected_targets: vec![stage.target().into()],
                graph: EffectGraphRecord::new(nodes)?,
                operations,
                budget: PlanBudgetRecord::new(
                    stage.budget().mutations(),
                    stage.budget().observations(),
                )?,
            })?)
        };
        Ok(Self {
            workflow,
            sequence,
            metadata: allocation.request(),
            destination: snapshot_id.clone(),
            allocation_evidence: allocation.digest(),
            kinds,
            plan,
        })
    }

    /// Read exact ordered source extents. Operation sequences are zero-based indices.
    #[must_use]
    pub fn kinds(&self) -> &[SnapshotDataKind] {
        &self.kinds
    }
    /// Read the original metadata allocation declaration and retained source association.
    #[must_use]
    pub const fn metadata(&self) -> &'request IcSnapshotUploadRequest<'source> {
        self.metadata
    }
    /// Read exact declared destination bytes; they grant no allocation authority.
    #[must_use]
    pub fn destination(&self) -> &[u8] {
        &self.destination
    }
    /// Read the immutable child plan, or None when there are no data writes.
    #[must_use]
    pub const fn plan(&self) -> Option<&OperationPlanRecord> {
        self.plan.as_ref()
    }

    /// Bind the exact original singleton allocation and request/raw-reply evidence.
    ///
    /// Existing stage persistence requires complete Applied predecessor journals and
    /// their chronological checkpoint. This check authenticates neither allocation
    /// attribution nor byte custody and grants no dispatch/load/start authority.
    /// # Errors
    /// Rejects an empty plan, changed allocation/request/evidence or original dependency.
    pub fn bind(
        &self,
        allocation_binding: &ExecutionStageBindingRecord,
        allocation_plan: &OperationPlanRecord,
        predecessors: Vec<ExecutionStagePredecessorRecord>,
    ) -> Result<ExecutionStageBindingRecord, IcSnapshotDataUploadPlanningError> {
        let plan = self
            .plan
            .as_ref()
            .ok_or(IcSnapshotDataUploadPlanningError::NoDataWrites)?;
        allocation_binding.validate(self.workflow, allocation_plan)?;
        if allocation_plan.operations().len() != 1
            || allocation_plan.operations()[0].request() != self.metadata.binding_digest().hash()
            || !predecessors.iter().any(|row| {
                row.stage_sequence() == allocation_binding.stage_sequence()
                    && row.binding() == &allocation_binding.digest()
                    && row.learned_evidence() == &self.allocation_evidence
            })
        {
            return Err(IcSnapshotDataUploadPlanningError::OriginalMismatch);
        }
        Ok(ExecutionStageBindingRecord::new(
            self.workflow,
            self.sequence,
            plan,
            predecessors,
        )?)
    }
    pub(crate) fn validate_binding(
        &self,
        binding: &ExecutionStageBindingRecord,
    ) -> Result<(), IcSnapshotDataUploadPlanningError> {
        let plan = self
            .plan
            .as_ref()
            .ok_or(IcSnapshotDataUploadPlanningError::NoDataWrites)?;
        binding.validate(self.workflow, plan)?;
        if binding.stage_sequence() != self.sequence
            || !binding
                .predecessors()
                .iter()
                .any(|row| row.learned_evidence() == &self.allocation_evidence)
        {
            return Err(IcSnapshotDataUploadPlanningError::OriginalMismatch);
        }
        Ok(())
    }
}

/// Planning rejection changes no source bytes, journal, allowance or remote effects.
#[derive(Debug, Error)]
pub enum IcSnapshotDataUploadPlanningError {
    /// Chunk size is outside the existing 1..=1 MiB data boundary.
    #[error("invalid snapshot data upload chunk size")]
    InvalidChunkSize,
    /// Complete declared extent counts overflow nat64 or host indexing.
    #[error("snapshot data upload count overflow")]
    CountOverflow,
    /// Complete data writes cannot fit their original stage allocation.
    #[error("snapshot data upload needs {required} updates; original allocation is {original}")]
    InsufficientAllowance {
        /// Complete required update count, excluding metadata allocation.
        required: u64,
        /// Original data-stage ceiling, excluding unassigned workflow headroom.
        original: u32,
    },
    /// Original context, source, allocation, request or predecessor evidence differs.
    #[error("snapshot data upload originals differ")]
    OriginalMismatch,
    /// Empty data has no placeholder child plan or checkpoint.
    #[error("snapshot data upload has no data writes to bind")]
    NoDataWrites,
    /// Existing immutable operation plan admission.
    #[error(transparent)]
    Plan(#[from] OperationPlanError),
    /// Existing explicit graph admission.
    #[error(transparent)]
    Graph(#[from] EffectGraphError),
    /// Existing original stage binding admission.
    #[error(transparent)]
    Workflow(#[from] ExecutionWorkflowError),
    /// Existing exact source/destination codec admission.
    #[error(transparent)]
    Upload(#[from] IcSnapshotUploadError),
    /// Existing original per-operation attempt admission.
    #[error(transparent)]
    Budget(#[from] AttemptJournalRecordError),
}
impl From<ExtentPlanningError> for IcSnapshotDataUploadPlanningError {
    fn from(error: ExtentPlanningError) -> Self {
        match error {
            ExtentPlanningError::InvalidChunkSize => Self::InvalidChunkSize,
            ExtentPlanningError::CountOverflow => Self::CountOverflow,
            ExtentPlanningError::InsufficientAllowance { required, original } => {
                Self::InsufficientAllowance { required, original }
            }
        }
    }
}

#[cfg(test)]
mod tests;
