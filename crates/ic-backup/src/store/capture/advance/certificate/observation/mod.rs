//! Pure original-request certificate observation. No lookup, reservation or settlement.

#[cfg(test)]
use super::super::CaptureCall;
#[cfg(test)]
use super::CapturePolicy;
use super::{BackupDigest, LookupResult, StatusCertificateError, TimePolicy};
#[cfg(test)]
use super::{PreparedStatus, StatusAdmissionRecord};
use crate::store::StatusRequestError;
use thiserror::Error;

/// Separately admitted observation limits, not replacement authority for the original call.
/// A workflow must retain these exact values before performing a remote observation.
#[derive(Clone, Copy, Debug)]
pub struct CaptureObservationPolicy {
    /// Explicit observation-certificate and delegation windows; no clock is read here.
    pub time: TimePolicy,
    /// Combined certificate, projected reply and five-byte outcome framing ceiling.
    /// Must be nonzero and no larger than the original call's result allowance.
    pub evidence_bytes: u32,
}

/// Authenticated view of one original request at the admitted observation time.
///
/// This is not a receipt or dispatch capability. The caller retains the exact input
/// certificate and original observation authority separately. Raw replies/diagnostics
/// are deliberately not exposed through Debug. No variant refunds original spending.
pub enum CaptureObservation {
    /// Exact original request reply. Application policy still needs validation.
    Replied(Vec<u8>),
    /// Original certified rejection in the existing bounded Backup diagnostic framing.
    Rejected(Vec<u8>),
    /// Request was received; no terminal outcome is available.
    Received,
    /// Request is processing; no terminal outcome is available.
    Processing,
    /// Certified done marker, with the original result no longer available.
    Done,
    /// The exact status path is certified absent; this does not prove non-execution.
    Absent,
    /// The exact status path is pruned/unknown; this is not certified absence.
    Unknown,
}

/// Refused original-request observation; no failure permits another management call.
#[derive(Debug, Error)]
pub enum CaptureObservationError {
    /// The original exact signed call, sender, framing or admission is invalid.
    #[error(transparent)]
    Request(#[from] StatusRequestError),
    /// Observation bounds, original trust, certificate or outcome fields refused.
    #[error(transparent)]
    Certificate(#[from] StatusCertificateError),
}

/// Authenticate a certificate for the original call under a separate observation policy.
///
/// Verifies the original direct-key signature and exact call content, then authenticates
/// the certificate with the plan's original root/effective target and the supplied
/// observation windows. Only `request_status/<original request ID>` is inspected.
/// Original request bytes, expiry and retained certificate policy remain untouched.
/// Reply and rejection projection share the original direct-call certificate decoder.
/// Unsupported status values, malformed rejection fields and malformed lookup trees
/// refuse; missing/pruned/pending/done states never become a success or retry decision.
///
/// This is a local provider helper, not the remote recovery workflow. The caller must
/// separately prove observation authority, finite prior reservations, fresh admission,
/// read-state request binding and retention of exact certificate/policy bytes. Passing
/// new windows here cannot widen the original `settle_capture_result` acceptance.
/// Neither a clock nor network, signer, application guard or journal is accessed.
///
///
/// # Errors
/// Refuses wrong call/sender/signature, malformed original policy, invalid observation
/// bounds/windows, wrong root/delegation/target, malformed or unauthenticated certificates,
/// unknown status labels and unavailable or oversized terminal fields.
#[cfg(test)]
pub fn observe_capture_certificate(
    plan: &CapturePolicy,
    admission: &StatusAdmissionRecord,
    prepared: &PreparedStatus,
    call: CaptureCall,
    certificate: &[u8],
    root_key_der: &[u8],
    policy: CaptureObservationPolicy,
) -> Result<CaptureObservation, CaptureObservationError> {
    // Check limits before signature or certificate decoding/allocation.
    let maximum = policy.evidence_bytes as usize;
    if maximum == 0
        || policy.evidence_bytes > admission.result_bytes
        || certificate.len().checked_add(5).is_none_or(|n| n > maximum)
    {
        return Err(
            StatusCertificateError::Admission(super::OperationStoreError::StatusAdmission).into(),
        );
    }
    super::super::request::authenticate_call_request(plan, admission, prepared, call)?;
    // Retain original policy validity, but never replace it with observation time.
    super::observation(&prepared.observation)?;
    observe_authenticated_request(
        &super::CertificateRequirements {
            canister: plan.request().target.canister,
            trust_root: plan.request().trust_root,
        },
        prepared.request_id,
        certificate,
        root_key_der,
        policy,
    )
}

// Signature and exact derived call admission remain with the original request owner.
pub(in crate::store) fn observe_authenticated_request(
    authority: &super::CertificateRequirements,
    request_id: [u8; 32],
    certificate: &[u8],
    root_key_der: &[u8],
    policy: CaptureObservationPolicy,
) -> Result<CaptureObservation, CaptureObservationError> {
    let maximum = policy.evidence_bytes as usize;
    if maximum < 5 || certificate.len().checked_add(5).is_none_or(|n| n > maximum) {
        return Err(StatusCertificateError::Outcome.into());
    }
    super::validate_window(policy.time.certificate)?;
    super::validate_window(policy.time.delegation)?;
    if root_key_der.len() != 133 || BackupDigest::compute(root_key_der) != authority.trust_root {
        return Err(StatusCertificateError::TrustRoot.into());
    }
    let authenticated =
        super::authenticate(certificate, root_key_der, authority.canister, policy.time)?;
    let tree = authenticated.tree();
    let path = [b"request_status".as_slice(), &request_id, b"status"];
    Ok(match tree.lookup_path(path) {
        LookupResult::Found(b"replied" | b"rejected") => {
            let (rejected, response) =
                super::terminal_response(tree, &request_id, maximum - certificate.len() - 5)?;
            if rejected {
                CaptureObservation::Rejected(response)
            } else {
                CaptureObservation::Replied(response)
            }
        }
        LookupResult::Found(b"received") => CaptureObservation::Received,
        LookupResult::Found(b"processing") => CaptureObservation::Processing,
        LookupResult::Found(b"done") => CaptureObservation::Done,
        LookupResult::Absent => CaptureObservation::Absent,
        LookupResult::Unknown => CaptureObservation::Unknown,
        LookupResult::Found(_) | LookupResult::Error => {
            return Err(StatusCertificateError::Outcome.into());
        }
    })
}
