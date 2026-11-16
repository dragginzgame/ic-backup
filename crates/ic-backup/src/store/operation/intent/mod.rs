//! Original direct profile and borrowed approval shared by the two fixed procedures.
use crate::store::BackupDigest;
use crate::store::{
    BackupTarget, CaptureLookupError, CaptureObservationError, CapturePlan, CaptureTiming,
    OperationLimits, OperationStoreError, PreparedStatus, RestorePlan, StatusAdmissionRecord,
    StatusCertificateError, StatusEvidence, StatusRequestError,
    capture::{
        CertificateRequirements,
        advance::{
            CaptureLookupPolicy, CaptureObservation, CaptureObservationPolicy,
            authenticate_lookup_envelope, observe_authenticated_request,
            status_certificate_observation,
        },
        authenticate_direct_request, certificate_observation,
    },
};
use ic_platform::{Principal, canister_sig_creation::extract_raw_root_pk_from_der};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::store) struct ProfileRecord {
    pub transport: [u8; 32],
    pub signer: [u8; 32],
    pub guard: [u8; 32],
    pub application: [u8; 32],
    pub root_der: Vec<u8>,
    pub snapshot_capacity: u32,
    pub prepare_duration_ns: u64,
    pub submit_duration_ns: u64,
    pub certificate_max_age_ns: u64,
    pub delegation_max_age_ns: u64,
    pub clock_skew_ns: u64,
    pub signer_network: u32,
    pub request_bytes: u32,
    pub result_bytes: u32,
}

/// Exact original approval supplied to signing and live application admission.
#[derive(Clone, Copy, Debug)]
pub enum Plan<'a> {
    /// Original stopped-canister capture.
    Capture(&'a CapturePlan),
    /// Original source-bound same-canister restore.
    Restore(&'a RestorePlan),
}
impl<'a> Plan<'a> {
    /// Original operation ID, never renewed on continuation.
    #[must_use]
    pub fn operation_id(self) -> &'a str {
        match self {
            Self::Capture(p) => &p.request().operation_id,
            Self::Restore(p) => &p.request().operation_id,
        }
    }
    /// Actual approved network and target.
    #[must_use]
    pub fn target(self) -> BackupTarget {
        match self {
            Self::Capture(p) => p.request().target,
            Self::Restore(p) => p.request().target,
        }
    }
    /// Actual direct management sender.
    #[must_use]
    pub fn sender(self) -> Principal {
        match self {
            Self::Capture(p) => p.request().sender,
            Self::Restore(p) => p.request().sender,
        }
    }
    /// Exact original approval commitment.
    #[must_use]
    pub fn digest(self) -> BackupDigest {
        match self {
            Self::Capture(p) => p.digest(),
            Self::Restore(p) => p.digest(),
        }
    }
    /// Exact original encoded declaration.
    #[must_use]
    pub fn encoded(self) -> &'a [u8] {
        match self {
            Self::Capture(p) => p.encoded(),
            Self::Restore(p) => p.encoded(),
        }
    }
    pub(in crate::store) fn trust_root(self) -> BackupDigest {
        match self {
            Self::Capture(p) => p.request().trust_root,
            Self::Restore(p) => p.request().trust_root,
        }
    }
    pub(in crate::store) fn limits(self) -> OperationLimits {
        match self {
            Self::Capture(p) => p.request().limits,
            Self::Restore(p) => p.request().limits,
        }
    }
    pub(in crate::store) fn profile(self) -> &'a ProfileRecord {
        match self {
            Self::Capture(p) => p.profile(),
            Self::Restore(p) => p.profile(),
        }
    }
    pub(in crate::store) fn requirements(
        self,
    ) -> crate::store::capture::advance::DirectRequirements {
        crate::store::capture::advance::DirectRequirements {
            target: self.target(),
            sender: self.sender(),
            trust_root: self.trust_root(),
        }
    }
    pub(in crate::store) fn certificate(self) -> CertificateRequirements {
        CertificateRequirements {
            canister: self.target().canister,
            trust_root: self.trust_root(),
        }
    }
    pub(in crate::store) fn validate(self) -> Result<(), Error> {
        let p = self.profile();
        let limits = self.limits();
        if !limits.valid()
            || !(1..=1024).contains(&p.snapshot_capacity)
            || p.root_der.len() != 133
            || extract_raw_root_pk_from_der(&p.root_der).is_err()
            || BackupDigest::compute(&p.root_der) != self.trust_root()
            || p.prepare_duration_ns == 0
            || p.prepare_duration_ns > p.submit_duration_ns
            || p.submit_duration_ns > 300_000_000_000
            || p.signer_network >= limits.network_requests
        {
            return Err(Error::Profile);
        }
        let reference = p.certificate_max_age_ns.max(p.delegation_max_age_ns).max(1);
        self.admission(reference)?;
        self.observation(reference)?;
        Ok(())
    }
    pub(in crate::store) fn admission(
        self,
        reference: u64,
    ) -> Result<StatusAdmissionRecord, Error> {
        if reference == 0 {
            return Err(Error::Timing);
        }
        let p = self.profile();
        let limits = self.limits();
        let a = StatusAdmissionRecord {
            provider: *self.digest().as_bytes(),
            application: p.application,
            prepare_deadline_ns: reference
                .checked_add(p.prepare_duration_ns)
                .ok_or(Error::Timing)?,
            submit_deadline_ns: reference
                .checked_add(p.submit_duration_ns)
                .ok_or(Error::Timing)?,
            prepare_requests: p.signer_network,
            submit_requests: 1,
            request_bytes: p.request_bytes,
            result_bytes: p.result_bytes,
        };
        a.validate_limits(limits.network_requests, limits.evidence_bytes, 0)?;
        Ok(a)
    }
    pub(in crate::store) fn observation(self, reference: u64) -> Result<Vec<u8>, Error> {
        let p = self.profile();
        let timing = CaptureTiming {
            certificate_max_age_ns: p.certificate_max_age_ns,
            delegation_max_age_ns: p.delegation_max_age_ns,
            max_clock_skew_ns: p.clock_skew_ns,
        };
        Ok(status_certificate_observation(
            timing
                .policy(&self.admission(reference)?, reference)
                .ok_or(Error::Timing)?,
        )?)
    }
    pub(in crate::store) fn decode_request(
        self,
        reference: u64,
        envelope: Vec<u8>,
    ) -> Result<PreparedStatus, Error> {
        use ic_platform::ic_protocol::{EnvelopeContent, decode_call_content_strict};
        if envelope.len() > self.profile().request_bytes as usize {
            return Err(Error::Profile);
        }
        let content =
            decode_call_content_strict(&envelope, envelope.len()).map_err(|_| Error::Profile)?;
        let EnvelopeContent::Call { ingress_expiry, .. } = &content else {
            return Err(Error::Profile);
        };
        Ok(PreparedStatus {
            request_id: *content.to_request_id(),
            expires_ns: *ingress_expiry,
            envelope,
            observation: self.observation(reference)?,
        })
    }
    pub(in crate::store) fn authenticate_request(
        self,
        reference: u64,
        prepared: &PreparedStatus,
        method: &str,
        arguments: &[u8],
    ) -> Result<(), Error> {
        let a = self.admission(reference)?;
        if prepared.expires_ns != a.submit_deadline_ns
            || prepared.observation != self.observation(reference)?
        {
            return Err(Error::Timing);
        }
        authenticate_direct_request(self.sender(), a.request_bytes, prepared, method, arguments)?;
        Ok(())
    }
    pub(in crate::store) fn authenticate(
        self,
        reference: u64,
        prepared: &PreparedStatus,
        evidence: &StatusEvidence,
        method: &str,
        arguments: &[u8],
    ) -> Result<(), Error> {
        self.authenticate_request(reference, prepared, method, arguments)?;
        let a = self.admission(reference)?;
        evidence.encode(a.result_bytes)?;
        self.certificate()
            .authenticate(&a, prepared, evidence, &self.profile().root_der)?;
        Ok(())
    }
    pub(in crate::store) fn authenticate_lookup(
        self,
        reference: u64,
        original: &PreparedStatus,
        bytes: &[u8],
    ) -> Result<(), Error> {
        authenticate_lookup_envelope(
            self.sender(),
            original.request_id,
            bytes,
            CaptureLookupPolicy {
                expires_ns: self.admission(reference)?.submit_deadline_ns,
                envelope_bytes: self.profile().request_bytes,
            },
        )?;
        Ok(())
    }
    pub(in crate::store) fn observe(
        self,
        reference: u64,
        original: &PreparedStatus,
        certificate: &[u8],
    ) -> Result<CaptureObservation, Error> {
        Ok(observe_authenticated_request(
            &self.certificate(),
            original.request_id,
            certificate,
            &self.profile().root_der,
            CaptureObservationPolicy {
                time: certificate_observation(&self.observation(reference)?)?,
                evidence_bytes: self.profile().result_bytes,
            },
        )?)
    }
}

pub(in crate::store) trait Intent: Clone {
    fn view(&self) -> Plan<'_>;
    fn decode(bytes: &[u8]) -> Result<Self, OperationStoreError>;
    fn check_layout(
        record: &super::custody::Recorder<Self>,
        journal: &super::Journal,
    ) -> Result<(), OperationStoreError>;
    fn published_binding(
        record: &super::custody::Recorder<Self>,
        journal: &super::Journal,
    ) -> Result<bool, OperationStoreError>;
}

/// Original declaration, immutable profile or historical evidence refused.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Invalid original direct profile or finite declaration.
    #[error("original direct profile refused")]
    Profile,
    /// Original invocation timing differs from approved policy.
    #[error("original invocation timing refused")]
    Timing,
    /// Original file or finite admission refused.
    #[error(transparent)]
    Plan(#[from] OperationStoreError),
    /// Strict declaration encoding refused.
    #[error(transparent)]
    Encoding(#[from] serde_json::Error),
    /// Exact signed request refused.
    #[error(transparent)]
    Request(#[from] StatusRequestError),
    /// Original certified result refused.
    #[error(transparent)]
    Certificate(#[from] StatusCertificateError),
    /// Signed original-request observation refused.
    #[error(transparent)]
    Lookup(#[from] CaptureLookupError),
    /// Separately timed observation certificate refused.
    #[error(transparent)]
    Observation(#[from] CaptureObservationError),
}
