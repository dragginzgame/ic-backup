//! Decode-only status projection; other upstream values are bounded skipped work.

use super::{IcCanisterStatusInfo, IcLifecycleReplyError};
use crate::model::{
    bounded_sequence,
    control_authority::{ControllerSet, MAX_CONTROLLERS},
    ic_wire::{self, DecoderLimits},
};
use candid::{CandidType, Principal};
use ic_management_canister_types::CanisterStatusType;
use serde::{Deserialize, Deserializer};

const DECODING_QUOTA: usize = 2 * 1024 * 1024;
const SKIPPING_QUOTA: usize = 64 * 1024;
const MAX_TYPE_TABLE_ENTRIES: usize = 64;

#[derive(CandidType, Deserialize)]
struct WireStatus {
    status: CanisterStatusType,
    settings: WireSettings,
}

#[derive(CandidType, Deserialize)]
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
    let value: WireStatus = ic_wire::decode_one(
        bytes,
        DecoderLimits {
            decoding: DECODING_QUOTA,
            skipping: SKIPPING_QUOTA,
            type_table: MAX_TYPE_TABLE_ENTRIES,
            header: None,
        },
    )
    .map_err(|()| IcLifecycleReplyError::InvalidReply)?;
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
