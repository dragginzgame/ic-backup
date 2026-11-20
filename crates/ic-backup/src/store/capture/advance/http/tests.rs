//! Test-only lookup policy and authenticated HTTP observation fixtures.

use super::{
    BackupHttpError, BackupHttpTransport, PreparedStatus, StatusAdmissionRecord, StatusEvidence,
    remaining,
};
use crate::store::capture::{
    CapturePolicy,
    advance::{
        CaptureCall, CaptureLookupError, CaptureLookupPolicy, CaptureObservation,
        CaptureObservationError, CaptureObservationPolicy, authenticate_capture_lookup,
        observe_capture_certificate, status_certificate_observation,
    },
};
use thiserror::Error;

///
/// CaptureLookupHttpPolicy
/// Separately approved lookup bounds. These fields do not reserve any authority.
///

#[derive(Clone, Copy, Debug)]
pub struct CaptureLookupHttpPolicy {
    /// Exact signed lookup expiry and raw envelope cap.
    pub request: CaptureLookupPolicy,
    /// Explicit certificate/delegation windows and combined proof/result cap.
    /// The same cap also bounds the complete encoded HTTP response body.
    pub observation: CaptureObservationPolicy,
    /// Absolute Unix nanosecond deadline for connection, body and authentication.
    pub deadline_ns: u64,
}

///
/// CaptureLookupEvidence
/// Exact authenticated certificate and its original-request projection, not settlement.
/// Contains sensitive bytes and deliberately has no Debug implementation.
///

pub struct CaptureLookupEvidence {
    /// Exact original certificate bytes, retained by the caller for historical replay.
    pub certificate: Vec<u8>,
    /// Original request's state; absent/done/unknown never prove non-execution.
    pub outcome: CaptureObservation,
}

///
/// CaptureLookupHttpError
/// Lookup refusal or uncertain observation; no variant grants another attempt.
///

#[derive(Debug, Error)]
pub enum CaptureLookupHttpError {
    /// Invalid observation bounds or time policy, refused before dispatch.
    #[error("invalid capture lookup HTTP policy")]
    Policy,
    /// Original or lookup request authentication refused before dispatch.
    #[error(transparent)]
    Request(#[from] CaptureLookupError),
    /// Shared HTTP admission, deadline, response bound or uncertain transport failure.
    #[error(transparent)]
    Http(#[from] BackupHttpError),
    /// Certificate or original-result association refused after dispatch.
    #[error(transparent)]
    Observation(#[from] CaptureObservationError),
}

impl BackupHttpTransport {
    /// Fetch one original request-status certificate using exact retained lookup bytes.
    ///
    /// Call only after the integrating workflow has durably reserved this observation
    /// and retained its request and policy. This helper does not inspect a journal or
    /// establish those reservations. It neither signs nor submits a management call.
    /// Original calls and lookup signatures authenticate before connection; route and
    /// root must match the original plan. Only the original target's v2 read-state
    /// endpoint is used, through the existing no-proxy/no-redirect/no-retry client.
    /// The deadline is bounded by the lookup expiry and configured transport timeout;
    /// original call expiry is not extended and may already have elapsed.
    ///
    /// HTTP/body handling shares the existing bounded CBOR path. The exact certificate
    /// authenticates under the explicitly supplied observation windows and original
    /// root/target/request ID. The caller owns freshness-policy selection, custody and
    /// persistence. A return value is not a settled result or permission for dependent
    /// work. Pending, done, absent and unknown stay distinct; there is no polling.
    /// Cancellation or lost response leaves all external reservations spent.
    ///
    /// # Errors
    /// Refuses invalid policy, request authentication, route/root, elapsed deadlines,
    /// HTTP or byte bounds, malformed responses and failed certificate authentication.
    ///
    /// # Panics
    /// Polling requires a Tokio runtime with IO and timers enabled.
    pub(in crate::store) async fn lookup_once(
        &self,
        plan: &CapturePolicy,
        admission: &StatusAdmissionRecord,
        original: &PreparedStatus,
        call: CaptureCall,
        lookup: &[u8],
        policy: CaptureLookupHttpPolicy,
    ) -> Result<CaptureLookupEvidence, CaptureLookupHttpError> {
        if policy.observation.evidence_bytes < 5
            || policy.observation.evidence_bytes > admission.result_bytes
            || status_certificate_observation(policy.observation.time).is_err()
        {
            return Err(CaptureLookupHttpError::Policy);
        }
        authenticate_capture_lookup(plan, admission, original, call, lookup, policy.request)?;
        self.admit_plan(plan)?;
        let deadline = policy.deadline_ns.min(policy.request.expires_ns);
        remaining(deadline)?;
        let maximum = policy.observation.evidence_bytes as usize;
        let certificate = self
            .read_state_certificate(
                plan.request().target.canister,
                lookup,
                original.request_id,
                deadline,
                maximum,
            )
            .await?;
        let outcome = observe_capture_certificate(
            plan,
            admission,
            original,
            call,
            &certificate,
            self.root_key_der(),
            policy.observation,
        )?;
        remaining(deadline)?;

        Ok(CaptureLookupEvidence {
            certificate,
            outcome,
        })
    }
}

impl BackupHttpTransport {
    fn admit_plan(&self, plan: &CapturePolicy) -> Result<(), BackupHttpError> {
        self.admit_binding(plan.request().target.network, plan.request().trust_root)
    }

    /// Send the original envelope once, then authenticate a retained terminal result.
    ///
    /// Call only from the live provider invocation backed by the operation's durable
    /// submission reservation. Calling this helper directly does not prove that
    /// reservation and never creates authority to replay a request. It does not
    /// implement signing or application-continuity admission.
    ///
    /// The original network/root, exact status call and framed request bounds are
    /// checked before dispatch, along with direct-key signature authentication through
    /// [`crate::store::capture::advance::authenticate_status_request`]. Effective routing uses the original target. The
    /// request timeout is the lesser of the configured maximum, original submit
    /// deadline and ingress expiry. That timeout spans connection and body reads;
    /// absolute time is rechecked before returning authenticated evidence.
    ///
    /// Only HTTP 200 CBOR containing a certificate can settle. Raw body accumulation
    /// is capped at the original result allowance before each copy; compressed
    /// responses refuse. Original certificate/result limits still apply after decoding.
    /// Headers and internal network buffers retain the HTTP supplier's bounds.
    /// No redirects, proxies, connection reuse, decompression, retries, polling,
    /// root fetching, re-signing or detached recovery occurs. Cancellation leaves
    /// original spending intact; the server may still execute an accepted request.
    ///
    ///
    /// # Errors
    /// Refuses invalid bindings, exhausted time/bytes, nonterminal or malformed
    /// responses and failed certificate authentication. Errors never refund spending.
    ///
    /// # Panics
    /// The future requires a Tokio runtime with IO and timers enabled, as required
    /// by the selected HTTP client. Polling without that runtime can panic.
    pub(in crate::store) async fn submit_once(
        &self,
        plan: &CapturePolicy,
        admission: &StatusAdmissionRecord,
        prepared: &PreparedStatus,
    ) -> Result<StatusEvidence, BackupHttpError> {
        self.submit_call(plan, admission, prepared, CaptureCall::Status)
            .await
    }

    async fn submit_call(
        &self,
        plan: &CapturePolicy,
        admission: &StatusAdmissionRecord,
        prepared: &PreparedStatus,
        call: CaptureCall,
    ) -> Result<StatusEvidence, BackupHttpError> {
        let arguments = call.arguments(plan).map_err(BackupHttpError::Admission)?;

        self.submit_expected(
            crate::store::capture::advance::IngressRequirements {
                target: plan.request().target,
                sender: plan.request().sender,
                recipient: ic_platform_types::Principal::management_canister(),
                trust_root: plan.request().trust_root,
            },
            admission,
            prepared,
            call.method_name(),
            &arguments,
        )
        .await
    }
}
