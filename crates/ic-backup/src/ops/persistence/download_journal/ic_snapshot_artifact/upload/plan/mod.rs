//! Bounded exact source preparation before any data-stage spending.

mod source;

use super::{DownloadJournalGuard, IcSnapshotUploadArtifactError};
use crate::model::{
    execution_workflow::ExecutionWorkflowRecord,
    ic_snapshot_upload::{
        IcSnapshotDataUploadPlan, IcSnapshotDataUploadPlanningError, IcSnapshotUploadReply,
    },
};
use thiserror::Error;

impl DownloadJournalGuard<'_> {
    /// Prepare the complete finite original data plan, buffering one payload at a time.
    ///
    /// Count all regions/known chunks against the original workflow data-stage ceiling
    /// before reading source bytes. Require the exact retained source plan/journal/tree
    /// and original metadata allocation reply. One bounded batch binds
    /// payloads and the canonical tree checksum to the same exact bytes. Retain only
    /// request digests and verify the tree before returning the plan. Repeated
    /// preparation creates no journal or allowance.
    ///
    /// The returned model binds declarations only. Independently authenticate exclusive
    /// allocation attribution and durable original request/reply custody before binding
    /// its Applied predecessor. Later writes freshly prepare/check their exact payload;
    /// source stability and future command/destination custody stay integration-owned.
    /// # Errors
    /// Rejects wrong originals, insufficient original allocation before IO, changed or
    /// unsafe source bytes and existing bounded plan/request failures. No records change.
    pub fn prepare_ic_snapshot_data_upload_plan<'workflow, 'request, 'source>(
        &self,
        workflow: &'workflow ExecutionWorkflowRecord,
        sequence: u64,
        snapshot: &str,
        allocation: &IcSnapshotUploadReply<'request, 'source>,
        chunk_bytes: u64,
    ) -> Result<
        IcSnapshotDataUploadPlan<'workflow, 'request, 'source>,
        IcSnapshotDataUploadPreparationError,
    > {
        let kinds =
            IcSnapshotDataUploadPlan::derive_kinds(workflow, sequence, allocation, chunk_bytes)?;
        let metadata = allocation.request();
        let crate::model::ic_snapshot_upload::IcSnapshotUploadReplyKind::Metadata { snapshot_id } =
            allocation.kind()
        else {
            return Err(IcSnapshotDataUploadPlanningError::OriginalMismatch.into());
        };
        let bindings =
            self.with_upload_source(metadata.source_plan(), snapshot, metadata, |directory| {
                source::bindings(directory, metadata, snapshot_id, &kinds)
            })?;
        Ok(IcSnapshotDataUploadPlan::from_bindings(
            workflow, sequence, allocation, kinds, &bindings,
        )?)
    }
}

/// Complete local plan preparation failure retains every original source record.
#[derive(Debug, Error)]
pub enum IcSnapshotDataUploadPreparationError {
    /// Exact original workflow, allocation or finite plan admission failed.
    #[error(transparent)]
    Planning(#[from] IcSnapshotDataUploadPlanningError),
    /// Existing original guarded source/byte preparation failed.
    #[error(transparent)]
    Source(#[from] IcSnapshotUploadArtifactError),
}
