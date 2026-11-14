//! Ordered byte comparison against caller-held source slices, with bounded scratch space.
use super::{CaptureTransfer, ReadbackMetadataError};
use crate::store::TransferDataError;
use ic_platform::management_canister_types::{
    ReadCanisterSnapshotDataArgs, ReadCanisterSnapshotMetadataResult,
};
use std::fmt;
use thiserror::Error;

/// Ephemeral comparison coverage for one uploaded snapshot, not a restore receipt.
///
/// Every scheduled extent and named chunk must match in order. Only decoded
/// metadata and a counter are retained; payload comparison uses at most one
/// bounded decoded reply. Dropping this value discards local comparison progress.
/// It never changes durable spending or authorizes resending a request.
///
/// Callers must authenticate all destination replies against the exact requests,
/// retain raw evidence and supply source slices from the held verified bundle at
/// each original schedule position. This object cannot establish that custody,
/// exclude external snapshot changes or authenticate caller-supplied bytes.
pub struct UploadedSnapshotReadback<'transfer, 'source> {
    transfer: &'transfer CaptureTransfer<'source>,
    destination_id: &'transfer [u8],
    metadata: ReadCanisterSnapshotMetadataResult,
    compared: u32,
}

impl<'source> CaptureTransfer<'source> {
    /// Begin ordered payload comparison after validating uploaded metadata.
    ///
    /// The destination ID must be 1..=256 bytes and differ from the source ID.
    /// This does not prove allocation or bind the metadata reply to that ID;
    /// authenticated association remains the caller's responsibility. The original
    /// capture budget is not restore authority; restore needs independent intent,
    /// accounting, source custody and application admission before any effect.
    ///
    /// # Errors
    /// Refuses invalid/reused destination IDs or any uploaded-metadata mismatch.
    pub fn begin_readback<'transfer>(
        &'transfer self,
        destination_id: &'transfer [u8],
        metadata_reply: &[u8],
    ) -> Result<UploadedSnapshotReadback<'transfer, 'source>, ReadbackDataError> {
        if !(1..=256).contains(&destination_id.len()) || destination_id == self.snapshot_id {
            return Err(ReadbackDataError::Destination);
        }
        Ok(UploadedSnapshotReadback {
            transfer: self,
            destination_id,
            metadata: self.decode_uploaded_metadata(metadata_reply)?,
            compared: 0,
        })
    }
}

impl UploadedSnapshotReadback<'_, '_> {
    /// Next zero-based ordinal and destination request, or `None` after full comparison.
    ///
    /// Uses the source target and exact source range/hash with the destination ID.
    /// Repeated calls return the same request until comparison succeeds. Producing
    /// this DTO grants no signing, submission, retry or reconciliation authority.
    #[must_use]
    pub fn next_request(&self) -> Option<(u32, ReadCanisterSnapshotDataArgs)> {
        (self.compared < self.transfer.data_calls).then(|| {
            (
                self.compared,
                ReadCanisterSnapshotDataArgs {
                    canister_id: self.transfer.plan.request().target.canister,
                    snapshot_id: self.destination_id.to_vec(),
                    kind: self.transfer.kind(u64::from(self.compared)),
                },
            )
        })
    }

    /// Number of consecutive data positions compared successfully; not a traffic counter.
    #[must_use]
    pub const fn compared_data_calls(&self) -> u32 {
        self.compared
    }

    /// Compare one trusted source slice with its bounded destination reply.
    ///
    /// Source shape/hash is checked before reply decoding; the existing decoder
    /// checks destination shape/hash. Region bytes then compare exactly, including
    /// same-length corruption. Neither slice nor reply is retained by this object.
    /// Refusal leaves coverage unchanged and permits correcting local inputs;
    /// it does not permit a new remote attempt.
    ///
    /// # Errors
    /// Refuses skipped/repeated/out-of-range ordinals, invalid source bytes,
    /// malformed/excessive destination replies and unequal byte content.
    pub fn compare_data(
        &mut self,
        ordinal: u32,
        source: &[u8],
        reply: &[u8],
    ) -> Result<(), ReadbackDataError> {
        if ordinal != self.compared || ordinal >= self.transfer.data_calls {
            return Err(ReadbackDataError::Ordinal);
        }
        self.transfer
            .validate_data_bytes(ordinal, source)
            .map_err(ReadbackDataError::Source)?;
        let destination = self
            .transfer
            .decode_data(ordinal, reply)
            .map_err(ReadbackDataError::Reply)?;
        if source != destination {
            return Err(ReadbackDataError::Mismatch);
        }
        self.compared += 1;
        Ok(())
    }

    /// Return destination provenance only after metadata and every data position match.
    ///
    /// Success establishes equality to the supplied source slices only. It is not
    /// authenticated snapshot evidence, a durable completion or permission to load.
    ///
    /// # Errors
    /// Refuses incomplete coverage, including an unread named empty chunk.
    pub fn finish(self) -> Result<ReadCanisterSnapshotMetadataResult, ReadbackDataError> {
        if self.compared != self.transfer.data_calls {
            return Err(ReadbackDataError::Incomplete);
        }
        Ok(self.metadata)
    }
}

impl fmt::Debug for UploadedSnapshotReadback<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UploadedSnapshotReadback")
            .field("compared_data_calls", &self.compared)
            .field("data_calls", &self.transfer.data_calls)
            .finish_non_exhaustive()
    }
}

/// Typed local comparison refusal, without raw bytes, remote effects or spending changes.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum ReadbackDataError {
    /// Destination ID is empty, excessive or identical to the source ID.
    #[error("readback requires a bounded destination ID distinct from the source")]
    Destination,
    /// Uploaded metadata failed the shared bounded comparison.
    #[error(transparent)]
    Metadata(#[from] ReadbackMetadataError),
    /// Ordinal is not the next original scheduled position.
    #[error("readback ordinal is not the next scheduled data position")]
    Ordinal,
    /// Source slice violates its original scheduled length, chunk cap or hash.
    #[error("readback source bytes refused: {0}")]
    Source(#[source] TransferDataError),
    /// Destination raw reply violates the shared bounded data contract.
    #[error("readback destination reply refused: {0}")]
    Reply(#[source] TransferDataError),
    /// Validly shaped bytes differ from the supplied source slice.
    #[error("readback bytes differ from the source")]
    Mismatch,
    /// Some scheduled data positions have not been compared successfully.
    #[error("readback data coverage is incomplete")]
    Incomplete,
}

#[cfg(test)]
mod tests;
