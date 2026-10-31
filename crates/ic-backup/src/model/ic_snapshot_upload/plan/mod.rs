//! Exact bounded data-upload declarations, retaining no aggregate payload bytes.

use super::{
    IcSnapshotUploadError, IcSnapshotUploadReply, IcSnapshotUploadReplyKind,
    IcSnapshotUploadRequest,
};
use crate::model::{
    artifacts::ArtifactChecksumRecord,
    attempt_journal::AttemptJournalRecordError,
    data_stage_plan::{self, DataStagePlanError, LearnedDataStage, LearnedDataStageError},
    effect_graph::EffectGraphError,
    execution_workflow::{
        ExecutionStageBindingRecord, ExecutionStagePredecessorRecord, ExecutionWorkflowError,
        ExecutionWorkflowRecord,
    },
    ic_snapshot_data::{ExtentPlanningError, planned_extents},
    operation_plan::{OperationPlanError, OperationPlanRecord, PlannedOperationRecord},
};
use ic_platform::management_canister_types::SnapshotDataKind;
use thiserror::Error;

/// Complete ordered extents and source-bound request digests for one original data stage.
///
/// Guarded local source preparation creates this declaration. Module, heap and stable
/// regions precede known chunks in original metadata order. Every operation has one
/// update and zero observations with an explicit predecessor dependency. Original
/// stage headroom stays unassigned. No upload bytes, spending, authenticated allocation
/// or complete-transfer proof are retained here. Empty data needs no placeholder plan.
pub struct IcSnapshotDataUploadPlan<'workflow, 'request, 'source> {
    binding: LearnedDataStage<'workflow>,
    metadata: &'request IcSnapshotUploadRequest<'source>,
    destination: Vec<u8>,
    writes: DataWrites,
}

// Empty data has neither extents nor a placeholder plan; writes retain both together.
#[expect(
    clippy::large_enum_variant,
    reason = "keep the original plan inline rather than add a heap allocation to every nonempty upload"
)]
enum DataWrites {
    Empty,
    Writes {
        kinds: Vec<SnapshotDataKind>,
        plan: OperationPlanRecord,
    },
}
// Completion records the admitted operation, never a new effect or persisted mode.
#[derive(Clone)]
pub(crate) enum UploadCompletion {
    Data,
    Allocation {
        sequence: u64,
        evidence: ArtifactChecksumRecord,
    },
}

// Carry the admitted allocation and bounded extents across guarded source IO.
// Only finish can turn their exact ordered byte bindings into a public plan.
pub(crate) struct DataUploadPreparation<'workflow, 'request, 'source> {
    workflow: &'workflow ExecutionWorkflowRecord,
    stage: &'workflow PlannedOperationRecord,
    metadata: &'request IcSnapshotUploadRequest<'source>,
    destination: Vec<u8>,
    allocation_evidence: ArtifactChecksumRecord,
    kinds: Vec<SnapshotDataKind>,
}

impl<'workflow, 'request, 'source> DataUploadPreparation<'workflow, 'request, 'source> {
    pub(crate) fn new(
        workflow: &'workflow ExecutionWorkflowRecord,
        sequence: u64,
        allocation: &IcSnapshotUploadReply<'request, 'source>,
        chunk_bytes: u64,
    ) -> Result<Self, IcSnapshotDataUploadPlanningError> {
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
        let kinds = planned_extents(metadata.source(), chunk_bytes, stage.budget().mutations())?;
        Ok(Self {
            workflow,
            stage,
            metadata,
            destination: snapshot_id.clone(),
            allocation_evidence: allocation.digest(),
            kinds,
        })
    }

    pub(crate) const fn metadata(&self) -> &'request IcSnapshotUploadRequest<'source> {
        self.metadata
    }
    pub(crate) fn destination(&self) -> &[u8] {
        &self.destination
    }
    pub(crate) fn kinds(&self) -> &[SnapshotDataKind] {
        &self.kinds
    }

    pub(crate) fn finish(
        self,
        bindings: &[ArtifactChecksumRecord],
    ) -> Result<
        IcSnapshotDataUploadPlan<'workflow, 'request, 'source>,
        IcSnapshotDataUploadPlanningError,
    > {
        if bindings.len() != self.kinds.len() {
            return Err(IcSnapshotDataUploadPlanningError::OriginalMismatch);
        }
        let plan = data_stage_plan::build(
            self.workflow.allocation(),
            self.stage,
            bindings.iter().cloned(),
        )?;
        Ok(IcSnapshotDataUploadPlan {
            binding: LearnedDataStage {
                workflow: self.workflow,
                sequence: self.stage.operation_sequence(),
                evidence: self.allocation_evidence,
            },
            metadata: self.metadata,
            destination: self.destination,
            writes: match plan {
                Some(plan) => DataWrites::Writes {
                    kinds: self.kinds,
                    plan,
                },
                None => DataWrites::Empty,
            },
        })
    }
}

impl<'request, 'source> IcSnapshotDataUploadPlan<'_, 'request, 'source> {
    pub(crate) fn workflow_digest(&self) -> ArtifactChecksumRecord {
        self.binding.workflow.digest()
    }
    pub(crate) fn validate_completion(
        &self,
        binding: &ExecutionStageBindingRecord,
        plan: &OperationPlanRecord,
        learned: &ArtifactChecksumRecord,
    ) -> Result<UploadCompletion, IcSnapshotDataUploadPlanningError> {
        if let Some(expected) = self.plan() {
            self.validate_binding(binding)?;
            if plan != expected {
                return Err(IcSnapshotDataUploadPlanningError::OriginalMismatch);
            }
            Ok(UploadCompletion::Data)
        } else {
            binding.validate(self.binding.workflow, plan)?;
            let [operation] = plan.operations() else {
                return Err(IcSnapshotDataUploadPlanningError::OriginalMismatch);
            };
            if operation.target() != self.metadata.target()
                || operation.request() != self.metadata.binding_digest().hash()
                || learned != &self.binding.evidence
            {
                return Err(IcSnapshotDataUploadPlanningError::OriginalMismatch);
            }
            Ok(UploadCompletion::Allocation {
                sequence: operation.operation_sequence(),
                evidence: self.binding.evidence.clone(),
            })
        }
    }
    /// Read exact ordered source extents. Operation sequences are zero-based indices.
    #[must_use]
    pub fn kinds(&self) -> &[SnapshotDataKind] {
        match &self.writes {
            DataWrites::Empty => &[],
            DataWrites::Writes { kinds, .. } => kinds,
        }
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
        match &self.writes {
            DataWrites::Empty => None,
            DataWrites::Writes { plan, .. } => Some(plan),
        }
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
            .plan()
            .ok_or(IcSnapshotDataUploadPlanningError::NoDataWrites)?;
        self.binding
            .bind(
                plan,
                allocation_binding,
                allocation_plan,
                self.metadata.binding_digest().hash(),
                predecessors,
            )
            .map_err(Into::into)
    }
    pub(crate) fn validate_binding(
        &self,
        binding: &ExecutionStageBindingRecord,
    ) -> Result<(), IcSnapshotDataUploadPlanningError> {
        let plan = self
            .plan()
            .ok_or(IcSnapshotDataUploadPlanningError::NoDataWrites)?;
        Ok(self.binding.validate(binding, plan)?)
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

impl From<DataStagePlanError> for IcSnapshotDataUploadPlanningError {
    fn from(error: DataStagePlanError) -> Self {
        match error {
            DataStagePlanError::CountOverflow => Self::CountOverflow,
            DataStagePlanError::Graph(error) => Self::Graph(error),
            DataStagePlanError::Plan(error) => Self::Plan(error),
            DataStagePlanError::Budget(error) => Self::Budget(error),
        }
    }
}

impl From<LearnedDataStageError> for IcSnapshotDataUploadPlanningError {
    fn from(error: LearnedDataStageError) -> Self {
        match error {
            LearnedDataStageError::Workflow(error) => Self::Workflow(error),
            LearnedDataStageError::Mismatch => Self::OriginalMismatch,
        }
    }
}

#[cfg(test)]
mod tests;
