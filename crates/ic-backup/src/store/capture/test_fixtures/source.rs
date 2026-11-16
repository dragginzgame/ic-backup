use crate::store::BackupSummary;
use ic_platform::management_canister_types::{ReadCanisterSnapshotDataArgs, SnapshotDataKind};
use std::num::NonZeroU64;
pub(in crate::store) fn extent(receipt: &BackupSummary) -> ReadCanisterSnapshotDataArgs {
    ReadCanisterSnapshotDataArgs {
        canister_id: receipt.target.canister,
        snapshot_id: vec![8],
        kind: SnapshotDataKind::WasmModule { offset: 0, size: 5 },
    }
}
pub(in crate::store) fn maximum() -> NonZeroU64 {
    NonZeroU64::new(10).unwrap()
}
