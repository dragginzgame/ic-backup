//! Shared bounded Candid decoder setup; each method owns its exact limits.

use candid::{
    CandidType,
    de::{DecoderConfig, IDLDeserialize},
};
use serde::Deserialize;

#[derive(Clone, Copy)]
pub(super) struct DecoderLimits {
    pub decoding: usize,
    pub skipping: usize,
    pub type_table: usize,
    pub header: Option<usize>,
}

pub(super) fn decoder(bytes: &[u8], limits: DecoderLimits) -> Result<IDLDeserialize<'_>, ()> {
    let mut config = DecoderConfig::new();
    config
        .set_decoding_quota(limits.decoding)
        .set_skipping_quota(limits.skipping)
        .set_max_type_len(limits.type_table)
        .set_full_error_message(false);
    if let Some(header) = limits.header {
        config.set_max_header_len(header);
    }
    IDLDeserialize::new_with_config(bytes, &config).map_err(|_| ())
}

pub(super) fn finish(decoder: &mut IDLDeserialize<'_>) -> Result<(), ()> {
    if !decoder.is_done() {
        return Err(());
    }
    decoder.done().map_err(|_| ())
}

pub(super) fn decode_one<'de, T>(bytes: &'de [u8], limits: DecoderLimits) -> Result<T, ()>
where
    T: CandidType + Deserialize<'de>,
{
    let mut decoder = decoder(bytes, limits)?;
    let value = decoder.get_value::<T>().map_err(|_| ())?;
    finish(&mut decoder)?;
    Ok(value)
}
