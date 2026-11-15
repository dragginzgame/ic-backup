//! Local authentication of original status outcomes; no transport or clock.
use super::{CaptureError, CapturePlan, PreparedStatus, StatusAdmissionRecord, StatusEvidence};
use ic_host_artifacts::artifact::Sha256Digest;
use ic_platform::certification::{
    HashTree, LookupResult,
    historical::{self, Limits, TimePolicy, TimeWindow},
};
use thiserror::Error;

const OBSERVATION: &[u8] = b"ic-backup/status-certificate/1\n";
const LIMITS: Limits = Limits {
    certificate_bytes: 1024 * 1024,
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
    Admission(#[source] CaptureError),
    /// Missing, malformed or arithmetically invalid original time policy.
    #[error("invalid original status certificate observation")]
    Observation,
    /// The supplied DER key does not match the original plan's root commitment.
    #[error("status certificate root differs from original trust")]
    TrustRoot,
    /// Platform refused bounded certificate authentication.
    #[error("status certificate authentication failed")]
    Certificate(#[source] historical::Error),
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

/// Build bounded evidence from an original certificate under retained request authority.
///
/// This local helper is for a provider receiving a certificate, including certified
/// rejections. It authenticates before projecting the response and retains exact
/// certificate bytes. It never submits, polls, retries or changes an operation.
/// Successful responses retain the raw reply. Rejections use Backup's framed
/// diagnostic representation; callers need not implement that representation.
/// Direct ingress accepts canonical single-byte reject codes 1 through 5 and
/// exact UTF-8 messages. Unknown, overlong and inter-canister-only codes refuse.
/// Optional error codes require either a certified leaf or certified absence;
/// pruned/unknown evidence cannot be treated as absence.
///
/// The original time policy must already be retained through
/// [`status_certificate_observation`]. Root/network admission, signed-envelope
/// authentication, current authority and application continuity remain provider-owned.
///
/// ```no_run
/// use ic_backup::store::{CapturePlan, PreparedStatus, StatusAdmissionRecord,
///     StatusEvidence, StatusCertificateError, status_certificate_evidence};
///
/// fn retainable_result(
///     plan: &CapturePlan, admission: &StatusAdmissionRecord,
///     request: &PreparedStatus, certificate: &[u8], root_der: &[u8],
/// ) -> Result<StatusEvidence, StatusCertificateError> {
///     status_certificate_evidence(plan, admission, request, certificate, root_der)
/// }
/// ```
///
/// # Errors
/// Refuses invalid original admission, failed authentication, unavailable outcomes
/// and responses that cannot fit the original complete result/proof allowance.
pub fn status_certificate_evidence(
    plan: &CapturePlan,
    admission: &StatusAdmissionRecord,
    prepared: &PreparedStatus,
    certificate: &[u8],
    root_key_der: &[u8],
) -> Result<StatusEvidence, StatusCertificateError> {
    call_certificate_evidence(
        plan,
        admission,
        prepared,
        certificate,
        root_key_der,
        super::CaptureCall::Status,
    )
}

pub(super) fn call_certificate_evidence(
    plan: &CapturePlan,
    admission: &StatusAdmissionRecord,
    prepared: &PreparedStatus,
    certificate: &[u8],
    root_key_der: &[u8],
    call: super::CaptureCall,
) -> Result<StatusEvidence, StatusCertificateError> {
    let (rejected, response) =
        authenticated_response(plan, admission, prepared, certificate, root_key_der, call)?;
    let evidence = StatusEvidence {
        rejected,
        response,
        proof: certificate.to_vec(),
    };
    evidence
        .encode(admission.result_bytes)
        .map_err(StatusCertificateError::Admission)?;
    Ok(evidence)
}

/// Authenticate retained status evidence against its original request.
///
/// Uses the same bounded projection as [`status_certificate_evidence`], checking
/// the outcome flag and every retained response byte. The DER root must hash to
/// the approved plan's trust-root commitment. Platform authenticates signatures,
/// delegated canister authority and both retained time windows. Backup binds the
/// exact request-status path, reply or rejection diagnostics to the admitted call.
/// This is a local helper for [`super::StatusProvider::authenticate`], not a provider.
///
/// Original request/proof custody is required; freely constructed arguments do not
/// establish it. Success grants no new effect, rollback claim or retry allowance.
/// The workflow separately checks successful Candid status replies. Certified
/// rejections leave original spending intact, including transient reject codes.
/// Missing/pruned evidence and pending/forgotten outcomes refuse. No clock, network,
/// cache or alternative proof is used. Diagnostic bytes are never included in errors.
///
/// Certificate/delegation caps are 1 MiB each, combined trees 8192 nodes, depth 64
/// and selected canister ranges 1024. Complete evidence obeys original admission;
/// these limits are not allocation-quota units.
///
/// ```no_run
/// use ic_backup::store::{CapturePlan, PreparedStatus, StatusAdmissionRecord,
///     StatusEvidence, StatusCertificateError, authenticate_status_evidence};
///
/// fn authenticate_retained(
///     plan: &CapturePlan, admission: &StatusAdmissionRecord,
///     request: &PreparedStatus, result: &StatusEvidence, root_der: &[u8],
/// ) -> Result<(), StatusCertificateError> {
///     authenticate_status_evidence(plan, admission, request, result, root_der)
/// }
/// ```
///
/// # Errors
/// Refuses changed bindings, invalid framing/time policy, failed Platform
/// authentication, unsupported outcomes and unequal or unavailable responses.
pub fn authenticate_status_evidence(
    plan: &CapturePlan,
    admission: &StatusAdmissionRecord,
    prepared: &PreparedStatus,
    evidence: &StatusEvidence,
    root_key_der: &[u8],
) -> Result<(), StatusCertificateError> {
    authenticate_call_evidence(
        plan,
        admission,
        prepared,
        evidence,
        root_key_der,
        super::CaptureCall::Status,
    )
}

pub(super) fn authenticate_call_evidence(
    plan: &CapturePlan,
    admission: &StatusAdmissionRecord,
    prepared: &PreparedStatus,
    evidence: &StatusEvidence,
    root_key_der: &[u8],
    call: super::CaptureCall,
) -> Result<(), StatusCertificateError> {
    authenticate_expected_evidence(
        plan,
        admission,
        prepared,
        evidence,
        root_key_der,
        call.method_name(),
        &call
            .arguments(plan)
            .map_err(StatusCertificateError::Admission)?,
    )
}

pub(in crate::store::capture) fn authenticate_expected_evidence(
    plan: &CapturePlan,
    admission: &StatusAdmissionRecord,
    prepared: &PreparedStatus,
    evidence: &StatusEvidence,
    root_key_der: &[u8],
    method: &str,
    arguments: &[u8],
) -> Result<(), StatusCertificateError> {
    admission
        .validate(plan)
        .map_err(StatusCertificateError::Admission)?;
    evidence
        .encode(admission.result_bytes)
        .map_err(StatusCertificateError::Admission)?;
    validate_certificate_request(plan, admission, prepared, method, arguments)?;
    CertificateRequirements {
        canister: plan.request().target.canister,
        trust_root: plan.request().trust_root,
    }
    .authenticate(admission, prepared, evidence, root_key_der)
}

fn authenticated_response(
    plan: &CapturePlan,
    admission: &StatusAdmissionRecord,
    prepared: &PreparedStatus,
    certificate: &[u8],
    root_key_der: &[u8],
    call: super::CaptureCall,
) -> Result<(bool, Vec<u8>), StatusCertificateError> {
    authenticated_expected_response(
        plan,
        admission,
        prepared,
        certificate,
        root_key_der,
        call.method_name(),
        &call
            .arguments(plan)
            .map_err(StatusCertificateError::Admission)?,
    )
}

pub(super) fn authenticated_expected_response(
    plan: &CapturePlan,
    admission: &StatusAdmissionRecord,
    prepared: &PreparedStatus,
    certificate: &[u8],
    root_key_der: &[u8],
    method: &str,
    arguments: &[u8],
) -> Result<(bool, Vec<u8>), StatusCertificateError> {
    validate_certificate_request(plan, admission, prepared, method, arguments)?;
    CertificateRequirements {
        canister: plan.request().target.canister,
        trust_root: plan.request().trust_root,
    }
    .response(admission, prepared, certificate, root_key_der)
}

fn validate_certificate_request(
    plan: &CapturePlan,
    admission: &StatusAdmissionRecord,
    prepared: &PreparedStatus,
    method: &str,
    arguments: &[u8],
) -> Result<(), StatusCertificateError> {
    admission
        .validate(plan)
        .map_err(StatusCertificateError::Admission)?;
    // Reuse original framing policy before inspecting the retained signed call.
    prepared
        .encode(admission.request_bytes)
        .map_err(StatusCertificateError::Admission)?;
    prepared
        .validate_expected_call(plan, method, arguments)
        .map_err(StatusCertificateError::Admission)?;
    Ok(())
}

/// Original certificate authority only; callers admit request identity and full frame bounds.
pub(in crate::store) struct CertificateRequirements {
    pub canister: ic_platform::Principal,
    pub trust_root: Sha256Digest,
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
    fn response(
        &self,
        admission: &StatusAdmissionRecord,
        prepared: &PreparedStatus,
        certificate: &[u8],
        root_key_der: &[u8],
    ) -> Result<(bool, Vec<u8>), StatusCertificateError> {
        if root_key_der.len() != 133 || Sha256Digest::compute(root_key_der) != self.trust_root {
            return Err(StatusCertificateError::TrustRoot);
        }
        let maximum = (admission.result_bytes as usize)
            .checked_sub(certificate.len())
            .and_then(|n| n.checked_sub(5))
            .ok_or(StatusCertificateError::Admission(
                CaptureError::StatusAdmission,
            ))?;
        let authenticated = historical::authenticate_certificate_at(
            certificate,
            root_key_der,
            self.canister,
            observation(&prepared.observation)?,
            LIMITS,
        )
        .map_err(StatusCertificateError::Certificate)?;
        terminal_response(
            &authenticated.certificate.tree,
            &prepared.request_id,
            maximum,
        )
    }
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
                    CaptureError::StatusAdmission,
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
    // All leaves come from a <=1 MiB admitted certificate. Check complete response
    // size before allocation; the optional tag preserves absent versus present-empty.
    let length = FORMAT.len() + 1 + 4 + message.len() + 1 + error.map_or(0, |s| 4 + s.len());
    if length > maximum {
        return Err(StatusCertificateError::Admission(
            CaptureError::StatusAdmission,
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

pub(super) fn observation(bytes: &[u8]) -> Result<TimePolicy, StatusCertificateError> {
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

mod observation;
pub use observation::{
    CaptureObservation, CaptureObservationError, CaptureObservationPolicy,
    observe_capture_certificate,
};
