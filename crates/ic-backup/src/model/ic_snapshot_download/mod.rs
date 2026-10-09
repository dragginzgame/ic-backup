//! Bounded metadata-derived exact download plans; no calls, spending or transfer proof.

use crate::model::{
    attempt_journal::{AttemptBudgetRecord, AttemptJournalRecordError},
    effect_graph::{EffectGraphError, EffectGraphRecord, EffectNodeRecord, EffectNodeRequest},
    execution_workflow::{
        ExecutionStageBindingRecord, ExecutionStagePredecessorRecord, ExecutionWorkflowError,
        ExecutionWorkflowRecord,
    },
    ic_snapshot_data::{
        IcSnapshotDataError, IcSnapshotDataRequest, MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES,
    },
    ic_snapshot_metadata::IcSnapshotMetadataReply,
    operation_plan::{
        OperationPlanError, OperationPlanRecord, OperationPlanRequest, PlanBudgetRecord,
        PlannedOperationRecord, PlannedOperationRequest,
    },
};
use ic_management_canister_types::SnapshotDataKind;
use thiserror::Error;

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
    workflow: &'workflow ExecutionWorkflowRecord,
    sequence: u64,
    metadata: &'metadata IcSnapshotMetadataReply<'metadata>,
    requests: Vec<IcSnapshotDataRequest<'metadata>>,
    plan: Option<OperationPlanRecord>,
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
    ) -> Result<Self, IcSnapshotDownloadPlanningError> {
        let stage = workflow.stage(sequence)?;
        if stage.target() != metadata.request().target() {
            return Err(IcSnapshotDownloadPlanningError::MetadataMismatch);
        }
        if chunk_bytes == 0 || chunk_bytes > MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES as u64 {
            return Err(IcSnapshotDownloadPlanningError::InvalidChunkSize);
        }
        let values = metadata.metadata();
        let sizes = [
            values.wasm_module_size,
            values.wasm_memory_size,
            values.stable_memory_size,
        ];
        let mut count = u64::try_from(values.wasm_chunk_store.len())
            .map_err(|_| IcSnapshotDownloadPlanningError::CountOverflow)?;
        for size in sizes {
            let region = size / chunk_bytes + u64::from(size % chunk_bytes != 0);
            count = count
                .checked_add(region)
                .ok_or(IcSnapshotDownloadPlanningError::CountOverflow)?;
        }
        if count > u64::from(stage.budget().mutations()) {
            return Err(IcSnapshotDownloadPlanningError::InsufficientAllowance {
                required: count,
                original: stage.budget().mutations(),
            });
        }
        let capacity =
            usize::try_from(count).map_err(|_| IcSnapshotDownloadPlanningError::CountOverflow)?;
        let mut requests = Vec::with_capacity(capacity);
        for (region, total) in sizes.into_iter().enumerate() {
            let mut offset = 0;
            while offset < total {
                let size = (total - offset).min(chunk_bytes);
                let kind = match region {
                    0 => SnapshotDataKind::WasmModule { offset, size },
                    1 => SnapshotDataKind::WasmMemory { offset, size },
                    _ => SnapshotDataKind::StableMemory { offset, size },
                };
                requests.push(IcSnapshotDataRequest::new(metadata, kind)?);
                offset += size; // The admitted size never exceeds total - offset.
            }
        }
        for chunk in &values.wasm_chunk_store {
            requests.push(IcSnapshotDataRequest::new(
                metadata,
                SnapshotDataKind::WasmChunk {
                    hash: chunk.hash.clone(),
                },
            )?);
        }
        let plan = if requests.is_empty() {
            None
        } else {
            let mut nodes = Vec::with_capacity(capacity);
            let mut operations = Vec::with_capacity(capacity);
            for (index, request) in requests.iter().enumerate() {
                let sequence = u64::try_from(index)
                    .map_err(|_| IcSnapshotDownloadPlanningError::CountOverflow)?;
                nodes.push(EffectNodeRecord::new(EffectNodeRequest {
                    operation_sequence: sequence,
                    depends_on: sequence.checked_sub(1).into_iter().collect(),
                })?);
                operations.push(PlannedOperationRecord::new(PlannedOperationRequest {
                    operation_sequence: sequence,
                    target: stage.target().into(),
                    request: request.digest().hash().into(),
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
            metadata,
            requests,
            plan,
        })
    }
    /// Read exact ordered requests; operation IDs are their zero-based indices.
    #[must_use]
    pub fn requests(&self) -> &[IcSnapshotDataRequest<'metadata>] {
        &self.requests
    }
    /// Read the original-allocation child plan. None means no data calls are needed,
    /// not that capture, durable publication or complete transfer is qualified.
    #[must_use]
    pub const fn plan(&self) -> Option<&OperationPlanRecord> {
        self.plan.as_ref()
    }

    /// Re-admit an exact retained data-stage binding and its original metadata evidence.
    /// This structural association grants no fresh permission or transfer attestation.
    pub(crate) fn validate_binding(
        &self,
        binding: &ExecutionStageBindingRecord,
    ) -> Result<(), IcSnapshotDownloadPlanningError> {
        let plan = self
            .plan
            .as_ref()
            .ok_or(IcSnapshotDownloadPlanningError::NoDataReads)?;
        binding.validate(self.workflow, plan)?;
        if binding.stage_sequence() != self.sequence
            || !binding
                .predecessors()
                .iter()
                .any(|row| row.learned_evidence() == &self.metadata.digest())
        {
            return Err(IcSnapshotDownloadPlanningError::MetadataMismatch);
        }
        Ok(())
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
    ) -> Result<ExecutionStageBindingRecord, IcSnapshotDownloadPlanningError> {
        let plan = self
            .plan
            .as_ref()
            .ok_or(IcSnapshotDownloadPlanningError::NoDataReads)?;
        metadata_binding.validate(self.workflow, metadata_plan)?;
        if metadata_plan.operations().len() != 1
            || metadata_plan.operations()[0].request() != self.metadata.request().digest().hash()
            || !predecessors.iter().any(|row| {
                row.stage_sequence() == metadata_binding.stage_sequence()
                    && row.binding() == &metadata_binding.digest()
                    && row.learned_evidence() == &self.metadata.digest()
            })
        {
            return Err(IcSnapshotDownloadPlanningError::MetadataMismatch);
        }
        Ok(ExecutionStageBindingRecord::new(
            self.workflow,
            self.sequence,
            plan,
            predecessors,
        )?)
    }
}

/// Pure planning failure; no filesystem, provider, reservation or receipt changes.
#[derive(Debug, Error)]
pub enum IcSnapshotDownloadPlanningError {
    /// Chunk size is outside the existing 1..=1 MiB request boundary.
    #[error("invalid snapshot download chunk size")]
    InvalidChunkSize,
    /// Combined remote declared region counts cannot fit nat64.
    #[error("snapshot download request count overflow")]
    CountOverflow,
    /// Original stage cannot pay for every exact required data request.
    #[error("snapshot download needs {required} updates; original allocation is {original}")]
    InsufficientAllowance {
        /// Complete exact required update count.
        required: u64,
        /// Original stage mutation ceiling, excluding workflow headroom.
        original: u32,
    },
    /// Target, original metadata stage/request or exact reply evidence changed.
    #[error("snapshot download metadata differs from original stage")]
    MetadataMismatch,
    /// There is no data call to bind; no placeholder operation is created.
    #[error("snapshot download has no data reads to bind")]
    NoDataReads,
    /// Canonical original plan or allocation rejected.
    #[error(transparent)]
    Plan(#[from] OperationPlanError),
    /// Existing explicit graph owner rejected.
    #[error(transparent)]
    Graph(#[from] EffectGraphError),
    /// Existing immutable stage owner rejected.
    #[error(transparent)]
    Workflow(#[from] ExecutionWorkflowError),
    /// Existing exact metadata-bound data request owner rejected.
    #[error(transparent)]
    Data(#[from] IcSnapshotDataError),
    /// Existing per-operation budget owner rejected.
    #[error(transparent)]
    Budget(#[from] AttemptJournalRecordError),
}

#[cfg(test)]
pub(crate) mod tests;
