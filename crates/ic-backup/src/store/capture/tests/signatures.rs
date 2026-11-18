//! Real signatures over external protocol envelopes; no signing service or IC effects.
use super::*;
use ic_platform::{
    ecdsa, ed25519,
    ingress::{Envelope, decode_envelope, encode_envelope},
};
use ic_platform_types::Principal;

#[derive(Clone, Copy)]
pub(in crate::store) enum Scheme {
    Ed25519,
    Secp256k1,
}
impl Scheme {
    pub(in crate::store) fn key(self) -> Vec<u8> {
        // Independently written standard SPKI prefixes; exact lengths/bytes are exercised.
        match self {
            Self::Ed25519 => [
                &[48, 42, 48, 5, 6, 3, 43, 101, 112, 3, 33, 0][..],
                ed25519::SigningKey::from_bytes(&[17; 32])
                    .verifying_key()
                    .as_bytes(),
            ]
            .concat(),
            Self::Secp256k1 => [
                &[
                    48, 86, 48, 16, 6, 7, 42, 134, 72, 206, 61, 2, 1, 6, 5, 43, 129, 4, 0, 10, 3,
                    66, 0,
                ][..],
                ecdsa::SigningKey::from_slice(&[17; 32])
                    .unwrap()
                    .verifying_key()
                    .to_sec1_point(false)
                    .as_bytes(),
            ]
            .concat(),
        }
    }

    pub(in crate::store) fn sign(self, message: &[u8]) -> Vec<u8> {
        match self {
            Self::Ed25519 => {
                use ed25519::Signer;
                ed25519::SigningKey::from_bytes(&[17; 32])
                    .sign(message)
                    .to_bytes()
                    .to_vec()
            }
            Self::Secp256k1 => {
                use ecdsa::PrehashSigner;
                let signature: ecdsa::Signature = ecdsa::SigningKey::from_slice(&[17; 32])
                    .unwrap()
                    .sign_prehash(BackupDigest::compute(message).as_bytes())
                    .unwrap();
                signature.to_bytes().to_vec()
            }
        }
    }
}

pub(in crate::store) fn fixture(scheme: Scheme) -> (CapturePolicy, PreparedStatus) {
    let der = scheme.key();
    let sender = Principal::self_authenticating(&der);
    let mut request = plan().request().clone();
    request.sender = sender;
    let plan = CapturePolicy::new(request).unwrap();
    let mut content = call_content();
    if let EnvelopeContent::Call { sender: caller, .. } = &mut content {
        *caller = sender.as_slice().try_into().unwrap();
    }
    let id = content.request_id().unwrap();
    // Independent literal request domain, rather than using the verifier's signable method.
    let message = [b"\x0aic-request".as_slice(), id.as_slice()].concat();
    let envelope = Envelope {
        content,
        sender_pubkey: Some(der),
        sender_sig: Some(scheme.sign(&message)),
        sender_delegation: None,
    };
    (
        plan,
        PreparedStatus {
            request_id: id,
            expires_ns: 21,
            envelope: encode_envelope(&envelope, 4096).unwrap(),
            observation: vec![1],
        },
    )
}

pub(in crate::store) fn change(prepared: &mut PreparedStatus, action: impl FnOnce(&mut Envelope)) {
    let mut envelope = decode_envelope(&prepared.envelope, 4096).unwrap().value;
    action(&mut envelope);
    prepared.request_id = envelope.content.request_id().unwrap();
    prepared.envelope = encode_envelope(&envelope, 8192).unwrap();
}
