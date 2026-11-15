//! One metadata read after authenticated snapshot capture, with original-journal spending.
use super::{
    CaptureError, CapturePlan, METHOD, MetadataReadError, PreparedStatus, Snapshot,
    StatusAdmissionRecord, StatusEvidence, authenticate_metadata_reply, certificate, request,
};
use crate::store::capture::ledger::{EventRecord, EvidenceRecord, State};
use crate::store::{
    CaptureOperation, CaptureProvider, CaptureSnapshotPhase, SnapshotAdvanceError,
    SnapshotProgress, StatusHttpError, StatusHttpTransport,
};
use ic_platform::{
    ic_protocol::decode_call_content_strict,
    management_canister_types::ReadCanisterSnapshotMetadataResult,
};
use serde::{Deserialize, Serialize};
use std::{future::Future, time::Duration};
use thiserror::Error;

/// Exact authority for one metadata read; retained before external signing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureMetadataAdmissionRecord {
    /// Original provider/application bindings and finite signing/network/byte allowances.
    pub call: StatusAdmissionRecord,
    /// Exact configured transport commitment.
    pub transport: [u8; 32],
    /// Exact external signer protocol/configuration commitment.
    pub signer: [u8; 32],
    /// Integration-owned current read permission and snapshot-custody witness.
    pub read_authority: [u8; 32],
    /// Exact signed ingress expiry, no earlier than the submission deadline.
    pub expires_ns: u64,
    /// Retained certificate/delegation windows from `status_certificate_observation`.
    pub observation: Vec<u8>,
}
impl CaptureMetadataAdmissionRecord {
    fn authenticate_prepared(
        &self,
        plan: &CapturePlan,
        prepared: &PreparedStatus,
        args: &[u8],
    ) -> Result<(), MetadataReadError> {
        if prepared.expires_ns != self.expires_ns || prepared.observation != self.observation {
            return Err(CaptureError::StatusAdmission.into());
        }
        request::authenticate_expected_request(plan, &self.call, prepared, METHOD, args)?;
        Ok(())
    }
    pub(in crate::store::capture) fn validate(
        &self,
        plan: &CapturePlan,
    ) -> Result<(), CaptureError> {
        self.call.validate(plan)?;
        if self.call.submit_requests != 1 || self.expires_ns < self.call.submit_deadline_ns {
            return Err(CaptureError::StatusAdmission);
        }
        certificate::observation(&self.observation).map_err(|_| CaptureError::StatusAdmission)?;
        Ok(())
    }
}

/// Trusted bounded sign-only integration; admission is local and effect-free.
pub trait CaptureMetadataSigner {
    /// Redacted failure without keys or raw payloads.
    type Error;
    /// Stable signer/configuration identity.
    fn protocol(&self) -> [u8; 32];
    /// Check current read permission, original application continuity and exclusive snapshot custody.
    fn admit_read(
        &self,
        plan: &CapturePlan,
        admission: &CaptureMetadataAdmissionRecord,
        snapshot: &Snapshot,
    ) -> Result<(), Self::Error>;
    /// Sign the exact fixed metadata method/arguments, sender and retained expiry once.
    /// Enforce declared byte/network bounds; never dispatch, retry or detach work.
    /// Cancellation must preserve external process custody and spent allowance.
    fn sign_metadata(
        &mut self,
        plan: &CapturePlan,
        admission: &CaptureMetadataAdmissionRecord,
        snapshot: &Snapshot,
    ) -> impl Future<Output = Result<Vec<u8>, Self::Error>> + Send;
}

/// Metadata journal stage; inspection alone authenticates no result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureMetadataPhase {
    /// Signing invocation consumed without a complete retained request.
    SigningReserved,
    /// Exact signed request retained; submission not yet reserved.
    Prepared,
    /// One HTTP invocation consumed; its result remains unresolved.
    SubmissionReserved,
    /// Exact certified result retained; application policy must still be checked.
    Settled,
}
/// Read-only metadata authority and its additional reserved counts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureMetadataView {
    /// Current retained phase.
    pub phase: CaptureMetadataPhase,
    /// Exact original admission.
    pub admission: CaptureMetadataAdmissionRecord,
    /// One once this metadata read exists.
    pub signing_requests_reserved: u32,
    /// Signer network allowance plus one after submission reservation.
    pub network_requests_reserved: u32,
}
/// Metadata continuation outcome; no variant completes a local backup.
pub enum CaptureMetadataProgress {
    /// Signing is unresolved and must not be repeated.
    SigningPending,
    /// Submission is unresolved and must not be repeated.
    SubmissionPending,
    /// Authenticated, supported metadata. No durable data-read schedule exists yet.
    Ready(Box<ReadCanisterSnapshotMetadataResult>),
}
/// Metadata refusal or uncertainty; original evidence and spending remain retained.
#[derive(Debug, Error)]
pub enum CaptureMetadataAdvanceError<P, S> {
    /// Original custody, cumulative budgets or persistence failed.
    #[error(transparent)]
    Capture(#[from] CaptureError),
    /// Original snapshot is not settled or has no successful capture result.
    #[error("metadata requires the original settled capture")]
    SnapshotRequired,
    /// Original capture authentication or policy failed.
    #[error("original capture authentication refused")]
    Original(SnapshotAdvanceError<P>),
    /// Original signer, transport or derived snapshot binding differs.
    #[error("metadata authority binding differs")]
    Binding,
    /// Live read permission or external signing refused.
    #[error("metadata signer or read authority refused")]
    Signer(S),
    /// Preparation deadline expired or exceeds 300 seconds ahead.
    #[error("metadata signing deadline refused")]
    Deadline,
    /// One-shot HTTP refused or returned an uncertain outcome.
    #[error(transparent)]
    Http(#[from] StatusHttpError),
    /// Signed request, certificate or metadata profile refused.
    #[error(transparent)]
    Metadata(#[from] MetadataReadError),
}
impl CaptureOperation {
    // Replay only: no signer, live admission, clock or metadata submission port.
    pub(in crate::store::capture) async fn metadata_snapshot<P: CaptureProvider, S>(
        &mut self,
        admission: &CaptureMetadataAdmissionRecord,
        transport: &StatusHttpTransport,
        original_provider: &mut P,
        state: &State,
    ) -> Result<Snapshot, CaptureMetadataAdvanceError<P::Error, S>> {
        admission.validate(&self.plan)?;
        if state.snapshot.phase != CaptureSnapshotPhase::Settled {
            return Err(CaptureMetadataAdvanceError::SnapshotRequired);
        }
        if transport.lookup_commitment() != admission.transport
            || state
                .metadata
                .as_ref()
                .is_some_and(|m| m.admission != *admission)
        {
            return Err(CaptureMetadataAdvanceError::Binding);
        }
        let original_admission = state
            .snapshot
            .admission
            .clone()
            .ok_or(CaptureError::Incomplete)?;
        let SnapshotProgress::Captured(snapshot) = self
            .advance_snapshot(&original_admission, original_provider)
            .await
            .map_err(CaptureMetadataAdvanceError::Original)?
        else {
            return Err(CaptureMetadataAdvanceError::SnapshotRequired);
        };
        if self.check()? != *state {
            return Err(CaptureError::Changed.into());
        }
        if let Some(m) = &state.metadata
            && (m.snapshot_id != snapshot.id
                || m.timestamp != snapshot.taken_at_timestamp
                || m.total_size != snapshot.total_size)
        {
            return Err(CaptureMetadataAdvanceError::Binding);
        }
        Ok(snapshot)
    }

    pub(in crate::store::capture) fn retained_metadata<P, S>(
        &self,
        state: &State,
        snapshot: &Snapshot,
        transport: &StatusHttpTransport,
    ) -> Result<Box<ReadCanisterSnapshotMetadataResult>, CaptureMetadataAdvanceError<P, S>> {
        let m = state
            .metadata
            .as_ref()
            .filter(|m| m.phase == CaptureMetadataPhase::Settled)
            .ok_or(CaptureError::Incomplete)?;
        let admission = &m.admission;
        let r = PreparedStatus::decode(&self.evidence(
            "metadata-request.bin",
            m.request.as_ref().ok_or(CaptureError::Incomplete)?,
        )?)?;
        let e = StatusEvidence::decode(&self.evidence(
            "metadata-result.bin",
            m.result.as_ref().ok_or(CaptureError::Incomplete)?,
        )?)?;
        admission.authenticate_prepared(
            &self.plan,
            &r,
            &self.plan.metadata_read_arguments(snapshot)?,
        )?;
        let metadata = authenticate_metadata_reply(
            &self.plan,
            snapshot,
            &admission.call,
            &r,
            &e,
            transport.root_key_der(),
        )?;
        if self.check()? != *state {
            return Err(CaptureError::Changed.into());
        }
        Ok(Box::new(metadata))
    }

    /// Inspect only this metadata read's reservations, without provider or network work.
    /// # Errors
    /// Refuses changed custody, orphan evidence or invalid journals.
    pub fn inspect_metadata(&self) -> Result<Option<CaptureMetadataView>, CaptureError> {
        Ok(self.check()?.metadata.map(|m| CaptureMetadataView {
            phase: m.phase,
            signing_requests_reserved: 1,
            network_requests_reserved: m.admission.call.prepare_requests
                + u32::from(matches!(
                    m.phase,
                    CaptureMetadataPhase::SubmissionReserved | CaptureMetadataPhase::Settled
                )),
            admission: m.admission,
        }))
    }
    /// Acquire one metadata reply for the authenticated original captured snapshot.
    ///
    /// Requires an already settled take; original capture replay performs no live work.
    /// Retains signing reservation before signing, exact bytes before submission, and
    /// certified evidence before reporting metadata. Pending invocations never repeat.
    /// Prepared continuation reuses original bytes after live admission. Settled replay
    /// uses original proof and timing without a clock, metadata signer or live guard.
    /// Rejection/invalid profile evidence remains retained but never becomes Ready.
    /// No data transfer, schedule persistence, completion or snapshot cleanup occurs.
    /// ```no_run
    /// use ic_backup::store::{CaptureOperation, CaptureProvider, CaptureMetadataSigner,
    ///     CaptureMetadataAdmissionRecord, CaptureMetadataProgress, CaptureMetadataAdvanceError,
    ///     StatusHttpTransport};
    /// async fn metadata<P: CaptureProvider, S: CaptureMetadataSigner>(
    ///     op: &mut CaptureOperation, admission: &CaptureMetadataAdmissionRecord,
    ///     transport: &StatusHttpTransport, signer: &mut S, original: &mut P,
    /// ) -> Result<CaptureMetadataProgress, CaptureMetadataAdvanceError<P::Error, S::Error>> {
    ///     op.advance_metadata(admission, transport, signer, original).await
    /// }
    /// ```
    ///
    /// # Errors
    /// Refuses changed original bindings, exhausted budgets, failed admission/signing,
    /// authentication/policy failures and uncertain writes. No error refunds spending.
    /// # Panics
    /// New signing/submission requires a Tokio runtime with IO and timers enabled.
    pub async fn advance_metadata<P: CaptureProvider, S: CaptureMetadataSigner>(
        &mut self,
        admission: &CaptureMetadataAdmissionRecord,
        transport: &StatusHttpTransport,
        signer: &mut S,
        original_provider: &mut P,
    ) -> Result<CaptureMetadataProgress, CaptureMetadataAdvanceError<P::Error, S::Error>> {
        let mut state = self.check()?;
        let snapshot = self
            .metadata_snapshot(admission, transport, original_provider, &state)
            .await?;
        if let Some(m) = &state.metadata {
            match m.phase {
                CaptureMetadataPhase::SigningReserved => {
                    return Ok(CaptureMetadataProgress::SigningPending);
                }
                CaptureMetadataPhase::SubmissionReserved => {
                    return Ok(CaptureMetadataProgress::SubmissionPending);
                }
                CaptureMetadataPhase::Settled => {
                    return self
                        .retained_metadata(&state, &snapshot, transport)
                        .map(CaptureMetadataProgress::Ready);
                }
                CaptureMetadataPhase::Prepared => {}
            }
        }
        let arguments = self.plan.metadata_read_arguments(&snapshot)?;
        transport.admit_plan(&self.plan)?;
        if signer.protocol() != admission.signer {
            return Err(CaptureMetadataAdvanceError::Binding);
        }
        signer
            .admit_read(&self.plan, admission, &snapshot)
            .map_err(CaptureMetadataAdvanceError::Signer)?;
        if state.metadata.is_none() {
            if super::super::http::remaining(admission.call.prepare_deadline_ns)
                .map_err(|_| CaptureMetadataAdvanceError::Deadline)?
                > Duration::from_secs(300)
            {
                return Err(CaptureMetadataAdvanceError::Deadline);
            }
            self.commit(
                &EventRecord::MetadataStarted {
                    admission: admission.clone(),
                    snapshot_id: snapshot.id.clone(),
                    timestamp: snapshot.taken_at_timestamp,
                    total_size: snapshot.total_size,
                },
                None,
            )?;
            let duration = super::super::http::remaining(admission.call.prepare_deadline_ns)
                .map_err(|_| CaptureMetadataAdvanceError::Deadline)?;
            let envelope = tokio::time::timeout(
                duration,
                signer.sign_metadata(&self.plan, admission, &snapshot),
            )
            .await
            .map_err(|_| CaptureMetadataAdvanceError::Deadline)?
            .map_err(CaptureMetadataAdvanceError::Signer)?;
            self.check()?;
            if signer.protocol() != admission.signer {
                return Err(CaptureMetadataAdvanceError::Binding);
            }
            super::super::http::remaining(admission.call.prepare_deadline_ns)
                .map_err(|_| CaptureMetadataAdvanceError::Deadline)?;
            let content =
                decode_call_content_strict(&envelope, admission.call.request_bytes as usize)
                    .map_err(|_| CaptureError::StatusAdmission)?;
            let prepared = PreparedStatus {
                request_id: *content.to_request_id(),
                expires_ns: admission.expires_ns,
                envelope,
                observation: admission.observation.clone(),
            };
            admission.authenticate_prepared(&self.plan, &prepared, &arguments)?;
            let bytes = prepared.encode(admission.call.request_bytes)?;
            self.commit(
                &EventRecord::MetadataPrepared {
                    request: EvidenceRecord::new(&bytes),
                },
                Some(("metadata-request.bin", &bytes)),
            )?;
            state = self.check()?;
        }
        let m = state.metadata.as_ref().ok_or(CaptureError::Incomplete)?;
        let prepared = PreparedStatus::decode(&self.evidence(
            "metadata-request.bin",
            m.request.as_ref().ok_or(CaptureError::Incomplete)?,
        )?)?;
        admission.authenticate_prepared(&self.plan, &prepared, &arguments)?;
        signer
            .admit_read(&self.plan, admission, &snapshot)
            .map_err(CaptureMetadataAdvanceError::Signer)?;
        super::super::http::remaining(admission.call.submit_deadline_ns)?;
        self.commit(&EventRecord::MetadataSubmissionReserved, None)?;
        let evidence = transport
            .submit_expected(&self.plan, &admission.call, &prepared, METHOD, &arguments)
            .await?;
        self.check()?;
        let bytes = evidence.encode(admission.call.result_bytes)?;
        self.commit(
            &EventRecord::MetadataSettled {
                result: EvidenceRecord::new(&bytes),
            },
            Some(("metadata-result.bin", &bytes)),
        )?;
        let metadata = authenticate_metadata_reply(
            &self.plan,
            &snapshot,
            &admission.call,
            &prepared,
            &evidence,
            transport.root_key_der(),
        )?;
        self.check()?;
        Ok(CaptureMetadataProgress::Ready(Box::new(metadata)))
    }
}
