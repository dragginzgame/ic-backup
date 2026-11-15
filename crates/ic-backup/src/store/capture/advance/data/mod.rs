//! Sequential scheduled payload acquisition with original-journal authority.
use super::{
    CaptureError, CaptureOperation, CapturePlan, CaptureProvider, PreparedStatus,
    StatusAdmissionRecord, StatusEvidence, StatusHttpError, StatusHttpTransport, certificate, http,
    request,
};
use crate::store::capture::ledger::{EventRecord, EvidenceRecord};
use crate::store::{
    CaptureMetadataAdvanceError, CaptureMetadataPhase, CaptureTransfer, DataReadError,
    TransferError,
};
use ic_platform::{
    ic_protocol::decode_call_content_strict,
    management_canister_types::{Snapshot, SnapshotDataKind},
};
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, future::Future, time::Duration};
use thiserror::Error;

const METHOD: &str = "read_canister_snapshot_data";
const PROOF_BYTES: u32 = 1024 * 1024;
const REPLY_OVERHEAD: u32 = 8192;

/// Original authority for one ordinal; a new ordinal needs its own admission.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureDataAdmissionRecord {
    /// Complete request/result and signing/network bounds, with original application binding.
    pub call: StatusAdmissionRecord,
    /// Exact configured transport commitment.
    pub transport: [u8; 32],
    /// External signer protocol/configuration identity.
    pub signer: [u8; 32],
    /// Current-read permission and continuous snapshot custody witness.
    pub read_authority: [u8; 32],
    /// Exact ingress expiry, at least the submission deadline.
    pub expires_ns: u64,
    /// Retained original certificate/delegation windows.
    pub observation: Vec<u8>,
}
impl CaptureDataAdmissionRecord {
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
    fn prepared(
        &self,
        plan: &CapturePlan,
        r: &PreparedStatus,
        args: &[u8],
    ) -> Result<(), DataReadError> {
        if r.expires_ns != self.expires_ns || r.observation != self.observation {
            return Err(CaptureError::StatusAdmission.into());
        }
        request::authenticate_expected_request(plan, &self.call, r, METHOD, args)?;
        Ok(())
    }
}
/// Trusted bounded sign-only adapter; admission must be local and effect-free.
pub trait CaptureDataSigner {
    /// Redacted adapter error.
    type Error;
    /// Stable signer/configuration identity.
    fn protocol(&self) -> [u8; 32];
    /// Require current permission and continuous exclusive custody for this original snapshot.
    fn admit_read(
        &self,
        plan: &CapturePlan,
        admission: &CaptureDataAdmissionRecord,
        ordinal: u32,
        snapshot: &Snapshot,
    ) -> Result<(), Self::Error>;
    /// Sign the fixed data method and exact supplied arguments, sender and expiry once.
    /// Enforce declared byte/network limits; never submit, retry or detach work.
    /// Cancellation retains external process custody and spent allowance.
    fn sign_data(
        &mut self,
        plan: &CapturePlan,
        admission: &CaptureDataAdmissionRecord,
        ordinal: u32,
        arguments: &[u8],
    ) -> impl Future<Output = Result<Vec<u8>, Self::Error>> + Send;
}
/// Retained phase; inspection alone is not authenticated coverage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureDataPhase {
    /// Signing invocation consumed without a retained request.
    SigningReserved,
    /// Exact signed bytes retained.
    Prepared,
    /// Submission consumed, outcome unresolved.
    SubmissionReserved,
    /// Certified evidence retained, payload not yet accepted.
    Settled,
    /// Exact validated payload length/hash retained; bytes remain in certified evidence.
    Accepted,
}
/// Read-only projection for one original data ordinal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureDataView {
    /// Current phase.
    pub phase: CaptureDataPhase,
    /// Original finite authority.
    pub admission: CaptureDataAdmissionRecord,
    /// One signing invocation once the ordinal exists.
    pub signing_requests_reserved: u32,
    /// Signer network allowance plus one after submission reservation.
    pub network_requests_reserved: u32,
}
/// One-ordinal continuation result; no outcome completes a local backup.
pub enum CaptureDataProgress {
    /// Signing remains spent and unresolved.
    SigningPending,
    /// HTTP remains spent and unresolved.
    SubmissionPending,
    /// Authenticated payload retained in original certified evidence.
    Ready(Vec<u8>),
}
/// Data acquisition refusal preserves original spending and evidence.
#[derive(Debug, Error)]
pub enum CaptureDataAdvanceError<P, S> {
    /// Local journal, custody, bounds or persistence failed.
    #[error(transparent)]
    Capture(#[from] CaptureError),
    /// Missing source/binding, changed admission or out-of-order ordinal.
    #[error("data read differs from the original bound schedule")]
    Binding,
    /// Not every scheduled ordinal has an accepted payload.
    #[error("scheduled payload coverage is incomplete")]
    Coverage,
    /// The combined module differs from the original release identity.
    #[error("complete module hash differs from the capture plan")]
    ModuleHash,
    /// Original capture/metadata authentication failed.
    #[error("original metadata authentication refused")]
    Metadata(Box<CaptureMetadataAdvanceError<P, Infallible>>),
    /// Existing transfer profile refused.
    #[error(transparent)]
    Transfer(#[from] TransferError),
    /// Exact request, certificate or payload refused.
    #[error(transparent)]
    Data(#[from] DataReadError),
    /// Current permission or external signing refused.
    #[error("data signer or read authority refused")]
    Signer(S),
    /// Signing expired or lies more than 300 seconds ahead.
    #[error("data signing deadline refused")]
    Deadline,
    /// The one HTTP invocation refused or became uncertain.
    #[error(transparent)]
    Http(#[from] StatusHttpError),
}
impl CaptureOperation {
    /// Inspect one data ordinal without authentication or remote work.
    /// # Errors
    /// Refuses damaged journals or changed evidence/custody.
    pub fn inspect_data(&self, ordinal: u32) -> Result<Option<CaptureDataView>, CaptureError> {
        Ok(self
            .check()?
            .data
            .get(ordinal as usize)
            .map(|d| CaptureDataView {
                phase: d.phase,
                admission: d.admission.clone(),
                signing_requests_reserved: 1,
                network_requests_reserved: d.admission.call.prepare_requests
                    + u32::from(matches!(
                        d.phase,
                        CaptureDataPhase::SubmissionReserved
                            | CaptureDataPhase::Settled
                            | CaptureDataPhase::Accepted
                    )),
            }))
    }

    /// Authenticate complete retained coverage and the combined original module hash.
    /// Returns the total payload byte count, processing one bounded reply at a time.
    /// This is offline replay: no signing, live admission, HTTP, journal changes or
    /// artifact publication. The result is not a completion or restore receipt.
    /// # Errors
    /// Refuses incomplete acceptance, changed custody/binding, invalid original
    /// evidence and a module hash differing from the immutable capture plan.
    pub async fn verify_data<P: CaptureProvider>(
        &mut self,
        transport: &StatusHttpTransport,
        original: &mut P,
    ) -> Result<u64, CaptureDataAdvanceError<P::Error, Infallible>> {
        use ic_host_artifacts::artifact::HashingWriter;
        use std::io::{Write, sink};

        let state = self.check()?;
        let bound = state
            .transfer
            .as_ref()
            .ok_or(CaptureDataAdvanceError::Binding)?;
        if state.data.len() != bound.data_calls as usize
            || state
                .data
                .iter()
                .any(|d| d.phase != CaptureDataPhase::Accepted)
        {
            return Err(CaptureDataAdvanceError::Coverage);
        }
        let m = state
            .metadata
            .as_ref()
            .ok_or(CaptureDataAdvanceError::Binding)?;
        let snapshot = self
            .metadata_snapshot(&m.admission, transport, original, &state)
            .await
            .map_err(|e| CaptureDataAdvanceError::Metadata(Box::new(e)))?;
        let metadata = self
            .retained_metadata(&state, &snapshot, transport)
            .map_err(|e| CaptureDataAdvanceError::Metadata(Box::new(e)))?;
        let transfer = self.plan.plan_transfer(&snapshot.id, &metadata)?;
        if transfer.binding(
            m.request.as_ref().ok_or(CaptureError::Incomplete)?,
            m.result.as_ref().ok_or(CaptureError::Incomplete)?,
        ) != *bound
        {
            return Err(CaptureDataAdvanceError::Binding);
        }
        let mut module = HashingWriter::new(sink(), metadata.wasm_module_size);
        let mut total = 0_u64;
        for (ordinal, (request, d)) in transfer.requests().zip(&state.data).enumerate() {
            let bytes = self.data_payload(ordinal as u32, d, &transfer, transport)?;
            total = total
                .checked_add(bytes.len() as u64)
                .filter(|n| *n <= transfer.payload_ceiling())
                .ok_or(CaptureError::Changed)?;
            if matches!(request.kind, SnapshotDataKind::WasmModule { .. }) {
                module.write_all(&bytes).map_err(CaptureError::from)?;
            }
        }
        if self.check()? != state {
            return Err(CaptureError::Changed.into());
        }
        if module.identity().sha256 != self.plan.request().module_sha256 {
            return Err(CaptureDataAdvanceError::ModuleHash);
        }
        Ok(total)
    }

    /// Acquire or replay one exact scheduled payload ordinal.
    ///
    /// Requires a retained transfer binding and all preceding ordinals accepted.
    /// Reauthenticates original capture/metadata and prior accepted payloads before
    /// new effects. Reserves signing and submission before their one-shot effects;
    /// retains certified evidence before validating payload and recording acceptance.
    /// Prepared continuation reuses exact bytes after fresh read admission. Pending
    /// calls never repeat. Accepted replay is offline, does not invoke the signer,
    /// and returns bytes decoded from retained evidence without journal writes.
    /// No separate payload file, final coverage receipt, cleanup or backup completion.
    /// ```no_run
    /// use ic_backup::store::{CaptureOperation, CaptureDataAdmissionRecord, CaptureDataSigner,
    ///     CaptureProvider, StatusHttpTransport, CaptureDataProgress, CaptureDataAdvanceError};
    /// async fn read<P: CaptureProvider, S: CaptureDataSigner>(op: &mut CaptureOperation,
    ///     ordinal: u32, a: &CaptureDataAdmissionRecord, t: &StatusHttpTransport,
    ///     signer: &mut S, original: &mut P)
    ///     -> Result<CaptureDataProgress, CaptureDataAdvanceError<P::Error, S::Error>> {
    ///     op.advance_data(ordinal, a, t, signer, original).await
    /// }
    /// ```
    /// # Errors
    /// Refuses wrong identity/sequence, insufficient complete-proof allowance,
    /// changed custody, expired admission, failed authentication and uncertain writes.
    /// Errors never refund spending. Partial frames and orphan files refuse reopen.
    /// # Panics
    /// New signing/submission needs a Tokio runtime with IO and timers enabled.
    pub async fn advance_data<P: CaptureProvider, S: CaptureDataSigner>(
        &mut self,
        ordinal: u32,
        admission: &CaptureDataAdmissionRecord,
        transport: &StatusHttpTransport,
        signer: &mut S,
        original: &mut P,
    ) -> Result<CaptureDataProgress, CaptureDataAdvanceError<P::Error, S::Error>> {
        let mut state = self.check()?;
        admission.validate(&self.plan)?;
        let bound = state
            .transfer
            .as_ref()
            .ok_or(CaptureDataAdvanceError::Binding)?;
        let m = state
            .metadata
            .as_ref()
            .filter(|m| m.phase == CaptureMetadataPhase::Settled)
            .ok_or(CaptureDataAdvanceError::Binding)?;
        if ordinal >= bound.data_calls
            || ordinal as usize > state.data.len()
            || transport.lookup_commitment() != admission.transport
            || state
                .data
                .get(ordinal as usize)
                .is_some_and(|d| d.admission != *admission)
            || (ordinal as usize == state.data.len()
                && state
                    .data
                    .last()
                    .is_some_and(|d| d.phase != CaptureDataPhase::Accepted))
        {
            return Err(CaptureDataAdvanceError::Binding);
        }
        let snapshot = self
            .metadata_snapshot(&m.admission, transport, original, &state)
            .await
            .map_err(|error| CaptureDataAdvanceError::Metadata(Box::new(error)))?;
        let metadata = self
            .retained_metadata(&state, &snapshot, transport)
            .map_err(|error| CaptureDataAdvanceError::Metadata(Box::new(error)))?;
        // Keep the immutable plan local so persistence can mutably borrow the operation.
        let plan = CapturePlan::decode(self.plan.encoded().to_vec())?;
        let transfer = plan.plan_transfer(&snapshot.id, &metadata)?;
        if transfer.binding(
            m.request.as_ref().ok_or(CaptureError::Incomplete)?,
            m.result.as_ref().ok_or(CaptureError::Incomplete)?,
        ) != *bound
        {
            return Err(CaptureDataAdvanceError::Binding);
        }
        for (n, d) in state.data.iter().enumerate().take(ordinal as usize) {
            if d.phase != CaptureDataPhase::Accepted {
                return Err(CaptureDataAdvanceError::Binding);
            }
            self.data_payload(n as u32, d, &transfer, transport)?;
        }
        if self.check()? != state {
            return Err(CaptureError::Changed.into());
        }
        if let Some(d) = state.data.get(ordinal as usize) {
            match d.phase {
                CaptureDataPhase::SigningReserved => {
                    return Ok(CaptureDataProgress::SigningPending);
                }
                CaptureDataPhase::SubmissionReserved => {
                    return Ok(CaptureDataProgress::SubmissionPending);
                }
                CaptureDataPhase::Settled | CaptureDataPhase::Accepted => {
                    let bytes = self.data_payload(ordinal, d, &transfer, transport)?;
                    if self.check()? != state {
                        return Err(CaptureError::Changed.into());
                    }
                    if d.phase == CaptureDataPhase::Settled {
                        self.commit(
                            &EventRecord::DataAccepted {
                                ordinal,
                                payload: EvidenceRecord::new(&bytes),
                            },
                            None,
                        )?;
                    }
                    return Ok(CaptureDataProgress::Ready(bytes));
                }
                CaptureDataPhase::Prepared => {}
            }
        }
        let args = transfer.data_arguments(ordinal)?;
        let payload_limit = match transfer
            .requests()
            .nth(ordinal as usize)
            .ok_or(CaptureDataAdvanceError::Binding)?
            .kind
        {
            SnapshotDataKind::WasmModule { size, .. }
            | SnapshotDataKind::WasmMemory { size, .. }
            | SnapshotDataKind::StableMemory { size, .. } => size as u32,
            SnapshotDataKind::WasmChunk { .. } => plan.request().limits.extent_bytes,
        };
        if admission.call.result_bytes < PROOF_BYTES + payload_limit + REPLY_OVERHEAD + 5 {
            return Err(CaptureError::StatusAdmission.into());
        }
        transport.admit_plan(&plan)?;
        if signer.protocol() != admission.signer {
            return Err(CaptureDataAdvanceError::Binding);
        }
        signer
            .admit_read(&plan, admission, ordinal, &snapshot)
            .map_err(CaptureDataAdvanceError::Signer)?;
        let names = super::super::ledger::data_names(ordinal);
        if ordinal as usize == state.data.len() {
            if http::remaining(admission.call.prepare_deadline_ns)
                .map_err(|_| CaptureDataAdvanceError::Deadline)?
                > Duration::from_secs(300)
            {
                return Err(CaptureDataAdvanceError::Deadline);
            }
            self.commit(
                &EventRecord::DataStarted {
                    ordinal,
                    admission: admission.clone(),
                },
                None,
            )?;
            let envelope = tokio::time::timeout(
                http::remaining(admission.call.prepare_deadline_ns)
                    .map_err(|_| CaptureDataAdvanceError::Deadline)?,
                signer.sign_data(&plan, admission, ordinal, &args),
            )
            .await
            .map_err(|_| CaptureDataAdvanceError::Deadline)?
            .map_err(CaptureDataAdvanceError::Signer)?;
            self.check()?;
            if signer.protocol() != admission.signer {
                return Err(CaptureDataAdvanceError::Binding);
            }
            http::remaining(admission.call.prepare_deadline_ns)
                .map_err(|_| CaptureDataAdvanceError::Deadline)?;
            let content =
                decode_call_content_strict(&envelope, admission.call.request_bytes as usize)
                    .map_err(|_| CaptureError::StatusAdmission)?;
            let prepared = PreparedStatus {
                request_id: *content.to_request_id(),
                expires_ns: admission.expires_ns,
                envelope,
                observation: admission.observation.clone(),
            };
            admission.prepared(&plan, &prepared, &args)?;
            let bytes = prepared.encode(admission.call.request_bytes)?;
            self.commit(
                &EventRecord::DataPrepared {
                    ordinal,
                    request: EvidenceRecord::new(&bytes),
                },
                Some((&names.0, &bytes)),
            )?;
            state = self.check()?;
        }
        let d = state
            .data
            .get(ordinal as usize)
            .ok_or(CaptureError::Incomplete)?;
        let prepared = PreparedStatus::decode(&self.evidence(
            &names.0,
            d.request.as_ref().ok_or(CaptureError::Incomplete)?,
        )?)?;
        admission.prepared(&plan, &prepared, &args)?;
        signer
            .admit_read(&plan, admission, ordinal, &snapshot)
            .map_err(CaptureDataAdvanceError::Signer)?;
        http::remaining(admission.call.submit_deadline_ns)?;
        self.commit(&EventRecord::DataSubmissionReserved { ordinal }, None)?;
        let evidence = transport
            .submit_expected(&plan, &admission.call, &prepared, METHOD, &args)
            .await?;
        self.check()?;
        let bytes = evidence.encode(admission.call.result_bytes)?;
        self.commit(
            &EventRecord::DataSettled {
                ordinal,
                result: EvidenceRecord::new(&bytes),
            },
            Some((&names.1, &bytes)),
        )?;
        let state = self.check()?;
        let bytes = self.data_payload(
            ordinal,
            state
                .data
                .get(ordinal as usize)
                .ok_or(CaptureError::Incomplete)?,
            &transfer,
            transport,
        )?;
        self.commit(
            &EventRecord::DataAccepted {
                ordinal,
                payload: EvidenceRecord::new(&bytes),
            },
            None,
        )?;
        Ok(CaptureDataProgress::Ready(bytes))
    }

    pub(in crate::store::capture) fn data_payload(
        &self,
        ordinal: u32,
        d: &super::super::ledger::DataState,
        transfer: &CaptureTransfer<'_>,
        transport: &StatusHttpTransport,
    ) -> Result<Vec<u8>, DataReadError> {
        if d.admission.transport != transport.lookup_commitment() {
            return Err(CaptureError::Changed.into());
        }
        let names = super::super::ledger::data_names(ordinal);
        let r = PreparedStatus::decode(&self.evidence(
            &names.0,
            d.request.as_ref().ok_or(CaptureError::Incomplete)?,
        )?)?;
        let e = StatusEvidence::decode(
            &self.evidence(&names.1, d.result.as_ref().ok_or(CaptureError::Incomplete)?)?,
        )?;
        d.admission
            .prepared(&self.plan, &r, &transfer.data_arguments(ordinal)?)?;
        let bytes = transfer.authenticate_data_reply(
            ordinal,
            &d.admission.call,
            &r,
            &e,
            transport.root_key_der(),
        )?;
        if e.response.len() > bytes.len() + REPLY_OVERHEAD as usize
            || d.payload
                .as_ref()
                .is_some_and(|p| *p != EvidenceRecord::new(&bytes))
        {
            return Err(CaptureError::Changed.into());
        }
        Ok(bytes)
    }
}
