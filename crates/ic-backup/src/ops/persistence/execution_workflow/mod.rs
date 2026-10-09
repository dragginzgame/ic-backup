//! Fixed immutable stage admission using the existing layout, plan and settlement owners.

use super::{
    BackupLayoutGuard, ExecutionSettlementPersistenceError, JournalLock, JournalLockError,
    OperationPlanPersistenceError, PersistenceError, create_json_durable, create_operation_plan,
    read_execution_settlement, read_json, read_operation_plan,
};
use crate::model::{
    artifacts::ArtifactChecksumRecord,
    execution_workflow::{
        ExecutionStageBindingRecord, ExecutionWorkflowError, ExecutionWorkflowRecord,
        MAX_EXECUTION_STAGE_BYTES, MAX_EXECUTION_WORKFLOW_BYTES,
    },
    operation_plan::OperationPlanRecord,
};
use std::{fs, path::PathBuf};
use thiserror::Error;

const WORKFLOW_FILE: &str = "execution-workflow.json";
const BINDING_FILE: &str = "stage-binding.json";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StagePreparationBarrier {
    AfterPlanPublication,
    AfterBindingPublication,
}

/// Durably create the full original workflow allocation without replacement.
/// # Errors
/// Rejects occupied/unsafe paths, bounds, ownership and publication failures.
pub fn create_execution_workflow(
    layout: &BackupLayoutGuard,
    record: &ExecutionWorkflowRecord,
) -> Result<(), ExecutionWorkflowPersistenceError> {
    layout.check_root()?;
    super::json::check_json_size(record, MAX_EXECUTION_WORKFLOW_BYTES)?;
    let path = layout.root().join(WORKFLOW_FILE);
    let _lock = JournalLock::acquire(&path)?;
    create_json_durable(&path, record)?;
    Ok(())
}
/// Read exact original workflow allocation; missing evidence never supplies a new budget.
/// # Errors
/// Rejects missing/changed/unsafe/oversized records and ownership failures.
pub fn read_execution_workflow(
    layout: &BackupLayoutGuard,
    expected: &ArtifactChecksumRecord,
) -> Result<ExecutionWorkflowRecord, ExecutionWorkflowPersistenceError> {
    layout.check_root()?;
    let path = layout.root().join(WORKFLOW_FILE);
    let _lock = JournalLock::acquire(&path)?;
    let record: ExecutionWorkflowRecord = read_json(&path, MAX_EXECUTION_WORKFLOW_BYTES)?;
    super::json::check_json_size(&record, MAX_EXECUTION_WORKFLOW_BYTES)?;
    if &record.digest() != expected {
        return Err(ExecutionWorkflowPersistenceError::DigestMismatch);
    }
    Ok(record)
}

/// One original stage's fixed layout, retained under both workflow and child exclusion.
///
/// Existing attempt journals remain the only spending owner. Creation supplies no
/// journals and reopening never creates, resets or reconstructs them. Integrations
/// retain exact learned inputs, authenticate their relation to settled originals,
/// and qualify fresh authority, command custody and proof of no previous dispatch.
#[derive(Debug)]
pub struct ExecutionStageGuard<'a> {
    workflow_layout: &'a BackupLayoutGuard,
    stage_layout: BackupLayoutGuard,
    binding: ExecutionStageBindingRecord,
    plan: OperationPlanRecord,
}
impl<'a> ExecutionStageGuard<'a> {
    /// Create a fixed private stage directory, exact child plan and immutable binding.
    ///
    /// Every original predecessor must have its exact retained binding, plan and
    /// complete chronological Applied settlement. The directory is create-only:
    /// occupied or interrupted preparation is retained and cannot be recreated
    /// with another plan. No journal or backend effect occurs before return.
    /// # Errors
    /// Rejects changed allocation, unavailable prerequisites, occupied stages and IO failures.
    pub fn create(
        workflow_layout: &'a BackupLayoutGuard,
        binding: ExecutionStageBindingRecord,
        plan: OperationPlanRecord,
    ) -> Result<Self, ExecutionWorkflowPersistenceError> {
        Self::create_with(workflow_layout, binding, plan, |_| {})
    }
    fn create_with(
        workflow_layout: &'a BackupLayoutGuard,
        binding: ExecutionStageBindingRecord,
        plan: OperationPlanRecord,
        mut barrier: impl FnMut(StagePreparationBarrier),
    ) -> Result<Self, ExecutionWorkflowPersistenceError> {
        let workflow = read_execution_workflow(workflow_layout, binding.workflow())?;
        binding.validate(&workflow, &plan)?;
        super::json::check_json_size(&binding, MAX_EXECUTION_STAGE_BYTES)?;
        // Validate both documents before allocating any stage paths.
        super::json::check_json_size(
            &plan,
            crate::model::operation_plan::MAX_OPERATION_PLAN_BYTES,
        )?;
        validate_predecessors(workflow_layout, &workflow, &binding)?;
        let path = stage_path(workflow_layout, binding.stage_sequence());
        create_private_directory(&path)?;
        fs::File::open(workflow_layout.root())
            .and_then(|directory| directory.sync_all())
            .map_err(PersistenceError::from)?;
        let stage_layout = acquire_stage(workflow_layout, binding.stage_sequence())?;
        create_operation_plan(&stage_layout, &plan)?;
        barrier(StagePreparationBarrier::AfterPlanPublication);
        create_json_durable(&stage_layout.root().join(BINDING_FILE), &binding)?;
        barrier(StagePreparationBarrier::AfterBindingPublication);
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
        let (binding, plan) = read_binding(&stage_layout, &workflow, sequence, expected_binding)?;
        validate_predecessors(workflow_layout, &workflow, &binding)?;
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
    /// Re-admit both original records and expose the existing journal/layout owner.
    ///
    /// Callers must use complete original journal admission for resume. Never
    /// create a missing journal on resume or substitute a freshly derived plan.
    /// # Errors
    /// Rejects changed roots, retained records or predecessor settlement histories.
    pub fn layout(&self) -> Result<&BackupLayoutGuard, ExecutionWorkflowPersistenceError> {
        let workflow = read_execution_workflow(self.workflow_layout, self.binding.workflow())?;
        read_binding(
            &self.stage_layout,
            &workflow,
            self.binding.stage_sequence(),
            &self.binding.digest(),
        )?;
        validate_predecessors(self.workflow_layout, &workflow, &self.binding)?;
        Ok(&self.stage_layout)
    }
}
fn read_binding(
    layout: &BackupLayoutGuard,
    workflow: &ExecutionWorkflowRecord,
    sequence: u64,
    expected: &ArtifactChecksumRecord,
) -> Result<(ExecutionStageBindingRecord, OperationPlanRecord), ExecutionWorkflowPersistenceError> {
    layout.check_root()?;
    let binding: ExecutionStageBindingRecord =
        read_json(&layout.root().join(BINDING_FILE), MAX_EXECUTION_STAGE_BYTES)?;
    super::json::check_json_size(&binding, MAX_EXECUTION_STAGE_BYTES)?;
    if binding.stage_sequence() != sequence || &binding.digest() != expected {
        return Err(ExecutionWorkflowPersistenceError::DigestMismatch);
    }
    let plan = read_operation_plan(layout, binding.plan())?;
    binding.validate(workflow, &plan)?;
    Ok((binding, plan))
}
fn validate_predecessors(
    layout: &BackupLayoutGuard,
    workflow: &ExecutionWorkflowRecord,
    binding: &ExecutionStageBindingRecord,
) -> Result<(), ExecutionWorkflowPersistenceError> {
    // Sequential existing layout/journal admission avoids holding multiple
    // predecessor locks or inventing a second outcome/accounting projection.
    for row in binding.predecessors() {
        let predecessor = acquire_stage(layout, row.stage_sequence())?;
        let (original, _) =
            read_binding(&predecessor, workflow, row.stage_sequence(), row.binding())?;
        read_execution_settlement(&predecessor, original.plan(), row.settlement())?;
    }
    Ok(())
}
fn stage_path(layout: &BackupLayoutGuard, sequence: u64) -> PathBuf {
    layout.root().join(format!("execution-stage-{sequence}"))
}
fn acquire_stage(
    layout: &BackupLayoutGuard,
    sequence: u64,
) -> Result<BackupLayoutGuard, ExecutionWorkflowPersistenceError> {
    layout.check_root()?;
    let path = stage_path(layout, sequence);
    let metadata = fs::symlink_metadata(&path).map_err(PersistenceError::from)?;
    if !metadata.is_dir() {
        return Err(ExecutionWorkflowPersistenceError::UnsafeStage);
    }
    let stage = BackupLayoutGuard::acquire(&path)?;
    // Unlike an explicitly operator-selected root, a derived stage symlink is
    // never adopted. Noncooperating namespace stability remains caller-owned.
    if stage.root() != path {
        return Err(ExecutionWorkflowPersistenceError::UnsafeStage);
    }
    layout.check_root()?;
    Ok(stage)
}
fn create_private_directory(
    path: &std::path::Path,
) -> Result<(), ExecutionWorkflowPersistenceError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        fs::DirBuilder::new()
            .mode(0o700)
            .create(path)
            .map_err(PersistenceError::from)?;
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Err(PersistenceError::Io(std::io::Error::from(std::io::ErrorKind::Unsupported)).into())
    }
}

/// Typed immutable workflow/stage admission failure; no recovery evidence is removed.
#[derive(Debug, Error)]
pub enum ExecutionWorkflowPersistenceError {
    /// Workflow/stage identity differs from the retained expected digest.
    #[error("execution workflow/stage digest mismatch")]
    DigestMismatch,
    /// A derived stage directory is a symlink or another unsafe entry.
    #[error("unsafe execution stage directory")]
    UnsafeStage,
    /// Original model allocation or learned binding was rejected.
    #[error(transparent)]
    Model(#[from] ExecutionWorkflowError),
    /// Existing immutable operation-plan admission failed.
    #[error(transparent)]
    Plan(#[from] OperationPlanPersistenceError),
    /// Existing complete chronological Applied settlement admission failed.
    #[error(transparent)]
    Settlement(#[from] ExecutionSettlementPersistenceError),
    /// Cooperating layout or journal exclusion failed.
    #[error(transparent)]
    Lock(#[from] JournalLockError),
    /// Existing bounded JSON/durable filesystem owner failed.
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
}

#[cfg(all(test, unix))]
mod tests;
