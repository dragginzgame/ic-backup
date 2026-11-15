//! Explicit local schedule binding; retained metadata remains the source of truth.
use super::{CaptureTransfer, TransferError};
use crate::store::capture::{
    CaptureError, CaptureMetadataAdvanceError, CaptureMetadataPhase, CaptureOperation,
    CaptureProvider, StatusHttpTransport,
    ledger::{EventRecord, EvidenceRecord},
};
use ic_host_artifacts::artifact::Sha256Digest;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use thiserror::Error;

/// Inspection of a retained schedule; not authentication or permission to dispatch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureTransferView {
    /// Version-one schedule commitment to original intent, evidence and geometry.
    pub commitment: [u8; 32],
    /// Exact number of scheduled data requests.
    pub data_calls: u32,
    /// Region bytes plus a full extent allowance per unknown-length chunk.
    pub payload_ceiling: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::store::capture) struct TransferBindingRecord {
    pub version: u32,
    pub commitment: [u8; 32],
    pub data_calls: u32,
    pub payload_ceiling: u64,
}
impl TransferBindingRecord {
    pub fn view(&self) -> CaptureTransferView {
        CaptureTransferView {
            commitment: self.commitment,
            data_calls: self.data_calls,
            payload_ceiling: self.payload_ceiling,
        }
    }
}
impl CaptureTransfer<'_> {
    // Fixed geometry encodes the same lazy schedule without iterating potentially
    // billions of extents. Only the already-bounded chunk list varies in length.
    pub(in crate::store::capture) fn binding(
        &self,
        request: &EvidenceRecord,
        result: &EvidenceRecord,
    ) -> TransferBindingRecord {
        let mut bytes = b"ic-backup/transfer-schedule/1\0".to_vec();
        bytes.extend_from_slice(Sha256Digest::compute(self.plan.encoded()).as_bytes());
        bytes.extend_from_slice(&request.sha256);
        bytes.extend_from_slice(&result.sha256);
        bytes.extend_from_slice(&(self.snapshot_id.len() as u32).to_le_bytes());
        bytes.extend_from_slice(self.snapshot_id);
        bytes.extend_from_slice(&self.plan.request().limits.extent_bytes.to_le_bytes());
        for size in [
            self.metadata.wasm_module_size,
            self.metadata.wasm_memory_size,
            self.metadata.stable_memory_size,
        ] {
            bytes.extend_from_slice(&size.to_le_bytes());
        }
        bytes.extend_from_slice(&(self.chunks.len() as u32).to_le_bytes());
        for chunk in &self.chunks {
            bytes.extend_from_slice(chunk);
        }
        bytes.extend_from_slice(&self.data_calls.to_le_bytes());
        bytes.extend_from_slice(&self.payload_ceiling.to_le_bytes());
        TransferBindingRecord {
            version: 1,
            commitment: *Sha256Digest::compute(&bytes).as_bytes(),
            data_calls: self.data_calls,
            payload_ceiling: self.payload_ceiling,
        }
    }
}

/// Local binding refuses without granting new signing or network allowance.
#[derive(Debug, Error)]
pub enum CaptureTransferBindError<P> {
    /// Local custody, journal or finite evidence allowance refused.
    #[error(transparent)]
    Capture(#[from] CaptureError),
    /// Metadata must already be settled before this local operation.
    #[error("transfer binding requires settled metadata")]
    MetadataRequired,
    /// Retained original capture or metadata authentication failed.
    #[error("retained metadata authentication refused")]
    Metadata(CaptureMetadataAdvanceError<P, Infallible>),
    /// Existing transfer profile refused.
    #[error(transparent)]
    Transfer(#[from] TransferError),
    /// Recomputed schedule differs from the retained version or commitment.
    #[error("retained transfer schedule differs")]
    Binding,
}
impl CaptureOperation {
    /// Inspect the retained schedule without authenticating or dispatching anything.
    /// # Errors
    /// Refuses invalid journals or changed evidence/custody.
    pub fn inspect_transfer(&self) -> Result<Option<CaptureTransferView>, CaptureError> {
        Ok(self
            .check()?
            .transfer
            .as_ref()
            .map(TransferBindingRecord::view))
    }

    /// Bind the schedule derived from already-settled, authenticated metadata once.
    ///
    /// Uses shared retained-evidence authentication; neither live read permission nor
    /// metadata signing/submission is invoked. The original provider must support
    /// effect-free historical capture authentication. A first call appends one
    /// local journal frame; identical subsequent calls authenticate without writes.
    /// No data request is reserved or dispatched and no backup is completed.
    /// ```no_run
    /// use ic_backup::store::{CaptureOperation, CaptureProvider, StatusHttpTransport,
    ///     CaptureTransferView, CaptureTransferBindError};
    /// async fn bind<P: CaptureProvider>(op: &mut CaptureOperation,
    ///     transport: &StatusHttpTransport, verifier: &mut P)
    ///     -> Result<CaptureTransferView, CaptureTransferBindError<P::Error>> {
    ///     op.bind_transfer(transport, verifier).await
    /// }
    /// ```
    /// # Errors
    /// Refuses unsettled metadata, authentication failures, changed bindings,
    /// exhausted local evidence allowance and uncertain writes. Partial tails
    /// refuse reopening; a complete uncertain binding is retained on reopen.
    pub async fn bind_transfer<P: CaptureProvider>(
        &mut self,
        transport: &StatusHttpTransport,
        original: &mut P,
    ) -> Result<CaptureTransferView, CaptureTransferBindError<P::Error>> {
        let state = self.check()?;
        let m = state
            .metadata
            .as_ref()
            .filter(|m| m.phase == CaptureMetadataPhase::Settled)
            .ok_or(CaptureTransferBindError::MetadataRequired)?;
        let snapshot = self
            .metadata_snapshot(&m.admission, transport, original, &state)
            .await
            .map_err(CaptureTransferBindError::Metadata)?;
        let metadata = self
            .retained_metadata(&state, &snapshot, transport)
            .map_err(CaptureTransferBindError::Metadata)?;
        let schedule = self.plan.plan_transfer(&m.snapshot_id, &metadata)?;
        let record = schedule.binding(
            m.request.as_ref().ok_or(CaptureError::Incomplete)?,
            m.result.as_ref().ok_or(CaptureError::Incomplete)?,
        );
        if let Some(retained) = state.transfer {
            if retained != record {
                return Err(CaptureTransferBindError::Binding);
            }
        } else {
            self.commit(
                &EventRecord::TransferBound {
                    binding: record.clone(),
                },
                None,
            )?;
        }
        Ok(record.view())
    }
}

#[cfg(test)]
mod tests;
