use crate::store::BackupSummary;
use ic_platform_protocol::management_snapshots::Extent;
use ic_platform_protocol::management_snapshots::{ReadDataArgs, ReadKind};
use std::num::NonZeroU64;
pub(in crate::store) fn extent(receipt: &BackupSummary) -> ReadDataArgs {
    ReadDataArgs {
        canister_id: receipt.target.canister,
        snapshot_id: vec![8],
        kind: ReadKind::WasmModule(Extent { offset: 0, size: 5 }),
    }
}
pub(in crate::store) fn maximum() -> NonZeroU64 {
    NonZeroU64::new(10).unwrap()
}
