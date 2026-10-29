//! Fixed immutable stage admission using the existing layout, plan and settlement owners.

mod ancestry;
mod progress;
mod stage;
use ancestry::{
    acquire_stage, read_admitted_binding, read_binding, stage_path, validate_predecessors,
};
pub use progress::{ExecutionWorkflowProgressError, read_execution_workflow_progress};
pub use stage::ExecutionStageGuard;
#[cfg(test)]
use stage::StagePreparationBarrier;

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

/// Durably create the full original workflow allocation without replacement.
/// # Errors
/// Rejects occupied/unsafe paths, bounds, ownership and publication failures.
pub fn create_execution_workflow(
    layout: &BackupLayoutGuard,
    record: &ExecutionWorkflowRecord,
) -> Result<(), ExecutionWorkflowPersistenceError> {
    layout.check_root()?;
    let bytes = super::json::encode_json(record, MAX_EXECUTION_WORKFLOW_BYTES)?;
    super::immutable_record::create_encoded_record(layout, WORKFLOW_FILE, &bytes)
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
    for allocated in workflow.stages() {
        let sequence = allocated.operation_sequence();
        let stage = acquire_stage(layout, sequence)?;
        let binding: ExecutionStageBindingRecord =
            read_json(&stage.root().join(BINDING_FILE), MAX_EXECUTION_STAGE_BYTES)
                .map_err(ExecutionWorkflowPersistenceError::from)?;
        let (binding, plan) = read_binding(&stage, &workflow, sequence, &binding.digest())?;
        let settlement = replay_execution_settlement(&stage, &plan.digest())?;
        read_binding(&stage, &workflow, sequence, &binding.digest())?;
        let view = ExecutionStageSettlementView {
            stage_sequence: sequence,
            binding: binding.digest(),
            settlement: settlement.digest(),
        };
        stages.push((binding, view));
    }
    for (binding, _) in &stages {
        for predecessor in binding.predecessors() {
            // The original catalog supplies canonical sequence order.
            let index = stages
                .binary_search_by_key(&predecessor.stage_sequence(), |(_, view)| {
                    view.stage_sequence()
                })
                .map_err(|_| ExecutionWorkflowPersistenceError::DigestMismatch)?;
            let actual = &stages[index].1;
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
        stages: stages.into_iter().map(|(_, view)| view).collect(),
    })
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
