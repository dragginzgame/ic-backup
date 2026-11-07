//! Bounded metadata-derived exact download plans; no calls, spending or transfer proof.

use crate::model::{
    data_stage_plan::{self, LearnedDataStage, SnapshotDataPlanningError},
    execution_workflow::{
        ExecutionStageBindingRecord, ExecutionStagePredecessorRecord, ExecutionWorkflowRecord,
    },
    ic_snapshot_data::{IcSnapshotDataRequest, planned_extents},
    ic_snapshot_metadata::IcSnapshotMetadataReply,
    operation_plan::OperationPlanRecord,
};

/// Exact finite data requests and their original allocation-bound ordinary child plan.
///
/// Requests cover module, heap and stable regions in that order, then every known
/// chunk in original metadata order. Empty regions need no requests; even empty
/// known chunks require a hash-checked reply. Opaque operation IDs are zero-based
/// ordinals with explicit sequential dependencies for the contiguous writer.
/// Every request has one mutation-lane replicated update and zero recovery calls.
/// Lost replies therefore stop with pending spending; this is not a retry schedule.
/// The child aggregate retains original stage ceilings and leaves spare allowance
/// unassigned. An entirely read-free snapshot has no child plan or binding.
pub struct IcSnapshotDownloadPlan<'workflow, 'metadata> {
    binding: LearnedDataStage<'workflow>,
    metadata: &'metadata IcSnapshotMetadataReply<'metadata>,
    reads: Option<DataReads<'metadata>>,
}

// Read-free snapshots have no requests or placeholder plan; nonempty reads carry both.
struct DataReads<'metadata> {
    requests: Vec<IcSnapshotDataRequest<'metadata>>,
    plan: OperationPlanRecord,
}
impl<'workflow, 'metadata> IcSnapshotDownloadPlan<'workflow, 'metadata> {
    /// Derive exact data payloads within the original target and mutation allocation.
    ///
    /// Count checked region ceilings and known chunks before allocating/encoding
    /// requests. No iteration or allocation grows with an unadmitted remote nat64.
    /// Fresh access, authentic metadata/extent custody and execution remain separate.
    /// # Errors
    /// Rejects unknown/wrong stages, invalid chunks, arithmetic overflow, insufficient
    /// original allowance and existing bounded request/plan admission failures.
    pub fn new(
        workflow: &'workflow ExecutionWorkflowRecord,
        sequence: u64,
        metadata: &'metadata IcSnapshotMetadataReply<'metadata>,
        chunk_bytes: u64,
    ) -> Result<Self, SnapshotDataPlanningError> {
        let stage = workflow.stage(sequence)?;
        if stage.target() != metadata.request().target() {
            return Err(SnapshotDataPlanningError::OriginalMismatch);
        }
        let kinds = planned_extents(metadata, chunk_bytes, stage.budget().mutations())?;
        let requests = kinds
            .into_iter()
            .map(|kind| IcSnapshotDataRequest::new(metadata, kind))
            .collect::<Result<Vec<_>, _>>()?;
        let plan = data_stage_plan::build(
            workflow.allocation(),
            stage,
            requests.iter().map(IcSnapshotDataRequest::digest),
        )?;
        Ok(Self {
            binding: LearnedDataStage {
                workflow,
                sequence,
                evidence: metadata.digest(),
            },
            metadata,
            reads: plan.map(|plan| DataReads { requests, plan }),
        })
    }
    /// Read exact ordered requests; operation IDs are their zero-based indices.
    #[must_use]
    pub fn requests(&self) -> &[IcSnapshotDataRequest<'metadata>] {
        self.reads
            .as_ref()
            .map_or(&[], |reads| reads.requests.as_slice())
    }
    // Every immutable request was constructed from this same original metadata.
    pub(crate) fn matches_metadata(&self, metadata: &IcSnapshotMetadataReply<'_>) -> bool {
        self.metadata.digest() == metadata.digest()
    }
    /// Read the original-allocation child plan. None means no data calls are needed,
    /// not that capture, durable publication or complete transfer is qualified.
    #[must_use]
    pub const fn plan(&self) -> Option<&OperationPlanRecord> {
        match &self.reads {
            Some(reads) => Some(&reads.plan),
            None => None,
        }
    }

    /// Re-admit an exact retained data-stage binding and its original metadata evidence.
    /// This structural association grants no fresh permission or transfer attestation.
    pub(crate) fn validate_binding(
        &self,
        binding: &ExecutionStageBindingRecord,
    ) -> Result<(), SnapshotDataPlanningError> {
        let plan = self
            .plan()
            .ok_or(SnapshotDataPlanningError::NoDataOperations)?;
        self.binding.validate(binding, plan)
    }
    /// Bind the exact original metadata stage and reply evidence before stage creation.
    ///
    /// The source stage must contain exactly one metadata request under this original
    /// workflow. Its direct predecessor row must retain its binding and this exact
    /// metadata request/raw-reply evidence digest. Existing stage persistence still
    /// admits the complete Applied source journals and settlement. These checks do
    /// not authenticate metadata or prove that an opaque receipt describes its reply.
    /// # Errors
    /// Rejects a read-free plan, wrong metadata stage/plan/input or original dependencies.
    pub fn bind(
        &self,
        metadata_binding: &ExecutionStageBindingRecord,
        metadata_plan: &OperationPlanRecord,
        predecessors: Vec<ExecutionStagePredecessorRecord>,
    ) -> Result<ExecutionStageBindingRecord, SnapshotDataPlanningError> {
        let plan = self
            .plan()
            .ok_or(SnapshotDataPlanningError::NoDataOperations)?;
        self.binding.bind(
            plan,
            metadata_binding,
            metadata_plan,
            self.metadata.request().digest().hash(),
            predecessors,
        )
    }
}

#[cfg(test)]
mod tests;
