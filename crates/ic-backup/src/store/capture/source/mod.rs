//! Authenticated capture evidence and held payload custody, without restore authority.

pub(in crate::store) mod transfer;

use super::{
    BackupStore, CaptureOperation, OperationReplayError, OperationStoreError, same_identity,
};
use crate::store::{BackupSummary, PayloadReadError, PayloadSource, StoreError};
use ic_platform_protocol::management_snapshots::ReadDataArgs;
use std::{fmt, num::NonZeroU64};
use thiserror::Error;
pub use transfer::{RestoreTransfer, RestoreTransferError, RestoreTransferRequest};

/// Original capture lock and authenticated payload descriptors. No restore authority.
pub struct CaptureSource<'capture, 'store> {
    _capture: &'capture CaptureOperation,
    payload: PayloadSource<'store>,
    metadata: ic_platform_protocol::management_snapshots::SnapshotMetadata,
}
/// Source selection, original evidence or held byte custody refused without repair.
#[derive(Debug, Error)]
pub enum CaptureSourceError {
    /// Caller-selected receipt differs from the original operation.
    #[error("capture source receipt does not match the selected operation")]
    Selection,
    /// Original local custody refused.
    #[error(transparent)]
    Capture(#[from] OperationStoreError),
    /// Original certified completion or payload policy refused.
    #[error(transparent)]
    Completion(#[from] OperationReplayError),
    /// Bounded payload/manifest admission refused.
    #[error(transparent)]
    Store(#[from] StoreError),
}
impl BackupStore {
    /// Authenticate an exact completed receipt and retain its original source files.
    ///
    /// Requires this same held store, capture lock and caller-selected payload limit.
    /// Reads use bounded block checks after admission; no signer, guard, clock, HTTP,
    /// synchronization or publication occurs. The returned borrow holds capture custody.
    /// # Errors
    /// Refuses wrong receipts/stores, missing or changed evidence, unsafe files or limits.
    pub fn open_capture_source<'capture, 'store>(
        &'store self,
        operation: &'capture mut CaptureOperation,
        expected: &BackupSummary,
        maximum_payload_bytes: NonZeroU64,
    ) -> Result<CaptureSource<'capture, 'store>, CaptureSourceError> {
        same_identity(&self.root, &operation.store)?;
        if expected.bundle_id != operation.plan().request().operation_id {
            return Err(CaptureSourceError::Selection);
        }
        let payload = self.open_payload(
            &expected.bundle_id,
            expected.manifest_sha256,
            maximum_payload_bytes,
        )?;
        if payload.summary() != expected {
            return Err(CaptureSourceError::Selection);
        }
        let (verified, flow) = operation.verified_completion(expected.manifest_sha256)?;
        let metadata = flow
            .transfer()
            .map_err(OperationReplayError::from)?
            .metadata()
            .clone();
        if verified != *expected {
            return Err(CaptureSourceError::Selection);
        }
        payload.check_files()?;
        same_identity(&self.root, &operation.store)?;

        Ok(CaptureSource {
            _capture: operation,
            payload,
            metadata,
        })
    }
}
impl<'store> CaptureSource<'_, 'store> {
    // Restore retains the owning CaptureOperation beside these descriptors.
    pub(in crate::store) fn into_parts(
        self,
    ) -> (
        PayloadSource<'store>,
        ic_platform_protocol::management_snapshots::SnapshotMetadata,
    ) {
        (self.payload, self.metadata)
    }

    pub(in crate::store) fn metadata(
        &self,
    ) -> &ic_platform_protocol::management_snapshots::SnapshotMetadata {
        &self.metadata
    }

    pub(in crate::store) fn maximum_member_bytes(&self) -> u64 {
        self.payload
            .member_lengths()
            .into_iter()
            .flatten()
            .map(|(_, n)| n)
            .max()
            .unwrap_or(0)
    }

    pub(in crate::store) fn original_plan(&self) -> &super::CapturePlan {
        self._capture.plan()
    }

    /// Original authenticated receipt selected at admission; no restore permission.
    #[must_use]
    pub fn summary(&self) -> &BackupSummary {
        self.payload.summary()
    }

    /// Read original source bytes through the held payload reader.
    ///
    /// Requires the original canister/snapshot and a valid extent or chunk. Returns
    /// at most 1 MiB after checking at most two original blocks and held custody.
    /// # Errors
    /// Invalid requests leave the source usable. IO, integrity or custody failures
    /// invalidate payload reads; reacquisition is explicit and never repairs files.
    pub fn read_data(&mut self, request: &ReadDataArgs) -> Result<Vec<u8>, PayloadReadError> {
        self.payload.read_data(request)
    }
}
impl fmt::Debug for CaptureSource<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CaptureSource")
            .field("payload", &self.payload)
            .finish_non_exhaustive()
    }
}
