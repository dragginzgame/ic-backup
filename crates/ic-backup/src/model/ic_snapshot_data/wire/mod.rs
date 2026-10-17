//! Bounded data vector decoding; no skipped extensions or untrusted allocation.

use super::MAX_IC_SNAPSHOT_DATA_REPLY_BYTES;
use super::{IcSnapshotDataError, MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES};
use crate::model::bounded_sequence;
use candid::CandidType;
use ic_testkit_candid::{DecodeLimits, decode_exactly_one};
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
    let value: WireData = decode_exactly_one(
        bytes,
        DecodeLimits {
            reply_bytes: MAX_IC_SNAPSHOT_DATA_REPLY_BYTES,
            decoding_quota: 8 * 1024 * 1024,
            skipping_quota: 0,
            type_entries: 16,
            header_bytes: 4096,
        },
    )
    .map_err(|_| IcSnapshotDataError::InvalidReply)?;
    Ok(value.chunk)
}
