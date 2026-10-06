//! Decode-only pinned wire fields with bounded allocation and no skipped work.

use super::{IcSnapshotMetadataError, MAX_IC_SNAPSHOT_CHUNKS, MAX_IC_SNAPSHOT_GLOBALS};
use candid::{
    CandidType,
    de::{DecoderConfig, IDLDeserialize},
};
use ic_management_canister_types::{
    CanisterTimer, ChunkHash, OnLowWasmMemoryHookStatus, ReadCanisterSnapshotMetadataResult,
    SnapshotMetadataGlobal, SnapshotSource,
};
use serde::{Deserialize, Deserializer, de};
use std::{collections::BTreeSet, fmt};

#[derive(CandidType, Deserialize)]
struct WireMetadata {
    source: Option<WireSource>,
    taken_at_timestamp: u64,
    wasm_module_size: u64,
    #[serde(deserialize_with = "globals")]
    globals: Vec<Option<SnapshotMetadataGlobal>>,
    wasm_memory_size: u64,
    stable_memory_size: u64,
    #[serde(deserialize_with = "chunks")]
    wasm_chunk_store: Vec<WireChunk>,
    canister_version: u64,
    #[serde(deserialize_with = "bytes32")]
    certified_data: Vec<u8>,
    global_timer: Option<CanisterTimer>,
    on_low_wasm_memory_hook_status: Option<OnLowWasmMemoryHookStatus>,
}

#[derive(CandidType, Deserialize)]
struct WireChunk {
    #[serde(deserialize_with = "bytes32")]
    hash: Vec<u8>,
}

#[derive(CandidType, Deserialize)]
enum WireSource {
    #[serde(rename = "taken_from_canister")]
    TakenFromCanister(WireReserved),
    #[serde(rename = "metadata_upload")]
    MetadataUpload(WireReserved),
}

// The SDK's Reserved deserializer deliberately skips arbitrary values. Read the
// actual Candid reserved marker instead so recognized source tags need no skipped
// work and cannot silently discard an unknown payload under our zero-skip quota.
struct WireReserved;

impl CandidType for WireReserved {
    fn _ty() -> candid::types::Type {
        candid::types::TypeInner::Reserved.into()
    }
    fn idl_serialize<S: candid::types::Serializer>(&self, serializer: S) -> Result<(), S::Error> {
        serializer.serialize_null(())
    }
}

impl<'de> Deserialize<'de> for WireReserved {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct ReservedVisitor;
        impl de::Visitor<'_> for ReservedVisitor {
            type Value = WireReserved;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("the Candid reserved marker")
            }
            fn visit_byte_buf<E: de::Error>(self, bytes: Vec<u8>) -> Result<Self::Value, E> {
                if bytes == [3] {
                    Ok(WireReserved)
                } else {
                    Err(E::custom("invalid reserved marker"))
                }
            }
        }
        d.deserialize_any(ReservedVisitor)
    }
}

fn globals<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<Vec<Option<SnapshotMetadataGlobal>>, D::Error> {
    sequence::<_, _, MAX_IC_SNAPSHOT_GLOBALS>(d)
}

fn chunks<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<WireChunk>, D::Error> {
    sequence::<_, _, MAX_IC_SNAPSHOT_CHUNKS>(d)
}

fn bytes32<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
    sequence::<_, _, 32>(d)
}

// Never allocate using an untrusted size hint. Each retained vector has one
// explicit bound; the decoder's separate quota also bounds value/type work.
fn sequence<'de, D, T, const LIMIT: usize>(d: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct SequenceVisitor<T, const LIMIT: usize>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const LIMIT: usize> de::Visitor<'de> for SequenceVisitor<T, LIMIT> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "at most {LIMIT} metadata elements")
        }
        fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            if seq.size_hint().is_some_and(|n| n > LIMIT) {
                return Err(de::Error::custom("metadata sequence exceeds bound"));
            }
            let mut values = Vec::new();
            while values.len() < LIMIT {
                match seq.next_element()? {
                    Some(value) => values.push(value),
                    None => return Ok(values),
                }
            }
            if seq.next_element::<T>()?.is_some() {
                return Err(de::Error::custom("metadata sequence exceeds bound"));
            }
            Ok(values)
        }
    }
    d.deserialize_seq(SequenceVisitor::<T, LIMIT>(std::marker::PhantomData))
}

pub(super) fn decode(
    bytes: &[u8],
) -> Result<ReadCanisterSnapshotMetadataResult, IcSnapshotMetadataError> {
    let mut config = DecoderConfig::new();
    config
        .set_decoding_quota(2 * 1024 * 1024)
        .set_skipping_quota(0)
        .set_max_type_len(64)
        .set_max_header_len(16 * 1024)
        .set_full_error_message(false);
    let mut decoder = IDLDeserialize::new_with_config(bytes, &config)
        .map_err(|_| IcSnapshotMetadataError::InvalidReply)?;
    let value = decoder
        .get_value::<WireMetadata>()
        .map_err(|_| IcSnapshotMetadataError::InvalidReply)?;
    if !decoder.is_done() {
        return Err(IcSnapshotMetadataError::InvalidReply);
    }
    decoder
        .done()
        .map_err(|_| IcSnapshotMetadataError::InvalidReply)?;
    if value
        .globals
        .iter()
        .flatten()
        .any(|global| matches!(global, SnapshotMetadataGlobal::V128(n) if n.0.bits() > 128))
    {
        return Err(IcSnapshotMetadataError::InvalidGlobal);
    }
    let mut hashes = BTreeSet::new();
    for chunk in &value.wasm_chunk_store {
        if chunk.hash.len() != 32 || !hashes.insert(chunk.hash.as_slice()) {
            return Err(IcSnapshotMetadataError::InvalidChunkHash);
        }
    }
    Ok(ReadCanisterSnapshotMetadataResult {
        source: value.source.map(|source| match source {
            WireSource::TakenFromCanister(_) => SnapshotSource::TakenFromCanister(candid::Reserved),
            WireSource::MetadataUpload(_) => SnapshotSource::MetadataUpload(candid::Reserved),
        }),
        taken_at_timestamp: value.taken_at_timestamp,
        wasm_module_size: value.wasm_module_size,
        globals: value.globals,
        wasm_memory_size: value.wasm_memory_size,
        stable_memory_size: value.stable_memory_size,
        wasm_chunk_store: value
            .wasm_chunk_store
            .into_iter()
            .map(|c| ChunkHash { hash: c.hash })
            .collect(),
        canister_version: value.canister_version,
        certified_data: value.certified_data,
        global_timer: value.global_timer,
        on_low_wasm_memory_hook_status: value.on_low_wasm_memory_hook_status,
    })
}
