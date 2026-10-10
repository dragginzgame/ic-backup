//! One fresh original source-bound metadata/data upload; no automatic receipt or retry.

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
    policy::ic_snapshot_upload::{IcSnapshotUploadAssociationError, validate_acknowledgement},
    ports::{ic_mutation::IcMutationProviderError, ic_snapshot_upload::IcSnapshotUploadProvider},
};
use thiserror::Error;

/// Durably reserve one exact original upload, freshly admit it and submit once.
///
/// Prepare the retained stage and every original journal before entry; hold no attempt
/// guards. Guarded source preparation and exact metadata/tree/payload retention precede
/// the upload plan. Data bytes and their independently attributed destination must be
/// bound before that data stage is published, within its original workflow allocation.
/// This checks the canonical full source/upload context and binding before spending,
/// then delegates reservation/prerequisites to the existing complete-plan owner.
/// Missing, pending, Applied or exhausted originals never reach provider invocation.
///
/// The mandatory fallible `admit` callback qualifies authentic complete source bytes,
/// actual fresh controllers and application/restore requirements, exact metadata/data
/// destination attribution, stable source/payload custody and exclusive never-dispatched
/// command custody. It runs under the selected journal lock; remote preflight requires
/// its own prior accounting. Local checks grant no byte fence or allocation authority.
/// The existing provider sends only the exact accounted replicated update, with no
/// batching, retries, hidden observations or implicit funding. No default is installed.
///
/// Re-admit original stage/ancestor evidence before and after dispatch and check the
/// bounded acknowledgement through the canonical upload policy/decoder. Success
/// remains pending; integrations independently authenticate attribution and retain
/// exact replies before explicitly recording receipts with the existing journal owner.
/// A returned ID/empty reply alone grants no allocation outcome, learned data authority,
/// complete transfer, load/start admission, terminal or fence/source-reference release.
/// # Errors
/// Rejects changed originals/payload/context, spending, fresh admission, provider or
/// reply failures. Post-reservation failures retain consumption; post-reply rejections
/// retain the bounded acknowledgement. No failure grants redispatch/refund/cleanup.
pub fn upload_snapshot<E: std::error::Error + 'static>(
    stage: &ExecutionStageGuard<'_>,
    operation_sequence: u64,
    payload: &IcSnapshotUploadRequest<'_>,
    provider: &mut impl IcSnapshotUploadProvider,
    admit: impl FnOnce(&IcSnapshotUploadAttempt<'_, '_>) -> Result<(), E>,
) -> Result<IcMutationAcknowledgement, IcSnapshotUploadExecutionError<E>> {
    let plan = stage.plan();
    let authority = plan.attempt_authority(operation_sequence)?;
    let mut journal = AttemptJournalGuard::open(stage.layout()?, &authority)?;
    original_authority(plan, operation_sequence, journal.record()?, payload)?;
    journal.reserve_planned_mutation(&plan.digest())?;
    let request =
        IcSnapshotUploadAttempt::new(plan, operation_sequence, journal.record()?, payload)?;
    admit(&request).map_err(IcSnapshotUploadExecutionError::Admission)?;
    stage.layout()?;
    let acknowledgement = provider.submit_upload(&request)?;
    let association = match journal.record() {
        Ok(record) => record,
        Err(source) => {
            return Err(IcSnapshotUploadExecutionError::AfterReplyJournal {
                source,
                acknowledgement: Box::new(acknowledgement),
            });
        }
    };
    if let Err(source) = validate_acknowledgement(&request, association, &acknowledgement) {
        return Err(IcSnapshotUploadExecutionError::Association {
            source,
            acknowledgement: Box::new(acknowledgement),
        });
    }
    if let Err(source) = stage.layout() {
        return Err(IcSnapshotUploadExecutionError::AfterReplyStage {
            source,
            acknowledgement: Box::new(acknowledgement),
        });
    }
    Ok(acknowledgement)
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
    /// Passive bounded association rejected a returned acknowledgement.
    #[error("snapshot upload acknowledgement association failed: {source}")]
    Association {
        /// Existing canonical mutation association rejection.
        source: IcSnapshotUploadAssociationError,
        /// Exact returned acknowledgement, without authenticated outcome.
        acknowledgement: Box<IcMutationAcknowledgement>,
    },
    /// Selected journal re-admission failed after a reply.
    #[error("snapshot upload journal changed after reply: {source}")]
    AfterReplyJournal {
        /// Original journal rejection.
        source: AttemptJournalError,
        /// Exact bounded returned acknowledgement.
        acknowledgement: Box<IcMutationAcknowledgement>,
    },
    /// Stage or ancestor re-admission failed after a reply.
    #[error("snapshot upload stage changed after reply: {source}")]
    AfterReplyStage {
        /// Original stage or ancestor rejection.
        source: ExecutionWorkflowPersistenceError,
        /// Exact bounded returned acknowledgement.
        acknowledgement: Box<IcMutationAcknowledgement>,
    },
}

#[cfg(all(test, unix))]
mod tests;
