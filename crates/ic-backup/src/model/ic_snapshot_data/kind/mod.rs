//! Borrowed region/chunk classification shared by model and filesystem projections.

use ic_platform::management_canister_types::SnapshotDataKind;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum DataExtent<'a> {
    Region {
        index: usize,
        offset: u64,
        size: u64,
    },
    Chunk {
        hash: &'a [u8],
    },
}

impl<'a> From<&'a SnapshotDataKind> for DataExtent<'a> {
    fn from(kind: &'a SnapshotDataKind) -> Self {
        match kind {
            SnapshotDataKind::WasmModule { offset, size } => Self::Region {
                index: 0,
                offset: *offset,
                size: *size,
            },
            SnapshotDataKind::WasmMemory { offset, size } => Self::Region {
                index: 1,
                offset: *offset,
                size: *size,
            },
            SnapshotDataKind::StableMemory { offset, size } => Self::Region {
                index: 2,
                offset: *offset,
                size: *size,
            },
            SnapshotDataKind::WasmChunk { hash } => Self::Chunk { hash },
        }
    }
}
