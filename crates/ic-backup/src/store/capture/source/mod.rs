//! Authenticated capture evidence and held payload custody, without restore authority.
mod transfer;
pub use transfer::{RestoreTransfer, RestoreTransferError, RestoreTransferRequest};

use super::{
    BackupStore, CaptureCompletionError, CaptureError, CaptureOperation, CaptureProvider,
    StatusHttpTransport, same_identity,
};
use crate::store::{BackupSummary, PayloadReadError, PayloadSource, StoreError};
use ic_platform::management_canister_types::ReadCanisterSnapshotDataArgs;
use std::{fmt, num::NonZeroU64};
use thiserror::Error;

/// Held original capture lock and authenticated payload descriptors.
///
/// Admission authenticates original evidence once. Reads use the existing bounded
/// payload reader and its held identities/block hashes; they do not reauthenticate
/// all certificates or rehash the whole bundle on each extent. Exclude external
/// namespace/content writers for the handle's lifetime. This supplies no restore
/// intent, durable source reference, target claim or permission to upload/load.
pub struct CaptureSource<'capture, 'store> {
    _capture: &'capture CaptureOperation,
    payload: PayloadSource<'store>,
}

/// Source selection or original custody/evidence refused without modifying files.
#[derive(Debug, Error)]
pub enum CaptureSourceError<E> {
    /// Caller-selected receipt does not match the original operation and payload.
    #[error("capture source receipt does not match the selected operation")]
    Selection,
    /// Original capture/store custody refused.
    #[error(transparent)]
    Capture(#[from] CaptureError),
    /// Original completed evidence failed authentication or exact source comparison.
    #[error("capture source completion verification refused")]
    Completion(Box<CaptureCompletionError<E>>),
    /// Payload bounds, bytes or held filesystem admission refused.
    #[error(transparent)]
    Store(#[from] StoreError),
}

impl BackupStore {
    /// Authenticate the selected completed receipt and hold its source bytes.
    ///
    /// The operation must belong to this same held store. The exact receipt binds
    /// network, canister, release, module, snapshot and manifest identity. Payload
    /// admission enforces the caller's total byte limit before certificate replay.
    /// The returned handle borrows the original operation so its lock cannot be
    /// dropped or its workflow advanced while source reads are in use.
    ///
    /// No files are written/synchronized, no signing or HTTP occurs, and no restore
    /// authority is granted. A future restore must acquire target custody before
    /// source operation custody and retain its own intent/reference before effects.
    ///
    /// # Errors
    /// Refuses mixed stores, wrong receipts, incomplete/changed original evidence,
    /// excessive payloads and unsafe/changed source files. All files remain intact.
    pub async fn open_capture_source<'capture, 'store, P: CaptureProvider>(
        &'store self,
        operation: &'capture mut CaptureOperation,
        expected: &BackupSummary,
        maximum_payload_bytes: NonZeroU64,
        transport: &StatusHttpTransport,
        provider: &mut P,
    ) -> Result<CaptureSource<'capture, 'store>, CaptureSourceError<P::Error>> {
        same_identity(&self.root, &operation.store)?;
        if expected.bundle_id != operation.plan.request().operation_id {
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
        let verified = operation
            .verify_completion(expected.manifest_sha256, transport, provider)
            .await
            .map_err(|e| CaptureSourceError::Completion(Box::new(e)))?;
        if verified != *expected {
            return Err(CaptureSourceError::Selection);
        }
        payload.check_files()?;
        same_identity(&self.root, &operation.store)?;
        Ok(CaptureSource {
            _capture: operation,
            payload,
        })
    }
}

impl CaptureSource<'_, '_> {
    pub(in crate::store) fn original_plan(&self) -> &super::CapturePlan {
        &self._capture.plan
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
    pub fn read_data(
        &mut self,
        request: &ReadCanisterSnapshotDataArgs,
    ) -> Result<Vec<u8>, PayloadReadError> {
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
