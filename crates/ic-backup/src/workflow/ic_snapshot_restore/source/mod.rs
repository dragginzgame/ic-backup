//! Exact existing-snapshot or completed-upload provenance for one restore step.

use crate::{
    model::{
        attempt_journal::AppliedReceiptEvidence,
        execution_workflow::ExecutionStagePredecessorRecord,
        ic_request::{IcManagementMethodRecord, IcManagementRequestRecord},
        ic_snapshot_metadata::IcSnapshotMetadataRequest,
        ic_snapshot_upload::{IcSnapshotDataUploadPlan, IcSnapshotDataUploadPlanningError},
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{
        AttemptJournalError, AttemptJournalGuard, BackupLayoutGuard, ExecutionStageCheckpointError,
        ExecutionStageGuard, ExecutionWorkflowPersistenceError,
    },
};
use thiserror::Error;

/// Source identity retained independently of the restore request being admitted.
///
/// Existing snapshots require authenticated metadata/source association from the
/// integration. Uploaded snapshots additionally require the exact complete retained
/// data stage and allocation predecessor. Neither form authenticates backend bytes,
/// permissions or application safety; the mandatory admission callback still does.
#[derive(Clone)]
pub struct IcRestoreSource<'a> {
    pub(super) layout: &'a BackupLayoutGuard,
    pub(super) plan: &'a OperationPlanRecord,
    target: String,
    snapshot_id: Vec<u8>,
    upload: Option<UploadOrigin<'a>>,
}

#[derive(Clone)]
struct UploadOrigin<'a> {
    layout: &'a BackupLayoutGuard,
    workflow: crate::model::artifacts::ArtifactChecksumRecord,
    completed: ExecutionStagePredecessorRecord,
    allocation_receipt: Option<crate::model::artifacts::ArtifactChecksumRecord>,
}

impl<'a> IcRestoreSource<'a> {
    /// Bind an existing snapshot's independently qualified metadata request.
    ///
    /// Use only for an existing source snapshot, not as a substitute for upload
    /// completion. This passive declaration supplies no authentic byte association.
    #[must_use]
    pub fn existing(
        layout: &'a BackupLayoutGuard,
        plan: &'a OperationPlanRecord,
        metadata: &IcSnapshotMetadataRequest,
    ) -> Self {
        Self {
            layout,
            plan,
            target: metadata.target().into(),
            snapshot_id: metadata.snapshot_id().into(),
            upload: None,
        }
    }

    /// Bind an allocated destination to its complete original upload stage.
    ///
    /// Hold no upload attempt/stage guards. Reopening the existing stage checks
    /// allocation ancestry, every original Applied history and its checkpoint;
    /// it creates no receipt, stage, allowance or replacement authority.
    /// # Errors
    /// Rejects incomplete/pending, substituted or cross-plan upload evidence.
    pub fn uploaded(
        layout: &'a BackupLayoutGuard,
        upload: &IcSnapshotDataUploadPlan<'_, '_, 'a>,
        workflow_layout: &'a BackupLayoutGuard,
        completed: &ExecutionStagePredecessorRecord,
    ) -> Result<Self, IcRestoreSourceError> {
        let origin = UploadOrigin {
            layout: workflow_layout,
            workflow: upload.workflow_digest(),
            completed: completed.clone(),
            allocation_receipt: upload
                .plan()
                .is_none()
                .then(|| completed.learned_evidence().clone()),
        };
        let stage = origin.open()?;
        upload.validate_completion(stage.binding(), stage.plan(), completed.learned_evidence())?;
        origin.replay(&stage)?;
        Ok(Self {
            layout,
            plan: upload.metadata().source_plan(),
            target: upload.metadata().target().into(),
            snapshot_id: upload.destination().into(),
            upload: Some(origin),
        })
    }

    pub(crate) fn validate(
        &self,
        stage: &ExecutionStageGuard<'_>,
        wire: &IcManagementRequestRecord,
    ) -> Result<(), IcRestoreSourceError> {
        if wire.target() != self.target
            || (wire.method() == IcManagementMethodRecord::LoadCanisterSnapshot
                && wire.snapshot_id() != Some(self.snapshot_id.as_slice()))
        {
            return Err(IcRestoreSourceError::Mismatch);
        }
        if let Some(origin) = &self.upload {
            if stage.binding().workflow() == &origin.workflow
                && (wire.method() == IcManagementMethodRecord::LoadCanisterSnapshot
                    && !stage.binding().predecessors().contains(&origin.completed))
            {
                return Err(IcRestoreSourceError::Mismatch);
            }
            let upload = origin.open()?;
            origin.replay(&upload)?;
        }
        Ok(())
    }
}

impl UploadOrigin<'_> {
    fn open(&self) -> Result<ExecutionStageGuard<'_>, ExecutionWorkflowPersistenceError> {
        ExecutionStageGuard::open(
            self.layout,
            &self.workflow,
            self.completed.stage_sequence(),
            self.completed.binding(),
        )
    }

    fn replay(&self, stage: &ExecutionStageGuard<'_>) -> Result<(), IcRestoreSourceError> {
        if stage.replay_checkpoint(self.completed.learned_evidence().clone())? != self.completed {
            return Err(IcRestoreSourceError::Mismatch);
        }
        if let Some(expected) = &self.allocation_receipt {
            let sequence = stage.plan().operations()[0].operation_sequence();
            let authority = stage
                .plan()
                .attempt_authority(sequence)
                .map_err(|_| IcRestoreSourceError::Mismatch)?;
            let journal = AttemptJournalGuard::open(stage.layout()?, &authority)?;
            if !matches!(journal.record()?.applied_receipt(), Some(AppliedReceiptEvidence::Mutation { evidence, .. }) if evidence == expected.hash())
            {
                return Err(IcRestoreSourceError::Mismatch);
            }
        }
        Ok(())
    }
}

/// Retained upload/load provenance rejection; no variant permits reissue.
#[derive(Debug, Error)]
pub enum IcRestoreSourceError {
    /// Original allocation receipt cannot be admitted under its journal exclusion.
    #[error(transparent)]
    Journal(#[from] AttemptJournalError),
    /// Target, snapshot, original plan or predecessor identity differs.
    #[error("restore snapshot source differs from its original upload or metadata")]
    Mismatch,
    /// Original allocation/data-plan admission failed.
    #[error(transparent)]
    Planning(#[from] IcSnapshotDataUploadPlanningError),
    /// Original stage/ancestor admission failed.
    #[error(transparent)]
    Stage(#[from] ExecutionWorkflowPersistenceError),
    /// Complete original upload settlement is absent or changed.
    #[error(transparent)]
    Checkpoint(#[from] ExecutionStageCheckpointError),
}
