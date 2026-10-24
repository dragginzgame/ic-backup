//! Fixed immutable stage admission using the existing layout, plan and settlement owners.

use super::{
    AttemptJournalError, AttemptJournalGuard, BackupLayoutGuard, ExecutionProgressPersistenceError,
    ExecutionSettlementCheckpointError, ExecutionSettlementPersistenceError, JournalLock,
    JournalLockError, OperationPlanPersistenceError, PersistenceError, read_execution_progress,
    read_execution_settlement, read_json, read_operation_plan, reconcile_execution_settlement,
    replay_execution_settlement,
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
    collections::{BTreeMap, BTreeSet, btree_map::Entry},
    fs,
    path::PathBuf,
};
use thiserror::Error;

const WORKFLOW_FILE: &str = "execution-workflow.json";
const BINDING_FILE: &str = "stage-binding.json";

/// Exact local identities of every original stage's retained checkpoint.
///
/// This is a sequential local projection, not an immutable product completion
/// receipt. It authenticates no learned evidence, remote effect, application
/// safety or command/byte custody and grants no terminal or reference release.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionWorkflowSettlementView {
    workflow: ArtifactChecksumRecord,
    stages: Vec<ExecutionStageSettlementView>,
}
impl ExecutionWorkflowSettlementView {
    /// Read the exact original workflow declaration identity.
    #[must_use]
    pub const fn workflow(&self) -> &ArtifactChecksumRecord {
        &self.workflow
    }
    /// Read one row per original stage in canonical allocation order.
    #[must_use]
    pub fn stages(&self) -> &[ExecutionStageSettlementView] {
        &self.stages
    }
}

/// One original stage's bound plan and complete chronological local checkpoint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionStageSettlementView {
    stage_sequence: u64,
    binding: ArtifactChecksumRecord,
    settlement: ArtifactChecksumRecord,
}
impl ExecutionStageSettlementView {
    /// Read the original opaque stage identity.
    #[must_use]
    pub const fn stage_sequence(&self) -> u64 {
        self.stage_sequence
    }
    /// Read the exact retained binding identity.
    #[must_use]
    pub const fn binding(&self) -> &ArtifactChecksumRecord {
        &self.binding
    }
    /// Read the complete original journal-history checkpoint identity.
    #[must_use]
    pub const fn settlement(&self) -> &ArtifactChecksumRecord {
        &self.settlement
    }
}

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
    let bytes = super::json::encode_json(record, MAX_EXECUTION_WORKFLOW_BYTES)?;
    let path = layout.root().join(WORKFLOW_FILE);
    let _lock = JournalLock::acquire(&path)?;
    super::json::create_encoded(&path, &bytes)?;
    Ok(())
}
/// Read exact original workflow allocation; missing evidence never supplies a new budget.
/// # Errors
/// Rejects missing/changed/unsafe/oversized records and ownership failures.
pub fn read_execution_workflow(
    layout: &BackupLayoutGuard,
    expected: &ArtifactChecksumRecord,
) -> Result<ExecutionWorkflowRecord, ExecutionWorkflowPersistenceError> {
    super::immutable_record::read(
        layout,
        WORKFLOW_FILE,
        MAX_EXECUTION_WORKFLOW_BYTES,
        |record: &ExecutionWorkflowRecord| &record.digest() == expected,
        || ExecutionWorkflowPersistenceError::DigestMismatch,
    )
}

/// Read which allocated stage paths are already occupied under the held layout.
///
/// This is a namespace preflight for a retained run, not evidence that an absent
/// stage has never dispatched. An occupied partial stage remains occupied and is
/// never repaired. Derived symlinks and non-directories refuse admission.
/// # Errors
/// Rejects changed workflow/layout, unsafe stage entries and IO failures.
pub(crate) fn read_execution_stage_occupancy(
    layout: &BackupLayoutGuard,
    expected: &ArtifactChecksumRecord,
) -> Result<BTreeSet<u64>, ExecutionWorkflowPersistenceError> {
    let workflow = read_execution_workflow(layout, expected)?;
    let mut occupied = BTreeSet::new();
    for stage in workflow.stages() {
        let sequence = stage.operation_sequence();
        match fs::symlink_metadata(stage_path(layout, sequence)) {
            Ok(metadata) if metadata.is_dir() => {
                occupied.insert(sequence);
            }
            Ok(_) => return Err(ExecutionWorkflowPersistenceError::UnsafeStage),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(PersistenceError::from(error).into()),
        }
    }
    read_execution_workflow(layout, expected)?;
    Ok(occupied)
}

/// Replay every allocated stage's exact retained binding and all-Applied checkpoint.
///
/// The original workflow determines the complete stage set. Every fixed stage must
/// exist with a valid child plan, immutable binding, complete original journal set
/// and exact published settlement. Every binding's predecessor identities must
/// match this complete original set. Stage locks are held one at a time under
/// the caller's workflow layout guard. No absent stage means zero spending, and no
/// occupied checkpoint is repaired or republished. The resulting digest rows
/// are local evidence only; integration-owned authenticity, complete transfer,
/// application safety, command quiescence and terminal/release proof remain separate.
/// # Errors
/// Rejects missing, unsafe, changed, held or unsettled original evidence without effects.
pub fn read_complete_execution_workflow(
    layout: &BackupLayoutGuard,
    expected: &ArtifactChecksumRecord,
) -> Result<ExecutionWorkflowSettlementView, ExecutionStageCheckpointError> {
    read_complete_with(layout, expected, || {
        read_execution_workflow(layout, expected)
    })
}

/// Replay complete original settlement while retaining the exact workflow journal lock.
///
/// Terminal-evidence providers may use this instead of reacquiring their held journal
/// lock. The guard must protect this layout's original `execution-workflow.json`.
/// Both opening and closing reads retain the ordinary bounds, strict decoding and
/// expected original digest checks; child locks and checkpoint admission are unchanged.
/// This supplies local settlement only, never application or reference-release authority.
/// # Errors
/// Rejects a guard for another journal, changed originals, or incomplete settlement.
pub fn read_complete_execution_workflow_locked(
    layout: &BackupLayoutGuard,
    journal: &JournalLock,
    expected: &ArtifactChecksumRecord,
) -> Result<ExecutionWorkflowSettlementView, ExecutionStageCheckpointError> {
    if journal.journal() != layout.root().join(WORKFLOW_FILE) {
        return Err(ExecutionWorkflowPersistenceError::DigestMismatch.into());
    }
    read_complete_with(layout, expected, || {
        layout.check_root()?;
        let workflow: ExecutionWorkflowRecord =
            read_json(journal.journal(), MAX_EXECUTION_WORKFLOW_BYTES)?;
        super::json::check_json_size(&workflow, MAX_EXECUTION_WORKFLOW_BYTES)?;
        if &workflow.digest() != expected {
            return Err(ExecutionWorkflowPersistenceError::DigestMismatch);
        }
        Ok(workflow)
    })
}

fn read_complete_with(
    layout: &BackupLayoutGuard,
    expected: &ArtifactChecksumRecord,
    read_original: impl Fn() -> Result<ExecutionWorkflowRecord, ExecutionWorkflowPersistenceError>,
) -> Result<ExecutionWorkflowSettlementView, ExecutionStageCheckpointError> {
    let workflow = read_original()?;
    let mut stages = Vec::with_capacity(workflow.stages().len());
    let mut bindings = Vec::with_capacity(workflow.stages().len());
    let mut by_sequence = BTreeMap::new();
    for allocated in workflow.stages() {
        let sequence = allocated.operation_sequence();
        let stage = acquire_stage(layout, sequence)?;
        let binding: ExecutionStageBindingRecord =
            read_json(&stage.root().join(BINDING_FILE), MAX_EXECUTION_STAGE_BYTES)
                .map_err(ExecutionWorkflowPersistenceError::from)?;
        let (binding, plan) = read_binding(&stage, &workflow, sequence, &binding.digest())?;
        let settlement = replay_execution_settlement(&stage, &plan.digest())?;
        read_binding(&stage, &workflow, sequence, &binding.digest())?;
        by_sequence.insert(sequence, stages.len());
        stages.push(ExecutionStageSettlementView {
            stage_sequence: sequence,
            binding: binding.digest(),
            settlement: settlement.digest(),
        });
        bindings.push(binding);
    }
    for binding in &bindings {
        for predecessor in binding.predecessors() {
            let Some(&index) = by_sequence.get(&predecessor.stage_sequence()) else {
                return Err(ExecutionWorkflowPersistenceError::DigestMismatch.into());
            };
            let actual = &stages[index];
            if predecessor.binding() != actual.binding()
                || predecessor.settlement() != actual.settlement()
            {
                return Err(ExecutionWorkflowPersistenceError::DigestMismatch.into());
            }
        }
    }
    read_original()?;
    Ok(ExecutionWorkflowSettlementView {
        workflow: expected.clone(),
        stages,
    })
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
    fn reconcile_checkpoint_with(
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
    // Allocate immutable stage records internally before preparing original journals.
    fn create_with(
        workflow_layout: &'a BackupLayoutGuard,
        binding: ExecutionStageBindingRecord,
        plan: OperationPlanRecord,
        mut barrier: impl FnMut(StagePreparationBarrier),
    ) -> Result<Self, ExecutionWorkflowPersistenceError> {
        let workflow = read_execution_workflow(workflow_layout, binding.workflow())?;
        binding.validate(&workflow, &plan)?;
        let binding_bytes = super::json::encode_json(&binding, MAX_EXECUTION_STAGE_BYTES)?;
        // Encode both documents before allocating any stage paths.
        let plan_bytes = super::json::encode_json(
            &plan,
            crate::model::operation_plan::MAX_OPERATION_PLAN_BYTES,
        )?;
        validate_predecessors(workflow_layout, &workflow, &binding)?;
        let path = stage_path(workflow_layout, binding.stage_sequence());
        ic_host_fs::durable::create_private_directory(&path)
            .map_err(|error| PersistenceError::Io(crate::ops::directory_creation_io(error)))?;
        let stage_layout = acquire_stage(workflow_layout, binding.stage_sequence())?;
        super::operation_plan::create_encoded(&stage_layout, &plan_bytes)?;
        barrier(StagePreparationBarrier::Plan);
        super::json::create_encoded(&stage_layout.root().join(BINDING_FILE), &binding_bytes)?;
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

/// Stage-bound checkpoint publication/replay refusal; originals remain retained.
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
