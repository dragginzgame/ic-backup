//! Single originally accounted exact destination-data observation contract.

use crate::{
    model::ic_snapshot_upload_data_observation::{
        IcSnapshotUploadDataObservationRequest, IcSnapshotUploadDataObservationResponse,
    },
    ports::ic_observation::IcObservationProviderError,
};

/// Integration-owned authenticated readback of one unresolved original data upload.
///
/// Retain exact source/upload/destination-metadata/read originals and durably reserve
/// the observation first. Qualify original allocation, fresh snapshot read access,
/// actual context/timing, stable command/byte custody and proof this observation was
/// never dispatched. Reconstructed requests grant no repeat call.
///
/// Invoke only the one originally accounted exact replicated host data-read update;
/// no hidden retry, metadata/list/status read, upload, proxy or query substitution.
/// Preserve actual original claim association and bounded raw evidence. Matching
/// bytes may already exist and prove no upload occurrence/attribution; differing or
/// absent data prove no `NotApplied` outcome. Backend readback ability remains qualified
/// by the integration. No default provider or complete-upload admission is installed.
///
/// Failures/drop/death retain both pending attempts, source references and application/
/// fence obligations. A lost observation reply stays pending rather than Uncertain.
/// No receipt, refund, load/start, terminal or fence/reference-release permit follows.
pub trait IcSnapshotUploadDataObservationProvider {
    /// Read one exact original destination extent/hash under its spent observation.
    /// # Errors
    /// Existing typed failures retain accounting and authorize no implicit reissue.
    fn observe_upload_data(
        &mut self,
        request: &IcSnapshotUploadDataObservationRequest<'_, '_, '_>,
    ) -> Result<IcSnapshotUploadDataObservationResponse, IcObservationProviderError>;
}

#[cfg(test)]
mod tests;
