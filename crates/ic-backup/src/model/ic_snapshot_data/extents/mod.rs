//! One bounded declaration owner for complete ordered snapshot extents.

use super::{IcSnapshotMetadataReply, MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES, SnapshotRegion};
use ic_platform::management_canister_types::SnapshotDataKind;

use crate::model::data_stage_plan::SnapshotDataPlanningError;

pub(crate) fn planned_extents(
    metadata: &IcSnapshotMetadataReply<'_>,
    chunk_bytes: u64,
    original: u32,
) -> Result<Vec<SnapshotDataKind>, SnapshotDataPlanningError> {
    if chunk_bytes == 0 || chunk_bytes > MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES as u64 {
        return Err(SnapshotDataPlanningError::InvalidChunkSize);
    }
    let values = metadata.metadata();
    let sizes = metadata.region_sizes();
    let mut count = u64::try_from(values.wasm_chunk_store.len())
        .map_err(|_| SnapshotDataPlanningError::CountOverflow)?;
    for size in sizes {
        let region = size / chunk_bytes + u64::from(size % chunk_bytes != 0);
        count = count
            .checked_add(region)
            .ok_or(SnapshotDataPlanningError::CountOverflow)?;
    }
    if count > u64::from(original) {
        return Err(SnapshotDataPlanningError::InsufficientAllowance {
            required: count,
            original,
        });
    }
    let capacity = usize::try_from(count).map_err(|_| SnapshotDataPlanningError::CountOverflow)?;
    let mut kinds = Vec::with_capacity(capacity);
    for (region, total) in SnapshotRegion::ALL.into_iter().zip(sizes) {
        let mut offset = 0;
        while offset < total {
            let size = (total - offset).min(chunk_bytes);
            kinds.push(region.extent(offset, size));
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
