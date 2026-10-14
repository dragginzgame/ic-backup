//! Exact v1 snapshot member names and data-kind mapping.

use crate::hash::hex_bytes;
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

pub(super) fn select(kind: &SnapshotDataKind) -> Member {
    match kind {
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
            name: chunk_name(hash),
        },
    }
}

pub(super) fn chunk_name(hash: &[u8]) -> String {
    format!("chunk-{}.bin", hex_bytes(hash))
}
