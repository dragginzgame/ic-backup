//! Single originally reserved metadata-upload inventory observation contract.

use crate::{
    model::{
        ic_observation::IcObservationResponse,
        ic_snapshot_upload_observation::IcSnapshotUploadObservationRequest,
    },
    ports::ic_observation::IcObservationProviderError,
};

/// Integration-owned authenticated list evidence for one unresolved metadata upload.
///
/// Retain exact source/upload/list originals and durably reserve this observation
/// before invoking the provider. Independently qualify current context and snapshot
/// read permission, original recovery chronology and exclusive command custody with
/// proof this observation was never dispatched. Reconstruction proves none of these.
///
/// Invoke exactly one originally accounted host replicated list update using the
/// request's management receiver, effective target, method and bytes. No hidden
/// retry, query/proxy substitution, extra observation or upload is permitted.
/// Retain the raw reply and actual authority/attempt/context/target association.
/// Freshness, authentication, baseline custody and exclusive allocation attribution
/// remain integration-owned; zero/one/many snapshots establish no upload outcome.
///
/// Failure/drop/death retain pending mutation/observation spending, original source
/// references and application/fence obligations. Lost observation replies stay
/// pending, not settled Uncertain. No receipt, refund, default provider, data-upload
/// reconciliation, terminal or reference-release authority is installed.
pub trait IcSnapshotUploadObservationProvider {
    /// Observe once under the exact original reserved list request.
    /// # Errors
    /// All existing typed failures preserve accounting and permit no implicit reissue.
    fn observe_upload(
        &mut self,
        request: &IcSnapshotUploadObservationRequest<'_, '_>,
    ) -> Result<IcObservationResponse, IcObservationProviderError>;
}

#[cfg(test)]
mod tests;
