//! Original direct profile and borrowed approval shared by the two fixed procedures.
use crate::store::{
    BackupDigest, BackupTarget, CaptureLookupError, CaptureObservationError, CapturePlan,
    CaptureTiming, OperationLimits, OperationStoreError, PreparedStatus, RestorePlan,
    StatusAdmissionRecord, StatusCertificateError, StatusEvidence, StatusRequestError,
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
use ic_platform::certificate::PreparedRootKey;
use ic_platform_types::Principal;
use serde::{Deserialize, Serialize};

/// Immutable Root authority and per-slot limits from the original approval.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManagerRecord {
    /// Approved Root principal, distinct from the host signer and child.
    pub root: Principal,
    /// Approved manager authority generation.
    pub generation: u64,
    /// Commitment to the Root wire protocol and codec.
    pub protocol: [u8; 32],
    /// Commitment to the provisioned manager implementation and controller policy.
    pub authority_policy: [u8; 32],
    /// Cycles attached per original management call.
    pub cycles: u64,
    /// Maximum cost including attachment per inner call.
    pub cycle_limit: u64,
    /// Maximum retained inner reply or rejection per slot.
    pub reply_bytes: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::store) struct ProfileRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manager: Option<ManagerRecord>,
    #[serde(skip)]
    pub codec: Option<&'static dyn super::manager::RootCodec>,
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

    /// Host signing identity from the original approval.
    #[must_use]
    pub fn sender(self) -> Principal {
        match self {
            Self::Capture(p) => p.request().sender,
            Self::Restore(p) => p.request().sender,
        }
    }

    /// Original Root policy for the application guard and sign-only adapter.
    #[must_use]
    pub fn root_manager(self) -> Option<&'a ManagerRecord> {
        self.profile().manager.as_ref()
    }

    /// Signed ingress recipient for updates under this approval.
    /// Signers use this identity separately from the child selected by `target`.
    #[must_use]
    pub fn ingress_recipient(self) -> Principal {
        self.requirements().recipient
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
    ) -> crate::store::capture::advance::IngressRequirements {
        crate::store::capture::advance::IngressRequirements {
            target: self.target(),
            sender: self.sender(),
            recipient: self
                .profile()
                .manager
                .as_ref()
                .map_or_else(Principal::management_canister, |manager| manager.root),
            trust_root: self.trust_root(),
        }
    }

    pub(in crate::store) fn status_requirements(
        self,
    ) -> crate::store::capture::status::StatusRequirements {
        let manager = self.profile().manager.as_ref();
        crate::store::capture::status::StatusRequirements {
            module_sha256: match self {
                Self::Capture(plan) => plan.request().module_sha256,
                Self::Restore(plan) => plan.module_sha256(),
            },
            sender: manager.map_or_else(|| self.sender(), |binding| binding.root),
            sole_controller: manager.is_some(),
        }
    }

    pub(in crate::store) fn certificate(self) -> CertificateRequirements {
        CertificateRequirements {
            canister: self.requirements().effective_canister(),
            trust_root: self.trust_root(),
        }
    }

    pub(in crate::store) fn validate(self) -> Result<(), Error> {
        let p = self.profile();
        let limits = self.limits();
        if let Some(manager) = &p.manager
            && (p
                .codec
                .is_some_and(|codec| codec.protocol() != manager.protocol)
                || manager.generation != 1
                || manager.root == Principal::management_canister()
                || manager.root == Principal::anonymous()
                || manager.root == self.target().canister
                || manager.root == self.sender()
                || manager.protocol == [0; 32]
                || manager.authority_policy == [0; 32]
                || manager.cycles > manager.cycle_limit
                || manager.cycle_limit > 1_000_000_000_000
                || !(1..=1_048_576 + 16_384).contains(&manager.reply_bytes)
                || self.operation_id() == "00000000000000000000000000000000"
                || self.target().network.as_bytes() == &[0; 32])
        {
            return Err(Error::Profile);
        }
        if !limits.valid()
            || !(1..=1024).contains(&p.snapshot_capacity)
            || p.root_der.len() != 133
            || PreparedRootKey::from_der(&p.root_der).is_err()
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
        use ic_platform::ingress::{Content as EnvelopeContent, decode_envelope};
        if envelope.len() > self.profile().request_bytes as usize {
            return Err(Error::Profile);
        }
        let content = decode_envelope(&envelope, envelope.len())
            .map_err(|_| Error::Profile)?
            .value
            .content;
        let EnvelopeContent::Call { ingress_expiry, .. } = &content else {
            return Err(Error::Profile);
        };
        Ok(PreparedStatus {
            request_id: content.request_id().map_err(|_| Error::Profile)?,
            expires_ns: *ingress_expiry,
            envelope,
            observation: self.observation(reference)?,
        })
    }

    #[cfg(test)]
    pub(in crate::store) fn authenticate_request(
        self,
        reference: u64,
        prepared: &PreparedStatus,
        method: &str,
        arguments: &[u8],
    ) -> Result<(), Error> {
        // Direct call evidence has no Root binding or inner outcome.
        if self.profile().manager.is_some() {
            return Err(Error::ManagerExchange);
        }
        self.authenticate_ingress(reference, prepared, method, arguments)
    }

    pub(in crate::store) fn authenticate_ingress(
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
        let requirements = self.requirements();
        authenticate_direct_request(
            requirements.sender,
            requirements.recipient,
            a.request_bytes,
            prepared,
            method,
            arguments,
        )?;
        Ok(())
    }
    // The caller authenticates this owned request before resolving its result.
    // No await or replacement of those prepared bytes may intervene.
    pub(in crate::store) fn authenticate_result(
        self,
        reference: u64,
        prepared: &PreparedStatus,
        evidence: &StatusEvidence,
    ) -> Result<(), Error> {
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
    fn attach_codec(&mut self, codec: Option<&'static dyn super::manager::RootCodec>);
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
    /// The manager attested a definitive pre-dispatch refusal for the original slot.
    #[error("Root refused the original operation before dispatch")]
    RootRefused,
    /// Manager evidence requires the original Root exchange codec.
    #[error("original Root exchange codec required")]
    ManagerExchange,
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
