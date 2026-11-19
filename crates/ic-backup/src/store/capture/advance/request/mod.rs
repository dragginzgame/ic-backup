//! Direct-key authentication of original status ingress; no signing, clock or effects.

mod lookup;
#[cfg(test)]
pub(super) mod tests;

use super::{OperationStoreError, PreparedStatus};
use crate::store::BackupDigest;
use ic_platform::{
    ecdsa,
    ecdsa::PrehashVerifier,
    ed25519,
    ingress::{Content as EnvelopeContent, Envelope, decode_envelope},
};
use ic_platform_types::Principal;
pub(in crate::store) use lookup::authenticate_lookup_envelope;
pub use lookup::{CaptureLookupError, CaptureLookupPolicy};
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
    Admission(#[source] OperationStoreError),
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

pub(in crate::store) fn authenticate_direct_request(
    sender: Principal,
    recipient: Principal,
    request_bytes: u32,
    prepared: &PreparedStatus,
    method: &str,
    arguments: &[u8],
) -> Result<(), StatusRequestError> {
    // Bound the complete caller-owned record before either protocol decoder allocates.
    prepared
        .encode(request_bytes)
        .map_err(StatusRequestError::Admission)?;
    prepared
        .validate_direct_call(sender, recipient, method, arguments)
        .map_err(StatusRequestError::Admission)?;
    let envelope = decode_envelope(&prepared.envelope, request_bytes as usize)
        .map_err(|_| StatusRequestError::Envelope)?
        .value;
    let EnvelopeContent::Call {
        sender,
        sender_info,
        ..
    } = &envelope.content
    else {
        return Err(StatusRequestError::Envelope);
    };
    if sender_info.is_some() {
        return Err(StatusRequestError::Identity);
    }
    authenticate_direct_envelope(&envelope, sender.as_slice())
}

fn authenticate_direct_envelope(
    envelope: &Envelope,
    sender: &[u8],
) -> Result<(), StatusRequestError> {
    if envelope.sender_delegation.is_some() {
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
    let message = envelope
        .content
        .signable()
        .map_err(|_| StatusRequestError::Envelope)?;
    if let Some(raw) = der.strip_prefix(ED25519_DER).filter(|raw| raw.len() == 32) {
        verify_sender(der, sender)?;
        let raw = raw.try_into().map_err(|_| StatusRequestError::Identity)?;
        let key =
            ed25519::VerifyingKey::from_bytes(raw).map_err(|_| StatusRequestError::Signature)?;
        key.verify_strict(&message, &ed25519::Signature::from_bytes(signature))
            .map_err(|_| StatusRequestError::Signature)
    } else if let Some(raw) = der
        .strip_prefix(SECP256K1_DER)
        .filter(|raw| raw.len() == 65 && raw[0] == 4)
    {
        verify_sender(der, sender)?;
        let key =
            ecdsa::VerifyingKey::from_sec1_bytes(raw).map_err(|_| StatusRequestError::Signature)?;
        let signature =
            ecdsa::Signature::from_slice(signature).map_err(|_| StatusRequestError::Signature)?;
        if signature.normalize_s() != signature {
            return Err(StatusRequestError::Signature);
        }
        key.verify_prehash(BackupDigest::compute(&message).as_bytes(), &signature)
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

/// Exact direct management identity selected by an original intent or native protocol fixture.
pub(in crate::store) struct IngressRequirements {
    pub target: crate::store::BackupTarget,
    pub sender: Principal,
    pub recipient: Principal,
    pub trust_root: crate::store::BackupDigest,
}

impl IngressRequirements {
    // Management ingress is routed by its child argument. An update to Root
    // is routed and certified by the signed recipient, independently of the child.
    pub(in crate::store) fn effective_canister(&self) -> Principal {
        if self.recipient == Principal::management_canister() {
            self.target.canister
        } else {
            self.recipient
        }
    }
}
