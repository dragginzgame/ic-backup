//! Settle only from the one retained original-request observation; no new remote work.
use super::{
    CaptureCall, CaptureError, CaptureLookupError, CaptureLookupPhase, CaptureObservation,
    CaptureObservationError, CaptureOperation, Error, EventRecord, EvidenceRecord, PreparedStatus,
    StatusHttpTransport, authenticate_capture_lookup, observe_capture_certificate,
};
use crate::store::CaptureCallEvidence;

/// Retained lookup settlement refusal. No variant grants new remote allowance.
#[derive(Debug, Error)]
pub enum CaptureLookupSettlementError {
    /// Original custody, bounds or persistence failed.
    #[error(transparent)]
    Capture(#[from] CaptureError),
    /// No complete retained observation exists.
    #[error("capture lookup has no retained observation")]
    NotObserved,
    /// The authenticated observation has no reply or rejection to settle.
    #[error("capture lookup outcome remains unresolved")]
    Unresolved,
    /// The original call already has a different settlement or transport binding.
    #[error("capture lookup settlement conflicts with retained authority or evidence")]
    Conflict,
    /// Exact signed lookup/original request association refused.
    #[error(transparent)]
    Request(#[from] CaptureLookupError),
    /// Original certificate authentication under retained observation windows refused.
    #[error(transparent)]
    Observation(#[from] CaptureObservationError),
}

impl CaptureOperation {
    /// Settle the original call from its already retained terminal lookup certificate.
    ///
    /// Performs only local authentication and durable persistence. No signing, clock,
    /// live guard, network or extra allowance is needed. Original request bytes and
    /// timing are unchanged; this settlement retains its distinct observation source.
    /// Replied/rejected evidence is retained without declaring application-policy
    /// success. Unresolved observations leave the call pending. Continue separately
    /// with `advance_status` or `advance_snapshot`, which authenticate the retained
    /// observation and apply existing policy and live admission before dependent work.
    /// Identical replay is effect-free; a different original settlement cannot be replaced.
    ///
    /// ```no_run
    /// use ic_backup::store::{CaptureOperation, StatusHttpTransport, CaptureLookupSettlementError};
    /// fn settle(op: &mut CaptureOperation, transport: &StatusHttpTransport)
    ///     -> Result<(), CaptureLookupSettlementError> {
    ///     op.settle_capture_lookup(transport)
    /// }
    /// ```
    ///
    /// # Errors
    /// Refuses missing/nonterminal observations, different transport or settlement,
    /// failed authentication, changed custody and uncertain writes.
    pub fn settle_capture_lookup(
        &mut self,
        transport: &StatusHttpTransport,
    ) -> Result<(), CaptureLookupSettlementError> {
        let state = self.check()?;
        let lookup = state
            .lookup
            .as_ref()
            .ok_or(CaptureLookupSettlementError::NotObserved)?;
        if transport.lookup_commitment() != lookup.admission.transport {
            return Err(CaptureLookupSettlementError::Conflict);
        }
        let call = lookup.admission.call;
        let evidence = self.lookup_terminal(transport.root_key_der())?;
        let (original, _, result) = state.original(call)?;
        let bytes = evidence.encode(original.result_bytes)?;
        if let Some(result) = result {
            if lookup.settlement_root.as_deref() != Some(transport.root_key_der())
                || self.evidence(super::result_name(call), result)? != bytes
            {
                return Err(CaptureLookupSettlementError::Conflict);
            }
        } else if !state.pending(call) {
            return Err(CaptureLookupSettlementError::Conflict);
        }
        if self.check()? != state {
            return Err(CaptureError::Changed.into());
        }
        if result.is_none() {
            self.commit(
                &EventRecord::LookupSettled {
                    root_key_der: transport.root_key_der().to_vec(),
                    result: EvidenceRecord::new(&bytes),
                },
                Some((super::result_name(call), &bytes)),
            )?;
        }
        Ok(())
    }

    fn lookup_terminal(
        &self,
        root: &[u8],
    ) -> Result<CaptureCallEvidence, CaptureLookupSettlementError> {
        let state = self.check()?;
        let lookup = state
            .lookup
            .as_ref()
            .ok_or(CaptureLookupSettlementError::NotObserved)?;
        if lookup.phase != CaptureLookupPhase::Observed {
            return Err(CaptureLookupSettlementError::NotObserved);
        }
        let call = lookup.admission.call;
        let (admission, original, _) = state.original(call)?;
        let prepared =
            PreparedStatus::decode(&self.evidence(super::original_name(call), original)?)?;
        let request = self.evidence(
            "lookup-request.bin",
            lookup.request.as_ref().ok_or(CaptureError::Incomplete)?,
        )?;
        let policy = lookup.admission.policy()?;
        authenticate_capture_lookup(
            &self.plan,
            admission,
            &prepared,
            call,
            &request,
            policy.request,
        )?;
        let certificate = self.evidence(
            "lookup-certificate.bin",
            lookup
                .certificate
                .as_ref()
                .ok_or(CaptureError::Incomplete)?,
        )?;
        let outcome = observe_capture_certificate(
            &self.plan,
            admission,
            &prepared,
            call,
            &certificate,
            root,
            policy.observation,
        )?;
        let (rejected, response) = match outcome {
            CaptureObservation::Replied(bytes) => (false, bytes),
            CaptureObservation::Rejected(bytes) => (true, bytes),
            _ => return Err(CaptureLookupSettlementError::Unresolved),
        };
        if self.check()? != state {
            return Err(CaptureError::Changed.into());
        }
        Ok(CaptureCallEvidence {
            response,
            proof: certificate,
            rejected,
        })
    }

    pub(in crate::store::capture::advance) fn authenticate_lookup_settlement(
        &self,
        call: CaptureCall,
        evidence: &CaptureCallEvidence,
    ) -> Result<bool, CaptureLookupSettlementError> {
        let state = self.check()?;
        let Some(lookup) = state.lookup.as_ref().filter(|l| l.admission.call == call) else {
            return Ok(false);
        };
        let Some(root) = &lookup.settlement_root else {
            return Ok(false);
        };
        let expected = self.lookup_terminal(root)?;
        if expected.response != evidence.response
            || expected.proof != evidence.proof
            || expected.rejected != evidence.rejected
        {
            return Err(CaptureLookupSettlementError::Conflict);
        }
        if self.check()? != state {
            return Err(CaptureError::Changed.into());
        }
        Ok(true)
    }
}
