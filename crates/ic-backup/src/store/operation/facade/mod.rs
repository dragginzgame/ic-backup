//! Public operation boundary; no phase commands or caller-selected journal facts.
//!
//! ```no_run
//! use ic_backup::store::{BackupStore, BackupSummary, CapturePlan, BackupSigner,
//!     BackupApplicationGuard, BackupConnection, OperationAdvanceError, OperationProgress};
//! use ic_backup::store::BackupDigest;
//!
//! // First construct/review connection.plan(request, options), then supply its approval.
//! async fn capture<S: BackupSigner, G: BackupApplicationGuard>(
//!     store: &BackupStore, connection: &mut BackupConnection<S, G>,
//!     plan: CapturePlan, approval: BackupDigest,
//! ) -> Result<Option<BackupSummary>, OperationAdvanceError<S::Error, G::Error>> {
//!     let mut operation = store.create_capture(plan, approval)?;
//!     loop {
//!         match operation.advance(connection).await? {
//!             OperationProgress::Advanced => continue,
//!             OperationProgress::NeedsAttention(_) => return Ok(None),
//!             OperationProgress::Complete(receipt) => return Ok(Some(*receipt)),
//!         }
//!     }
//! }
//! ```

#[cfg(test)]
mod tests;

use super::{
    Step,
    custody::{CaptureOperation, Role},
    replay::Error,
    runtime::{Connection, Guard, Signer},
};
use crate::store::{
    BackupDigest, BackupStore, BackupSummary,
    capture::{CaptureRequest, CaptureTiming, OperationStoreError, intent::CapturePlan},
    operation::intent::ProfileRecord,
};

/// Per-invocation costs and timing to approve once with the capture declaration.
#[derive(Clone, Copy, Debug)]
pub struct OperationOptions {
    /// Integration-owned continuous application witness.
    pub application: [u8; 32],
    /// Maximum retained snapshot count admitted before a non-replacing take.
    pub snapshot_capacity: u32,
    /// Maximum duration of one already-reserved signing invocation.
    pub prepare_duration_ns: u64,
    /// Maximum duration from invocation reservation through submission.
    pub submit_duration_ns: u64,
    /// Separate historical certificate and delegation windows.
    pub timing: CaptureTiming,
    /// Maximum signing-service network requests per invocation.
    pub signer_network: u32,
    /// Complete retained request ceiling, at most 2 MiB.
    pub request_bytes: u32,
    /// Complete retained proof/result ceiling, at most 4 MiB.
    pub result_bytes: u32,
}

/// Authenticated local progress and conservative spending, never effect authority.
#[derive(Debug)]
pub struct OperationView<T = BackupSummary> {
    /// Next fixed management call, if local work or completion is not next.
    pub next_call: Option<Step>,
    /// Latest retained update identity; pending dispatch must never be repeated.
    pub request_id: Option<[u8; 32]>,
    /// Signatures reserved, including interrupted signing and observations.
    pub signing_requests_reserved: u32,
    /// Network ceiling reserved, including all signing-service allowance.
    pub network_requests_reserved: u32,
    /// Explicit observations reserved; no result refunds them.
    pub observations_reserved: u32,
    /// Exact completed receipt, absent until original local completion closes.
    pub completed: Option<T>,
}

impl<S: Signer, G: Guard> Connection<S, G> {
    /// Compose one bounded transport, sign-only adapter and application guard.
    #[must_use]
    pub fn new(transport: crate::store::BackupHttpTransport, signer: S, guard: G) -> Self {
        Self {
            manager: None,
            transport,
            signer,
            guard,
        }
    }

    /// Select Root for new capture and restore approvals. The adapter performs pure
    /// framing and retained-result decoding. Backup owns all signing and submission.
    #[must_use]
    pub fn with_root_manager(
        mut self,
        manager: crate::store::RootManager,
        codec: &'static dyn crate::store::RootCodec,
    ) -> Self {
        self.manager = Some((manager, codec));

        self
    }

    /// Bind an immutable approval to this actual connection and its finite policy.
    ///
    /// No clock, signing, filesystem or network work occurs. A changed connection
    /// cannot reopen or replenish an existing operation's original authority.
    /// # Errors
    /// Refuses invalid declarations, trust, capacity, finite costs or timing.
    pub fn plan(
        &self,
        request: CaptureRequest,
        options: OperationOptions,
    ) -> Result<CapturePlan, crate::store::OperationPlanError> {
        self.transport
            .admit_binding(request.target.network, request.trust_root)
            .map_err(|_| crate::store::OperationPlanError::Profile)?;

        CapturePlan::new(request, self.profile(options))
    }

    fn profile(&self, options: OperationOptions) -> ProfileRecord {
        ProfileRecord {
            manager: self.manager.as_ref().map(|(manager, _)| manager.clone()),
            codec: self.manager.as_ref().map(|(_, codec)| *codec),
            transport: self.transport.commitment(),
            signer: self.signer.protocol(),
            guard: self.guard.protocol(),
            application: options.application,
            root_der: self.transport.root_key_der().to_vec(),
            snapshot_capacity: options.snapshot_capacity,
            prepare_duration_ns: options.prepare_duration_ns,
            submit_duration_ns: options.submit_duration_ns,
            certificate_max_age_ns: options.timing.certificate_max_age_ns,
            delegation_max_age_ns: options.timing.delegation_max_age_ns,
            clock_skew_ns: options.timing.max_clock_skew_ns,
            signer_network: options.signer_network,
            request_bytes: options.request_bytes,
            result_bytes: options.result_bytes,
        }
    }

    /// Approve a same-canister restore from an authenticated held capture source.
    /// # Errors
    /// Refuses mismatched source, trust, geometry, finite allowance or request capacity.
    pub fn plan_restore(
        &self,
        source: &crate::store::CaptureSource<'_, '_>,
        request: crate::store::RestoreRequest,
        options: OperationOptions,
    ) -> Result<crate::store::RestorePlan, crate::store::RestorePlanError> {
        self.transport
            .admit_binding(request.target.network, request.trust_root)
            .map_err(|_| crate::store::RestorePlanError::SourceBinding)?;

        crate::store::RestorePlan::new(source, request, self.profile(options))
    }
}

impl BackupStore {
    /// Retain exact approved intent and exclusive target ownership before effects.
    /// # Errors
    /// Refuses mismatched approval, occupied/partial intent, unsafe custody or IO.
    /// Failure preserves every original file and may leave an occupied target.
    pub fn create_capture(
        &self,
        plan: CapturePlan,
        approval: BackupDigest,
    ) -> Result<CaptureOperation, Error> {
        if plan.digest() != approval {
            return Err(OperationStoreError::ApprovalMismatch.into());
        }

        Ok(CaptureOperation::create(&self.root, plan)?)
    }

    /// Reopen the exact approved operation without creating, synchronizing or repairing.
    /// # Errors
    /// Refuses changed approval, files, evidence, unknown tails or held locks.
    pub fn open_capture(
        &self,
        id: &str,
        approval: BackupDigest,
    ) -> Result<CaptureOperation, Error> {
        let operation = CaptureOperation::open_with_codec(&self.root, id, self.codec)?;
        if operation.intent.digest() != approval {
            return Err(OperationStoreError::ApprovalMismatch.into());
        }
        operation.inspect()?;

        Ok(operation)
    }
}

impl CaptureOperation {
    pub(in crate::store) fn check_custody(&self) -> Result<(), OperationStoreError> {
        self.check().map(|_| ())
    }

    /// Authenticate retained progress without clock, signer, guard, HTTP or writes.
    /// # Errors
    /// Refuses incomplete/changed custody, invalid evidence or settled policy failures.
    pub fn inspect(&self) -> Result<OperationView, Error> {
        let journal = self.check()?;
        let flow = self.project(&journal, &self.intent)?;
        let next_call = match flow.next()? {
            crate::store::capture::procedure::Next::Call(step) => Some(step),
            _ => None,
        };

        self.view(&journal, next_call, self.published(&journal, &flow)?)
    }

    /// Verify an exact already-published receipt and payload, with no publication or sync.
    /// # Errors
    /// Refuses missing/staged publication, wrong selection, changed evidence or payload.
    pub fn verify_completion(&self, expected: BackupDigest) -> Result<BackupSummary, Error> {
        self.verified_completion(expected)
            .map(|(summary, _)| summary)
    }

    pub(in crate::store) fn verified_completion(
        &self,
        expected: BackupDigest,
    ) -> Result<
        (
            BackupSummary,
            crate::store::capture::procedure::Procedure<'_>,
        ),
        Error,
    > {
        let journal = self.check()?;
        let flow = self.project(&journal, &self.intent)?;
        let summary = self
            .published(&journal, &flow)?
            .ok_or(OperationStoreError::Incomplete)?;
        if summary.manifest_sha256 != expected {
            return Err(OperationStoreError::ApprovalMismatch.into());
        }

        Ok((summary, flow))
    }

    /// Import one surviving original terminal certificate without new spending or effects.
    /// # Errors
    /// Refuses wrong request/trust/time, nonterminal proof, prior resolution or changed custody.
    /// Certified policy failures stay durably settled even when this returns an error.
    pub fn reconcile_reply(&mut self, certificate: &[u8]) -> Result<(), Error> {
        self.reconcile_certificate(certificate, false)
    }

    /// Import the surviving certificate for the original read-state or Root lookup.
    /// Uses that lookup's retained time policy without new spending, clock or effects.
    /// # Errors
    /// Refuses missing/resolved lookup, changed custody, invalid proof or failed policy.
    pub fn reconcile_observation_reply(&mut self, certificate: &[u8]) -> Result<(), Error> {
        self.reconcile_certificate(certificate, true)
    }

    fn reconcile_certificate(
        &mut self,
        certificate: &[u8],
        observation: bool,
    ) -> Result<(), Error> {
        let intent = self.intent.clone();
        let journal = self.check()?;
        let mut flow = self.project(&journal, &intent)?;
        let call = journal
            .calls
            .last()
            .ok_or(OperationStoreError::Incomplete)?;
        if flow.next()? != crate::store::capture::procedure::Next::Call(call.step) {
            return Err(Error::History);
        }
        let result = if observation && intent.profile().manager.is_none() {
            self.import_observation(&journal, certificate)?
        } else {
            let exchange = if observation {
                crate::store::RootExchange::Lookup
            } else {
                crate::store::RootExchange::Execute
            };

            self.import_reply(&journal, &flow.effect()?, exchange, certificate)?
        };
        let Some(result) = result else {
            return Ok(());
        };
        if call.step == Step::FinalStatus {
            self.materialize(&self.check()?, &flow, false)?;
        }
        flow.accept(
            result.rejected,
            call.step,
            result.request_id,
            &result.response,
        )?;

        Ok(())
    }
}

#[expect(
    private_bounds,
    reason = "only the two sealed concrete intent types own recorder views"
)]
impl<P: crate::store::operation::intent::Intent> super::custody::Recorder<P> {
    pub(in crate::store) fn view<T>(
        &self,
        journal: &super::Journal,
        next_call: Option<Step>,
        completed: Option<T>,
    ) -> Result<OperationView<T>, Error> {
        let request_id = journal
            .calls
            .last()
            .filter(|c| c.update.request.is_some())
            .map(|c| {
                self.intent
                    .view()
                    .decode_request(c.update.reference_ns, self.read_role(c, Role::Request)?)
                    .map(|r| r.request_id)
            })
            .transpose()?;

        Ok(OperationView {
            next_call,
            request_id,
            signing_requests_reserved: journal.signatures,
            network_requests_reserved: journal.network,
            observations_reserved: journal.observations,
            completed,
        })
    }
}
