//! Bounded data vector decoding; no skipped extensions or untrusted allocation.

use super::{IcSnapshotDataError, MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES};
use candid::{
    CandidType,
    de::{DecoderConfig, IDLDeserialize},
};
use serde::{Deserialize, Deserializer, de};
use std::fmt;

#[derive(CandidType, Deserialize)]
struct WireData {
    #[serde(deserialize_with = "chunk")]
    chunk: Vec<u8>,
}

fn chunk<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
    struct ChunkVisitor;
    impl<'de> de::Visitor<'de> for ChunkVisitor {
        type Value = Vec<u8>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("at most 1 MiB of snapshot data")
        }
        fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            if seq
                .size_hint()
                .is_some_and(|n| n > MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES)
            {
                return Err(de::Error::custom("snapshot chunk exceeds bound"));
            }
            let mut bytes = Vec::new();
            while bytes.len() < MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES {
                match seq.next_element()? {
                    Some(byte) => bytes.push(byte),
                    None => return Ok(bytes),
                }
            }
            if seq.next_element::<u8>()?.is_some() {
                return Err(de::Error::custom("snapshot chunk exceeds bound"));
            }
            Ok(bytes)
        }
    }
    d.deserialize_seq(ChunkVisitor)
}

pub(super) fn decode(bytes: &[u8]) -> Result<Vec<u8>, IcSnapshotDataError> {
    let mut config = DecoderConfig::new();
    config
        .set_decoding_quota(8 * 1024 * 1024)
        .set_skipping_quota(0)
        .set_max_type_len(16)
        .set_max_header_len(4096)
        .set_full_error_message(false);
    let mut decoder = IDLDeserialize::new_with_config(bytes, &config)
        .map_err(|_| IcSnapshotDataError::InvalidReply)?;
    let value = decoder
        .get_value::<WireData>()
        .map_err(|_| IcSnapshotDataError::InvalidReply)?;
    if !decoder.is_done() {
        return Err(IcSnapshotDataError::InvalidReply);
    }
    decoder
        .done()
        .map_err(|_| IcSnapshotDataError::InvalidReply)?;
    Ok(value.chunk)
}
