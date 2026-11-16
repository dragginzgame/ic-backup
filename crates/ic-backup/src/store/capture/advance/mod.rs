//! Exact direct call, certificate and bounded HTTP primitives. The recorder owns execution.
#[cfg(test)]
use super::CapturePolicy;
use super::OperationStoreError;
use ic_platform::ic_protocol::{EnvelopeContent, decode_call_content_strict};
#[cfg(test)]
use ic_platform::{candid_codec, management_canister_types::CanisterIdRecord};
use serde::{Deserialize, Serialize};
#[cfg(test)]
mod call;
mod certificate;
mod direct;
mod http;
mod request;
#[cfg(test)]
pub(in crate::store) use call::CaptureCall;
pub(in crate::store) use certificate::CaptureObservation;
pub use certificate::CaptureObservationError;
pub(in crate::store) use certificate::CaptureObservationPolicy;
pub(in crate::store) use certificate::observe_authenticated_request;
pub(in crate::store) use certificate::{
    CertificateRequirements, observation as certificate_observation,
};
pub use certificate::{StatusCertificateError, status_certificate_observation};
pub use direct::CaptureTiming;
pub use http::{BackupHttpError, BackupHttpTransport, ReceivedCertificate};
pub use request::CaptureLookupError;
pub(in crate::store) use request::CaptureLookupPolicy;
pub(in crate::store) use request::DirectRequirements;
pub use request::StatusRequestError;
pub(in crate::store) use request::authenticate_direct_request;
pub(in crate::store) use request::authenticate_lookup_envelope;
/// Original finite call authority, retained before preparation.
/// Capture derives this from its original profile; restore retains its status admission.
/// Bindings identify the provider protocol and continuous application maintenance witness.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatusAdmissionRecord {
    /// Exact integration/provider protocol commitment.
    pub provider: [u8; 32],
    /// Exact continuous-maintenance and external-obligation witness commitment.
    pub application: [u8; 32],
    /// Absolute preparation deadline in Unix nanoseconds; providers enforce it.
    pub prepare_deadline_ns: u64,
    /// Absolute submission deadline in Unix nanoseconds; providers enforce it.
    pub submit_deadline_ns: u64,
    /// Total network requests allowed during preparation, including external signing.
    pub prepare_requests: u32,
    /// Total network requests allowed during submission and response authentication.
    pub submit_requests: u32,
    /// Maximum retained request file size, including its framing (at most 2 MiB).
    pub request_bytes: u32,
    /// Maximum retained result/proof file size, including framing (at most 4 MiB).
    pub result_bytes: u32,
}
impl StatusAdmissionRecord {
    #[cfg(test)]
    pub(in crate::store) fn validate(
        &self,
        plan: &CapturePolicy,
    ) -> Result<(), OperationStoreError> {
        let initial_bytes = 0;
        self.validate_limits(
            plan.request().limits.network_requests,
            plan.request().limits.evidence_bytes,
            initial_bytes,
        )
    }

    pub(in crate::store) fn validate_limits(
        &self,
        network_requests: u32,
        evidence_bytes: u64,
        initial_bytes: usize,
    ) -> Result<(), OperationStoreError> {
        // Reserve all four maximum-size transition frames before any provider output.
        let evidence = u64::from(self.request_bytes)
            + u64::from(self.result_bytes)
            + initial_bytes as u64
            + 4 * (16 * 1024 + 36);
        let requests = self.prepare_requests.checked_add(self.submit_requests);
        if self.prepare_deadline_ns == 0
            || self.submit_deadline_ns < self.prepare_deadline_ns
            || !(46..=crate::store::limits::REQUEST_BYTES).contains(&self.request_bytes)
            || !(7..=crate::store::limits::RESULT_BYTES).contains(&self.result_bytes)
            || self.submit_requests == 0
            || requests.is_none_or(|n| n > network_requests)
            || evidence > evidence_bytes
        {
            return Err(OperationStoreError::StatusAdmission);
        }
        Ok(())
    }
}

/// Exact signed request and its original observation binding. Contains sensitive bytes.
/// Backup checks immutable call content; the provider must authenticate the envelope,
/// routing, observation binding and original admission.
pub struct PreparedStatus {
    /// Original ingress request identity.
    pub request_id: [u8; 32],
    /// Original expiry in Unix nanoseconds.
    pub expires_ns: u64,
    /// Exact envelope to submit without re-encoding or re-signing.
    pub envelope: Vec<u8>,
    /// Provider-owned bounded information needed to authenticate the original result.
    pub observation: Vec<u8>,
}
/// Raw original response and retained authentication proof. Contains sensitive bytes.
pub struct StatusEvidence {
    /// True only for an authenticated remote rejection of this exact request.
    pub rejected: bool,
    /// Raw Candid reply, or provider-defined original rejection bytes.
    pub response: Vec<u8>,
    /// Original authentication evidence, sufficient for historical local replay.
    pub proof: Vec<u8>,
}

#[cfg(test)]
pub(in crate::store) fn arguments(plan: &CapturePolicy) -> Result<Vec<u8>, OperationStoreError> {
    candid_codec::encode_one(
        &CanisterIdRecord {
            canister_id: plan.request().target.canister,
        },
        256,
    )
    .map_err(|_| OperationStoreError::StatusAdmission)
}

// Private exact binary evidence formats. Bounds are checked before concatenation;
// reopening reads at the committed length before parsing or allocating components.
impl PreparedStatus {
    // Inspect the original bytes; never normalize or re-encode a signed envelope.
    // Authentication metadata, routing, observation and freshness stay provider-owned.
    #[cfg(test)]
    pub(in crate::store) fn validate_call(
        &self,
        plan: &CapturePolicy,
    ) -> Result<(), OperationStoreError> {
        self.validate_call_for(plan, CaptureCall::Status)
    }

    #[cfg(test)]
    pub(in crate::store) fn validate_call_for(
        &self,
        plan: &CapturePolicy,
        call: CaptureCall,
    ) -> Result<(), OperationStoreError> {
        self.validate_expected_call(plan, call.method_name(), &call.arguments(plan)?)
    }

    #[cfg(test)]
    pub(in crate::store) fn validate_expected_call(
        &self,
        plan: &CapturePolicy,
        method: &str,
        arguments: &[u8],
    ) -> Result<(), OperationStoreError> {
        let expected = plan.request().sender;
        self.validate_direct_call(expected, method, arguments)
    }

    fn validate_direct_call(
        &self,
        expected: ic_platform::Principal,
        method: &str,
        arguments: &[u8],
    ) -> Result<(), OperationStoreError> {
        let content = decode_call_content_strict(
            &self.envelope,
            crate::store::limits::REQUEST_BYTES as usize,
        )
        .map_err(|_| OperationStoreError::StatusAdmission)?;
        let EnvelopeContent::Call {
            ingress_expiry,
            sender,
            canister_id,
            method_name,
            arg,
            ..
        } = &content
        else {
            return Err(OperationStoreError::StatusAdmission);
        };
        let matches_status = sender.as_slice() == expected.as_slice()
            && canister_id.as_slice().is_empty()
            && method_name == method
            && arg.as_slice() == arguments;
        if !matches_status
            || *ingress_expiry != self.expires_ns
            || *content.to_request_id() != self.request_id
        {
            return Err(OperationStoreError::StatusAdmission);
        }
        Ok(())
    }

    pub(in crate::store) fn encode(&self, limit: u32) -> Result<Vec<u8>, OperationStoreError> {
        if self.expires_ns == 0 {
            return Err(OperationStoreError::StatusAdmission);
        }
        let tail = pair(&self.envelope, &self.observation, limit.saturating_sub(40))?;
        Ok([
            self.request_id.as_slice(),
            &self.expires_ns.to_le_bytes(),
            &tail,
        ]
        .concat())
    }
    #[cfg(test)]
    pub(in crate::store) fn decode(bytes: &[u8]) -> Result<Self, OperationStoreError> {
        if bytes.len() < 46 {
            return Err(OperationStoreError::Changed);
        }
        let (envelope, observation) = split(&bytes[40..])?;
        let expires_ns = u64::from_le_bytes(
            bytes[32..40]
                .try_into()
                .map_err(|_| OperationStoreError::Changed)?,
        );
        if expires_ns == 0 {
            return Err(OperationStoreError::Changed);
        }
        Ok(Self {
            request_id: bytes[..32]
                .try_into()
                .map_err(|_| OperationStoreError::Changed)?,
            expires_ns,
            envelope,
            observation,
        })
    }
}
impl StatusEvidence {
    pub(in crate::store) fn encode(&self, limit: u32) -> Result<Vec<u8>, OperationStoreError> {
        let tail = pair(&self.response, &self.proof, limit.saturating_sub(1))?;
        Ok([&[u8::from(self.rejected)], tail.as_slice()].concat())
    }
    pub(in crate::store) fn decode(bytes: &[u8]) -> Result<Self, OperationStoreError> {
        let Some((&kind, tail)) = bytes.split_first() else {
            return Err(OperationStoreError::Changed);
        };
        if kind > 1 {
            return Err(OperationStoreError::Changed);
        }
        let (response, proof) = split(tail)?;
        Ok(Self {
            rejected: kind == 1,
            response,
            proof,
        })
    }
}
fn pair(first: &[u8], second: &[u8], limit: u32) -> Result<Vec<u8>, OperationStoreError> {
    if first.is_empty()
        || second.is_empty()
        || first
            .len()
            .checked_add(second.len())
            .and_then(|n| n.checked_add(4))
            .is_none_or(|n| n > limit as usize)
    {
        return Err(OperationStoreError::StatusAdmission);
    }
    Ok([&(first.len() as u32).to_le_bytes(), first, second].concat())
}
fn split(bytes: &[u8]) -> Result<(Vec<u8>, Vec<u8>), OperationStoreError> {
    let header = bytes.get(..4).ok_or(OperationStoreError::Changed)?;
    let length = u32::from_le_bytes(
        header
            .try_into()
            .map_err(|_| OperationStoreError::Changed)?,
    ) as usize;
    if length == 0 || length >= bytes.len().saturating_sub(4) {
        return Err(OperationStoreError::Changed);
    }
    Ok((bytes[4..4 + length].to_vec(), bytes[4 + length..].to_vec()))
}

#[cfg(test)]
pub(in crate::store) mod tests;
#[cfg(test)]
pub(in crate::store) use certificate::{
    authenticate_status_evidence, observe_capture_certificate, status_certificate_evidence,
};
#[cfg(test)]
pub(in crate::store) use http::{CaptureLookupHttpError, CaptureLookupHttpPolicy};
#[cfg(test)]
pub(in crate::store) use request::{authenticate_capture_lookup, authenticate_status_request};
