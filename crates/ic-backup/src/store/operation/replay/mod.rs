//! Semantic admission of retained call facts. No clock, signer, guard or network.

use super::{Journal, Step, runtime::Applied};
use crate::store::{
    capture::{
        CaptureObservation, OperationStoreError,
        advance::StatusEvidence,
        intent::{self, CapturePlan},
        procedure::{self, Next, Procedure},
    },
    operation::intent::{Intent, Plan},
};
use thiserror::Error;

///
/// Error
/// Original file custody, call authentication or fixed capture policy refused.
///

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    /// Original local files, ownership or finite admission refused.
    Capture(#[from] OperationStoreError),
    #[error(transparent)]
    /// Original retained intent or evidence failed authentication.
    Authentication(#[from] intent::Error),
    #[error(transparent)]
    /// Certified call result failed fixed capture policy.
    Policy(Box<procedure::Error>),
    #[error("retained call history does not follow the fixed capture procedure")]
    /// Retained call order or resolution conflicts with the fixed procedure.
    History,
    /// Certified restore result failed fixed restore policy.
    #[error(transparent)]
    Restore(Box<crate::store::restore::procedure::Error>),
}

impl From<procedure::Error> for Error {
    fn from(error: procedure::Error) -> Self {
        Self::Policy(Box::new(error))
    }
}

impl Journal {
    pub(in crate::store) fn replay<'a>(
        &self,
        intent: &'a CapturePlan,
        mut read: impl FnMut(&super::Call, super::custody::Role) -> Result<Vec<u8>, OperationStoreError>,
        mut verify_materialized: impl FnMut(&Procedure<'_>) -> Result<(), Error>,
    ) -> Result<Procedure<'a>, Error> {
        let mut flow = Procedure::new(intent)?;
        for (index, call) in self.calls.iter().enumerate() {
            if flow.next()? == Next::Materialize {
                if !self.materialization || call.step != Step::FinalStatus {
                    return Err(Error::History);
                }
                verify_materialized(&flow)?;
                flow.after_materialization()?;
            }
            if flow.next()? != Next::Call(call.step) {
                return Err(Error::History);
            }
            let effect = flow.effect()?;

            if let Some(result) = resolve_call(intent.view(), call, &effect, &mut read)? {
                flow.accept(
                    result.rejected,
                    call.step,
                    result.request_id,
                    &result.response,
                )?;
            } else {
                if index + 1 != self.calls.len()
                    || (self.materialization && call.step != Step::FinalStatus)
                {
                    return Err(Error::History);
                }

                return Ok(flow);
            }
        }
        if self.materialization && !matches!(flow.next()?, Next::Materialize | Next::Publish) {
            return Err(Error::History);
        }
        if self.completed_at_ns.is_some() && flow.next()? != Next::Publish {
            return Err(Error::History);
        }

        Ok(flow)
    }
}

impl From<crate::store::restore::procedure::Error> for Error {
    fn from(e: crate::store::restore::procedure::Error) -> Self {
        Self::Restore(Box::new(e))
    }
}

pub(in crate::store) fn resolve_call(
    intent: Plan<'_>,
    call: &super::Call,
    effect: &(super::effect::Effect, usize),
    read: &mut impl FnMut(&super::Call, super::custody::Role) -> Result<Vec<u8>, OperationStoreError>,
) -> Result<Option<Applied>, Error> {
    use super::custody::Role;
    if call.update.request.is_none() {
        return Ok(None);
    }
    let prepared = intent.decode_request(call.update.reference_ns, read(call, Role::Request)?)?;
    use super::manager::{Call, RootExchange};
    let original = Call::prepare(
        intent,
        call.step,
        call.update.reference_ns,
        &effect.0,
        effect.1,
    )?;
    let (method, arguments) = original.execute();
    intent.authenticate_ingress(call.update.reference_ns, &prepared, method, arguments)?;
    let mut terminal = None;
    if let Some(o) = &call.observation {
        if o.request.is_some() {
            intent.authenticate_lookup(
                o.reference_ns,
                &prepared,
                &read(call, Role::ObservationRequest)?,
            )?;
        }

        if o.result.is_some() {
            terminal = match intent.observe(
                o.reference_ns,
                &prepared,
                &read(call, Role::ObservationResult)?,
            )? {
                CaptureObservation::Replied(r) => Some((false, r)),
                CaptureObservation::Rejected(r) => Some((true, r)),
                _ => None,
            };
        }
    }
    if call.update.result.is_some() {
        if terminal.is_some() {
            return Err(Error::History);
        }
        let evidence = StatusEvidence::decode(&read(call, Role::Result)?)?;
        intent.authenticate_result(call.update.reference_ns, &prepared, &evidence)?;
        terminal = Some((evidence.rejected, evidence.response));
    }
    let mut inner = match terminal {
        Some((rejected, bytes)) => original.result(RootExchange::Execute, rejected, bytes)?,
        None => None,
    };
    if let Some(lookup) = &call.manager_lookup {
        let Call::Root { binding, .. } = &original else {
            return Err(Error::History);
        };

        if lookup.request.is_some() {
            let signed = intent
                .decode_request(lookup.reference_ns, read(call, Role::ManagerLookupRequest)?)?;
            if signed.request_id == prepared.request_id {
                return Err(Error::History);
            }
            intent.authenticate_ingress(
                lookup.reference_ns,
                &signed,
                "snapshot_lookup",
                binding.lookup(),
            )?;

            if lookup.result.is_some() {
                let evidence = StatusEvidence::decode(&read(call, Role::ManagerLookupResult)?)?;
                intent.authenticate_result(lookup.reference_ns, &signed, &evidence)?;

                if let Some(result) =
                    original.result(RootExchange::Lookup, evidence.rejected, evidence.response)?
                {
                    if inner.as_ref().is_some_and(|prior| prior != &result) {
                        return Err(Error::History);
                    }
                    inner = Some(result);
                }
            }
        }
    }

    Ok(inner.map(|(rejected, response)| Applied {
        request_id: prepared.request_id,
        rejected,
        response,
    }))
}
