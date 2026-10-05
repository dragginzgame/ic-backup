//! Decode-only status projection; other upstream values are bounded skipped work.

use super::{IcCanisterStatusInfo, IcLifecycleReplyError};
use crate::model::control_authority::{ControllerSet, MAX_CONTROLLERS};
use candid::{
    CandidType, Principal,
    de::{DecoderConfig, IDLDeserialize},
};
use ic_management_canister_types::CanisterStatusType;
use serde::{Deserialize, Deserializer, de};
use std::fmt;

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
    struct ControllersVisitor;
    impl<'de> de::Visitor<'de> for ControllersVisitor {
        type Value = Vec<Principal>;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a bounded controller principal sequence")
        }
        fn visit_seq<A: de::SeqAccess<'de>>(
            self,
            mut sequence: A,
        ) -> Result<Self::Value, A::Error> {
            let mut principals = Vec::new();
            while let Some(principal) = sequence.next_element::<Principal>()? {
                if principals.len() == MAX_CONTROLLERS {
                    return Err(de::Error::custom("controller sequence exceeds bound"));
                }
                principals.push(principal);
            }
            Ok(principals)
        }
    }
    deserializer.deserialize_seq(ControllersVisitor)
}

pub(super) fn status(bytes: &[u8]) -> Result<IcCanisterStatusInfo, IcLifecycleReplyError> {
    let mut config = DecoderConfig::new();
    config.set_decoding_quota(DECODING_QUOTA);
    config.set_skipping_quota(SKIPPING_QUOTA);
    config.set_max_type_len(MAX_TYPE_TABLE_ENTRIES);
    config.set_full_error_message(false);
    let mut decoder = IDLDeserialize::new_with_config(bytes, &config)
        .map_err(|_| IcLifecycleReplyError::InvalidReply)?;
    let value = decoder
        .get_value::<WireStatus>()
        .map_err(|_| IcLifecycleReplyError::InvalidReply)?;
    if !decoder.is_done() {
        return Err(IcLifecycleReplyError::InvalidReply);
    }
    decoder
        .done()
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
