//! Decode-only status projection; other upstream values are bounded skipped work.

use super::MAX_IC_LIFECYCLE_REPLY_BYTES;
use super::{IcCanisterStatusInfo, IcLifecycleReplyError};
use crate::model::{
    bounded_sequence,
    control_authority::{ControllerSet, MAX_CONTROLLERS},
};
use ic_platform::candid::contract::{CandidType, Principal};
use ic_platform::candid::{DecodeLimits, decode_exactly_one};
use ic_platform::management_canister_types::CanisterStatusType;
use serde::{Deserialize, Deserializer};

const DECODING_QUOTA: usize = 2 * 1024 * 1024;
const SKIPPING_QUOTA: usize = 64 * 1024;
const MAX_TYPE_TABLE_ENTRIES: usize = 64;

#[derive(CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
struct WireStatus {
    status: CanisterStatusType,
    settings: WireSettings,
}

#[derive(CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
struct WireSettings {
    #[serde(deserialize_with = "controllers")]
    controllers: Vec<Principal>,
}

fn controllers<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<Principal>, D::Error> {
    bounded_sequence::deserialize_wire::<_, Principal, MAX_CONTROLLERS>(
        deserializer,
        "a bounded controller principal sequence",
        "controller sequence exceeds bound",
    )
}

pub(super) fn status(bytes: &[u8]) -> Result<IcCanisterStatusInfo, IcLifecycleReplyError> {
    let value: WireStatus = decode_exactly_one(
        bytes,
        DecodeLimits {
            reply_bytes: MAX_IC_LIFECYCLE_REPLY_BYTES,
            decoding_quota: DECODING_QUOTA,
            skipping_quota: SKIPPING_QUOTA,
            type_entries: MAX_TYPE_TABLE_ENTRIES,
            header_bytes: 64 * 1024,
        },
    )
    .map_err(|_| IcLifecycleReplyError::InvalidReply)?;
    let controllers = ControllerSet::new(
        value
            .settings
            .controllers
            .into_iter()
            .map(|principal| principal.to_text())
            .collect(),
    )?;
    Ok(IcCanisterStatusInfo {
        status: value.status,
        controllers,
    })
}
