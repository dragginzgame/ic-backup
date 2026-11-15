//! Direct-host composition; integrations retain signing and application authority.
use super::{
    CaptureCall, CaptureError, CapturePlan, CaptureProvider, PreparedStatus, StatusAdmissionRecord,
    StatusCertificateError, StatusEvidence, StatusHttpError, StatusHttpTransport, StatusProvider,
    StatusRequestError, arguments, certificate, status_certificate_observation,
};
use ic_host_artifacts::artifact::Sha256Digest;
use ic_platform::{
    certification::historical::{TimePolicy, TimeWindow},
    ic_protocol::{EnvelopeContent, decode_call_content_strict},
};
use std::{
    future::Future,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use thiserror::Error;

const MAX_LIVE_WINDOW_NS: u64 = 300_000_000_000;

/// One sign-only invocation, already covered by the operation's durable reservation.
/// Only the selected replicated call to the management canister is permitted,
/// routed to the plan's effective target. No management request may be submitted.
pub struct CaptureSigningRequest<'a> {
    /// Exact finite management call; sign its method and supplied canonical arguments.
    pub call: CaptureCall,
    /// Original approved target, sender, network and trust commitments.
    pub plan: &'a CapturePlan,
    /// Exact Platform-encoded management arguments; do not reconstruct them.
    pub arguments: &'a [u8],
    /// Absolute completion deadline in Unix nanoseconds.
    pub deadline_ns: u64,
    /// Upper bound on signed ingress expiry, in Unix nanoseconds.
    pub max_expiry_ns: u64,
    /// Maximum complete envelope bytes, enforced while receiving signer output.
    pub envelope_bytes: u32,
    /// Total network requests available to this invocation, including signing services.
    pub network_requests: u32,
}

/// Trusted integration-owned sign-only adapter; Backup never owns a private key.
///
/// Invoke at most one signer. Enforce the supplied byte/network limits while producing
/// output, before allocation. The future must be cooperative and nonblocking; dropping
/// it must preserve child-process/request custody without retries or detached submission.
/// Backup bounds the awaited duration and verifies the returned envelope independently.
/// A timeout cannot undo an external signature or replenish original spending.
pub trait CaptureSigner {
    /// Typed redacted error; never include keys, envelopes or credentials.
    type Error;
    /// Stable commitment to the adapter protocol and relevant configuration.
    fn protocol(&self) -> [u8; 32];
    /// Return only the exact externally signed envelope, without submitting it.
    fn sign_call(
        &mut self,
        request: CaptureSigningRequest<'_>,
    ) -> impl Future<Output = Result<Vec<u8>, Self::Error>> + Send;
}

/// Integration-owned continuous maintenance and external-obligation authority.
///
/// This check is local, bounded and effect-free. It must verify the original witness
/// and its uninterrupted validity for the exact plan, not infer safety from a stopped
/// status or acquire fresh maintenance. Canic owns its own witness implementation.
pub trait CaptureApplicationGuard {
    /// Typed redacted refusal.
    type Error;
    /// Stable commitment to the witness protocol and relevant configuration.
    fn protocol(&self) -> [u8; 32];
    /// Check the original application commitment immediately before live work.
    fn admit(&self, plan: &CapturePlan, application: &[u8; 32]) -> Result<(), Self::Error>;
}

/// Explicit original certificate policy; there are no implicit freshness defaults.
#[derive(Clone, Copy, Debug)]
pub struct CaptureTiming {
    /// Maximum certificate age relative to the start of preparation.
    pub certificate_max_age_ns: u64,
    /// Separate maximum root-delegation age relative to that same reference.
    pub delegation_max_age_ns: u64,
    /// Extra future allowance beyond the original submission deadline.
    pub max_clock_skew_ns: u64,
}

impl CaptureTiming {
    // Shared exact time-window arithmetic for direct capture and restore preparation.
    pub(in crate::store) fn policy(
        &self,
        admission: &StatusAdmissionRecord,
        reference: u64,
    ) -> Option<TimePolicy> {
        let future = admission
            .submit_deadline_ns
            .checked_sub(reference)
            .filter(|span| *span <= MAX_LIVE_WINDOW_NS)?
            .checked_add(self.max_clock_skew_ns)?;
        if reference >= admission.prepare_deadline_ns {
            return None;
        }
        let window = |max_age_ns| TimeWindow {
            reference_ns: reference,
            max_age_ns,
            max_future_skew_ns: future,
        };
        Some(TimePolicy {
            certificate: window(self.certificate_max_age_ns),
            delegation: window(self.delegation_max_age_ns),
        })
    }
}

/// Direct provider using one external signer invocation and one authenticated HTTP call.
///
/// Use through [`super::CaptureOperation::advance_status`] and
/// [`super::CaptureOperation::advance_snapshot`] to obtain durable per-call reservations.
/// The caller supplies a Tokio runtime with IO and timers. This provider owns no runtime,
/// identity key, process launcher, polling, recovery retry or application maintenance.
/// Retain the original configuration and integration protocols for historical replay.
pub struct DirectCaptureProvider<S, A> {
    transport: StatusHttpTransport,
    signer: S,
    application: A,
    timing: CaptureTiming,
}

/// Redacted direct-provider refusal. No error grants new spending or retry authority.
#[derive(Debug, Error)]
pub enum DirectCaptureError<S, A> {
    /// Original finite authority or exact arguments refused.
    #[error("direct capture admission refused")]
    Admission(#[source] CaptureError),
    /// Provider protocol/configuration or retained observation differs from admission.
    #[error("direct capture provider binding refused")]
    Binding,
    /// Host time, original live window or awaited preparation deadline refused.
    #[error("direct capture deadline or clock refused")]
    Deadline,
    /// Sign-only adapter failed; its reservation remains spent.
    #[error("direct capture signing failed")]
    Signer(S),
    /// Original continuous application authority is unavailable.
    #[error("direct capture application authority refused")]
    Application(A),
    /// Exact envelope or signature refused.
    #[error("direct capture request authentication failed")]
    Request(#[from] StatusRequestError),
    /// Original observation or certificate refused.
    #[error("direct capture certificate authentication failed")]
    Certificate(#[from] StatusCertificateError),
    /// HTTP refusal or uncertain outcome.
    #[error("direct capture HTTP failed")]
    Http(#[from] StatusHttpError),
}

impl<S: CaptureSigner, A: CaptureApplicationGuard> DirectCaptureProvider<S, A> {
    /// Compose explicit transport, sign-only and application policies without effects.
    /// Time-window arithmetic is checked against the actual original reference before signing.
    pub fn new(
        transport: StatusHttpTransport,
        signer: S,
        application: A,
        timing: CaptureTiming,
    ) -> Self {
        Self {
            transport,
            signer,
            application,
            timing,
        }
    }

    /// Commitment to put in the original [`StatusAdmissionRecord::provider`].
    /// Includes the route/network/root/timeout, signer and guard protocols and timing.
    #[must_use]
    pub fn commitment(&self) -> [u8; 32] {
        let mut bytes = b"ic-backup/direct-capture-provider/1\n".to_vec();
        bytes.extend_from_slice(&self.transport.commitment());
        bytes.extend_from_slice(&self.signer.protocol());
        bytes.extend_from_slice(&self.application.protocol());
        for value in [
            self.timing.certificate_max_age_ns,
            self.timing.delegation_max_age_ns,
            self.timing.max_clock_skew_ns,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        *Sha256Digest::compute(&bytes).as_bytes()
    }

    fn binding(
        &self,
        plan: &CapturePlan,
        admission: &StatusAdmissionRecord,
    ) -> Result<(), DirectCaptureError<S::Error, A::Error>> {
        admission
            .validate(plan)
            .map_err(DirectCaptureError::Admission)?;
        self.transport.admit_plan(plan)?;
        if admission.provider != self.commitment() || admission.submit_requests != 1 {
            return Err(DirectCaptureError::Binding);
        }
        Ok(())
    }

    fn observation(
        &self,
        admission: &StatusAdmissionRecord,
        reference: u64,
    ) -> Result<Vec<u8>, DirectCaptureError<S::Error, A::Error>> {
        let policy = self
            .timing
            .policy(admission, reference)
            .ok_or(DirectCaptureError::Deadline)?;
        Ok(status_certificate_observation(policy)?)
    }

    fn original_request(
        &self,
        plan: &CapturePlan,
        admission: &StatusAdmissionRecord,
        prepared: &PreparedStatus,
        call: CaptureCall,
    ) -> Result<u64, DirectCaptureError<S::Error, A::Error>> {
        self.binding(plan, admission)?;
        super::request::authenticate_call_request(plan, admission, prepared, call)?;
        let reference = certificate::observation(&prepared.observation)?
            .certificate
            .reference_ns;
        if prepared.observation != self.observation(admission, reference)? {
            return Err(DirectCaptureError::Binding);
        }
        if prepared.expires_ns <= reference || prepared.expires_ns > admission.submit_deadline_ns {
            return Err(DirectCaptureError::Deadline);
        }
        Ok(reference)
    }
}

impl<S: CaptureSigner + Send, A: CaptureApplicationGuard + Send> CaptureProvider
    for DirectCaptureProvider<S, A>
{
    fn admit_call(
        &self,
        plan: &CapturePlan,
        admission: &StatusAdmissionRecord,
        call: CaptureCall,
        prepared: Option<&PreparedStatus>,
    ) -> Result<(), Self::Error> {
        self.binding(plan, admission)?;
        let now = now().ok_or(DirectCaptureError::Deadline)?;
        if let Some(prepared) = prepared {
            let reference = self.original_request(plan, admission, prepared, call)?;
            if now < reference || now >= prepared.expires_ns || now >= admission.submit_deadline_ns
            {
                return Err(DirectCaptureError::Deadline);
            }
        } else {
            // Validate the finite observation before spending any signing authority.
            let observation = self.observation(admission, now)?;
            PreparedStatus::envelope_limit(admission.request_bytes, &observation)
                .ok_or(DirectCaptureError::Binding)?;
        }
        self.application
            .admit(plan, &admission.application)
            .map_err(DirectCaptureError::Application)
    }

    async fn prepare_call(
        &mut self,
        plan: &CapturePlan,
        admission: &StatusAdmissionRecord,
        call: CaptureCall,
    ) -> Result<PreparedStatus, Self::Error> {
        self.admit_call(plan, admission, call, None)?;
        let args = call
            .arguments(plan)
            .map_err(DirectCaptureError::Admission)?;
        let reference = now().ok_or(DirectCaptureError::Deadline)?;
        let observation = self.observation(admission, reference)?;
        let envelope_bytes = PreparedStatus::envelope_limit(admission.request_bytes, &observation)
            .ok_or(DirectCaptureError::Binding)?;
        let remaining = admission
            .prepare_deadline_ns
            .checked_sub(reference)
            .filter(|span| *span > 0)
            .ok_or(DirectCaptureError::Deadline)?;
        let envelope = tokio::time::timeout(
            Duration::from_nanos(remaining),
            self.signer.sign_call(CaptureSigningRequest {
                call,
                plan,
                arguments: &args,
                deadline_ns: admission.prepare_deadline_ns,
                max_expiry_ns: admission.submit_deadline_ns,
                envelope_bytes,
                network_requests: admission.prepare_requests,
            }),
        )
        .await
        .map_err(|_| DirectCaptureError::Deadline)?
        .map_err(DirectCaptureError::Signer)?;
        let completed = now().ok_or(DirectCaptureError::Deadline)?;
        if completed < reference || completed >= admission.prepare_deadline_ns {
            return Err(DirectCaptureError::Deadline);
        }
        if envelope.len() > envelope_bytes as usize {
            return Err(DirectCaptureError::Binding);
        }
        let content = decode_call_content_strict(&envelope, envelope_bytes as usize)
            .map_err(|_| DirectCaptureError::Binding)?;
        let EnvelopeContent::Call { ingress_expiry, .. } = &content else {
            return Err(DirectCaptureError::Binding);
        };
        let prepared = PreparedStatus {
            request_id: *content.to_request_id(),
            expires_ns: *ingress_expiry,
            envelope,
            observation,
        };
        self.admit_call(plan, admission, call, Some(&prepared))?;
        let completed = now().ok_or(DirectCaptureError::Deadline)?;
        if completed < reference
            || completed >= admission.prepare_deadline_ns
            || completed >= prepared.expires_ns
        {
            return Err(DirectCaptureError::Deadline);
        }
        Ok(prepared)
    }

    async fn submit_call(
        &mut self,
        plan: &CapturePlan,
        admission: &StatusAdmissionRecord,
        call: CaptureCall,
        prepared: &PreparedStatus,
    ) -> Result<StatusEvidence, Self::Error> {
        self.admit_call(plan, admission, call, Some(prepared))?;
        Ok(self
            .transport
            .submit_call(plan, admission, prepared, call)
            .await?)
    }

    fn authenticate_call(
        &self,
        plan: &CapturePlan,
        admission: &StatusAdmissionRecord,
        call: CaptureCall,
        prepared: &PreparedStatus,
        evidence: &StatusEvidence,
    ) -> Result<(), Self::Error> {
        self.original_request(plan, admission, prepared, call)?;
        // Historical verification does not reacquire live application authority or read time.
        Ok(super::certificate::authenticate_call_evidence(
            plan,
            admission,
            prepared,
            evidence,
            self.transport.root_key_der(),
            call,
        )?)
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

impl<S: CaptureSigner + Send, A: CaptureApplicationGuard + Send> StatusProvider
    for DirectCaptureProvider<S, A>
{
    type Error = DirectCaptureError<S::Error, A::Error>;
    fn admit(
        &self,
        plan: &CapturePlan,
        admission: &StatusAdmissionRecord,
        args: &[u8],
        prepared: Option<&PreparedStatus>,
    ) -> Result<(), Self::Error> {
        if args != arguments(plan).map_err(DirectCaptureError::Admission)? {
            return Err(DirectCaptureError::Binding);
        }
        self.admit_call(plan, admission, CaptureCall::Status, prepared)
    }
    async fn prepare(
        &mut self,
        plan: &CapturePlan,
        admission: &StatusAdmissionRecord,
        args: &[u8],
    ) -> Result<PreparedStatus, Self::Error> {
        if args != arguments(plan).map_err(DirectCaptureError::Admission)? {
            return Err(DirectCaptureError::Binding);
        }
        self.prepare_call(plan, admission, CaptureCall::Status)
            .await
    }
    async fn submit_once(
        &mut self,
        plan: &CapturePlan,
        admission: &StatusAdmissionRecord,
        prepared: &PreparedStatus,
    ) -> Result<StatusEvidence, Self::Error> {
        self.submit_call(plan, admission, CaptureCall::Status, prepared)
            .await
    }
    fn authenticate(
        &self,
        plan: &CapturePlan,
        admission: &StatusAdmissionRecord,
        prepared: &PreparedStatus,
        evidence: &StatusEvidence,
    ) -> Result<(), Self::Error> {
        self.authenticate_call(plan, admission, CaptureCall::Status, prepared, evidence)
    }
}
