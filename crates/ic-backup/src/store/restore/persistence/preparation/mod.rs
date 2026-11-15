//! One durable signing attempt; no submission or automatic signing retry.
use super::{
    CaptureError, Error, FileIdentity, PRIVATE, RestoreOperation, RestorePlan, Sha256Digest, Write,
    child, initial, open_optional, retain,
};
use crate::store::capture::ledger::{CREATED, EvidenceRecord, Frames, encode_frame};
use crate::store::{PreparedStatus, StatusAdmissionRecord, StatusRequestError, read_optional};
use ic_platform::{candid_codec, management_canister_types::CanisterIdRecord};
use serde::{Deserialize, Serialize};
use std::{
    future::Future,
    io::{Seek, SeekFrom},
    os::unix::fs::MetadataExt,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// External sign-only integration. No method may submit management ingress.
/// Admission is local/effect-free. Signing must enforce original output/network
/// ceilings, stop on cancellation and never retry or spawn detached work.
pub trait RestoreStatusSigner {
    /// Redacted integration-owned error.
    type Error;
    /// Exact signer protocol bound in the admission.
    fn protocol(&self) -> [u8; 32];
    /// Check current original sender/application custody; never perform network IO.
    fn admit(
        &self,
        plan: &RestorePlan,
        admission: &StatusAdmissionRecord,
    ) -> Result<(), Self::Error>;
    /// Sign only `canister_status` for the management canister with these exact arguments.
    /// One invocation is already spent. Preparation network traffic, including any
    /// external signing service, cannot exceed `admission.prepare_requests`.
    fn sign_status(
        &mut self,
        plan: &RestorePlan,
        admission: &StatusAdmissionRecord,
        arguments: &[u8],
    ) -> impl Future<Output = Result<PreparedStatus, Self::Error>> + Send;
}

/// Preparation progress, never submission or fresh retry permission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RestoreStatusPreparation {
    /// One original authenticated request is retained. No management ingress was submitted.
    Prepared,
    /// Signing was spent and its outcome is unresolved; never automatically sign again.
    SigningPending,
}
/// Read-only original reservation view; no raw signed bytes or effect authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RestoreStatusView {
    /// Exact original admission, absent before signing reservation.
    pub admission: Option<StatusAdmissionRecord>,
    /// Reserved signer invocations, zero or one.
    pub signing_requests_reserved: u32,
    /// Reserved preparation traffic; not measured usage and never refunded.
    pub network_requests_reserved: u32,
    /// Retained authenticated request ID, absent while unprepared/pending.
    pub request_id: Option<[u8; 32]>,
}
/// Preparation failure never grants another signing invocation.
#[derive(Debug, Error)]
pub enum RestoreStatusPreparationError<E> {
    /// Original local custody, budget or persistence refused.
    #[error(transparent)]
    Capture(#[from] CaptureError),
    /// Integration refused or returned an uncertain signing outcome.
    #[error("restore status signer refused or has an uncertain outcome")]
    Signer(E),
    /// Signer output failed exact request/signature admission.
    #[error(transparent)]
    Request(#[from] StatusRequestError),
    /// Preparation expired, exceeded 300 seconds, or the clock was unavailable.
    #[error("restore status preparation timing refused")]
    Timing,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct State {
    admission: Option<StatusAdmissionRecord>,
    pub(super) request: Option<EvidenceRecord>,
    head: Sha256Digest,
    bytes: u64,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Event {
    StatusSigningReserved { admission: StatusAdmissionRecord },
    StatusPrepared { request: EvidenceRecord },
}
fn validate(plan: &RestorePlan, a: &StatusAdmissionRecord) -> Result<(), CaptureError> {
    a.validate_limits(
        plan.request().limits.network_requests,
        plan.request().limits.evidence_bytes,
        initial(plan)?.len(),
    )
}
fn apply(state: &mut State, event: &Event, plan: &RestorePlan) -> Result<(), CaptureError> {
    match event {
        Event::StatusSigningReserved { admission } if state.admission.is_none() => {
            validate(plan, admission)?;
            state.admission = Some(admission.clone());
        }
        Event::StatusPrepared { request }
            if state.admission.is_some() && state.request.is_none() =>
        {
            let a = state.admission.as_ref().ok_or(CaptureError::Incomplete)?;
            if !(46..=a.request_bytes).contains(&request.bytes) {
                return Err(CaptureError::Changed);
            }
            state.request = Some(request.clone());
        }
        _ => return Err(CaptureError::Changed),
    }
    Ok(())
}
pub(super) fn read(op: &RestoreOperation<'_>) -> Result<State, CaptureError> {
    // Only Created, signing reservation and preparation exist in this reader.
    let limit = op
        .plan
        .request()
        .limits
        .evidence_bytes
        .min(3 * (16 * 1024 + 36));
    let bytes = read_optional(&op.directory, "ledger.bin", limit as usize)?
        .ok_or(CaptureError::Incomplete)?;
    let mut frames = Frames::for_original(bytes.as_slice(), op.plan.encoded(), limit);
    if frames.next()?.as_deref() != Some(CREATED) {
        return Err(CaptureError::Changed);
    }
    let mut state = State {
        admission: None,
        request: None,
        head: frames.head,
        bytes: frames.bytes,
    };
    while let Some(bytes) = frames.next()? {
        let event: Event = serde_json::from_slice(&bytes)?;
        if serde_json::to_vec(&event)? != bytes {
            return Err(CaptureError::Changed);
        }
        apply(&mut state, &event, &op.plan)?;
    }
    state.head = frames.head;
    state.bytes = frames.bytes;
    if let Some(reference) = &state.request {
        let prepared = request(op, reference)?;
        if prepared.expires_ns
            < state
                .admission
                .as_ref()
                .ok_or(CaptureError::Incomplete)?
                .submit_deadline_ns
        {
            return Err(CaptureError::Changed);
        }
        op.plan
            .authenticate_status_request(
                state.admission.as_ref().ok_or(CaptureError::Incomplete)?,
                &prepared,
            )
            .map_err(|_| CaptureError::Changed)?;
    }
    let current = open_optional(&op.directory, "ledger.bin")?.ok_or(CaptureError::Incomplete)?;
    if FileIdentity::from_file(&current)? != FileIdentity::from_file(&op.ledger_file)?
        || current.metadata()?.len() != state.bytes
    {
        return Err(CaptureError::Changed);
    }
    Ok(state)
}
fn request(
    op: &RestoreOperation<'_>,
    reference: &EvidenceRecord,
) -> Result<PreparedStatus, CaptureError> {
    let bytes = read_optional(
        &op.directory,
        "status-request.bin",
        reference.bytes as usize,
    )?
    .ok_or(CaptureError::Incomplete)?;
    if EvidenceRecord::new(&bytes) != *reference {
        return Err(CaptureError::Changed);
    }
    PreparedStatus::decode(&bytes)
}
fn remaining(deadline: u64) -> Option<Duration> {
    let now: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_nanos()
        .try_into()
        .ok()?;
    let ns = deadline.checked_sub(now)?;
    (ns > 0 && ns <= 300_000_000_000).then(|| Duration::from_nanos(ns))
}
impl RestoreOperation<'_> {
    /// Inspect original signing reservation and retained request without provider calls or writes.
    /// # Errors
    /// Refuses damaged/partial history, orphan evidence and changed original custody.
    pub fn inspect_status(&self) -> Result<RestoreStatusView, CaptureError> {
        let state = self.check()?;
        let request_id = state
            .request
            .as_ref()
            .map(|r| request(self, r).map(|p| p.request_id))
            .transpose()?;
        Ok(RestoreStatusView {
            signing_requests_reserved: u32::from(state.admission.is_some()),
            network_requests_reserved: state.admission.as_ref().map_or(0, |a| a.prepare_requests),
            admission: state.admission,
            request_id,
        })
    }
    /// Reserve one signer invocation and retain its exact authenticated status request.
    ///
    /// Pending replay never calls a signer. Prepared replay verifies original bytes
    /// locally without a clock or live guard. Fresh preparation admits current custody
    /// before reservation, bounds signing to the original deadline (at most 300 s),
    /// then retains the request create-only. Cancellation/failure stays spent. Signing
    /// requires a Tokio time driver. No status request is submitted or outcome inferred.
    /// # Errors
    /// Refuses changed admission/protocol, expired time, invalid output and uncertain
    /// persistence. Reopen after IO failure; preserve partial files without repair.
    pub async fn prepare_status<S: RestoreStatusSigner>(
        &mut self,
        a: &StatusAdmissionRecord,
        signer: &mut S,
    ) -> Result<RestoreStatusPreparation, RestoreStatusPreparationError<S::Error>> {
        let state = self.check()?;
        validate(&self.plan, a)?;
        if state
            .admission
            .as_ref()
            .is_some_and(|original| original != a)
        {
            return Err(CaptureError::StatusAdmission.into());
        }
        if state.request.is_some() {
            return Ok(RestoreStatusPreparation::Prepared);
        }
        if state.admission.is_some() {
            return Ok(RestoreStatusPreparation::SigningPending);
        }
        if signer.protocol() != a.provider {
            return Err(CaptureError::StatusAdmission.into());
        }
        remaining(a.prepare_deadline_ns).ok_or(RestoreStatusPreparationError::Timing)?;
        signer
            .admit(&self.plan, a)
            .map_err(RestoreStatusPreparationError::Signer)?;
        let args = candid_codec::encode_one(
            &CanisterIdRecord {
                canister_id: self.plan.request().target.canister,
            },
            512,
        )
        .map_err(|_| CaptureError::StatusAdmission)?;
        self.commit_status(
            &Event::StatusSigningReserved {
                admission: a.clone(),
            },
            None,
        )?;
        let timeout =
            remaining(a.prepare_deadline_ns).ok_or(RestoreStatusPreparationError::Timing)?;
        let prepared = tokio::time::timeout(timeout, signer.sign_status(&self.plan, a, &args))
            .await
            .map_err(|_| RestoreStatusPreparationError::Timing)?
            .map_err(RestoreStatusPreparationError::Signer)?;
        remaining(a.prepare_deadline_ns).ok_or(RestoreStatusPreparationError::Timing)?;
        self.plan.authenticate_status_request(a, &prepared)?;
        if prepared.expires_ns < a.submit_deadline_ns {
            return Err(CaptureError::StatusAdmission.into());
        }
        let bytes = prepared.encode(a.request_bytes)?;
        self.commit_status(
            &Event::StatusPrepared {
                request: EvidenceRecord::new(&bytes),
            },
            Some(&bytes),
        )?;
        Ok(RestoreStatusPreparation::Prepared)
    }
    fn commit_status(
        &mut self,
        event: &Event,
        evidence: Option<&[u8]>,
    ) -> Result<(), CaptureError> {
        let state = self.check()?;
        let mut next = state.clone();
        apply(&mut next, event, &self.plan)?;
        let frame = encode_frame(state.head, &serde_json::to_vec(event)?)?;
        if state.bytes + frame.len() as u64 > self.plan.request().limits.evidence_bytes {
            return Err(CaptureError::StatusAdmission);
        }
        let mut file = self
            .directory
            .resolve_parent(Path::new("ledger.bin"), PRIVATE)?
            .ok_or(CaptureError::Incomplete)?
            .open_regular_file_for_update(0o177)?
            .ok_or(CaptureError::Incomplete)?;
        if file.metadata()?.nlink() != 1
            || FileIdentity::from_file(&file)? != FileIdentity::from_file(&self.ledger_file)?
            || self.check()? != state
        {
            return Err(CaptureError::Changed);
        }
        self.usable = false;
        file.sync_all()?;
        self.plan_file.sync_all()?;
        self.lock.file().sync_all()?;
        self.directory.sync()?;
        child(&self.store.root, "operations", false)?.sync()?;
        self.target.sync()?;
        self.store.root.sync()?;
        if let Some(bytes) = evidence {
            retain(&self.directory, "status-request.bin", bytes)?;
            self.directory.sync()?;
        }
        file.seek(SeekFrom::Start(state.bytes))?;
        file.write_all(&frame)?;
        file.sync_all()?;
        self.usable = true;
        let checked = self.check();
        self.usable = checked.is_ok();
        checked?;
        Ok(())
    }
}
