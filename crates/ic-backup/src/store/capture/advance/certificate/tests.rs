//! Certificate evidence builders and checks used by capture regressions.

use super::{
    super::{CaptureCall, request::tests::authenticate_call_request},
    *,
};
use crate::store::{StatusAdmissionRecord, capture::CapturePolicy};

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
///
/// # Errors
/// Refuses invalid original admission, failed authentication, unavailable outcomes
/// and responses that cannot fit the original complete result/proof allowance.
pub fn status_certificate_evidence(
    plan: &CapturePolicy,
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
        CaptureCall::Status,
    )
}

pub(in crate::store::capture::advance) fn call_certificate_evidence(
    plan: &CapturePolicy,
    admission: &StatusAdmissionRecord,
    prepared: &PreparedStatus,
    certificate: &[u8],
    root_key_der: &[u8],
    call: CaptureCall,
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
/// This is a local authentication primitive used by the owning operation.
///
/// Original request/proof custody is required; freely constructed arguments do not
/// establish it. Success grants no new effect, rollback claim or retry allowance.
/// The workflow separately checks successful Candid status replies. Certified
/// rejections leave original spending intact, including transient reject codes.
/// Missing/pruned evidence and pending/forgotten outcomes refuse. No clock, network,
/// cache or alternative proof is used. Diagnostic bytes are never included in errors.
///
/// Certificate/delegation caps are 2 MiB / 1 MiB, combined trees 8192 nodes, depth 64
/// and selected canister ranges 1024. Complete evidence obeys original admission;
/// these limits are not allocation-quota units.
///
///
/// # Errors
/// Refuses changed bindings, invalid framing/time policy, failed Platform
/// authentication, unsupported outcomes and unequal or unavailable responses.
pub fn authenticate_status_evidence(
    plan: &CapturePolicy,
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
        CaptureCall::Status,
    )
}

fn authenticate_call_evidence(
    plan: &CapturePolicy,
    admission: &StatusAdmissionRecord,
    prepared: &PreparedStatus,
    evidence: &StatusEvidence,
    root_key_der: &[u8],
    call: CaptureCall,
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

fn authenticate_expected_evidence(
    plan: &CapturePolicy,
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
    plan: &CapturePolicy,
    admission: &StatusAdmissionRecord,
    prepared: &PreparedStatus,
    certificate: &[u8],
    root_key_der: &[u8],
    call: CaptureCall,
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

fn authenticated_expected_response(
    plan: &CapturePolicy,
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
    plan: &CapturePolicy,
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
    authenticate_call_request(plan, admission, prepared, call)?;
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
