//! Pure original-request certificate observation. No lookup, reservation or settlement.

use super::{BackupDigest, LookupResult, StatusCertificateError, TimePolicy};
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
