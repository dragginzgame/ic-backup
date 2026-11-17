//! Exact signed read-state admission for original-request recovery. No effects.
#[cfg(test)]
use super::super::CaptureCall;
use super::StatusRequestError;
#[cfg(test)]
use super::{CapturePolicy, PreparedStatus, StatusAdmissionRecord};
use crate::store::limits::REQUEST_BYTES;
use ic_platform::ingress::{Content as EnvelopeContent, decode_envelope};
use thiserror::Error;

/// Explicit constraints for one lookup envelope, not remote attempt authority.
/// The workflow must retain these separately from the original call's expiry.
#[derive(Clone, Copy, Debug)]
pub struct CaptureLookupPolicy {
    /// Exact nonzero lookup expiry in Unix nanoseconds, including historical replay.
    pub expires_ns: u64,
    /// Complete lookup envelope cap, from one byte through the shared 2 MiB request limit.
    /// This bounds input bytes, not total decoder work or allocations.
    pub envelope_bytes: u32,
}

/// A lookup cannot be associated with the authenticated original request.
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

/// Authenticate an externally signed lookup of exactly one original ingress request.
///
/// Authenticates the original status/list/take first, then requires a strict
/// `read_state` envelope from the same sender, the exact supplied lookup expiry,
/// and only `request_status/<original request ID>`. Extra or duplicate paths,
/// prefix lookups and paths to a different request refuse even with valid signatures.
/// Uses the same direct Ed25519/secp256k1 profile as original call authentication.
/// Returns Platform's computed lookup request ID, distinct from the original call
/// ID; it is not a receipt, reservation or submission capability. Neither envelope
/// is rewritten, resigned or renewed. Errors contain no raw request/key/signature.
///
/// The helper reads no clock, journal or network. Historical verification does not
/// establish current expiry or permission. Before a live lookup the workflow must
/// separately retain exact policy/bytes, reserve finite signing and network costs,
/// and admit route, original trust root, provider and observation time windows.
/// Reply authentication remains with the shared certificate authenticator;
/// this function does not settle uncertainty or authorize another management call.
///
/// # Errors
/// Refuses bounds, invalid original requests, malformed/ambiguous read-state content,
/// changed sender/expiry/paths, unsupported identity metadata and bad signatures.
#[cfg(test)]
pub fn authenticate_capture_lookup(
    plan: &CapturePolicy,
    admission: &StatusAdmissionRecord,
    original: &PreparedStatus,
    call: CaptureCall,
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
    super::authenticate_call_request(plan, admission, original, call)
        .map_err(CaptureLookupError::Original)?;
    let sender = plan.request().sender;
    authenticate_lookup_envelope(sender, original.request_id, bytes, policy)
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
