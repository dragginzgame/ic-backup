//! Exact signed read-state admission for original-request recovery. No effects.

use super::StatusRequestError;
use crate::store::limits::REQUEST_BYTES;
use ic_platform::ingress::{Content as EnvelopeContent, decode_envelope};
use thiserror::Error;

///
/// CaptureLookupPolicy
/// Explicit constraints for one lookup envelope, not remote attempt authority.
/// The workflow must retain these separately from the original call's expiry.
///

#[derive(Clone, Copy, Debug)]
pub struct CaptureLookupPolicy {
    /// Exact nonzero lookup expiry in Unix nanoseconds, including historical replay.
    pub expires_ns: u64,
    /// Complete lookup envelope cap, from one byte through the shared 2 MiB request limit.
    /// This bounds input bytes, not total decoder work or allocations.
    pub envelope_bytes: u32,
}

///
/// CaptureLookupError
/// A lookup cannot be associated with the authenticated original request.
///

#[derive(Debug, Error)]
pub enum CaptureLookupError {
    /// Invalid policy or lookup bytes outside its cap; checked before decoding.
    #[error("capture lookup exceeds its explicit bounds")]
    Bounds,
    /// Original signed call, sender, request identity or admission refused.
    #[error("capture lookup original request refused")]
    Original(#[source] StatusRequestError),
    /// Invalid strict read-state content or authentication envelope.
    #[error("invalid capture lookup envelope")]
    Envelope,
    /// Sender, expiry or exact request-status path differs from the approved inputs.
    #[error("capture lookup differs from its original request or policy")]
    Binding,
    /// Direct-key profile or signature failed the shared request authenticator.
    #[error("capture lookup authentication refused")]
    Authentication(#[source] StatusRequestError),
}

// The operation owner has already authenticated the original derived call.
pub(in crate::store) fn authenticate_lookup_envelope(
    sender: ic_platform_types::Principal,
    request_id: [u8; 32],
    bytes: &[u8],
    policy: CaptureLookupPolicy,
) -> Result<[u8; 32], CaptureLookupError> {
    if policy.expires_ns == 0
        || !(1..=REQUEST_BYTES).contains(&policy.envelope_bytes)
        || bytes.is_empty()
        || bytes.len() > policy.envelope_bytes as usize
    {
        return Err(CaptureLookupError::Bounds);
    }
    let expected = sender;
    let envelope = decode_envelope(bytes, policy.envelope_bytes as usize)
        .map_err(|_| CaptureLookupError::Envelope)?
        .value;
    let content = &envelope.content;
    let EnvelopeContent::ReadState {
        sender,
        ingress_expiry,
        paths,
    } = content
    else {
        return Err(CaptureLookupError::Envelope);
    };
    if sender.as_slice() != expected.as_slice()
        || *ingress_expiry != policy.expires_ns
        || paths.len() != 1
        || paths[0].len() != 2
        || paths[0][0].as_slice() != b"request_status"
        || paths[0][1].as_slice() != request_id
    {
        return Err(CaptureLookupError::Binding);
    }
    let id = content
        .request_id()
        .map_err(|_| CaptureLookupError::Envelope)?;
    super::authenticate_direct_envelope(&envelope, sender.as_slice())
        .map_err(CaptureLookupError::Authentication)?;

    Ok(id)
}
