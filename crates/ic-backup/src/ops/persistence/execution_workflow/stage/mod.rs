//! Guarded stage preparation, reopening and checkpoint completion.

use super::{
    ArtifactChecksumRecord, AttemptJournalGuard, BINDING_FILE, BackupLayoutGuard,
    ExecutionProgressView, ExecutionStageBindingRecord, ExecutionStageCheckpointError,
    ExecutionStagePredecessorRecord, ExecutionStagePreparationError, ExecutionStageResumeError,
    ExecutionWorkflowError, ExecutionWorkflowPersistenceError, MAX_EXECUTION_STAGE_BYTES,
    OperationPlanRecord, PersistenceError, acquire_stage, read_admitted_binding,
    read_execution_progress, read_execution_workflow, reconcile_execution_settlement,
    replay_execution_settlement, stage_path, validate_predecessors,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StagePreparationBarrier {
    Plan,
    Binding,
    Journal(u64),
}

/// One original stage's fixed layout, retained under both workflow and child exclusion.
///
/// Existing attempt journals remain the only spending owner. Preparation allocates
/// immutable stage records before creating the complete original journal set. Reopening never creates,
/// resets or reconstructs journals. Integrations
/// retain exact learned inputs, authenticate their relation to settled originals,
/// and qualify fresh authority, command custody and proof of no previous dispatch.
#[derive(Debug)]
pub struct ExecutionStageGuard<'a> {
    workflow_layout: &'a BackupLayoutGuard,
    pub(super) stage_layout: BackupLayoutGuard,
    binding: ExecutionStageBindingRecord,
    plan: OperationPlanRecord,
}
impl<'a> ExecutionStageGuard<'a> {
    /// Complete an absent local checkpoint or replay its exact occupied record.
    ///
    /// This is local recovery after uncertain checkpoint publication: all original
    /// journals and ancestor histories must still admit, and an occupied changed
    /// checkpoint is never replaced. The supplied learned evidence remains opaque
    /// integration-owned input, not an authenticated response or effect receipt.
    /// No provider, reservation, refund, terminal or reference release is involved.
    /// # Errors
    /// Rejects incomplete originals, unsafe or conflicting checkpoints and IO failures.
    pub fn reconcile_checkpoint(
        &self,
        learned_evidence: ArtifactChecksumRecord,
    ) -> Result<ExecutionStagePredecessorRecord, ExecutionStageCheckpointError> {
        self.reconcile_checkpoint_with(learned_evidence, || {})
    }
    pub(super) fn reconcile_checkpoint_with(
        &self,
        learned_evidence: ArtifactChecksumRecord,
        after_reconciliation: impl FnOnce(),
    ) -> Result<ExecutionStagePredecessorRecord, ExecutionStageCheckpointError> {
        let settlement = reconcile_execution_settlement(self.layout()?, &self.plan.digest())?;
        after_reconciliation();
        self.layout()?;
        Ok(ExecutionStagePredecessorRecord::new(
            self.binding.stage_sequence(),
            self.binding.digest(),
            settlement.digest(),
            learned_evidence,
        ))
    }
    /// Reconstruct an exact predecessor after a lost checkpoint response.
    ///
    /// Derives the settlement identity from all retained original journals and
    /// reads the already published checkpoint. It never creates a checkpoint,
    /// journal or receipt. The supplied learned evidence remains integration-owned
    /// and must be independently qualified before a successor is dispatched.
    /// # Errors
    /// Rejects missing, held, unsettled or changed originals and checkpoint drift.
    pub fn replay_checkpoint(
        &self,
        learned_evidence: ArtifactChecksumRecord,
    ) -> Result<ExecutionStagePredecessorRecord, ExecutionStageCheckpointError> {
        let settlement = replay_execution_settlement(self.layout()?, &self.plan.digest())?;
        self.layout()?;
        Ok(ExecutionStagePredecessorRecord::new(
            self.binding.stage_sequence(),
            self.binding.digest(),
            settlement.digest(),
            learned_evidence,
        ))
    }
    /// Resume an exact retained stage with its complete original journal progress.
    ///
    /// Reuses record-only `open` and canonical complete-journal admission, then
    /// rechecks the original stage and ancestor histories before returning. Hold
    /// no attempt guards. Reads only retained local records, without creating
    /// journals, reading artifact trees or calling providers. Pending and exhausted
    /// spending is retained.
    /// The returned progress is a sequential local projection, not atomic custody,
    /// fresh permission, dispatch, authenticated receipt or terminal/release proof.
    /// # Errors
    /// Rejects missing/changed/unsafe originals, journal contention and invalid
    /// retained accounting/causality without repair or replenishing allowance.
    pub fn resume(
        workflow_layout: &'a BackupLayoutGuard,
        expected_workflow: &ArtifactChecksumRecord,
        sequence: u64,
        expected_binding: &ArtifactChecksumRecord,
    ) -> Result<(Self, ExecutionProgressView), ExecutionStageResumeError> {
        let stage = Self::open(
            workflow_layout,
            expected_workflow,
            sequence,
            expected_binding,
        )?;
        let progress = read_execution_progress(&stage.stage_layout, &stage.plan.digest())?;
        stage.layout()?;
        Ok((stage, progress))
    }
    /// Durably prepare a new stage and its complete original attempt-journal set.
    ///
    /// Reuses create-only stage admission and the existing journal owner, taking
    /// one journal lock at a time. Every original authority is derived before
    /// stage allocation. No reservation, receipt, backend call or fresh dispatch
    /// authority follows. Hold no other attempt guards during preparation.
    ///
    /// Partial preparation remains occupied and is never repaired by retrying.
    /// Reopen only retained originals and require complete execution admission;
    /// missing journals never mean unused allowance. A lost successful response
    /// can reopen the complete exact stage without creating another journal.
    /// # Errors
    /// Rejects invalid originals, occupied stages, journal contention and failed
    /// durable publication, preserving all partial records and original limits.
    pub fn prepare(
        workflow_layout: &'a BackupLayoutGuard,
        binding: ExecutionStageBindingRecord,
        plan: OperationPlanRecord,
    ) -> Result<Self, ExecutionStagePreparationError> {
        Self::prepare_with(workflow_layout, binding, plan, |_| {})
    }
    pub(super) fn prepare_with(
        workflow_layout: &'a BackupLayoutGuard,
        binding: ExecutionStageBindingRecord,
        plan: OperationPlanRecord,
        mut barrier: impl FnMut(StagePreparationBarrier),
    ) -> Result<Self, ExecutionStagePreparationError> {
        let authorities = plan.attempt_authorities()?;
        let stage = Self::create_with(workflow_layout, binding, plan, &mut barrier)?;
        let layout = stage.layout()?;
        for authority in authorities {
            let sequence = authority.binding().operation_sequence();
            drop(AttemptJournalGuard::create(layout, authority)?);
            barrier(StagePreparationBarrier::Journal(sequence));
        }
        stage.layout()?;
        Ok(stage)
    }
    // Allocate immutable stage records internally before preparing original journals.
    pub(super) fn create_with(
        workflow_layout: &'a BackupLayoutGuard,
        binding: ExecutionStageBindingRecord,
        plan: OperationPlanRecord,
        mut barrier: impl FnMut(StagePreparationBarrier),
    ) -> Result<Self, ExecutionWorkflowPersistenceError> {
        let workflow = read_execution_workflow(workflow_layout, binding.workflow())?;
        binding.validate(&workflow, &plan)?;
        let binding_bytes = super::super::json::encode_json(&binding, MAX_EXECUTION_STAGE_BYTES)?;
        // Encode both documents before allocating any stage paths.
        let plan_bytes = super::super::json::encode_json(
            &plan,
            crate::model::operation_plan::MAX_OPERATION_PLAN_BYTES,
        )?;
        validate_predecessors(workflow_layout, &workflow, &binding)?;
        let path = stage_path(workflow_layout, binding.stage_sequence());
        ic_host_fs::durable::create_private_directory(&path)
            .map_err(|error| PersistenceError::Io(crate::ops::directory_creation_io(error)))?;
        let stage_layout = acquire_stage(workflow_layout, binding.stage_sequence())?;
        super::super::operation_plan::create_encoded(&stage_layout, &plan_bytes)?;
        barrier(StagePreparationBarrier::Plan);
        super::super::json::create_encoded(
            &stage_layout.root().join(BINDING_FILE),
            &binding_bytes,
        )?;
        barrier(StagePreparationBarrier::Binding);
        workflow_layout.check_root()?;
        Ok(Self {
            workflow_layout,
            stage_layout,
            binding,
            plan,
        })
    }
    /// Reopen an exact fixed stage without allocation, repair or journal creation.
    ///
    /// Lost successful creation responses reconcile here. A partially prepared
    /// directory without both immutable records rejects and remains retained.
    /// Missing pending journals cannot be treated as unused allowance by the
    /// existing execution-progress or settlement owners.
    /// # Errors
    /// Rejects missing/changed original records, prerequisite history or ownership failures.
    pub fn open(
        workflow_layout: &'a BackupLayoutGuard,
        expected_workflow: &ArtifactChecksumRecord,
        sequence: u64,
        expected_binding: &ArtifactChecksumRecord,
    ) -> Result<Self, ExecutionWorkflowPersistenceError> {
        let workflow = read_execution_workflow(workflow_layout, expected_workflow)?;
        workflow
            .stage(sequence)
            .map_err(|_| ExecutionWorkflowError::UnknownStage(sequence))?;
        let stage_layout = acquire_stage(workflow_layout, sequence)?;
        let (binding, plan) = read_admitted_binding(
            workflow_layout,
            &stage_layout,
            &workflow,
            sequence,
            expected_binding,
        )?;
        workflow_layout.check_root()?;
        Ok(Self {
            workflow_layout,
            stage_layout,
            binding,
            plan,
        })
    }
    /// Read the exact bound original child plan; no fresh permission is implied.
    #[must_use]
    pub const fn plan(&self) -> &OperationPlanRecord {
        &self.plan
    }
    /// Read the immutable original stage and predecessor commitments.
    #[must_use]
    pub const fn binding(&self) -> &ExecutionStageBindingRecord {
        &self.binding
    }

    /// Borrow the held original workflow layout for read-only predecessor admission.
    #[must_use]
    pub const fn workflow_layout(&self) -> &BackupLayoutGuard {
        self.workflow_layout
    }
    /// Re-admit both original records and expose the existing journal/layout owner.
    ///
    /// Callers must use complete original journal admission for resume. Never
    /// create a missing journal on resume or substitute a freshly derived plan.
    /// # Errors
    /// Rejects changed roots, retained records or any ancestor settlement history.
    pub fn layout(&self) -> Result<&BackupLayoutGuard, ExecutionWorkflowPersistenceError> {
        let workflow = read_execution_workflow(self.workflow_layout, self.binding.workflow())?;
        read_admitted_binding(
            self.workflow_layout,
            &self.stage_layout,
            &workflow,
            self.binding.stage_sequence(),
            &self.binding.digest(),
        )?;
        Ok(&self.stage_layout)
    }
}
