//! Initial status only. Providers are trusted integration boundaries, not IC emulators.
use super::{
    CaptureError, CaptureExecution, CaptureOperation, CapturePlan, CaptureStatusReplyError, ledger,
};
use ic_host_artifacts::artifact::Sha256Digest;
use ic_platform::{
    candid_codec,
    ic_protocol::{EnvelopeContent, decode_call_content_strict},
    management_canister_types::CanisterIdRecord,
};
use serde::{Deserialize, Serialize};
use std::future::Future;
use thiserror::Error;

mod certificate;
pub use certificate::{
    StatusCertificateError, authenticate_status_evidence, status_certificate_evidence,
    status_certificate_observation,
};

/// Original finite authority for the first status call, retained before preparation.
/// Bindings identify the provider protocol and continuous application maintenance witness.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatusAdmissionRecord {
    /// Exact integration/provider protocol commitment.
    pub provider: [u8; 32],
    /// Exact continuous-maintenance and external-obligation witness commitment.
    pub application: [u8; 32],
    /// Absolute preparation deadline in Unix nanoseconds; providers enforce it.
    pub prepare_deadline_ns: u64,
    /// Absolute submission deadline in Unix nanoseconds; providers enforce it.
    pub submit_deadline_ns: u64,
    /// Total network requests allowed during preparation, including external signing.
    pub prepare_requests: u32,
    /// Total network requests allowed during submission and response authentication.
    pub submit_requests: u32,
    /// Maximum retained request file size, including its framing (at most 1 MiB).
    pub request_bytes: u32,
    /// Maximum retained result/proof file size, including framing (at most 2 MiB).
    pub result_bytes: u32,
}
impl StatusAdmissionRecord {
    pub(super) fn validate(&self, plan: &CapturePlan) -> Result<(), CaptureError> {
        // Reserve all four maximum-size transition frames before any provider output.
        let evidence = u64::from(self.request_bytes)
            + u64::from(self.result_bytes)
            + ledger::initial(plan)?.len() as u64
            + 4 * (16 * 1024 + 36);
        let requests = self.prepare_requests.checked_add(self.submit_requests);
        if !matches!(plan.request().execution, CaptureExecution::Direct { .. })
            || self.prepare_deadline_ns == 0
            || self.submit_deadline_ns < self.prepare_deadline_ns
            || !(46..=1024 * 1024).contains(&self.request_bytes)
            || !(7..=2 * 1024 * 1024).contains(&self.result_bytes)
            || self.submit_requests == 0
            || requests.is_none_or(|n| n > plan.request().limits.network_requests)
            || evidence > plan.request().limits.evidence_bytes
        {
            return Err(CaptureError::StatusAdmission);
        }
        Ok(())
    }
}

/// Exact signed request and its original observation binding. Contains sensitive bytes.
/// Backup checks immutable call content; the provider must authenticate the envelope,
/// routing, observation binding and original admission.
pub struct PreparedStatus {
    /// Original ingress request identity.
    pub request_id: [u8; 32],
    /// Original expiry in Unix nanoseconds.
    pub expires_ns: u64,
    /// Exact envelope to submit without re-encoding or re-signing.
    pub envelope: Vec<u8>,
    /// Provider-owned bounded information needed to authenticate the original result.
    pub observation: Vec<u8>,
}
/// Raw original response and retained authentication proof. Contains sensitive bytes.
pub struct StatusEvidence {
    /// True only for an authenticated remote rejection of this exact request.
    pub rejected: bool,
    /// Raw Candid reply, or provider-defined original rejection bytes.
    pub response: Vec<u8>,
    /// Original authentication evidence, sufficient for historical local replay.
    pub proof: Vec<u8>,
}

/// Trusted direct-host status integration. No implementation is supplied or qualified here.
///
/// Admission and authentication are local, bounded, effect-free checks. Admission
/// verifies the exact network, trust root, sender, target, application continuity,
/// provider binding, deadlines and costs. Backup checks the exact status call content,
/// declared expiry and request ID through Platform, without re-encoding the envelope.
/// The provider still verifies signature/delegation authority, effective routing,
/// live expiry and the observation binding. Authentication verifies the retained original request/result association
/// and proof, including historical admission time, without imposing current freshness.
///
/// `prepare` invokes at most one signer and never submits management ingress.
/// `submit_once` sends the retained envelope once, without retry, polling, redirects,
/// fallback or re-signing. Both enforce admitted deadlines, network counts and byte
/// caps while receiving, before allocating unbounded output. Failures are uncertain.
/// Providers must be independently qualified; a successful predicate alone is not
/// proof of authentication or application safety. Never include secrets in errors.
pub trait StatusProvider {
    /// Provider-owned typed, redacted failure.
    type Error;
    /// Admit current authority and, when present, the exact retained signed request.
    fn admit(
        &self,
        plan: &CapturePlan,
        admission: &StatusAdmissionRecord,
        arguments: &[u8],
        prepared: Option<&PreparedStatus>,
    ) -> Result<(), Self::Error>;
    /// Prepare exactly this status call within its durable signing reservation.
    fn prepare(
        &mut self,
        plan: &CapturePlan,
        admission: &StatusAdmissionRecord,
        arguments: &[u8],
    ) -> impl Future<Output = Result<PreparedStatus, Self::Error>> + Send;
    /// Submit the original bytes within the durable submission reservation.
    fn submit_once(
        &mut self,
        plan: &CapturePlan,
        admission: &StatusAdmissionRecord,
        prepared: &PreparedStatus,
    ) -> impl Future<Output = Result<StatusEvidence, Self::Error>> + Send;
    /// Authenticate retained proof locally, including during cold settled replay.
    fn authenticate(
        &self,
        plan: &CapturePlan,
        admission: &StatusAdmissionRecord,
        prepared: &PreparedStatus,
        evidence: &StatusEvidence,
    ) -> Result<(), Self::Error>;
}

/// First-call result; none of these releases target ownership or completes a backup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusProgress {
    /// Original authenticated response meets the stopped/module/controller policy.
    Verified,
    /// Signing was reserved; its result is unresolved. Never automatically re-sign.
    PreparationPending,
    /// Submission was reserved; its result is unresolved. Never automatically resubmit.
    SubmissionPending,
    /// Retained authenticated rejection; original allowances remain spent.
    Rejected,
}
/// Status continuation failure. No error grants retry or fresh spending authority.
#[derive(Debug, Error)]
pub enum StatusAdvanceError<E> {
    /// Local original-custody or persistence failure.
    #[error(transparent)]
    Capture(#[from] CaptureError),
    /// Provider refusal or uncertain invocation; consult retained operation state.
    #[error("status provider refused or returned an uncertain outcome")]
    Provider(E),
    /// Retained authenticated response fails the capture policy or wire bounds.
    #[error(transparent)]
    Status(#[from] CaptureStatusReplyError),
}

pub(super) fn arguments(plan: &CapturePlan) -> Result<Vec<u8>, CaptureError> {
    candid_codec::encode_one(
        &CanisterIdRecord {
            canister_id: plan.request().target.canister,
        },
        256,
    )
    .map_err(|_| CaptureError::StatusAdmission)
}

impl CaptureOperation {
    /// Advance only the initial direct-host status call, using durable reservations.
    ///
    /// Cold pending reservations return attention without provider calls. A prepared
    /// request may submit only its original bytes after current admission. Settled
    /// replay performs local historical authentication and policy validation only.
    /// Prepared and settled calls must match the plan, exact arguments and request ID.
    /// Dropping the future never starts detached work or replenishes an allowance.
    /// No concrete provider, Canic executor, reconciliation or snapshot is implemented.
    ///
    /// ```no_run
    /// use ic_backup::store::{CaptureOperation, StatusAdmissionRecord, StatusProvider,
    ///     StatusProgress, StatusAdvanceError};
    ///
    /// async fn initial_status<P: StatusProvider>(
    ///     operation: &mut CaptureOperation, admission: &StatusAdmissionRecord, provider: &mut P,
    /// ) -> Result<StatusProgress, StatusAdvanceError<P::Error>> {
    ///     operation.advance_status(admission, provider).await
    /// }
    /// ```
    ///
    /// # Errors
    /// Refuses changed original authority/custody, unsupported Canic routing, exhausted
    /// bounds, uncertain persistence and provider/policy failures. Partial evidence
    /// remains retained. Reopen after uncertain writes; never reset the operation.
    pub async fn advance_status<P: StatusProvider>(
        &mut self,
        admission: &StatusAdmissionRecord,
        provider: &mut P,
    ) -> Result<StatusProgress, StatusAdvanceError<P::Error>> {
        use ledger::{CaptureStatusPhase, EventRecord};
        let mut state = self.check()?;
        admission.validate(&self.plan)?;
        if state
            .admission
            .as_ref()
            .is_some_and(|original| original != admission)
        {
            return Err(CaptureError::StatusAdmission.into());
        }
        match state.phase {
            CaptureStatusPhase::PreparationReserved => {
                return Ok(StatusProgress::PreparationPending);
            }
            CaptureStatusPhase::SubmissionReserved => return Ok(StatusProgress::SubmissionPending),
            CaptureStatusPhase::Settled => return self.status_result(provider, admission),
            CaptureStatusPhase::Created | CaptureStatusPhase::Prepared => {}
        }
        let args = arguments(&self.plan)?;
        if state.phase == CaptureStatusPhase::Created {
            provider
                .admit(&self.plan, admission, &args, None)
                .map_err(StatusAdvanceError::Provider)?;
            self.commit(
                &EventRecord::PreparationReserved {
                    sequence: 1,
                    admission: admission.clone(),
                    arguments_sha256: *Sha256Digest::compute(&args).as_bytes(),
                },
                None,
            )?;
            let prepared = provider
                .prepare(&self.plan, admission, &args)
                .await
                .map_err(StatusAdvanceError::Provider)?;
            self.check()?;
            let bytes = prepared.encode(admission.request_bytes)?;
            prepared.validate_call(&self.plan)?;
            provider
                .admit(&self.plan, admission, &args, Some(&prepared))
                .map_err(StatusAdvanceError::Provider)?;
            self.commit(
                &EventRecord::Prepared {
                    sequence: 2,
                    request: ledger::EvidenceRecord::new(&bytes),
                },
                Some(("status-request.bin", &bytes)),
            )?;
            state = self.check()?;
        }
        let prepared = PreparedStatus::decode(&self.evidence(
            "status-request.bin",
            state.request.as_ref().ok_or(CaptureError::Incomplete)?,
        )?)?;
        prepared.validate_call(&self.plan)?;
        provider
            .admit(&self.plan, admission, &args, Some(&prepared))
            .map_err(StatusAdvanceError::Provider)?;
        self.commit(&EventRecord::SubmissionReserved { sequence: 3 }, None)?;
        let evidence = provider
            .submit_once(&self.plan, admission, &prepared)
            .await
            .map_err(StatusAdvanceError::Provider)?;
        self.check()?;
        let bytes = evidence.encode(admission.result_bytes)?;
        provider
            .authenticate(&self.plan, admission, &prepared, &evidence)
            .map_err(StatusAdvanceError::Provider)?;
        self.commit(
            &EventRecord::Settled {
                sequence: 4,
                result: ledger::EvidenceRecord::new(&bytes),
            },
            Some(("status-result.bin", &bytes)),
        )?;
        self.status_result(provider, admission)
    }

    fn status_result<P: StatusProvider>(
        &self,
        provider: &P,
        admission: &StatusAdmissionRecord,
    ) -> Result<StatusProgress, StatusAdvanceError<P::Error>> {
        let state = self.check()?;
        let prepared = PreparedStatus::decode(&self.evidence(
            "status-request.bin",
            state.request.as_ref().ok_or(CaptureError::Incomplete)?,
        )?)?;
        prepared.validate_call(&self.plan)?;
        let evidence = StatusEvidence::decode(&self.evidence(
            "status-result.bin",
            state.result.as_ref().ok_or(CaptureError::Incomplete)?,
        )?)?;
        provider
            .authenticate(&self.plan, admission, &prepared, &evidence)
            .map_err(StatusAdvanceError::Provider)?;
        self.check()?;
        if evidence.rejected {
            return Ok(StatusProgress::Rejected);
        }
        self.plan.validate_status_reply(&evidence.response)?;
        Ok(StatusProgress::Verified)
    }
}

// Private exact binary evidence formats. Bounds are checked before concatenation;
// reopening reads at the committed length before parsing or allocating components.
impl PreparedStatus {
    // Inspect the original bytes; never normalize or re-encode a signed envelope.
    // Authentication metadata, routing, observation and freshness stay provider-owned.
    fn validate_call(&self, plan: &CapturePlan) -> Result<(), CaptureError> {
        let CaptureExecution::Direct { sender: expected } = plan.request().execution else {
            return Err(CaptureError::StatusAdmission);
        };
        let content = decode_call_content_strict(&self.envelope, 1024 * 1024)
            .map_err(|_| CaptureError::StatusAdmission)?;
        let EnvelopeContent::Call {
            ingress_expiry,
            sender,
            canister_id,
            method_name,
            arg,
            ..
        } = &content
        else {
            return Err(CaptureError::StatusAdmission);
        };
        let matches_status = sender.as_slice() == expected.as_slice()
            && canister_id.as_slice().is_empty()
            && method_name == "canister_status"
            && *arg == arguments(plan)?;
        if !matches_status
            || *ingress_expiry != self.expires_ns
            || *content.to_request_id() != self.request_id
        {
            return Err(CaptureError::StatusAdmission);
        }
        Ok(())
    }

    fn encode(&self, limit: u32) -> Result<Vec<u8>, CaptureError> {
        if self.expires_ns == 0 {
            return Err(CaptureError::StatusAdmission);
        }
        let tail = pair(&self.envelope, &self.observation, limit.saturating_sub(40))?;
        Ok([
            self.request_id.as_slice(),
            &self.expires_ns.to_le_bytes(),
            &tail,
        ]
        .concat())
    }
    fn decode(bytes: &[u8]) -> Result<Self, CaptureError> {
        if bytes.len() < 46 {
            return Err(CaptureError::Changed);
        }
        let (envelope, observation) = split(&bytes[40..])?;
        let expires_ns = u64::from_le_bytes(
            bytes[32..40]
                .try_into()
                .map_err(|_| CaptureError::Changed)?,
        );
        if expires_ns == 0 {
            return Err(CaptureError::Changed);
        }
        Ok(Self {
            request_id: bytes[..32].try_into().map_err(|_| CaptureError::Changed)?,
            expires_ns,
            envelope,
            observation,
        })
    }
}
impl StatusEvidence {
    fn encode(&self, limit: u32) -> Result<Vec<u8>, CaptureError> {
        let tail = pair(&self.response, &self.proof, limit.saturating_sub(1))?;
        Ok([&[u8::from(self.rejected)], tail.as_slice()].concat())
    }
    fn decode(bytes: &[u8]) -> Result<Self, CaptureError> {
        let Some((&kind, tail)) = bytes.split_first() else {
            return Err(CaptureError::Changed);
        };
        if kind > 1 {
            return Err(CaptureError::Changed);
        }
        let (response, proof) = split(tail)?;
        Ok(Self {
            rejected: kind == 1,
            response,
            proof,
        })
    }
}
fn pair(first: &[u8], second: &[u8], limit: u32) -> Result<Vec<u8>, CaptureError> {
    if first.is_empty()
        || second.is_empty()
        || first
            .len()
            .checked_add(second.len())
            .and_then(|n| n.checked_add(4))
            .is_none_or(|n| n > limit as usize)
    {
        return Err(CaptureError::StatusAdmission);
    }
    Ok([&(first.len() as u32).to_le_bytes(), first, second].concat())
}
fn split(bytes: &[u8]) -> Result<(Vec<u8>, Vec<u8>), CaptureError> {
    let header = bytes.get(..4).ok_or(CaptureError::Changed)?;
    let length = u32::from_le_bytes(header.try_into().map_err(|_| CaptureError::Changed)?) as usize;
    if length == 0 || length >= bytes.len().saturating_sub(4) {
        return Err(CaptureError::Changed);
    }
    Ok((bytes[4..4 + length].to_vec(), bytes[4 + length..].to_vec()))
}

#[cfg(test)]
mod tests;

/// Read-only projection of original status evidence and reserved allowances.
///
/// Counts are conservative reservations, not measured network traffic. Lost or
/// rejected results do not refund them. A settled stage is not an authentication
/// claim, a successful status predicate, backup completion or permission to release
/// the target. Signed envelopes and authentication-proof bytes are not exposed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureStatusView {
    /// Stage reconstructed from the complete admitted original ledger.
    pub phase: super::CaptureStatusPhase,
    /// Exact original admission, absent only before the first reservation.
    pub admission: Option<StatusAdmissionRecord>,
    /// Signing invocations already reserved under the original plan.
    pub signing_requests_reserved: u32,
    /// Network requests already reserved, including any external signing traffic.
    pub network_requests_reserved: u32,
    /// Original ingress identity when complete prepared evidence was retained.
    pub request_id: Option<[u8; 32]>,
}

impl CaptureOperation {
    /// Inspect retained initial-status progress without any provider or effect.
    ///
    /// This reads original custody and evidence, never signs, submits, authenticates,
    /// synchronizes or repairs. The returned admission lets a caller recover exact
    /// continuation inputs instead of reconstructing them from current configuration.
    /// `advance_status` still independently admits all authority before acting.
    ///
    /// ```no_run
    /// use ic_backup::store::{CaptureOperation, CaptureError, CaptureStatusPhase};
    ///
    /// fn report(operation: &CaptureOperation) -> Result<(), CaptureError> {
    ///     let status = operation.inspect_status()?;
    ///     if status.phase == CaptureStatusPhase::SubmissionReserved {
    ///         // Original submission is pending. Do not submit it again.
    ///         println!("pending ingress: {:?}", status.request_id);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    /// Refuses unusable sessions, partial/unknown history, missing or changed original
    /// evidence, malformed or plan-mismatched request content and changed closing custody. No partial
    /// view is returned and absent history never becomes unused authority.
    pub fn inspect_status(&self) -> Result<CaptureStatusView, CaptureError> {
        use super::CaptureStatusPhase;
        let state = self.check()?;
        let request_id = state
            .request
            .as_ref()
            .map(|reference| {
                let bytes = self.evidence("status-request.bin", reference)?;
                let prepared = PreparedStatus::decode(&bytes)?;
                prepared.validate_call(&self.plan)?;
                Ok::<_, CaptureError>(prepared.request_id)
            })
            .transpose()?;
        let signing_requests_reserved = u32::from(state.admission.is_some());
        let network_requests_reserved = match &state.admission {
            None => 0,
            Some(admission) => {
                let submitted = matches!(
                    state.phase,
                    CaptureStatusPhase::SubmissionReserved | CaptureStatusPhase::Settled
                );
                // Original admission checked this sum against the original finite plan.
                admission.prepare_requests
                    + if submitted {
                        admission.submit_requests
                    } else {
                        0
                    }
            }
        };
        if self.check()? != state {
            return Err(CaptureError::Changed);
        }
        Ok(CaptureStatusView {
            phase: state.phase,
            admission: state.admission,
            signing_requests_reserved,
            network_requests_reserved,
            request_id,
        })
    }
}
