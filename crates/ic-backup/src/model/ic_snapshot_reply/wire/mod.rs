//! Decode-only wire adapters with bounded Serde sequences and upstream Candid types.

use super::{IcSnapshotInfo, IcSnapshotReplyError, MAX_IC_SNAPSHOT_REPLY_ENTRIES};
use crate::model::{
    bounded_sequence,
    ic_request::{IcManagementMethodRecord, MAX_IC_SNAPSHOT_ID_BYTES},
    ic_wire::{self, DecoderLimits},
};
use candid::CandidType;
use serde::{Deserialize, Deserializer, de};

// These quotas bound decoder work and type-table retention independently of raw
// input and model sequence bounds. Zero skipped work rejects record extensions.
const DECODING_QUOTA: usize = 2 * 1024 * 1024;
const MAX_TYPE_TABLE_ENTRIES: usize = 16;

#[derive(CandidType, Deserialize)]
struct WireSnapshot {
    #[serde(deserialize_with = "snapshot_id")]
    id: Vec<u8>,
    taken_at_timestamp: u64,
    total_size: u64,
}

impl From<WireSnapshot> for IcSnapshotInfo {
    fn from(snapshot: WireSnapshot) -> Self {
        Self {
            id: snapshot.id,
            taken_at_timestamp: snapshot.taken_at_timestamp,
            total_size: snapshot.total_size,
        }
    }
}

#[derive(CandidType)]
struct WireInventory(Vec<WireSnapshot>);

impl<'de> Deserialize<'de> for WireInventory {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        bounded_sequence::deserialize_wire::<_, WireSnapshot, MAX_IC_SNAPSHOT_REPLY_ENTRIES>(
            deserializer,
            "at most 1024 snapshot descriptors",
            "snapshot inventory exceeds its entry bound",
        )
        .map(Self)
    }
}

pub(crate) fn snapshot_id<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
    let bytes = bounded_sequence::deserialize_wire::<_, u8, MAX_IC_SNAPSHOT_ID_BYTES>(
        deserializer,
        "1..=256 raw snapshot bytes",
        "snapshot identifier exceeds its bounds",
    )?;
    if bytes.is_empty() {
        return Err(de::Error::custom("empty snapshot identifier"));
    }
    Ok(bytes)
}

pub(super) fn decode(
    method: IcManagementMethodRecord,
    bytes: &[u8],
) -> Result<Vec<IcSnapshotInfo>, IcSnapshotReplyError> {
    let mut decoder = ic_wire::decoder(
        bytes,
        DecoderLimits {
            decoding: DECODING_QUOTA,
            skipping: 0,
            type_table: MAX_TYPE_TABLE_ENTRIES,
            header: None,
        },
    )
    .map_err(|()| IcSnapshotReplyError::InvalidReply)?;
    let snapshots = match method {
        IcManagementMethodRecord::TakeCanisterSnapshot => vec![
            decoder
                .get_value::<WireSnapshot>()
                .map_err(|_| IcSnapshotReplyError::InvalidReply)?,
        ],
        IcManagementMethodRecord::ListCanisterSnapshots => {
            decoder
                .get_value::<WireInventory>()
                .map_err(|_| IcSnapshotReplyError::InvalidReply)?
                .0
        }
        _ => return Err(IcSnapshotReplyError::UnsupportedMethod { method }),
    };
    ic_wire::finish(&mut decoder).map_err(|()| IcSnapshotReplyError::InvalidReply)?;
    Ok(snapshots.into_iter().map(IcSnapshotInfo::from).collect())
}
