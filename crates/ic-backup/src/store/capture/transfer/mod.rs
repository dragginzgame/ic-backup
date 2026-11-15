//! Pure bounded transfer planning over Platform metadata; no dispatch authority.

mod binding;
pub(super) use binding::TransferBindingRecord;
pub use binding::{CaptureTransferBindError, CaptureTransferView};
mod data;
mod metadata;
mod readback;
pub use data::{DataReadError, TransferDataError};
pub use metadata::TransferMetadataError;
pub use readback::{ReadbackDataError, ReadbackMetadataError, UploadedSnapshotReadback};

use super::{CaptureExecution, CapturePlan};
use ic_platform::management_canister_types::{
    ReadCanisterSnapshotDataArgs, ReadCanisterSnapshotMetadataResult, SnapshotDataKind,
    SnapshotMetadataGlobal, SnapshotSource,
};
use std::fmt;
use thiserror::Error;

const MAX_CHUNKS: usize = 1024;
const MAX_GLOBALS: usize = 4096;

/// Borrowed deterministic capture-data schedule, not authenticated snapshot evidence.
///
/// Metadata and the original plan remain immutably borrowed. No bytes are read or
/// written, attempts spent, providers called or external state authenticated.
/// Provider overhead, raw metadata decoding and durable schedule binding remain separate.
/// [`Self::decode_data`] validates individual data replies without recording progress.
pub struct CaptureTransfer<'a> {
    plan: &'a CapturePlan,
    metadata: &'a ReadCanisterSnapshotMetadataResult,
    snapshot_id: &'a [u8],
    chunks: Vec<[u8; 32]>,
    region_calls: [u64; 3],
    data_calls: u32,
    capture_calls: u32,
    minimum_network_requests: u32,
    payload_ceiling: u64,
}

impl CapturePlan {
    /// Plan module, heap, stable-memory and whole-chunk reads in fixed order.
    ///
    /// Reads use ascending extents followed by chunks sorted by hash. Unknown
    /// chunk lengths each reserve the original extent ceiling. Zero-length heap
    /// and stable memory emit no requests; a named empty chunk still emits one.
    /// The module must be nonempty, matching the local bundle profile.
    ///
    /// This accepts already-decoded Platform metadata. The caller still needs
    /// bounded raw decoding (available through [`Self::decode_metadata`]),
    /// authenticated target/snapshot association, original
    /// evidence, application admission and route-specific budgets before dispatch.
    ///
    /// ```
    /// use ic_backup::store::{CapturePlan, TransferError};
    /// use ic_platform::management_canister_types::ReadCanisterSnapshotMetadataResult;
    ///
    /// fn inspect_schedule(
    ///     plan: &CapturePlan,
    ///     snapshot: &[u8],
    ///     metadata: &ReadCanisterSnapshotMetadataResult,
    /// ) -> Result<(), TransferError> {
    ///     let transfer = plan.plan_transfer(snapshot, metadata)?;
    ///     assert_eq!(transfer.requests().len(), transfer.data_calls() as usize);
    ///     // These DTOs still require retained evidence and admitted dispatch authority.
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    /// Refuses unsupported metadata, malformed/duplicate chunks, overflow and
    /// schedules exceeding original payload or minimum request/signing allowances.
    pub fn plan_transfer<'a>(
        &'a self,
        snapshot_id: &'a [u8],
        metadata: &'a ReadCanisterSnapshotMetadataResult,
    ) -> Result<CaptureTransfer<'a>, TransferError> {
        if !(1..=256).contains(&snapshot_id.len()) {
            return Err(TransferError::SnapshotId);
        }
        admit_metadata(metadata)?;
        let limits = self.request().limits;
        let extent = u64::from(limits.extent_bytes);
        let sizes = [
            metadata.wasm_module_size,
            metadata.wasm_memory_size,
            metadata.stable_memory_size,
        ];
        let region_bytes = sizes
            .into_iter()
            .try_fold(0_u64, |total, size| total.checked_add(size))
            .ok_or(TransferError::Overflow)?;
        let chunk_bytes = (metadata.wasm_chunk_store.len() as u64)
            .checked_mul(extent)
            .ok_or(TransferError::Overflow)?;
        let payload_ceiling = region_bytes
            .checked_add(chunk_bytes)
            .ok_or(TransferError::Overflow)?;
        if payload_ceiling > limits.payload_bytes {
            return Err(TransferError::PayloadBudget {
                required: payload_ceiling,
                original: limits.payload_bytes,
            });
        }
        let region_calls = sizes.map(|size| size.div_ceil(extent));
        let data_calls = region_calls
            .into_iter()
            .try_fold(metadata.wasm_chunk_store.len() as u64, u64::checked_add)
            .ok_or(TransferError::Overflow)?;
        // Initial status, list, take, metadata and final status surround data reads.
        let capture_calls = data_calls.checked_add(5).ok_or(TransferError::Overflow)?;
        let hops = match self.request().execution {
            CaptureExecution::Direct { .. } => 1,
            CaptureExecution::Canic { .. } => 2,
        };
        let minimum_network_requests = capture_calls
            .checked_mul(hops)
            .ok_or(TransferError::Overflow)?;
        if minimum_network_requests > u64::from(limits.network_requests) {
            return Err(TransferError::NetworkBudget {
                required: minimum_network_requests,
                original: limits.network_requests,
            });
        }
        if capture_calls > u64::from(limits.signing_requests) {
            return Err(TransferError::SigningBudget {
                required: capture_calls,
                original: limits.signing_requests,
            });
        }
        // Bounded chunk identities are the only collection retained by the schedule.
        let mut chunks = Vec::with_capacity(metadata.wasm_chunk_store.len());
        for chunk in &metadata.wasm_chunk_store {
            chunks.push(
                <[u8; 32]>::try_from(chunk.hash.as_slice())
                    .map_err(|_| TransferError::ChunkHash)?,
            );
        }
        chunks.sort_unstable();
        if chunks.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(TransferError::ChunkHash);
        }
        Ok(CaptureTransfer {
            plan: self,
            metadata,
            snapshot_id,
            chunks,
            region_calls,
            data_calls: u32::try_from(data_calls).map_err(|_| TransferError::Overflow)?,
            capture_calls: u32::try_from(capture_calls).map_err(|_| TransferError::Overflow)?,
            minimum_network_requests: u32::try_from(minimum_network_requests)
                .map_err(|_| TransferError::Overflow)?,
            payload_ceiling,
        })
    }
}

impl CaptureTransfer<'_> {
    /// Number of data reads, including every named chunk regardless of its length.
    #[must_use]
    pub const fn data_calls(&self) -> u32 {
        self.data_calls
    }

    /// Whole-capture management calls: data reads plus the five fixed lifecycle reads/effects.
    #[must_use]
    pub const fn capture_calls(&self) -> u32 {
        self.capture_calls
    }

    /// Minimum network requests: one hop per direct call, two per Canic call.
    ///
    /// Authentication, status polling and executor receipt lookup may require more.
    /// This is a lower bound, not admission of a concrete provider or its costs.
    #[must_use]
    pub const fn minimum_network_requests(&self) -> u32 {
        self.minimum_network_requests
    }

    /// Exact region bytes plus one full extent allowance per unknown-length chunk.
    #[must_use]
    pub const fn payload_ceiling(&self) -> u64 {
        self.payload_ceiling
    }

    /// The unchanged borrowed metadata, retaining wire order, optional values and float bits.
    #[must_use]
    pub const fn metadata(&self) -> &ReadCanisterSnapshotMetadataResult {
        self.metadata
    }

    /// Generate Platform request DTOs lazily in deterministic order.
    ///
    /// Every request uses the original plan's target and exact supplied snapshot ID.
    /// Producing a DTO grants no signature, reservation or dispatch permission.
    pub fn requests(&self) -> impl ExactSizeIterator<Item = ReadCanisterSnapshotDataArgs> + '_ {
        (0..self.data_calls).map(|ordinal| ReadCanisterSnapshotDataArgs {
            canister_id: self.plan.request().target.canister,
            snapshot_id: self.snapshot_id.to_vec(),
            kind: self.kind(u64::from(ordinal)),
        })
    }

    fn kind(&self, ordinal: u64) -> SnapshotDataKind {
        data_kind(
            [
                self.metadata.wasm_module_size,
                self.metadata.wasm_memory_size,
                self.metadata.stable_memory_size,
            ],
            self.region_calls,
            u64::from(self.plan.request().limits.extent_bytes),
            &self.chunks,
            ordinal,
        )
    }
}

// Both capture and restore admit counts/lengths before using this fixed ordering.
pub(in crate::store::capture) fn data_kind(
    sizes: [u64; 3],
    region_calls: [u64; 3],
    extent: u64,
    chunks: &[[u8; 32]],
    mut ordinal: u64,
) -> SnapshotDataKind {
    for (region, count) in region_calls.into_iter().enumerate() {
        if ordinal < count {
            let offset = ordinal * extent;
            let size = extent.min(sizes[region] - offset);
            return match region {
                0 => SnapshotDataKind::WasmModule { offset, size },
                1 => SnapshotDataKind::WasmMemory { offset, size },
                _ => SnapshotDataKind::StableMemory { offset, size },
            };
        }
        ordinal -= count;
    }
    SnapshotDataKind::WasmChunk {
        hash: chunks[ordinal as usize].to_vec(),
    }
}

fn admit_metadata(metadata: &ReadCanisterSnapshotMetadataResult) -> Result<(), TransferError> {
    if metadata.wasm_chunk_store.len() > MAX_CHUNKS
        || metadata.globals.len() > MAX_GLOBALS
        || metadata.certified_data.len() > 32
    {
        return Err(TransferError::MetadataBounds);
    }
    if !matches!(metadata.source, Some(SnapshotSource::TakenFromCanister(_)))
        || metadata.wasm_module_size == 0
        || metadata.global_timer.is_none()
        || metadata.on_low_wasm_memory_hook_status.is_none()
        || metadata.globals.iter().any(Option::is_none)
    {
        return Err(TransferError::UnsupportedProfile);
    }
    for global in metadata.globals.iter().flatten() {
        if let SnapshotMetadataGlobal::V128(value) = global
            && value.0.bits() > 128
        {
            return Err(TransferError::GlobalRange);
        }
    }
    Ok(())
}

impl fmt::Debug for CaptureTransfer<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CaptureTransfer")
            .field("data_calls", &self.data_calls)
            .field("capture_calls", &self.capture_calls)
            .field("payload_ceiling", &self.payload_ceiling)
            .finish_non_exhaustive()
    }
}

/// Pure transfer-planning refusal; no error consumes or replenishes an attempt.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum TransferError {
    /// Snapshot identity must contain 1 through 256 bytes.
    #[error("invalid capture snapshot identity")]
    SnapshotId,
    /// A collection or certified-data value exceeded the product profile.
    #[error("snapshot metadata exceeds supported bounds")]
    MetadataBounds,
    /// Valid external metadata lacks required round-trip information or a nonempty module.
    #[error("snapshot is outside the supported capture profile")]
    UnsupportedProfile,
    /// A v128 global is not representable in 128 bits.
    #[error("snapshot vector global exceeds 128 bits")]
    GlobalRange,
    /// A chunk identity is malformed or duplicated.
    #[error("snapshot chunks must have unique 32-byte hashes")]
    ChunkHash,
    /// Declared sizes/counts cannot be summed without overflow.
    #[error("snapshot transfer arithmetic overflow")]
    Overflow,
    /// Conservative payload bytes exceed the original plan.
    #[error("transfer requires {required} payload bytes, exceeding original {original}")]
    PayloadBudget {
        /// Required region bytes plus conservative whole-chunk allowances.
        required: u64,
        /// Original payload allowance.
        original: u64,
    },
    /// Even the route's minimum network cost exceeds the original plan.
    #[error(
        "transfer requires at least {required} network requests, exceeding original {original}"
    )]
    NetworkBudget {
        /// Minimum whole-capture network request count.
        required: u64,
        /// Original total network allowance, including observations.
        original: u32,
    },
    /// Preparing the fixed capture calls exceeds the original signing allowance.
    #[error(
        "transfer requires at least {required} signing requests, exceeding original {original}"
    )]
    SigningBudget {
        /// Minimum signing preparations for the whole capture sequence.
        required: u64,
        /// Original signing allowance.
        original: u32,
    },
}

#[cfg(test)]
mod tests;
