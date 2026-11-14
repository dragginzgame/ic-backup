//! Direct-key authentication of original status ingress; no signing, clock or effects.
use super::{CaptureError, CapturePlan, PreparedStatus, StatusAdmissionRecord};
use ic_host_artifacts::artifact::Sha256Digest;
use ic_platform::{
    Principal, ecdsa,
    ecdsa::PrehashVerifier,
    ed25519,
    ic_protocol::{EnvelopeContent, decode_envelope},
};
use thiserror::Error;

// Exact SPKI profiles from RFC 8410 / RFC 5480 and the IC signature specification.
// No general ASN.1 parser or alternate key encoding is admitted here.
const ED25519_DER: &[u8] = b"\x30\x2a\x30\x05\x06\x03\x2b\x65\x70\x03\x21\x00";
const SECP256K1_DER: &[u8] =
    b"\x30\x56\x30\x10\x06\x07\x2a\x86\x48\xce\x3d\x02\x01\x06\x05\x2b\x81\x04\x00\x0a\x03\x42\x00";

/// Refusal to authenticate an original direct-key status request.
#[derive(Debug, Error)]
pub enum StatusRequestError {
    /// The original plan, call content or complete request bound differs.
    #[error("status request fails original admission")]
    Admission(#[source] CaptureError),
    /// Platform could not decode the bounded authentication envelope.
    #[error("invalid status authentication envelope")]
    Envelope,
    /// Unsupported key encoding, delegation or sender-info authentication.
    #[error("unsupported status signing identity")]
    Identity,
    /// The original direct sender is not the public key's self-authenticating ID.
    #[error("status signing key differs from original sender")]
    Sender,
    /// Missing, malformed or invalid signature, including a weak key or high-S ECDSA.
    #[error("invalid status request signature")]
    Signature,
}

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
/// it does not implement a signer or a complete [`super::StatusProvider`].
///
/// ```no_run
/// use ic_backup::store::{CapturePlan, PreparedStatus, StatusAdmissionRecord,
///     StatusRequestError, authenticate_status_request};
/// fn inspect_signer_output(
///     plan: &CapturePlan, admission: &StatusAdmissionRecord, request: &PreparedStatus,
/// ) -> Result<(), StatusRequestError> {
///     authenticate_status_request(plan, admission, request)
/// }
/// ```
///
/// # Errors
/// Refuses mismatched original content, malformed/bounded input, unsupported
/// identities and failed signatures. Errors contain no envelope, key or signature bytes.
pub fn authenticate_status_request(
    plan: &CapturePlan,
    admission: &StatusAdmissionRecord,
    prepared: &PreparedStatus,
) -> Result<(), StatusRequestError> {
    admission
        .validate(plan)
        .map_err(StatusRequestError::Admission)?;
    // Bound the complete caller-owned record before either protocol decoder allocates.
    prepared
        .encode(admission.request_bytes)
        .map_err(StatusRequestError::Admission)?;
    prepared
        .validate_call(plan)
        .map_err(StatusRequestError::Admission)?;
    let envelope = decode_envelope(&prepared.envelope, admission.request_bytes as usize)
        .map_err(|_| StatusRequestError::Envelope)?;
    let EnvelopeContent::Call {
        sender,
        sender_info,
        ..
    } = envelope.content.as_ref()
    else {
        return Err(StatusRequestError::Envelope);
    };
    if envelope.sender_delegation.is_some() || sender_info.is_some() {
        return Err(StatusRequestError::Identity);
    }
    let der = envelope
        .sender_pubkey
        .as_deref()
        .ok_or(StatusRequestError::Identity)?;
    let signature: &[u8; 64] = envelope
        .sender_sig
        .as_deref()
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or(StatusRequestError::Signature)?;
    let message = envelope.content.to_request_id().signable();
    if let Some(raw) = der.strip_prefix(ED25519_DER).filter(|raw| raw.len() == 32) {
        verify_sender(der, sender.as_slice())?;
        let raw = raw.try_into().map_err(|_| StatusRequestError::Identity)?;
        let key =
            ed25519::VerifyingKey::from_bytes(raw).map_err(|_| StatusRequestError::Signature)?;
        key.verify_strict(&message, &ed25519::Signature::from_bytes(signature))
            .map_err(|_| StatusRequestError::Signature)
    } else if let Some(raw) = der
        .strip_prefix(SECP256K1_DER)
        .filter(|raw| raw.len() == 65 && raw[0] == 4)
    {
        verify_sender(der, sender.as_slice())?;
        let key =
            ecdsa::VerifyingKey::from_sec1_bytes(raw).map_err(|_| StatusRequestError::Signature)?;
        let signature =
            ecdsa::Signature::from_slice(signature).map_err(|_| StatusRequestError::Signature)?;
        if signature.normalize_s() != signature {
            return Err(StatusRequestError::Signature);
        }
        key.verify_prehash(Sha256Digest::compute(&message).as_bytes(), &signature)
            .map_err(|_| StatusRequestError::Signature)
    } else {
        Err(StatusRequestError::Identity)
    }
}

fn verify_sender(der: &[u8], sender: &[u8]) -> Result<(), StatusRequestError> {
    if Principal::self_authenticating(der).as_slice() == sender {
        Ok(())
    } else {
        Err(StatusRequestError::Sender)
    }
}
