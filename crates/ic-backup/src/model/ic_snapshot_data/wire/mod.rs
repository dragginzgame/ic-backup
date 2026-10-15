//! Bounded data vector decoding; no skipped extensions or untrusted allocation.

use super::{IcSnapshotDataError, MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES};
use crate::model::{
    bounded_sequence,
    ic_wire::{self, DecoderLimits},
};
use candid::CandidType;
use serde::{Deserialize, Deserializer};

#[derive(CandidType, Deserialize)]
struct WireData {
    #[serde(deserialize_with = "chunk")]
    chunk: Vec<u8>,
}

fn chunk<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
    bounded_sequence::deserialize_wire::<_, u8, MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES>(
        d,
        "at most 1 MiB of snapshot data",
        "snapshot chunk exceeds bound",
    )
}

pub(super) fn decode(bytes: &[u8]) -> Result<Vec<u8>, IcSnapshotDataError> {
    let value: WireData = ic_wire::decode_one(
        bytes,
        DecoderLimits {
            decoding: 8 * 1024 * 1024,
            skipping: 0,
            type_table: 16,
            header: Some(4096),
        },
    )
    .map_err(|()| IcSnapshotDataError::InvalidReply)?;
    Ok(value.chunk)
}
