//! Exact v1 snapshot member names and data-kind mapping.

use super::IcSnapshotArtifactError;
use ic_host_artifacts::artifact::Sha256Digest;
use ic_management_canister_types::SnapshotDataKind;

pub(super) enum Member {
    Region {
        index: usize,
        offset: u64,
        size: u64,
    },
    Chunk {
        name: String,
    },
}

pub(super) fn select(kind: &SnapshotDataKind) -> Result<Member, IcSnapshotArtifactError> {
    Ok(match kind {
        SnapshotDataKind::WasmModule { offset, size } => Member::Region {
            index: 0,
            offset: *offset,
            size: *size,
        },
        SnapshotDataKind::WasmMemory { offset, size } => Member::Region {
            index: 1,
            offset: *offset,
            size: *size,
        },
        SnapshotDataKind::StableMemory { offset, size } => Member::Region {
            index: 2,
            offset: *offset,
            size: *size,
        },
        SnapshotDataKind::WasmChunk { hash } => Member::Chunk {
            name: chunk_name(Sha256Digest::from_bytes(
                hash.as_slice()
                    .try_into()
                    .map_err(|_| IcSnapshotArtifactError::OriginalMismatch)?,
            )),
        },
    })
}

pub(super) fn chunk_name(hash: Sha256Digest) -> String {
    format!("chunk-{hash}.bin")
}
