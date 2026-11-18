//! Fixture-only direct authority shared by the IC harness and isolated consumer.
use ic_backup::store::{
    BackupApplicationGuard, BackupDigest, BackupSigner, OperationAction, OperationPlan,
    OperationSigningContent, OperationSigningRequest,
};
use ic_platform::{
    ed25519::{Signer as _, SigningKey},
    ingress::{Content, Envelope, encode_envelope},
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

/// Current fixture wall-clock time in Unix nanoseconds.
/// # Panics
/// Panics if the fixture clock precedes Unix time or exceeds the u64 range.
#[must_use]
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .try_into()
        .unwrap()
}

/// Public DER for the fixed test-only signing key.
#[must_use]
pub fn der() -> Vec<u8> {
    [
        b"\x30\x2a\x30\x05\x06\x03\x2b\x65\x70\x03\x21\x00".as_slice(),
        SigningKey::from_bytes(&[17; 32]).verifying_key().as_bytes(),
    ]
    .concat()
}
/// Offline test signer whose counter records every signing invocation.
pub struct FixtureSigner(pub Arc<AtomicUsize>);
impl BackupSigner for FixtureSigner {
    type Error = ();
    fn protocol(&self) -> [u8; 32] {
        [1; 32]
    }

    async fn sign(&mut self, request: OperationSigningRequest<'_>) -> Result<Vec<u8>, ()> {
        let ordinal = self.0.fetch_add(1, Ordering::SeqCst);
        assert!(now() < request.deadline_ns);
        assert_eq!(request.network_requests, 0);
        let sender = request.plan.sender().as_slice().try_into().unwrap();
        let content = match request.signing {
            OperationSigningContent::Call { method, arguments } => Content::Call {
                nonce: Some(
                    BackupDigest::compute(
                        &[
                            request.plan.operation_id().as_bytes(),
                            &ordinal.to_le_bytes(),
                        ]
                        .concat(),
                    )
                    .as_bytes()
                    .to_vec(),
                ),
                ingress_expiry: request.expires_ns,
                sender,
                canister_id: request
                    .plan
                    .ingress_recipient()
                    .as_slice()
                    .try_into()
                    .unwrap(),
                method_name: method.into(),
                arg: arguments.to_vec(),
                sender_info: None,
            },
            OperationSigningContent::Observation { request_id } => Content::ReadState {
                sender,
                ingress_expiry: request.expires_ns,
                paths: vec![vec![b"request_status".to_vec(), request_id.to_vec()]],
            },
        };
        let signature =
            SigningKey::from_bytes(&[17; 32]).sign(&content.signable().map_err(|_| ())?);
        encode_envelope(
            &Envelope {
                content,
                sender_pubkey: Some(der()),
                sender_sig: Some(signature.to_bytes().to_vec()),
                sender_delegation: None,
            },
            request.envelope_bytes as usize,
        )
        .map_err(|_| ())
    }
}
/// Revocable test application authority shared with the harness.
pub struct FixtureGuard(pub Arc<AtomicBool>);
impl BackupApplicationGuard for FixtureGuard {
    type Error = ();
    fn protocol(&self) -> [u8; 32] {
        [2; 32]
    }

    fn admit(
        &self,
        _: OperationPlan<'_>,
        action: OperationAction,
        witness: &[u8; 32],
    ) -> Result<(), ()> {
        let _ = action;
        if witness == &[3; 32] && self.0.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err(())
        }
    }
}
