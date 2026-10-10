//! One bounded declaration owner for complete ordered snapshot extents.

use super::{IcSnapshotMetadataReply, MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES};
use ic_management_canister_types::SnapshotDataKind;

pub(crate) enum ExtentPlanningError {
    InvalidChunkSize,
    CountOverflow,
    InsufficientAllowance { required: u64, original: u32 },
}

pub(crate) fn planned_extents(
    metadata: &IcSnapshotMetadataReply<'_>,
    chunk_bytes: u64,
    original: u32,
) -> Result<Vec<SnapshotDataKind>, ExtentPlanningError> {
    if chunk_bytes == 0 || chunk_bytes > MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES as u64 {
        return Err(ExtentPlanningError::InvalidChunkSize);
    }
    let values = metadata.metadata();
    let sizes = [
        values.wasm_module_size,
        values.wasm_memory_size,
        values.stable_memory_size,
    ];
    let mut count = u64::try_from(values.wasm_chunk_store.len())
        .map_err(|_| ExtentPlanningError::CountOverflow)?;
    for size in sizes {
        let region = size / chunk_bytes + u64::from(size % chunk_bytes != 0);
        count = count
            .checked_add(region)
            .ok_or(ExtentPlanningError::CountOverflow)?;
    }
    if count > u64::from(original) {
        return Err(ExtentPlanningError::InsufficientAllowance {
            required: count,
            original,
        });
    }
    let capacity = usize::try_from(count).map_err(|_| ExtentPlanningError::CountOverflow)?;
    let mut kinds = Vec::with_capacity(capacity);
    for (region, total) in sizes.into_iter().enumerate() {
        let mut offset = 0;
        while offset < total {
            let size = (total - offset).min(chunk_bytes);
            kinds.push(match region {
                0 => SnapshotDataKind::WasmModule { offset, size },
                1 => SnapshotDataKind::WasmMemory { offset, size },
                _ => SnapshotDataKind::StableMemory { offset, size },
            });
            offset += size;
        }
    }
    for chunk in &values.wasm_chunk_store {
        kinds.push(SnapshotDataKind::WasmChunk {
            hash: chunk.hash.clone(),
        });
    }
    Ok(kinds)
}
