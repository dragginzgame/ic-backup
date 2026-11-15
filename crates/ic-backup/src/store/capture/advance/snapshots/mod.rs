//! Ordered, durable list/take continuation. Unknown results remain spent.
use super::{
    CaptureCall, CaptureError, CaptureOperation, CapturePlan, CaptureProvider, PreparedStatus,
    StatusAdmissionRecord, StatusAdvanceError, StatusEvidence, StatusProgress,
    ledger::{self, EventRecord, EvidenceRecord},
};
use crate::store::{SnapshotReplyError, decode_snapshot_list, decode_taken_snapshot};
use ic_host_artifacts::artifact::Sha256Digest;
use ic_platform::management_canister_types::Snapshot;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Original authority for one snapshot inventory and one non-replacing take.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotAdmissionRecord {
    /// Explicit remote capacity selected by the integration, 1..=1,024.
    pub capacity: u32,
    /// Original finite list-call authority, sharing initial status's provider/application.
    pub list: super::CaptureCallAdmissionRecord,
    /// Original finite take-call authority; no hidden replacement, cleanup or retry.
    pub take: super::CaptureCallAdmissionRecord,
}
impl SnapshotAdmissionRecord {
    pub(in crate::store::capture) fn validate(
        &self,
        plan: &CapturePlan,
        status: &StatusAdmissionRecord,
    ) -> Result<(), CaptureError> {
        let calls = [status, &self.list, &self.take];
        let mut requests = 0_u64;
        let mut bytes = ledger::initial(plan)?.len() as u64 + 12 * (16 * 1024 + 36);
        for admission in calls {
            admission.validate(plan)?;
            if admission.provider != status.provider || admission.application != status.application
            {
                return Err(CaptureError::StatusAdmission);
            }
            requests +=
                u64::from(admission.prepare_requests) + u64::from(admission.submit_requests);
            bytes += u64::from(admission.request_bytes) + u64::from(admission.result_bytes);
        }
        if !(1..=1024).contains(&self.capacity)
            || plan.request().limits.signing_requests < 3
            || requests > u64::from(plan.request().limits.network_requests)
            || bytes > plan.request().limits.evidence_bytes
        {
            return Err(CaptureError::StatusAdmission);
        }
        Ok(())
    }
}

/// Retained snapshot stage. Inspection alone authenticates no remote result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureSnapshotPhase {
    /// No snapshot-call reservation exists.
    Created,
    /// List signing was reserved; output is unresolved.
    ListPreparationReserved,
    /// Original signed list request is retained.
    ListPrepared,
    /// List submission was reserved; outcome is unresolved.
    ListSubmissionReserved,
    /// Original list evidence is retained; dependent take still requires authentication.
    ListSettled,
    /// Take signing was reserved; output is unresolved.
    TakePreparationReserved,
    /// Original signed take request is retained.
    TakePrepared,
    /// Take submission was reserved; outcome is unresolved.
    TakeSubmissionReserved,
    /// Original take evidence is retained; this is not backup completion.
    Settled,
}

/// Snapshot continuation result. None releases ownership or completes a local backup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SnapshotProgress {
    /// Exact original reply authenticates one newly allocated snapshot.
    Captured(Snapshot),
    /// Signing remains spent and unresolved for this call; do not sign again.
    PreparationPending(CaptureCall),
    /// Submission remains spent and unresolved; never infer success from another list.
    SubmissionPending(CaptureCall),
    /// Certified original rejection, with all reservations retained.
    Rejected(CaptureCall),
}

/// Snapshot refusal or uncertain outcome; no failure grants retry authority.
#[derive(Debug, Error)]
pub enum SnapshotAdvanceError<E> {
    /// Retained observation settlement failed historical authentication.
    #[error(transparent)]
    LookupSettlement(#[from] super::CaptureLookupSettlementError),
    /// Original local evidence, bounds or custody refused.
    #[error(transparent)]
    Capture(#[from] CaptureError),
    /// Initial status must be retained and independently verified first.
    #[error("snapshot capture requires original verified initial status")]
    InitialStatusRequired,
    /// Historical initial status failed authentication or policy.
    #[error("snapshot initial-status authentication failed")]
    Status(StatusAdvanceError<E>),
    /// Provider refused or returned an uncertain outcome.
    #[error("snapshot provider refused or returned an uncertain outcome")]
    Provider(E),
    /// Authenticated snapshot reply or capacity preflight refused.
    #[error(transparent)]
    Reply(#[from] SnapshotReplyError),
}

/// Read-only original snapshot stage and cumulative reserved costs, including status.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureSnapshotView {
    /// Retained stage, not a success/authentication claim.
    pub phase: CaptureSnapshotPhase,
    /// Original snapshot authority, absent until first list preparation reservation.
    pub admission: Option<SnapshotAdmissionRecord>,
    /// Total signing invocations reserved by initial status, list and take.
    pub signing_requests_reserved: u32,
    /// Total conservative network allowances reserved by those calls.
    pub network_requests_reserved: u32,
    /// Original list ingress identity when prepared evidence exists.
    pub list_request_id: Option<[u8; 32]>,
    /// Original take ingress identity when prepared evidence exists.
    pub take_request_id: Option<[u8; 32]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::store::capture) struct SnapshotState {
    pub phase: CaptureSnapshotPhase,
    pub admission: Option<SnapshotAdmissionRecord>,
    pub list_request: Option<EvidenceRecord>,
    pub list_result: Option<EvidenceRecord>,
    pub take_request: Option<EvidenceRecord>,
    pub take_result: Option<EvidenceRecord>,
}
impl SnapshotState {
    pub fn new() -> Self {
        Self {
            phase: CaptureSnapshotPhase::Created,
            admission: None,
            list_request: None,
            list_result: None,
            take_request: None,
            take_result: None,
        }
    }
    pub fn evidence(&self) -> [(&'static str, Option<&EvidenceRecord>); 4] {
        [
            ("snapshot-list-request.bin", self.list_request.as_ref()),
            ("snapshot-list-result.bin", self.list_result.as_ref()),
            ("snapshot-take-request.bin", self.take_request.as_ref()),
            ("snapshot-take-result.bin", self.take_result.as_ref()),
        ]
    }
    fn refs(
        &self,
        call: CaptureCall,
    ) -> Result<(Option<&EvidenceRecord>, Option<&EvidenceRecord>), CaptureError> {
        match call {
            CaptureCall::ListSnapshots => {
                Ok((self.list_request.as_ref(), self.list_result.as_ref()))
            }
            CaptureCall::TakeSnapshot => {
                Ok((self.take_request.as_ref(), self.take_result.as_ref()))
            }
            CaptureCall::Status => Err(CaptureError::StatusAdmission),
        }
    }
}

pub(in crate::store::capture) fn names(
    call: CaptureCall,
) -> Result<(&'static str, &'static str), CaptureError> {
    match call {
        CaptureCall::ListSnapshots => Ok(("snapshot-list-request.bin", "snapshot-list-result.bin")),
        CaptureCall::TakeSnapshot => Ok(("snapshot-take-request.bin", "snapshot-take-result.bin")),
        CaptureCall::Status => Err(CaptureError::StatusAdmission),
    }
}

impl CaptureOperation {
    /// List original snapshots, admit capacity and take one new snapshot durably.
    ///
    /// Initial status must already be verified through `advance_status`. Each call's
    /// signing/submission is reserved before invocation and exact evidence is retained
    /// before settlement. Reopen never repeats a pending effect. Settled replay is local.
    /// The integration must maintain uninterrupted stopped/application authority and
    /// exclusive snapshot custody. This never deletes/replaces a snapshot, uninstalls
    /// code, downloads a bundle, releases ownership or infers a lost result from a list.
    ///
    /// # Errors
    /// Refuses incomplete initial status, changed authority, insufficient cumulative
    /// budgets, capacity exhaustion, provider failures, unauthenticated or reused IDs
    /// and custody failures. All original evidence/spending remains retained.
    pub async fn advance_snapshot<P: CaptureProvider>(
        &mut self,
        admission: &SnapshotAdmissionRecord,
        provider: &mut P,
    ) -> Result<SnapshotProgress, SnapshotAdvanceError<P::Error>> {
        use CaptureSnapshotPhase as Phase;
        let state = self.check()?;
        if state.phase != super::super::CaptureStatusPhase::Settled {
            return Err(SnapshotAdvanceError::InitialStatusRequired);
        }
        let status = state.admission.as_ref().ok_or(CaptureError::Incomplete)?;
        admission.validate(&self.plan, status)?;
        if state
            .snapshot
            .admission
            .as_ref()
            .is_some_and(|original| original != admission)
        {
            return Err(CaptureError::StatusAdmission.into());
        }
        // An unresolved invocation is not a fresh capability, even if admission would pass.
        match state.snapshot.phase {
            Phase::ListPreparationReserved => {
                return Ok(SnapshotProgress::PreparationPending(
                    CaptureCall::ListSnapshots,
                ));
            }
            Phase::ListSubmissionReserved => {
                return Ok(SnapshotProgress::SubmissionPending(
                    CaptureCall::ListSnapshots,
                ));
            }
            Phase::TakePreparationReserved => {
                return Ok(SnapshotProgress::PreparationPending(
                    CaptureCall::TakeSnapshot,
                ));
            }
            Phase::TakeSubmissionReserved => {
                return Ok(SnapshotProgress::SubmissionPending(
                    CaptureCall::TakeSnapshot,
                ));
            }
            _ => {}
        }
        if self
            .status_result(provider, status)
            .map_err(SnapshotAdvanceError::Status)?
            != StatusProgress::Verified
        {
            return Err(SnapshotAdvanceError::InitialStatusRequired);
        }
        if matches!(state.snapshot.phase, Phase::Created | Phase::ListPrepared) {
            self.snapshot_call(admission, CaptureCall::ListSnapshots, provider)
                .await?;
        }
        let list = self.snapshot_result(&admission.list, CaptureCall::ListSnapshots, provider)?;
        if list.rejected {
            return Ok(SnapshotProgress::Rejected(CaptureCall::ListSnapshots));
        }
        let prior = decode_snapshot_list(&list.response)?;
        self.plan
            .take_snapshot_arguments(&prior, admission.capacity)?;
        if self.check()?.snapshot.phase != Phase::Settled {
            self.snapshot_call(admission, CaptureCall::TakeSnapshot, provider)
                .await?;
        }
        let result = self.snapshot_result(&admission.take, CaptureCall::TakeSnapshot, provider)?;
        if result.rejected {
            return Ok(SnapshotProgress::Rejected(CaptureCall::TakeSnapshot));
        }
        let snapshot = decode_taken_snapshot(&result.response, &prior)?;
        self.check()?;
        Ok(SnapshotProgress::Captured(snapshot))
    }

    async fn snapshot_call<P: CaptureProvider>(
        &mut self,
        admission: &SnapshotAdmissionRecord,
        call: CaptureCall,
        provider: &mut P,
    ) -> Result<(), SnapshotAdvanceError<P::Error>> {
        use CaptureSnapshotPhase as Phase;
        let (authority, start, prepared_phase, base) = match call {
            CaptureCall::ListSnapshots => (&admission.list, Phase::Created, Phase::ListPrepared, 5),
            CaptureCall::TakeSnapshot => {
                (&admission.take, Phase::ListSettled, Phase::TakePrepared, 9)
            }
            CaptureCall::Status => return Err(CaptureError::StatusAdmission.into()),
        };
        let (request_name, result_name) = names(call)?;
        let phase = self.check()?.snapshot.phase;
        if phase == start {
            provider
                .admit_call(&self.plan, authority, call, None)
                .map_err(SnapshotAdvanceError::Provider)?;
            let hash = *Sha256Digest::compute(&call.arguments(&self.plan)?).as_bytes();
            let event = match call {
                CaptureCall::ListSnapshots => EventRecord::SnapshotStarted {
                    sequence: base,
                    admission: admission.clone(),
                    arguments_sha256: hash,
                },
                CaptureCall::TakeSnapshot => EventRecord::SnapshotTakeReserved {
                    sequence: base,
                    arguments_sha256: hash,
                },
                CaptureCall::Status => return Err(CaptureError::StatusAdmission.into()),
            };
            self.commit(&event, None)?;
            let prepared = provider
                .prepare_call(&self.plan, authority, call)
                .await
                .map_err(SnapshotAdvanceError::Provider)?;
            self.check()?;
            let bytes = prepared.encode(authority.request_bytes)?;
            prepared.validate_call_for(&self.plan, call)?;
            provider
                .admit_call(&self.plan, authority, call, Some(&prepared))
                .map_err(SnapshotAdvanceError::Provider)?;
            self.commit(
                &EventRecord::SnapshotPrepared {
                    sequence: base + 1,
                    call,
                    request: EvidenceRecord::new(&bytes),
                },
                Some((request_name, &bytes)),
            )?;
        } else if phase != prepared_phase {
            return Err(CaptureError::StatusAdmission.into());
        }
        let prepared = self.snapshot_request(call)?;
        provider
            .admit_call(&self.plan, authority, call, Some(&prepared))
            .map_err(SnapshotAdvanceError::Provider)?;
        self.commit(
            &EventRecord::SnapshotSubmissionReserved {
                sequence: base + 2,
                call,
            },
            None,
        )?;
        let evidence = provider
            .submit_call(&self.plan, authority, call, &prepared)
            .await
            .map_err(SnapshotAdvanceError::Provider)?;
        self.check()?;
        let bytes = evidence.encode(authority.result_bytes)?;
        provider
            .authenticate_call(&self.plan, authority, call, &prepared, &evidence)
            .map_err(SnapshotAdvanceError::Provider)?;
        self.commit(
            &EventRecord::SnapshotSettled {
                sequence: base + 3,
                call,
                result: EvidenceRecord::new(&bytes),
            },
            Some((result_name, &bytes)),
        )?;
        Ok(())
    }

    fn snapshot_request(&self, call: CaptureCall) -> Result<PreparedStatus, CaptureError> {
        let state = self.check()?;
        let (request, _) = state.snapshot.refs(call)?;
        let prepared = PreparedStatus::decode(
            &self.evidence(names(call)?.0, request.ok_or(CaptureError::Incomplete)?)?,
        )?;
        prepared.validate_call_for(&self.plan, call)?;
        Ok(prepared)
    }
    fn snapshot_result<P: CaptureProvider>(
        &self,
        admission: &StatusAdmissionRecord,
        call: CaptureCall,
        provider: &P,
    ) -> Result<StatusEvidence, SnapshotAdvanceError<P::Error>> {
        let state = self.check()?;
        let prepared = self.snapshot_request(call)?;
        let (_, result) = state.snapshot.refs(call)?;
        let evidence = StatusEvidence::decode(
            &self.evidence(names(call)?.1, result.ok_or(CaptureError::Incomplete)?)?,
        )?;
        if !self.authenticate_lookup_settlement(call, &evidence)? {
            provider
                .authenticate_call(&self.plan, admission, call, &prepared, &evidence)
                .map_err(SnapshotAdvanceError::Provider)?;
        }
        if self.check()? != state {
            return Err(CaptureError::Changed.into());
        }
        Ok(evidence)
    }

    /// Inspect original snapshot progress and total reservations without effects.
    ///
    /// No signing, network, authentication, repair or inferred success occurs.
    /// # Errors
    /// Refuses unusable sessions, missing/changed evidence or invalid original custody.
    pub fn inspect_snapshot(&self) -> Result<CaptureSnapshotView, CaptureError> {
        use CaptureSnapshotPhase as Phase;
        let state = self.check()?;
        let status = self.inspect_status()?;
        let mut signing = status.signing_requests_reserved;
        let mut requests = status.network_requests_reserved;
        if let Some(a) = &state.snapshot.admission {
            signing += 1;
            requests += a.list.prepare_requests;
            if !matches!(
                state.snapshot.phase,
                Phase::ListPreparationReserved | Phase::ListPrepared
            ) {
                requests += a.list.submit_requests;
            }
            if matches!(
                state.snapshot.phase,
                Phase::TakePreparationReserved
                    | Phase::TakePrepared
                    | Phase::TakeSubmissionReserved
                    | Phase::Settled
            ) {
                signing += 1;
                requests += a.take.prepare_requests;
            }
            if matches!(
                state.snapshot.phase,
                Phase::TakeSubmissionReserved | Phase::Settled
            ) {
                requests += a.take.submit_requests;
            }
        }
        let list_request_id = state
            .snapshot
            .list_request
            .as_ref()
            .map(|_| {
                self.snapshot_request(CaptureCall::ListSnapshots)
                    .map(|r| r.request_id)
            })
            .transpose()?;
        let take_request_id = state
            .snapshot
            .take_request
            .as_ref()
            .map(|_| {
                self.snapshot_request(CaptureCall::TakeSnapshot)
                    .map(|r| r.request_id)
            })
            .transpose()?;
        if self.check()? != state {
            return Err(CaptureError::Changed);
        }
        Ok(CaptureSnapshotView {
            phase: state.snapshot.phase,
            admission: state.snapshot.admission,
            signing_requests_reserved: signing,
            network_requests_reserved: requests,
            list_request_id,
            take_request_id,
        })
    }
}
