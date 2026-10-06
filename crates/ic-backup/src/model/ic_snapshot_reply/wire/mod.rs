//! Decode-only wire adapters with bounded Serde sequences and upstream Candid types.

use super::{IcSnapshotInfo, IcSnapshotReplyError, MAX_IC_SNAPSHOT_REPLY_ENTRIES};
use crate::model::ic_request::{IcManagementMethodRecord, MAX_IC_SNAPSHOT_ID_BYTES};
use candid::{
    CandidType,
    de::{DecoderConfig, IDLDeserialize},
};
use serde::{Deserialize, Deserializer, de};
use std::fmt;

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
        struct InventoryVisitor;
        impl<'de> de::Visitor<'de> for InventoryVisitor {
            type Value = WireInventory;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("at most 1024 snapshot descriptors")
            }

            fn visit_seq<A: de::SeqAccess<'de>>(
                self,
                mut sequence: A,
            ) -> Result<Self::Value, A::Error> {
                if sequence
                    .size_hint()
                    .is_some_and(|size| size > MAX_IC_SNAPSHOT_REPLY_ENTRIES)
                {
                    return Err(de::Error::custom(
                        "snapshot inventory exceeds its entry bound",
                    ));
                }
                let mut snapshots = Vec::new();
                while snapshots.len() < MAX_IC_SNAPSHOT_REPLY_ENTRIES {
                    match sequence.next_element::<WireSnapshot>()? {
                        Some(snapshot) => snapshots.push(snapshot),
                        None => return Ok(WireInventory(snapshots)),
                    }
                }
                if sequence.next_element::<WireSnapshot>()?.is_some() {
                    return Err(de::Error::custom(
                        "snapshot inventory exceeds its entry bound",
                    ));
                }
                Ok(WireInventory(snapshots))
            }
        }
        deserializer.deserialize_seq(InventoryVisitor)
    }
}

pub(crate) fn snapshot_id<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
    struct IdVisitor;
    impl<'de> de::Visitor<'de> for IdVisitor {
        type Value = Vec<u8>;

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("1..=256 raw snapshot bytes")
        }

        fn visit_seq<A: de::SeqAccess<'de>>(
            self,
            mut sequence: A,
        ) -> Result<Self::Value, A::Error> {
            if sequence
                .size_hint()
                .is_some_and(|size| size == 0 || size > MAX_IC_SNAPSHOT_ID_BYTES)
            {
                return Err(de::Error::custom("snapshot identifier exceeds its bounds"));
            }
            let mut bytes = Vec::new();
            while bytes.len() < MAX_IC_SNAPSHOT_ID_BYTES {
                match sequence.next_element::<u8>()? {
                    Some(byte) => bytes.push(byte),
                    None if bytes.is_empty() => {
                        return Err(de::Error::custom("empty snapshot identifier"));
                    }
                    None => return Ok(bytes),
                }
            }
            if sequence.next_element::<u8>()?.is_some() {
                return Err(de::Error::custom("snapshot identifier exceeds its bounds"));
            }
            Ok(bytes)
        }
    }
    deserializer.deserialize_seq(IdVisitor)
}

pub(super) fn decode(
    method: IcManagementMethodRecord,
    bytes: &[u8],
) -> Result<Vec<IcSnapshotInfo>, IcSnapshotReplyError> {
    let mut config = DecoderConfig::new();
    config
        .set_decoding_quota(DECODING_QUOTA)
        .set_skipping_quota(0)
        .set_max_type_len(MAX_TYPE_TABLE_ENTRIES)
        .set_full_error_message(false);
    let mut decoder = IDLDeserialize::new_with_config(bytes, &config)
        .map_err(|_| IcSnapshotReplyError::InvalidReply)?;
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
    if !decoder.is_done() {
        return Err(IcSnapshotReplyError::InvalidReply);
    }
    decoder
        .done()
        .map_err(|_| IcSnapshotReplyError::InvalidReply)?;
    Ok(snapshots.into_iter().map(IcSnapshotInfo::from).collect())
}
