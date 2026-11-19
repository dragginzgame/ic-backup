//! Pure uploaded-state comparison; no authentication, persistence or load authority.

#[cfg(test)]
mod tests;

use super::{CaptureTransfer, TransferMetadataError, metadata};
use ic_platform_protocol::management_snapshots::{
    Global, LowMemoryHook, SnapshotMetadata, SnapshotSource, Timer,
};
use thiserror::Error;

impl CaptureTransfer<'_> {
    /// Decode uploaded-snapshot metadata and compare its restorable state with the source.
    ///
    /// Uses the same bounded decoder as capture metadata. Globals compare exact
    /// variants and float bits, including NaNs and signed zero. Chunk hashes must
    /// form the same unique set; original wire order stays in the returned value.
    /// Timers and hooks must be present and equal. No absent value is defaulted.
    ///
    /// Destination origin must be metadata upload. Its timestamp and canister
    /// version are returned as new provenance, not compared with source values.
    /// The caller must authenticate and retain the reply's original network,
    /// canister, destination snapshot and request association; the DTO has no ID.
    ///
    /// This checks metadata only. It grants no restore/load authority and proves
    /// neither complete payload readback nor application safety. Source bundle
    /// verification, independent restore intent/accounting and byte comparison
    /// remain required. No provider, journal or network is accessed.
    ///
    ///
    /// # Errors
    /// Refuses malformed/excessive input, wrong destination origin or any mismatch
    /// in region sizes, globals, certified data, timer, hook or chunk identities.
    pub fn decode_uploaded_metadata(
        &self,
        reply: &[u8],
    ) -> Result<SnapshotMetadata, ReadbackMetadataError> {
        let destination = metadata::decode(reply)?;
        if !matches!(destination.source, Some(SnapshotSource::MetadataUpload(_))) {
            return Err(ReadbackMetadataError::Origin);
        }
        let source = self.metadata;
        if destination.wasm_module_size != source.wasm_module_size
            || destination.wasm_memory_size != source.wasm_memory_size
            || destination.stable_memory_size != source.stable_memory_size
        {
            return Err(ReadbackMetadataError::RegionSizes);
        }
        if destination.globals.len() != source.globals.len()
            || !source
                .globals
                .iter()
                .zip(&destination.globals)
                .all(|pair| matches!(pair, (Some(left), Some(right)) if equal_global(left, right)))
        {
            return Err(ReadbackMetadataError::Globals);
        }
        if destination.certified_data != source.certified_data {
            return Err(ReadbackMetadataError::CertifiedData);
        }
        let equal_timer = match (&source.global_timer, &destination.global_timer) {
            (Some(Timer::Inactive), Some(Timer::Inactive)) => true,
            (Some(Timer::Active(left)), Some(Timer::Active(right))) => left == right,
            _ => false,
        };
        if !equal_timer {
            return Err(ReadbackMetadataError::GlobalTimer);
        }
        use LowMemoryHook::{ConditionNotSatisfied, Executed, Ready};
        if !matches!(
            (
                &source.on_low_wasm_memory_hook_status,
                &destination.on_low_wasm_memory_hook_status
            ),
            (Some(ConditionNotSatisfied), Some(ConditionNotSatisfied))
                | (Some(Ready), Some(Ready))
                | (Some(Executed), Some(Executed))
        ) {
            return Err(ReadbackMetadataError::LowMemoryHook);
        }
        if destination.wasm_chunk_store.len() != self.chunks.len() {
            return Err(ReadbackMetadataError::ChunkStore);
        }
        let mut chunks = Vec::with_capacity(self.chunks.len());
        for chunk in &destination.wasm_chunk_store {
            chunks.push(
                <[u8; 32]>::try_from(chunk.hash.as_slice())
                    .map_err(|_| ReadbackMetadataError::ChunkStore)?,
            );
        }
        chunks.sort_unstable();
        // The source index is already unique; equality also rejects destination duplicates.
        if chunks != self.chunks {
            return Err(ReadbackMetadataError::ChunkStore);
        }

        Ok(destination)
    }
}

fn equal_global(left: &Global, right: &Global) -> bool {
    use Global::{F32, F64, I32, I64, V128};

    match (left, right) {
        (I32(left), I32(right)) => left == right,
        (I64(left), I64(right)) => left == right,
        (F32(left), F32(right)) => left == right,
        (F64(left), F64(right)) => left == right,
        (V128(left), V128(right)) => left == right,
        _ => false,
    }
}

/// Uploaded metadata refusal; diagnostics never include payload or global values.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum ReadbackMetadataError {
    /// The shared bounded metadata decoder refused the input.
    #[error(transparent)]
    Metadata(#[from] TransferMetadataError),
    /// Destination metadata did not identify an uploaded snapshot.
    #[error("readback metadata must identify an uploaded snapshot")]
    Origin,
    /// One of the three memory/module lengths changed.
    #[error("readback region sizes differ from the source")]
    RegionSizes,
    /// Ordered global variants, presence or exact values changed.
    #[error("readback globals differ from the source")]
    Globals,
    /// Certified data changed.
    #[error("readback certified data differs from the source")]
    CertifiedData,
    /// The global timer is absent or differs from the source.
    #[error("readback global timer differs from the source")]
    GlobalTimer,
    /// The low-memory hook is absent or differs from the source.
    #[error("readback low-memory hook differs from the source")]
    LowMemoryHook,
    /// Chunk hashes are malformed, duplicated, missing or unexpected.
    #[error("readback chunk set differs from the source")]
    ChunkStore,
}
