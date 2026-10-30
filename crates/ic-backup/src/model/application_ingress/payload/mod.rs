//! Exact bounded application bytes; no application codec or effect permission.

use super::ApplicationIngressError;
use crate::model::{
    artifacts::ArtifactChecksumRecord, attempt_journal::OperationBindingRecord,
    evidence_digest::EvidenceDigest,
};
use std::fmt;

/// Maximum opaque argument bytes, checked before copying.
pub const MAX_APPLICATION_INGRESS_ARGUMENT_BYTES: usize = 1024 * 1024;
/// Maximum exact visible ASCII method bytes.
pub const MAX_APPLICATION_INGRESS_METHOD_BYTES: usize = 128;
/// Maximum caller-selected raw application reply ceiling.
pub const MAX_APPLICATION_INGRESS_REPLY_BYTES: usize = 1024 * 1024;

/// Immutable original receiver/routing target, update method, bytes and reply ceiling.
///
/// Retain these bytes before publishing their digest in the original plan or
/// observation reservation. Receiver and effective target may differ, but both
/// must be application principals. Management methods and query mode have separate
/// owners; this declaration supports replicated update ingress only. No Serde,
/// codec, fresh permission or application-specific interpretation is supplied.
#[derive(Clone)]
pub struct ApplicationIngressPayload {
    receiver: String,
    target: String,
    receiver_bytes: Vec<u8>,
    target_bytes: Vec<u8>,
    method: String,
    arguments: Vec<u8>,
    reply_limit: usize,
}

impl ApplicationIngressPayload {
    /// Admit exact opaque application ingress before allocating original spending.
    /// # Errors
    /// Rejects malformed/management principals, invalid methods and excessive bounds.
    pub fn new(
        receiver: &str,
        target: &str,
        method: &str,
        arguments: &[u8],
        reply_limit: usize,
    ) -> Result<Self, ApplicationIngressError> {
        crate::model::application_payload::validate_bytes(
            arguments,
            MAX_APPLICATION_INGRESS_ARGUMENT_BYTES,
            method,
            MAX_APPLICATION_INGRESS_METHOD_BYTES,
        )?;
        if !(1..=MAX_APPLICATION_INGRESS_REPLY_BYTES).contains(&reply_limit) {
            return Err(ApplicationIngressError::InvalidReplyLimit);
        }
        let (receiver, receiver_bytes) = crate::model::application_payload::principal(receiver)?;
        let (target, target_bytes) = crate::model::application_payload::principal(target)?;
        Ok(Self {
            receiver,
            target,
            receiver_bytes,
            target_bytes,
            method: method.into(),
            arguments: arguments.into(),
            reply_limit,
        })
    }
    /// Read the canonical receiver encoded into the signed update.
    #[must_use]
    pub fn receiver(&self) -> &str {
        &self.receiver
    }
    /// Read the canonical effective routing target bound to the original operation.
    #[must_use]
    pub fn target(&self) -> &str {
        &self.target
    }
    /// Read exact original method spelling.
    #[must_use]
    pub fn method(&self) -> &str {
        &self.method
    }
    /// Read exact opaque bytes without decoding application contracts.
    #[must_use]
    pub fn arguments(&self) -> &[u8] {
        &self.arguments
    }
    /// Read the original caller-selected raw reply ceiling.
    #[must_use]
    pub const fn reply_limit(&self) -> usize {
        self.reply_limit
    }
    /// Hash v1 NUL-terminated domain, u8-length/raw receiver and target principals,
    /// fixed update byte 1, then u64 BE length/method, length/arguments and reply ceiling.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        let mut framing = EvidenceDigest::new(b"ic-backup/application-ingress/v1\0");
        for principal in [&self.receiver_bytes, &self.target_bytes] {
            framing.bytes_u8(principal);
        }
        framing.byte(1);
        for bytes in [self.method.as_bytes(), self.arguments.as_slice()] {
            framing.bytes_u64(bytes);
        }
        framing.u64_be(self.reply_limit as u64);
        framing.finish()
    }
    /// Check original mutation target and every exact bound update field before spending.
    /// # Errors
    /// Rejects changed target or payload digest without IO or reservation.
    pub fn validate_mutation_binding(
        &self,
        binding: &OperationBindingRecord,
    ) -> Result<(), ApplicationIngressError> {
        self.validate_target(binding)?;
        if self.digest().hash() != binding.request() {
            return Err(ApplicationIngressError::PayloadMismatch);
        }
        Ok(())
    }
    /// Check recovery routing before reserving its separate exact payload digest.
    ///
    /// The original operation still binds the unresolved mutation's digest. The
    /// observation request separately checks its own digest and current reservation.
    /// # Errors
    /// Rejects a changed original target without IO or reservation.
    pub fn validate_observation_binding(
        &self,
        binding: &OperationBindingRecord,
    ) -> Result<(), ApplicationIngressError> {
        self.validate_target(binding)
    }
    fn validate_target(
        &self,
        binding: &OperationBindingRecord,
    ) -> Result<(), ApplicationIngressError> {
        if self.target != binding.target() {
            return Err(ApplicationIngressError::TargetMismatch);
        }
        Ok(())
    }
}

impl fmt::Debug for ApplicationIngressPayload {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ApplicationIngressPayload")
            .field("receiver", &self.receiver)
            .field("target", &self.target)
            .field("method", &self.method)
            .field("argument_bytes", &self.arguments.len())
            .field("reply_limit", &self.reply_limit)
            .finish_non_exhaustive()
    }
}
