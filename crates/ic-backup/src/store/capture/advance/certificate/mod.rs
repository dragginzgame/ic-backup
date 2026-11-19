//! Local authentication of original status outcomes; no transport or clock.

mod observation;
#[cfg(test)]
pub(super) mod tests;

use super::{OperationStoreError, PreparedStatus, StatusAdmissionRecord, StatusEvidence};
use crate::store::BackupDigest;
use ic_platform::certificate::{
    self, AuthenticatedCertificate, CertificateDecodeLimits, DecodeError, PreparedRootKey,
    TimePolicy, TimeWindow, VerificationLimits, VerifyError,
    construction::{HashTree, LookupResult},
};
pub(in crate::store) use observation::observe_authenticated_request;
pub use observation::{CaptureObservation, CaptureObservationError, CaptureObservationPolicy};
use thiserror::Error;

const OBSERVATION: &[u8] = b"ic-backup/status-certificate/1\n";
const LIMITS: VerificationLimits = VerificationLimits {
    delegation_bytes: 1024 * 1024,
    tree_nodes: 8192,
    tree_depth: 64,
    canister_ranges: 1024,
};

/// Refused historical status authentication. No failure grants retry authority.
#[derive(Debug, Error)]
pub enum StatusCertificateError {
    /// Original plan, call or complete evidence framing failed admission.
    #[error("status certificate evidence fails original admission")]
    Admission(#[source] OperationStoreError),
    /// Missing, malformed or arithmetically invalid original time policy.
    #[error("invalid original status certificate observation")]
    Observation,
    /// The supplied DER key does not match the original plan's root commitment.
    #[error("status certificate root differs from original trust")]
    TrustRoot,
    /// Platform refused bounded certificate authentication.
    #[error("status certificate authentication failed")]
    Certificate(#[source] VerifyError),
    /// Platform refused the bounded complete certificate encoding.
    #[error("status certificate decoding failed")]
    Decode(#[source] DecodeError),
    /// No supported certified terminal outcome matches the retained outcome flag.
    #[error("original request has no matching certified terminal outcome")]
    Outcome,
    /// The retained response differs from the certified outcome or its fields are unavailable.
    #[error("status response differs from the certified outcome")]
    Response,
    /// A rejection diagnostic is unavailable, malformed or outside the direct-ingress profile.
    #[error("unsupported or incomplete certified rejection")]
    Rejection,
}

/// Encode the time policy to retain in [`PreparedStatus::observation`] before submission.
///
/// The integration admits both original windows, including an explicitly selected
/// delegation age. This helper neither reads a clock nor chooses a freshness policy.
/// The original request evidence commits these bytes; replay never replaces them
/// with current configuration or derives them from an untrusted certificate.
///
/// # Errors
/// Refuses underflow or overflow in either inclusive time window.
pub fn status_certificate_observation(time: TimePolicy) -> Result<Vec<u8>, StatusCertificateError> {
    let mut bytes = OBSERVATION.to_vec();
    for window in [time.certificate, time.delegation] {
        validate_window(window)?;

        for value in [
            window.reference_ns,
            window.max_age_ns,
            window.max_future_skew_ns,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }

    Ok(bytes)
}

/// Original certificate authority only; callers admit request identity and full frame bounds.
pub(in crate::store) struct CertificateRequirements {
    pub canister: ic_platform_types::Principal,
    pub trust_root: BackupDigest,
}
impl CertificateRequirements {
    pub(in crate::store) fn authenticate(
        &self,
        admission: &StatusAdmissionRecord,
        prepared: &PreparedStatus,
        evidence: &StatusEvidence,
        root_key_der: &[u8],
    ) -> Result<(), StatusCertificateError> {
        let (rejected, response) =
            self.response(admission, prepared, &evidence.proof, root_key_der)?;
        if rejected != evidence.rejected {
            return Err(StatusCertificateError::Outcome);
        }
        if response != evidence.response {
            return Err(StatusCertificateError::Response);
        }

        Ok(())
    }

    pub(in crate::store) fn response(
        &self,
        admission: &StatusAdmissionRecord,
        prepared: &PreparedStatus,
        certificate: &[u8],
        root_key_der: &[u8],
    ) -> Result<(bool, Vec<u8>), StatusCertificateError> {
        if root_key_der.len() != 133 || BackupDigest::compute(root_key_der) != self.trust_root {
            return Err(StatusCertificateError::TrustRoot);
        }
        let maximum = (admission.result_bytes as usize)
            .checked_sub(certificate.len())
            .and_then(|n| n.checked_sub(5))
            .ok_or(StatusCertificateError::Admission(
                OperationStoreError::StatusAdmission,
            ))?;
        let authenticated = authenticate(
            certificate,
            root_key_der,
            self.canister,
            observation(&prepared.observation)?,
        )?;

        terminal_response(authenticated.tree(), &prepared.request_id, maximum)
    }
}

fn authenticate<'a>(
    bytes: &'a [u8],
    root: &[u8],
    canister: ic_platform_types::Principal,
    time: TimePolicy,
) -> Result<AuthenticatedCertificate<'a>, StatusCertificateError> {
    let root = PreparedRootKey::from_der(root).map_err(StatusCertificateError::Certificate)?;
    let decoded = certificate::decode_certificate(
        bytes,
        CertificateDecodeLimits {
            bytes: crate::store::limits::CERTIFICATE_BYTES,
        },
    )
    .map_err(StatusCertificateError::Decode)?;

    certificate::authenticate_certificate(decoded, &root, canister, time, LIMITS)
        .map_err(StatusCertificateError::Certificate)
}

fn terminal_response(
    tree: &HashTree,
    request_id: &[u8; 32],
    maximum: usize,
) -> Result<(bool, Vec<u8>), StatusCertificateError> {
    let path = [b"request_status".as_slice(), request_id, b"status"];

    match tree.lookup_path(path) {
        LookupResult::Found(b"replied") => {
            let reply = required_leaf(tree, request_id, b"reply")
                .ok_or(StatusCertificateError::Response)?;
            if reply.is_empty() || reply.len() > maximum {
                return Err(StatusCertificateError::Admission(
                    OperationStoreError::StatusAdmission,
                ));
            }

            Ok((false, reply.to_vec()))
        }
        LookupResult::Found(b"rejected") => {
            rejection(tree, request_id, maximum).map(|bytes| (true, bytes))
        }
        _ => Err(StatusCertificateError::Outcome),
    }
}

fn required_leaf<'a>(tree: &'a HashTree, id: &[u8; 32], name: &[u8]) -> Option<&'a [u8]> {
    match tree.lookup_path([b"request_status".as_slice(), id, name]) {
        LookupResult::Found(bytes) => Some(bytes),
        _ => None,
    }
}

fn rejection(
    tree: &HashTree,
    id: &[u8; 32],
    maximum: usize,
) -> Result<Vec<u8>, StatusCertificateError> {
    const FORMAT: &[u8] = b"ic-backup/status-rejection/1\n";
    let code = required_leaf(tree, id, b"reject_code").ok_or(StatusCertificateError::Rejection)?;
    // Direct ingress admits only the canonical one-byte encodings of codes 1..=5.
    // SYS_UNKNOWN is inter-canister-only. No general external natural codec or enum.
    if !matches!(code, [1..=5]) {
        return Err(StatusCertificateError::Rejection);
    }
    let message =
        required_leaf(tree, id, b"reject_message").ok_or(StatusCertificateError::Rejection)?;
    let error = match tree.lookup_path([b"request_status".as_slice(), id, b"error_code"]) {
        LookupResult::Found(bytes) => Some(bytes),
        LookupResult::Absent => None,
        LookupResult::Unknown | LookupResult::Error => {
            return Err(StatusCertificateError::Rejection);
        }
    };
    if std::str::from_utf8(message).is_err()
        || error.is_some_and(|s| std::str::from_utf8(s).is_err())
    {
        return Err(StatusCertificateError::Rejection);
    }
    // All leaves come from a <=2 MiB admitted certificate. Check complete response
    // size before allocation; the optional tag preserves absent versus present-empty.
    let length = FORMAT.len() + 1 + 4 + message.len() + 1 + error.map_or(0, |s| 4 + s.len());
    if length > maximum {
        return Err(StatusCertificateError::Admission(
            OperationStoreError::StatusAdmission,
        ));
    }
    let mut bytes = Vec::with_capacity(length);
    bytes.extend_from_slice(FORMAT);
    bytes.extend_from_slice(code);
    bytes.extend_from_slice(&(message.len() as u32).to_le_bytes());
    bytes.extend_from_slice(message);
    bytes.push(u8::from(error.is_some()));
    if let Some(error) = error {
        bytes.extend_from_slice(&(error.len() as u32).to_le_bytes());
        bytes.extend_from_slice(error);
    }

    Ok(bytes)
}

pub(in crate::store) fn observation(bytes: &[u8]) -> Result<TimePolicy, StatusCertificateError> {
    let values = bytes
        .strip_prefix(OBSERVATION)
        .filter(|bytes| bytes.len() == 48)
        .ok_or(StatusCertificateError::Observation)?;
    let mut decoded = [0; 6];
    for (value, bytes) in decoded.iter_mut().zip(values.as_chunks::<8>().0) {
        *value = u64::from_le_bytes(*bytes);
    }
    let certificate = TimeWindow {
        reference_ns: decoded[0],
        max_age_ns: decoded[1],
        max_future_skew_ns: decoded[2],
    };
    let delegation = TimeWindow {
        reference_ns: decoded[3],
        max_age_ns: decoded[4],
        max_future_skew_ns: decoded[5],
    };
    validate_window(certificate)?;
    validate_window(delegation)?;

    Ok(TimePolicy {
        certificate,
        delegation,
    })
}

fn validate_window(window: TimeWindow) -> Result<(), StatusCertificateError> {
    if window.reference_ns.checked_sub(window.max_age_ns).is_none()
        || window
            .reference_ns
            .checked_add(window.max_future_skew_ns)
            .is_none()
    {
        return Err(StatusCertificateError::Observation);
    }

    Ok(())
}
