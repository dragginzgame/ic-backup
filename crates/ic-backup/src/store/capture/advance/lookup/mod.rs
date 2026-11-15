//! One durable original-request lookup and explicit local settlement from its evidence.
mod settlement;
use super::{
    CaptureCall, CaptureError, CaptureLookupError, CaptureLookupHttpError, CaptureLookupHttpPolicy,
    CaptureLookupPolicy, CaptureObservation, CaptureObservationError, CaptureObservationPolicy,
    CaptureOperation, CapturePlan, PreparedStatus, StatusHttpTransport,
    authenticate_capture_lookup, certificate, ledger, observe_capture_certificate,
};
use ledger::{EventRecord, EvidenceRecord};
use serde::{Deserialize, Serialize};
pub use settlement::CaptureLookupSettlementError;
use std::future::Future;
use thiserror::Error;

/// Exact authority retained before the one supported lookup's signing invocation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureLookupAdmissionRecord {
    /// Original submitted status/list/take to observe.
    pub call: CaptureCall,
    /// Exact `StatusHttpTransport::lookup_commitment` configuration commitment.
    pub transport: [u8; 32],
    /// Exact integration sign-only protocol/configuration commitment.
    pub signer: [u8; 32],
    /// Integration-owned read permission witness; not fresh mutation authority.
    pub read_authority: [u8; 32],
    /// Absolute signing completion deadline in Unix nanoseconds, at most 300 seconds
    /// ahead when signing is first reserved.
    pub prepare_deadline_ns: u64,
    /// Absolute HTTP completion deadline in Unix nanoseconds.
    pub lookup_deadline_ns: u64,
    /// Exact signed lookup expiry; never replaces the original call expiry.
    pub expires_ns: u64,
    /// Network allowance for the signing invocation; one HTTP request is separate.
    pub prepare_requests: u32,
    /// Complete lookup envelope cap, at most 1 MiB.
    pub request_bytes: u32,
    /// Encoded HTTP and combined certificate/projected result cap.
    pub result_bytes: u32,
    /// Exact bytes produced by `status_certificate_observation` for this lookup.
    pub observation: Vec<u8>,
}
impl CaptureLookupAdmissionRecord {
    pub(in crate::store::capture) fn policy(
        &self,
    ) -> Result<CaptureLookupHttpPolicy, CaptureError> {
        if self.prepare_deadline_ns == 0
            || self.prepare_deadline_ns > self.lookup_deadline_ns
            || self.lookup_deadline_ns > self.expires_ns
            || !(1..=1024 * 1024).contains(&self.request_bytes)
            || !(5..=2 * 1024 * 1024).contains(&self.result_bytes)
        {
            return Err(CaptureError::StatusAdmission);
        }
        let time = certificate::observation(&self.observation)
            .map_err(|_| CaptureError::StatusAdmission)?;
        Ok(CaptureLookupHttpPolicy {
            request: CaptureLookupPolicy {
                expires_ns: self.expires_ns,
                envelope_bytes: self.request_bytes,
            },
            observation: CaptureObservationPolicy {
                time,
                evidence_bytes: self.result_bytes,
            },
            deadline_ns: self.lookup_deadline_ns,
        })
    }
}

/// Trusted sign-only/read-permission adapter for one separately reserved lookup.
/// All admission is local and effect-free. No method may submit management ingress.
pub trait CaptureLookupSigner {
    /// Redacted integration failure; never include raw signed bytes or keys.
    type Error;
    /// Stable protocol/configuration commitment retained in lookup admission.
    fn protocol(&self) -> [u8; 32];
    /// Check original plan, sender, route and current explicit read permission.
    /// Do not acquire new permission or perform hidden network work here.
    fn admit_read(
        &self,
        plan: &CapturePlan,
        admission: &CaptureLookupAdmissionRecord,
    ) -> Result<(), Self::Error>;
    /// Sign exactly `request_status/<original.request_id>` under the retained admission.
    /// Enforce byte/network/deadline limits while producing output. No retries or
    /// detached work; cancellation retains external process/request custody and spending.
    fn sign_lookup(
        &mut self,
        plan: &CapturePlan,
        admission: &CaptureLookupAdmissionRecord,
        original: &PreparedStatus,
    ) -> impl Future<Output = Result<Vec<u8>, Self::Error>> + Send;
}

/// Reconstructed lookup stage. Inspection authenticates no certificate or permission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureLookupPhase {
    /// Observation and signing allowances consumed; signer result unresolved.
    SigningReserved,
    /// Exact signed lookup retained; HTTP has not been reserved.
    Prepared,
    /// One HTTP request consumed; returned certificate unresolved.
    SubmissionReserved,
    /// Exact certificate retained; replay still authenticates it locally.
    Observed,
}

/// Read-only lookup authority and its lane's reserved counts, additional to original calls.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureLookupView {
    /// Retained stage, not outcome authentication.
    pub phase: CaptureLookupPhase,
    /// Exact original lookup authority, suitable for unchanged continuation.
    pub admission: CaptureLookupAdmissionRecord,
    /// Always one once this lookup exists; no refund after failure.
    pub signing_requests_reserved: u32,
    /// Signing-network allowance plus one after HTTP reservation.
    pub network_requests_reserved: u32,
    /// Always one once signing begins; this slice supports one lookup per operation.
    pub observations_reserved: u32,
}

/// No variant grants a repeat attempt, original mutation settlement or dependent effects.
pub enum CaptureLookupProgress {
    /// Signing was consumed without a complete retained request; never sign again.
    SigningPending,
    /// HTTP was consumed without a retained certificate; never dispatch again.
    LookupPending,
    /// Locally authenticated retained observation, including explicitly unresolved states.
    Observed(CaptureObservation),
}

/// Durable lookup refusal or uncertainty; original evidence and spending remain retained.
#[derive(Debug, Error)]
pub enum CaptureLookupAdvanceError<E> {
    /// Original custody, admission or persistence refused.
    #[error(transparent)]
    Capture(#[from] CaptureError),
    /// Signer protocol, transport commitment or exact admission differs.
    #[error("capture lookup binding differs")]
    Binding,
    /// Read permission or external signing refused; any reservation stays spent.
    #[error("capture lookup signer refused")]
    Signer(E),
    /// Signing deadline elapsed or the host clock refused.
    #[error("capture lookup preparation deadline refused")]
    Deadline,
    /// Exact original or signed lookup authentication failed.
    #[error(transparent)]
    Request(#[from] CaptureLookupError),
    /// One-shot HTTP failed or its result is uncertain.
    #[error(transparent)]
    Http(#[from] CaptureLookupHttpError),
    /// Historical certificate authentication refused.
    #[error(transparent)]
    Observation(#[from] CaptureObservationError),
}

impl CaptureOperation {
    /// Inspect lookup authority and lane-specific reservations without effects.
    /// # Errors
    /// Refuses changed custody, missing/orphan evidence and damaged journals.
    pub fn inspect_lookup(&self) -> Result<Option<CaptureLookupView>, CaptureError> {
        Ok(self.check()?.lookup.map(|state| CaptureLookupView {
            phase: state.phase,
            network_requests_reserved: state.admission.prepare_requests
                + u32::from(matches!(
                    state.phase,
                    CaptureLookupPhase::SubmissionReserved | CaptureLookupPhase::Observed
                )),
            admission: state.admission,
            signing_requests_reserved: 1,
            observations_reserved: 1,
        }))
    }

    /// Durably sign, submit once and retain one original-request observation.
    ///
    /// Starts only for an original submitted, unsettled status/list/take. Exact lookup
    /// admission, signing and observation allowance are retained before invoking the
    /// signer; the authenticated request is retained before reserving HTTP. A failed
    /// or cancelled invocation stays pending across reopen and never repeats. Prepared
    /// continuation reuses original bytes after fresh local read admission. Observed
    /// replay verifies the retained certificate without signer, live admission or IO
    /// beyond original local custody. One lookup is supported per operation, even if
    /// its authenticated outcome is pending/absent/done; no automatic second attempt.
    ///
    /// This retains observation evidence only. It neither settles the original call
    /// under a new time policy nor advances dependent work. Original evidence, transition
    /// ordinals and certificate windows remain unchanged. Older readers refuse new
    /// lookup events; original journals without lookup events remain readable unchanged.
    ///
    /// # Errors
    /// Refuses mismatched original authority, exhausted cumulative bounds, failed signing,
    /// invalid requests/proofs and uncertain persistence. No failure refunds spending.
    /// # Panics
    /// Requires a Tokio runtime with IO and timers enabled.
    pub async fn advance_lookup<S: CaptureLookupSigner>(
        &mut self,
        admission: &CaptureLookupAdmissionRecord,
        transport: &StatusHttpTransport,
        signer: &mut S,
    ) -> Result<CaptureLookupProgress, CaptureLookupAdvanceError<S::Error>> {
        let mut state = self.check()?;
        let policy = admission.policy()?;
        if state
            .lookup
            .as_ref()
            .is_some_and(|s| &s.admission != admission)
            || transport.lookup_commitment() != admission.transport
        {
            return Err(CaptureLookupAdvanceError::Binding);
        }
        let (original_admission, original_ref, _) = state.original(admission.call)?;
        let original_admission = original_admission.clone();
        let original =
            PreparedStatus::decode(&self.evidence(original_name(admission.call), original_ref)?)?;
        if let Some(lookup) = &state.lookup {
            match lookup.phase {
                CaptureLookupPhase::SigningReserved => {
                    return Ok(CaptureLookupProgress::SigningPending);
                }
                CaptureLookupPhase::SubmissionReserved => {
                    return Ok(CaptureLookupProgress::LookupPending);
                }
                CaptureLookupPhase::Observed => {
                    let bytes = self.evidence(
                        "lookup-request.bin",
                        lookup.request.as_ref().ok_or(CaptureError::Incomplete)?,
                    )?;
                    authenticate_capture_lookup(
                        &self.plan,
                        &original_admission,
                        &original,
                        admission.call,
                        &bytes,
                        policy.request,
                    )?;
                    let cert = self.evidence(
                        "lookup-certificate.bin",
                        lookup
                            .certificate
                            .as_ref()
                            .ok_or(CaptureError::Incomplete)?,
                    )?;
                    let outcome = observe_capture_certificate(
                        &self.plan,
                        &original_admission,
                        &original,
                        admission.call,
                        &cert,
                        transport.root_key_der(),
                        policy.observation,
                    )?;
                    if self.check()? != state {
                        return Err(CaptureError::Changed.into());
                    }
                    return Ok(CaptureLookupProgress::Observed(outcome));
                }
                CaptureLookupPhase::Prepared => {}
            }
        }
        if !state.pending(admission.call) {
            return Err(CaptureError::StatusAdmission.into());
        }
        if signer.protocol() != admission.signer {
            return Err(CaptureLookupAdvanceError::Binding);
        }
        transport
            .admit_plan(&self.plan)
            .map_err(CaptureLookupHttpError::from)?;
        super::request::authenticate_call_request(
            &self.plan,
            &original_admission,
            &original,
            admission.call,
        )
        .map_err(CaptureLookupError::Original)?;
        signer
            .admit_read(&self.plan, admission)
            .map_err(CaptureLookupAdvanceError::Signer)?;
        if state.lookup.is_none() {
            if super::http::remaining(admission.prepare_deadline_ns)
                .map_err(|_| CaptureLookupAdvanceError::Deadline)?
                > std::time::Duration::from_secs(300)
            {
                return Err(CaptureLookupAdvanceError::Deadline);
            }
            self.commit(
                &EventRecord::LookupStarted {
                    admission: admission.clone(),
                },
                None,
            )?;
            let duration = super::http::remaining(admission.prepare_deadline_ns)
                .map_err(|_| CaptureLookupAdvanceError::Deadline)?;
            let bytes = tokio::time::timeout(
                duration,
                signer.sign_lookup(&self.plan, admission, &original),
            )
            .await
            .map_err(|_| CaptureLookupAdvanceError::Deadline)?
            .map_err(CaptureLookupAdvanceError::Signer)?;
            self.check()?;
            if signer.protocol() != admission.signer {
                return Err(CaptureLookupAdvanceError::Binding);
            }
            super::http::remaining(admission.prepare_deadline_ns)
                .map_err(|_| CaptureLookupAdvanceError::Deadline)?;
            authenticate_capture_lookup(
                &self.plan,
                &original_admission,
                &original,
                admission.call,
                &bytes,
                policy.request,
            )?;
            self.commit(
                &EventRecord::LookupPrepared {
                    request: EvidenceRecord::new(&bytes),
                },
                Some(("lookup-request.bin", &bytes)),
            )?;
            state = self.check()?;
        }
        let lookup = state.lookup.as_ref().ok_or(CaptureError::Incomplete)?;
        let bytes = self.evidence(
            "lookup-request.bin",
            lookup.request.as_ref().ok_or(CaptureError::Incomplete)?,
        )?;
        authenticate_capture_lookup(
            &self.plan,
            &original_admission,
            &original,
            admission.call,
            &bytes,
            policy.request,
        )?;
        signer
            .admit_read(&self.plan, admission)
            .map_err(CaptureLookupAdvanceError::Signer)?;
        super::http::remaining(admission.lookup_deadline_ns)
            .map_err(|_| CaptureLookupAdvanceError::Deadline)?;
        self.commit(&EventRecord::LookupSubmissionReserved, None)?;
        let evidence = transport
            .lookup_once(
                &self.plan,
                &original_admission,
                &original,
                admission.call,
                &bytes,
                policy,
            )
            .await?;
        self.check()?;
        self.commit(
            &EventRecord::LookupObserved {
                certificate: EvidenceRecord::new(&evidence.certificate),
            },
            Some(("lookup-certificate.bin", &evidence.certificate)),
        )?;
        Ok(CaptureLookupProgress::Observed(evidence.outcome))
    }
}
pub(in crate::store::capture) fn original_name(call: CaptureCall) -> &'static str {
    match call {
        CaptureCall::Status => "status-request.bin",
        CaptureCall::ListSnapshots => "snapshot-list-request.bin",
        CaptureCall::TakeSnapshot => "snapshot-take-request.bin",
    }
}

pub(in crate::store::capture) fn result_name(call: CaptureCall) -> &'static str {
    match call {
        CaptureCall::Status => "status-result.bin",
        CaptureCall::ListSnapshots => "snapshot-list-result.bin",
        CaptureCall::TakeSnapshot => "snapshot-take-result.bin",
    }
}
