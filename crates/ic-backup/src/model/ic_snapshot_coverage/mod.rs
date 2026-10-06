//! Incremental coverage of admitted snapshot replies, without IO or durable progress.

use super::{ic_snapshot_data::IcSnapshotDataReply, ic_snapshot_metadata::IcSnapshotMetadataReply};
use ic_management_canister_types::SnapshotDataKind;
use std::fmt;
use thiserror::Error;

/// Ephemeral coverage of exact retained metadata by already decoded data replies.
///
/// Each region advances contiguously from zero; regions may be interleaved and
/// chunk-store replies may arrive in any order. Retained state is three nat64
/// cursors and at most 1,024 chunk-presence bits, regardless of snapshot size.
/// Admission retains no data bytes and performs no IO or provider calls. It does
/// not establish authenticated origin, byte custody, durable storage, accounting
/// or permission to repeat reads. Reconstructing this value starts empty; it is
/// never a resume journal or a substitute for one.
pub struct IcSnapshotDataCoverage<'metadata> {
    metadata: &'metadata IcSnapshotMetadataReply<'metadata>,
    regions: [u64; 3],
    chunks: Vec<bool>,
}

impl<'metadata> IcSnapshotDataCoverage<'metadata> {
    /// Start empty coverage under one exact original metadata evidence owner.
    #[must_use]
    pub fn new(metadata: &'metadata IcSnapshotMetadataReply<'metadata>) -> Self {
        Self {
            metadata,
            regions: [0; 3],
            chunks: vec![false; metadata.metadata().wasm_chunk_store.len()],
        }
    }

    /// Admit one validated reply, advancing only its region or exact chunk bit.
    ///
    /// Reply bytes may be dropped afterwards. That permits bounded streaming,
    /// but coverage alone therefore makes no retained-byte or transfer attestation.
    /// The order required within a region is a local admission order, not a grant
    /// of remote dispatch order or another accounted read.
    ///
    /// # Errors
    /// Rejects different metadata request/raw-reply evidence, gaps, overlaps,
    /// duplicate ranges or duplicate chunk identities. Every error preserves
    /// the previous coverage unchanged.
    pub fn admit(
        &mut self,
        reply: &IcSnapshotDataReply<'_, '_>,
    ) -> Result<(), IcSnapshotDataCoverageError> {
        if reply.request().metadata().digest() != self.metadata.digest() {
            return Err(IcSnapshotDataCoverageError::MetadataMismatch);
        }
        let (index, offset, size) = match reply.request().kind() {
            SnapshotDataKind::WasmModule { offset, size } => (0, *offset, *size),
            SnapshotDataKind::WasmMemory { offset, size } => (1, *offset, *size),
            SnapshotDataKind::StableMemory { offset, size } => (2, *offset, *size),
            SnapshotDataKind::WasmChunk { hash } => {
                let index = self
                    .metadata
                    .metadata()
                    .wasm_chunk_store
                    .iter()
                    .position(|chunk| chunk.hash == *hash)
                    .ok_or(IcSnapshotDataCoverageError::MetadataMismatch)?;
                if self.chunks[index] {
                    return Err(IcSnapshotDataCoverageError::DuplicateChunk);
                }
                self.chunks[index] = true;
                return Ok(());
            }
        };
        if offset != self.regions[index] {
            return Err(IcSnapshotDataCoverageError::NoncontiguousRange);
        }
        // Request admission already checked the exact metadata region and nat64
        // addition. Keep this transition fallible without a second range policy.
        let end = offset
            .checked_add(size)
            .ok_or(IcSnapshotDataCoverageError::NoncontiguousRange)?;
        self.regions[index] = end;
        Ok(())
    }

    /// Read exact original metadata, including non-data globals and optional fields.
    #[must_use]
    pub const fn metadata(&self) -> &'metadata IcSnapshotMetadataReply<'metadata> {
        self.metadata
    }

    /// Read admitted module, heap and stable byte counts in that order.
    #[must_use]
    pub const fn covered_region_bytes(&self) -> [u64; 3] {
        self.regions
    }

    /// Read the count of distinct admitted chunk-store replies, including empty chunks.
    #[must_use]
    pub fn covered_chunks(&self) -> usize {
        self.chunks.iter().filter(|covered| **covered).count()
    }

    /// Project complete declared data coverage, without an effect or storage permit.
    ///
    /// Every region must reach its exact declared size and every known chunk must
    /// have an admitted reply. Empty regions require no read; empty stored chunks
    /// still require their hash-checked reply. There is no combined-size sum that
    /// could overflow when independent regions use the full nat64 domain.
    #[must_use]
    pub fn complete(&self) -> Option<IcSnapshotDataCoverageView<'_, 'metadata>> {
        let values = self.metadata.metadata();
        let sizes = [
            values.wasm_module_size,
            values.wasm_memory_size,
            values.stable_memory_size,
        ];
        (self.regions == sizes && self.chunks.iter().all(|covered| *covered))
            .then_some(IcSnapshotDataCoverageView { coverage: self })
    }
}

impl fmt::Debug for IcSnapshotDataCoverage<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IcSnapshotDataCoverage")
            .field("covered_region_bytes", &self.regions)
            .field("covered_chunks", &self.covered_chunks())
            .finish_non_exhaustive()
    }
}

/// Read-only complete coverage of locally admitted replies, borrowing its owner.
///
/// This view authenticates no snapshot, retains no bytes and grants no durable
/// transfer, journal completion, upload/load/start, terminal or release authority.
#[derive(Debug)]
pub struct IcSnapshotDataCoverageView<'coverage, 'metadata> {
    coverage: &'coverage IcSnapshotDataCoverage<'metadata>,
}

impl<'metadata> IcSnapshotDataCoverageView<'_, 'metadata> {
    /// Read the exact original metadata whose declared data has been covered.
    #[must_use]
    pub const fn metadata(&self) -> &'metadata IcSnapshotMetadataReply<'metadata> {
        self.coverage.metadata()
    }
}

/// Typed coverage rejection without payloads, identifiers or raw decoder details.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum IcSnapshotDataCoverageError {
    /// The request or raw metadata evidence differs from the original owner.
    #[error("snapshot data coverage metadata mismatch")]
    MetadataMismatch,
    /// The next range leaves a gap, overlaps or duplicates admitted bytes.
    #[error("snapshot data coverage requires the next contiguous range")]
    NoncontiguousRange,
    /// The exact stored chunk already has an admitted reply.
    #[error("snapshot data coverage already contains this chunk")]
    DuplicateChunk,
}

#[cfg(test)]
mod tests;
