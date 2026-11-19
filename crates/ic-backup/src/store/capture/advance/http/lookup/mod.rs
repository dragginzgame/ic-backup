//! One externally reserved read-state dispatch through the existing HTTP owner.

use super::{BackupHttpError, BackupHttpTransport, remaining};
#[cfg(test)]
use super::{CapturePolicy, PreparedStatus, StatusAdmissionRecord};
#[cfg(test)]
use crate::store::capture::advance::{
    CaptureCall, CaptureLookupError, CaptureLookupPolicy, CaptureObservation,
    CaptureObservationError, CaptureObservationPolicy, authenticate_capture_lookup,
    observe_capture_certificate, status_certificate_observation,
};
use ic_platform::ingress::decode_read_state_response;
#[cfg(test)]
use thiserror::Error;

/// Separately approved lookup bounds. These fields do not reserve any authority.
#[derive(Clone, Copy, Debug)]
#[cfg(test)]
pub struct CaptureLookupHttpPolicy {
    /// Exact signed lookup expiry and raw envelope cap.
    pub request: CaptureLookupPolicy,
    /// Explicit certificate/delegation windows and combined proof/result cap.
    /// The same cap also bounds the complete encoded HTTP response body.
    pub observation: CaptureObservationPolicy,
    /// Absolute Unix nanosecond deadline for connection, body and authentication.
    pub deadline_ns: u64,
}

/// Exact authenticated certificate and its original-request projection, not settlement.
/// Contains sensitive bytes and deliberately has no Debug implementation.
#[cfg(test)]
pub struct CaptureLookupEvidence {
    /// Exact original certificate bytes, retained by the caller for historical replay.
    pub certificate: Vec<u8>,
    /// Original request's state; absent/done/unknown never prove non-execution.
    pub outcome: CaptureObservation,
}

/// Lookup refusal or uncertain observation; no variant grants another attempt.
#[derive(Debug, Error)]
#[cfg(test)]
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
    #[cfg(test)]
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

    // The owning operation admits the exact signed lookup and durable reservation.
    pub(in crate::store) async fn read_state_certificate(
        &self,
        canister: ic_platform_types::Principal,
        lookup: &[u8],
        request_id: [u8; 32],
        deadline: u64,
        maximum: usize,
    ) -> Result<Vec<u8>, BackupHttpError> {
        let mut url = self.origin.clone();
        url.set_path(&format!(
            "/api/v2/canister/{}/read_state",
            canister.to_text()
        ));
        let bytes = self
            .post_cbor(url, lookup.to_vec(), deadline, maximum)
            .await?;
        let certificate = decode_read_state_response(&bytes, maximum)
            .map_err(|_| BackupHttpError::Response)?
            .value
            .certificate;
        remaining(deadline)
            .map_err(|error| error.after_response(request_id, certificate.clone()))?;
        Ok(certificate)
    }
}
