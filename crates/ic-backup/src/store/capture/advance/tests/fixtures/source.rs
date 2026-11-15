use super::super::*;
use crate::store::BackupSummary;
use ic_platform::management_canister_types::{ReadCanisterSnapshotDataArgs, SnapshotDataKind};
use std::num::NonZeroU64;

pub(in crate::store) fn extent(receipt: &BackupSummary) -> ReadCanisterSnapshotDataArgs {
    ReadCanisterSnapshotDataArgs {
        canister_id: receipt.target.canister,
        snapshot_id: receipt
            .snapshot_id_hex
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|hex| u8::from_str_radix(std::str::from_utf8(hex).unwrap(), 16).unwrap())
            .collect(),
        kind: SnapshotDataKind::WasmModule { offset: 0, size: 5 },
    }
}

pub(in crate::store) fn maximum() -> NonZeroU64 {
    NonZeroU64::new(10).unwrap()
}

pub(in crate::store) fn status(
    plan: &CapturePlan,
) -> ic_platform::management_canister_types::CanisterStatusResult {
    crate::store::capture::status::tests::status(plan)
}
