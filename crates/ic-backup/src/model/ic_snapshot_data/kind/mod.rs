//! Borrowed region/chunk classification shared by model and filesystem projections.

use ic_platform::management_canister_types::{SnapshotDataKind, SnapshotDataOffset};

/// The fixed order used by metadata, contiguous coverage and artifact regions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SnapshotRegion {
    WasmModule,
    WasmMemory,
    StableMemory,
}

impl SnapshotRegion {
    pub(crate) const ALL: [Self; 3] = [Self::WasmModule, Self::WasmMemory, Self::StableMemory];

    pub(crate) const fn index(self) -> usize {
        match self {
            Self::WasmModule => 0,
            Self::WasmMemory => 1,
            Self::StableMemory => 2,
        }
    }

    pub(crate) const fn upload_offset(self, offset: u64) -> SnapshotDataOffset {
        match self {
            Self::WasmModule => SnapshotDataOffset::WasmModule { offset },
            Self::WasmMemory => SnapshotDataOffset::WasmMemory { offset },
            Self::StableMemory => SnapshotDataOffset::StableMemory { offset },
        }
    }

    pub(crate) const fn extent(self, offset: u64, size: u64) -> SnapshotDataKind {
        match self {
            Self::WasmModule => SnapshotDataKind::WasmModule { offset, size },
            Self::WasmMemory => SnapshotDataKind::WasmMemory { offset, size },
            Self::StableMemory => SnapshotDataKind::StableMemory { offset, size },
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum DataExtent<'a> {
    Region {
        region: SnapshotRegion,
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
                region: SnapshotRegion::WasmModule,
                offset: *offset,
                size: *size,
            },
            SnapshotDataKind::WasmMemory { offset, size } => Self::Region {
                region: SnapshotRegion::WasmMemory,
                offset: *offset,
                size: *size,
            },
            SnapshotDataKind::StableMemory { offset, size } => Self::Region {
                region: SnapshotRegion::StableMemory,
                offset: *offset,
                size: *size,
            },
            SnapshotDataKind::WasmChunk { hash } => Self::Chunk { hash },
        }
    }
}
