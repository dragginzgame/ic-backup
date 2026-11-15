//! Original direct-host profile, one-shot status submission and historical authentication.
use super::{RestorePlan, RestoreStatusSigner};
use crate::store::{
    CaptureTiming, PreparedStatus, RestoreStatusEvidenceError, StatusAdmissionRecord,
    StatusCertificateError, StatusEvidence, StatusHttpError, StatusHttpTransport,
    StatusRequestError,
    capture::{CertificateRequirements, certificate_observation},
    status_certificate_observation,
};
use ic_host_artifacts::artifact::Sha256Digest;
use std::time::{SystemTime, UNIX_EPOCH};

/// Direct-host restore profile around the integration's existing signer and guard.
///
/// Use through `RestoreOperation::prepare_status` to reserve signing durably.
/// The signer retains key custody and continuous application admission. Its protocol
/// must commit both signing and guard configuration; it receives the original composite
/// admission unchanged. Submission is only available through the operation’s durable
/// `submit_status` reservation. This adapter owns no keys or retry loop.
/// It admits the supplied original observation rather than replacing signer output.
/// The signer samples one reference during its reserved invocation for both windows.
/// Ages match the supplied timing; future allowance is the original submit deadline
/// minus that reference plus clock skew. Expiry equals the original submit deadline.
/// The reference must lie between invocation and completion, with preparation and
/// submission still live. No stale reference or wider policy is accepted.
/// Retain this exact configuration for later admission; opaque older preparation
/// records cannot be upgraded into this profile by supplying fresh configuration.
pub struct DirectRestoreStatus<S> {
    transport: StatusHttpTransport,
    signer: S,
    timing: CaptureTiming,
}

/// Redacted refusal of the direct restore profile; no variant grants a retry.
#[derive(Debug, thiserror::Error)]
pub enum DirectRestoreStatusError<E> {
    /// Provider commitment, observation profile or request capacity differs.
    #[error("direct restore profile binding refused")]
    Binding,
    /// Original finite preparation window, ingress expiry or host clock refused.
    #[error("direct restore timing refused")]
    Deadline,
    /// Integration signing or continuous application admission refused.
    #[error("direct restore integration refused")]
    Signer(E),
    /// Exact direct-key request refused.
    #[error(transparent)]
    Request(#[from] StatusRequestError),
    /// Original observation framing or arithmetic refused.
    #[error(transparent)]
    Certificate(#[from] StatusCertificateError),
    /// Transport binding/refusal or uncertain one-shot HTTP outcome.
    #[error(transparent)]
    Http(#[from] StatusHttpError),
}

impl<S: RestoreStatusSigner> DirectRestoreStatus<S> {
    /// Compose the original fixed transport, integration and explicit timing without IO.
    /// Timing arithmetic is checked against the live preparation reference before signing.
    #[must_use]
    pub fn new(transport: StatusHttpTransport, signer: S, timing: CaptureTiming) -> Self {
        Self {
            transport,
            signer,
            timing,
        }
    }

    /// Commitment for the original `StatusAdmissionRecord::provider` before preparation.
    /// Binds transport origin/network/root/timeout, signer/guard protocol and timing.
    #[must_use]
    pub fn commitment(&self) -> [u8; 32] {
        let mut bytes = b"ic-backup/direct-restore-status/1\n".to_vec();
        bytes.extend_from_slice(&self.transport.commitment());
        bytes.extend_from_slice(&self.signer.protocol());
        for value in [
            self.timing.certificate_max_age_ns,
            self.timing.delegation_max_age_ns,
            self.timing.max_clock_skew_ns,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        *Sha256Digest::compute(&bytes).as_bytes()
    }

    fn observation(
        &self,
        a: &StatusAdmissionRecord,
        reference: u64,
    ) -> Result<Vec<u8>, DirectRestoreStatusError<S::Error>> {
        let policy = self
            .timing
            .policy(a, reference)
            .ok_or(DirectRestoreStatusError::Deadline)?;
        Ok(status_certificate_observation(policy)?)
    }

    fn binding(
        &self,
        plan: &RestorePlan,
        a: &StatusAdmissionRecord,
    ) -> Result<(), DirectRestoreStatusError<S::Error>> {
        self.transport
            .admit_binding(plan.request().target.network, plan.request().trust_root)?;
        if a.provider != self.commitment() || a.submit_requests != 1 {
            return Err(DirectRestoreStatusError::Binding);
        }
        Ok(())
    }
    fn original_request(
        &self,
        plan: &RestorePlan,
        a: &StatusAdmissionRecord,
        prepared: &PreparedStatus,
    ) -> Result<u64, DirectRestoreStatusError<S::Error>> {
        self.binding(plan, a)?;
        plan.authenticate_status_request(a, prepared)?;
        let reference = certificate_observation(&prepared.observation)?
            .certificate
            .reference_ns;
        if prepared.observation != self.observation(a, reference)? {
            return Err(DirectRestoreStatusError::Binding);
        }
        if prepared.expires_ns != a.submit_deadline_ns {
            return Err(DirectRestoreStatusError::Deadline);
        }
        Ok(reference)
    }

    pub(super) fn admit_submission(
        &self,
        plan: &RestorePlan,
        a: &StatusAdmissionRecord,
        prepared: &PreparedStatus,
    ) -> Result<(), DirectRestoreStatusError<S::Error>> {
        let reference = self.original_request(plan, a, prepared)?;
        let current = now().ok_or(DirectRestoreStatusError::Deadline)?;
        if current < reference || current >= prepared.expires_ns {
            return Err(DirectRestoreStatusError::Deadline);
        }
        self.signer
            .admit(plan, a)
            .map_err(DirectRestoreStatusError::Signer)
    }

    pub(super) fn authenticate(
        &self,
        plan: &RestorePlan,
        a: &StatusAdmissionRecord,
        prepared: &PreparedStatus,
        evidence: &StatusEvidence,
    ) -> Result<(), DirectRestoreStatusError<S::Error>> {
        self.original_request(plan, a, prepared)?;
        plan.authenticate_status_evidence(a, prepared, evidence, self.transport.root_key_der())
            .map_err(|error| match error {
                RestoreStatusEvidenceError::Request(e) => DirectRestoreStatusError::Request(e),
                RestoreStatusEvidenceError::Certificate(e) => {
                    DirectRestoreStatusError::Certificate(e)
                }
            })
    }

    pub(super) async fn submit_reserved(
        &self,
        plan: &RestorePlan,
        a: &StatusAdmissionRecord,
        prepared: &PreparedStatus,
    ) -> Result<StatusEvidence, DirectRestoreStatusError<S::Error>> {
        self.admit_submission(plan, a, prepared)?;
        let proof = self
            .transport
            .call_certificate(plan.request().target.canister, a, prepared)
            .await?;
        let (rejected, response) = CertificateRequirements {
            canister: plan.request().target.canister,
            trust_root: plan.request().trust_root,
        }
        .response(a, prepared, &proof, self.transport.root_key_der())?;
        let evidence = StatusEvidence {
            rejected,
            response,
            proof,
        };
        evidence
            .encode(a.result_bytes)
            .map_err(StatusCertificateError::Admission)?;
        if now().is_none_or(|current| current >= a.submit_deadline_ns) {
            return Err(DirectRestoreStatusError::Deadline);
        }
        Ok(evidence)
    }
}

impl<S: RestoreStatusSigner + Send> RestoreStatusSigner for DirectRestoreStatus<S> {
    type Error = DirectRestoreStatusError<S::Error>;
    fn protocol(&self) -> [u8; 32] {
        self.commitment()
    }

    fn admit(&self, plan: &RestorePlan, a: &StatusAdmissionRecord) -> Result<(), Self::Error> {
        self.binding(plan, a)?;
        let reference = now().ok_or(DirectRestoreStatusError::Deadline)?;
        let observation = self.observation(a, reference)?;
        if PreparedStatus::envelope_limit(a.request_bytes, &observation).is_none() {
            return Err(DirectRestoreStatusError::Binding);
        }
        self.signer
            .admit(plan, a)
            .map_err(DirectRestoreStatusError::Signer)
    }

    async fn sign_status(
        &mut self,
        plan: &RestorePlan,
        a: &StatusAdmissionRecord,
        arguments: &[u8],
    ) -> Result<PreparedStatus, Self::Error> {
        self.admit(plan, a)?;
        let started = now().ok_or(DirectRestoreStatusError::Deadline)?;
        let prepared = self
            .signer
            .sign_status(plan, a, arguments)
            .await
            .map_err(DirectRestoreStatusError::Signer)?;
        let completed = now().ok_or(DirectRestoreStatusError::Deadline)?;
        let reference = self.original_request(plan, a, &prepared)?;
        if completed < started
            || reference < started
            || reference > completed
            || completed >= a.prepare_deadline_ns
            || completed >= a.submit_deadline_ns
            || prepared.expires_ns != a.submit_deadline_ns
        {
            return Err(DirectRestoreStatusError::Deadline);
        }
        self.signer
            .admit(plan, a)
            .map_err(DirectRestoreStatusError::Signer)?;
        Ok(prepared)
    }
}

fn now() -> Option<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_nanos()
        .try_into()
        .ok()
}
