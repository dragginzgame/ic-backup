//! Original-call and lookup authentication helpers for capture regressions.

use super::{super::CaptureCall, *};
use crate::store::{StatusAdmissionRecord, capture::CapturePolicy, limits::REQUEST_BYTES};

/// Authenticate an externally signed status request for a direct host identity.
///
/// Checks original admission, complete framing bounds, exact call content and
/// request ID before verifying the signature through Platform. Supports canonical
/// Ed25519 and uncompressed secp256k1 SPKI keys only, with 64-byte signatures.
/// Ed25519 verification is strict; secp256k1 requires low-S. Delegations (including
/// an explicit empty chain), sender-info, anonymous, P-256, WebAuthn and canister
/// signing identities are outside this profile. Original bytes are never changed.
///
/// This local, effect-free check reads no clock, so it can authenticate original
/// requests during historical replay. A provider must separately admit live expiry,
/// network/root/routing, original observation policy, application continuity and
/// its own binding/costs. Success grants no signing, submission or retry authority.
/// Use before retaining signer output, before live dispatch and during local replay;
/// signing and durable execution remain with the owning operation.
///
///
/// # Errors
/// Refuses mismatched original content, malformed/bounded input, unsupported
/// identities and failed signatures. Errors contain no envelope, key or signature bytes.
pub fn authenticate_status_request(
    plan: &CapturePolicy,
    admission: &StatusAdmissionRecord,
    prepared: &PreparedStatus,
) -> Result<(), StatusRequestError> {
    authenticate_call_request(plan, admission, prepared, CaptureCall::Status)
}

pub(in crate::store::capture::advance) fn authenticate_call_request(
    plan: &CapturePolicy,
    admission: &StatusAdmissionRecord,
    prepared: &PreparedStatus,
    call: CaptureCall,
) -> Result<(), StatusRequestError> {
    authenticate_expected_request(
        plan,
        admission,
        prepared,
        call.method_name(),
        &call
            .arguments(plan)
            .map_err(StatusRequestError::Admission)?,
    )
}

fn authenticate_expected_request(
    plan: &CapturePolicy,
    admission: &StatusAdmissionRecord,
    prepared: &PreparedStatus,
    method: &str,
    arguments: &[u8],
) -> Result<(), StatusRequestError> {
    admission
        .validate(plan)
        .map_err(StatusRequestError::Admission)?;
    let sender = plan.request().sender;
    authenticate_direct_request(
        sender,
        Principal::management_canister(),
        admission.request_bytes,
        prepared,
        method,
        arguments,
    )
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
    authenticate_call_request(plan, admission, original, call)
        .map_err(CaptureLookupError::Original)?;
    let sender = plan.request().sender;
    authenticate_lookup_envelope(sender, original.request_id, bytes, policy)
}
