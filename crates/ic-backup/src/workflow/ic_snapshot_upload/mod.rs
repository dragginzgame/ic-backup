//! One fresh original source-bound metadata/data upload; no automatic receipt or retry.

mod allocation;
#[cfg(unix)]
mod data;
#[cfg(unix)]
pub use data::{
    IcSnapshotDataUploadExecutionError, IcSnapshotDataUploadReplyError, upload_snapshot_data,
};

pub use allocation::{
    IcSnapshotAllocationExecutionError, IcSnapshotAllocationSettlementError, allocate_snapshot,
};

use crate::{
    model::{
        ic_mutation::IcMutationAcknowledgement,
        ic_snapshot_upload::{
            IcSnapshotUploadAttempt, IcSnapshotUploadAttemptError, IcSnapshotUploadRequest,
            original_authority,
        },
        operation_plan::OperationPlanError,
    },
    ops::persistence::{
        AttemptJournalError, AttemptJournalGuard, ExecutionProgressPersistenceError,
        ExecutionStageGuard, ExecutionWorkflowPersistenceError,
    },
    policy::ic_snapshot_upload::{
        IcSnapshotUploadAcknowledgementView, IcSnapshotUploadAssociationError,
        validate_acknowledgement,
    },
    ports::{ic_mutation::IcMutationProviderError, ic_snapshot_upload::IcSnapshotUploadProvider},
};
use thiserror::Error;

// Complete compound uploads under their original guard using the admitted reply.
// Return before checkpointing, which must run after journal exclusion is released.
async fn upload_snapshot_with<E: std::error::Error + 'static, T>(
    stage: &ExecutionStageGuard<'_>,
    operation_sequence: u64,
    payload: &IcSnapshotUploadRequest<'_>,
    provider: &mut impl IcSnapshotUploadProvider,
    admit: impl AsyncFnOnce(&IcSnapshotUploadAttempt<'_, '_>) -> Result<(), E>,
    complete: impl AsyncFnOnce(
        &mut AttemptJournalGuard<'_>,
        &IcSnapshotUploadAttempt<'_, '_>,
        &IcSnapshotUploadAcknowledgementView<'_, '_>,
    ) -> T,
) -> Result<(IcMutationAcknowledgement, T), IcSnapshotUploadExecutionError<E>> {
    let plan = stage.plan();
    let authority = plan.attempt_authority(operation_sequence)?;
    let mut journal = AttemptJournalGuard::open(stage.layout()?, &authority)?;
    original_authority(plan, operation_sequence, journal.record()?, payload)?;
    journal.reserve_planned_mutation(&plan.digest())?;
    let request =
        IcSnapshotUploadAttempt::new(plan, operation_sequence, journal.record()?, payload)?;
    admit(&request)
        .await
        .map_err(IcSnapshotUploadExecutionError::Admission)?;
    stage.layout()?;
    let acknowledgement = provider.submit_upload(&request, journal.record()?).await?;
    let admit_reply = || {
        let admitted = validate_acknowledgement(&request, journal.record()?, &acknowledgement)?;
        stage.layout()?;
        Ok::<_, IcSnapshotUploadReplyError>(admitted)
    };
    let admitted = match admit_reply() {
        Ok(admitted) => admitted,
        Err(source) => {
            return Err(IcSnapshotUploadExecutionError::AfterReply {
                source,
                acknowledgement: Box::new(acknowledgement),
            });
        }
    };
    let result = complete(&mut journal, &request, &admitted).await;
    Ok((acknowledgement, result))
}

/// Upload-step failures retain original spending and any bounded returned acknowledgement.
#[derive(Debug, Error)]
pub enum IcSnapshotUploadExecutionError<E: std::error::Error + 'static> {
    /// Original operation authority cannot be derived.
    #[error(transparent)]
    Plan(#[from] OperationPlanError),
    /// Workflow, stage or original ancestor evidence failed admission.
    #[error(transparent)]
    Stage(#[from] ExecutionWorkflowPersistenceError),
    /// Original selected journal admission failed.
    #[error(transparent)]
    Journal(#[from] AttemptJournalError),
    /// Complete original progress, prerequisites or durable reservation failed.
    #[error(transparent)]
    Progress(#[from] ExecutionProgressPersistenceError),
    /// Canonical original mutation binding or current reservation differs.
    #[error(transparent)]
    Request(#[from] IcSnapshotUploadAttemptError),
    /// Fresh integration-owned admission rejected after reservation.
    #[error("fresh snapshot upload admission failed: {0}")]
    Admission(#[source] E),
    /// The single provider call failed; spending remains pending.
    #[error(transparent)]
    Provider(#[from] IcMutationProviderError),
    /// Admission after a reply failed, retaining its complete bounded evidence.
    #[error("snapshot upload reply rejected: {source}")]
    AfterReply {
        /// Typed original admission failure.
        source: IcSnapshotUploadReplyError,
        /// Complete returned passive evidence, without settlement authority.
        acknowledgement: Box<IcMutationAcknowledgement>,
    },
}

/// Admission failure after a returned snapshot upload reply.
#[derive(Debug, Error)]
pub enum IcSnapshotUploadReplyError {
    /// Original selected journal cannot be re-admitted.
    #[error(transparent)]
    Journal(#[from] AttemptJournalError),
    /// Passive exact reply association failed.
    #[error(transparent)]
    Association(#[from] IcSnapshotUploadAssociationError),
    /// Original stage or ancestor evidence cannot be re-admitted.
    #[error(transparent)]
    Stage(#[from] ExecutionWorkflowPersistenceError),
}

#[cfg(all(test, unix))]
mod tests;
