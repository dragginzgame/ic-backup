//! Fixed immutable stage admission using the existing layout, plan and settlement owners.

use super::{
    AttemptJournalError, AttemptJournalGuard, BackupLayoutGuard, ExecutionProgressPersistenceError,
    ExecutionSettlementCheckpointError, ExecutionSettlementPersistenceError, JournalLock,
    JournalLockError, OperationPlanPersistenceError, PersistenceError,
    checkpoint_execution_settlement, create_json_durable, create_operation_plan,
    read_execution_progress, read_execution_settlement, read_json, read_operation_plan,
};
use crate::model::{
    artifacts::ArtifactChecksumRecord,
    execution_workflow::{
        ExecutionStageBindingRecord, ExecutionStagePredecessorRecord, ExecutionWorkflowError,
        ExecutionWorkflowRecord, MAX_EXECUTION_STAGE_BYTES, MAX_EXECUTION_WORKFLOW_BYTES,
    },
    operation_plan::{OperationPlanError, OperationPlanRecord},
};
use crate::policy::execution_progress::ExecutionProgressView;
use std::{
    collections::{BTreeMap, btree_map::Entry},
    fs,
    path::PathBuf,
};
use thiserror::Error;

const WORKFLOW_FILE: &str = "execution-workflow.json";
const BINDING_FILE: &str = "stage-binding.json";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StagePreparationBarrier {
    Plan,
    Binding,
    Journal(u64),
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
/// Existing attempt journals remain the only spending owner. `create` supplies no
/// journals; `prepare` creates the complete original set. Reopening never creates,
/// resets or reconstructs journals. Integrations
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
    /// Publish the original all-Applied checkpoint and derive its exact predecessor commitment.
    ///
    /// Re-admits the workflow, stage and complete ancestor histories before and after
    /// delegating to the canonical original-journal checkpoint owner. Hold no attempt
    /// guards. Failed post-publication admission retains the checkpoint and all originals;
    /// an occupied checkpoint is never replaced. Identity-bound local replay remains
    /// owned by `read_execution_settlement`.
    ///
    /// The supplied learned evidence stays integration-owned: this authenticates no
    /// learned input, receipt or effect, and grants no dispatch or terminal/release proof.
    /// # Errors
    /// Rejects changed/missing/held originals, unsettled journals and publication failures.
    pub fn checkpoint(
        &self,
        learned_evidence: ArtifactChecksumRecord,
    ) -> Result<ExecutionStagePredecessorRecord, ExecutionStageCheckpointError> {
        self.checkpoint_with(learned_evidence, || {})
    }
    fn checkpoint_with(
        &self,
        learned_evidence: ArtifactChecksumRecord,
        after_publication: impl FnOnce(),
    ) -> Result<ExecutionStagePredecessorRecord, ExecutionStageCheckpointError> {
        let settlement = checkpoint_execution_settlement(self.layout()?, &self.plan.digest())?;
        after_publication();
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
    fn prepare_with(
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
    /// Create a fixed private stage directory, exact child plan and immutable binding.
    ///
    /// Every original predecessor and ancestor needs its exact retained binding, plan and
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
        barrier(StagePreparationBarrier::Plan);
        create_json_durable(&stage_layout.root().join(BINDING_FILE), &binding)?;
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
    /// Rejects changed roots, retained records or any ancestor settlement history.
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

/// Stage-bound checkpoint refusal; original spending and published evidence remain retained.
#[derive(Debug, Error)]
pub enum ExecutionStageCheckpointError {
    /// Original workflow/stage or complete ancestor history cannot be admitted.
    #[error(transparent)]
    Stage(#[from] ExecutionWorkflowPersistenceError),
    /// Complete original journal checkpoint derivation/publication failed.
    #[error(transparent)]
    Settlement(#[from] ExecutionSettlementCheckpointError),
}

/// Failed create-only preparation; retained records never grant repair or extra spending.
#[derive(Debug, Error)]
pub enum ExecutionStagePreparationError {
    /// Original stage records, predecessor settlement or layout were rejected.
    #[error(transparent)]
    Stage(#[from] ExecutionWorkflowPersistenceError),
    /// Complete original child authority derivation failed before allocation.
    #[error(transparent)]
    Authority(#[from] OperationPlanError),
    /// Existing attempt-journal locking or durable create-only publication failed.
    #[error(transparent)]
    Journal(#[from] AttemptJournalError),
}

/// Complete retained stage-resume refusal; original evidence and spending remain unchanged.
#[derive(Debug, Error)]
pub enum ExecutionStageResumeError {
    /// Original stage, ancestor history or layout cannot be admitted.
    #[error(transparent)]
    Stage(#[from] ExecutionWorkflowPersistenceError),
    /// Complete original child journals, accounting or causality cannot be admitted.
    #[error(transparent)]
    Progress(#[from] ExecutionProgressPersistenceError),
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
    // Traverse the original acyclic catalog iteratively, admitting each ancestor
    // once. Keep at most one row per catalog node, not one per dependency path.
    // Learned evidence may differ between edges; binding/settlement identity cannot.
    let mut expected: BTreeMap<u64, ExecutionStagePredecessorRecord> = binding
        .predecessors()
        .iter()
        .map(|row| (row.stage_sequence(), row.clone()))
        .collect();
    let mut pending: Vec<_> = expected.keys().copied().collect();
    while let Some(sequence) = pending.pop() {
        let row = &expected[&sequence];
        let original = {
            // Release this layout and every journal lock before admitting another
            // ancestor, including an ancestor shared by multiple branches.
            let predecessor = acquire_stage(layout, sequence)?;
            let (original, _) = read_binding(&predecessor, workflow, sequence, row.binding())?;
            read_execution_settlement(&predecessor, original.plan(), row.settlement())?;
            original
        };
        for ancestor in original.predecessors() {
            match expected.entry(ancestor.stage_sequence()) {
                Entry::Vacant(entry) => {
                    pending.push(ancestor.stage_sequence());
                    entry.insert(ancestor.clone());
                }
                Entry::Occupied(entry) => {
                    if entry.get().binding() != ancestor.binding()
                        || entry.get().settlement() != ancestor.settlement()
                    {
                        return Err(ExecutionWorkflowPersistenceError::DigestMismatch);
                    }
                }
            }
        }
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
