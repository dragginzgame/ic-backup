//! Fixed capture/restore dispatch over the common recorder.
use super::custody::{Recorder, Role};
use super::manager::RootExchange;
use super::{FRAME_RESERVATION, FRAMES, FactRecord, Journal, ReferenceRecord, Step};
use crate::store::capture::{
    OperationStoreError,
    advance::{BackupHttpError, BackupHttpTransport, StatusEvidence},
    intent::{self},
};
use crate::store::operation::intent::{Intent, Plan};
use std::{
    future::Future,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Application permission required for the fixed next operation action.
pub enum Action {
    /// Inspect canister status or snapshot inventory; acquire no new application authority.
    Inspect,
    /// Take one snapshot without replacement or uninstalling code.
    TakeSnapshot,
    /// Read the retained snapshot under continuous application permission.
    ReadSnapshot,
    /// Allocate a fresh destination snapshot without replacement.
    AllocateSnapshot,
    /// Upload original source state into the allocated destination.
    UploadSnapshot,
    /// Rewind the stopped application by loading the verified destination.
    LoadSnapshot,
}

/// Exact derived content for one already-reserved sign-only invocation.
pub enum Signing<'a> {
    /// Sign the exact derived management call, without submission.
    Call {
        /// Exact fixed management method derived by Backup.
        method: &'static str,
        /// Exact canonical Platform arguments; do not reconstruct them.
        arguments: &'a [u8],
    },
    /// Sign the single original-request read-state path.
    Observation {
        /// Sole original ingress status path allowed for this lookup.
        request_id: [u8; 32],
    },
}

/// Original approval and finite costs for a single external signature.
pub struct SigningRequest<'a> {
    /// Original immutable operation approval.
    pub plan: Plan<'a>,
    /// Exact call or original-request observation to sign.
    pub signing: Signing<'a>,
    /// Absolute signing completion deadline in Unix nanoseconds.
    pub deadline_ns: u64,
    /// Exact required signed ingress expiry, never renewed on continuation.
    pub expires_ns: u64,
    /// Complete signed envelope byte ceiling.
    pub envelope_bytes: u32,
    /// Signing-service network ceiling already reserved by the operation.
    pub network_requests: u32,
}

/// Integration-owned bounded sign-only adapter. It never submits, retries or detaches work.
/// Enforce supplied byte/network limits while producing output; cancellation retains process custody.
pub trait Signer {
    /// Typed redacted adapter refusal; never include keys or sensitive bytes.
    type Error;
    /// Stable commitment to the adapter protocol and relevant configuration.
    fn protocol(&self) -> [u8; 32];
    /// Return only the bounded signed envelope for these exact approved inputs.
    fn sign(
        &mut self,
        request: SigningRequest<'_>,
    ) -> impl Future<Output = Result<Vec<u8>, Self::Error>> + Send;
}

/// Bounded local check of the original continuous application witness for this exact action.
/// It acquires no fresh maintenance and performs no effects. Integrations own external obligations.
pub trait Guard {
    /// Typed redacted adapter refusal; never include keys or sensitive bytes.
    type Error;
    /// Stable commitment to the adapter protocol and relevant configuration.
    fn protocol(&self) -> [u8; 32];
    /// Check the original continuous witness for the derived action, without effects.
    fn admit(
        &self,
        plan: Plan<'_>,
        action: Action,
        application: &[u8; 32],
    ) -> Result<(), Self::Error>;
}

/// One direct transport, external signer and action-aware application guard.
pub struct Connection<S, G> {
    pub(in crate::store) manager: Option<(
        crate::store::RootManager,
        &'static dyn crate::store::RootCodec,
    )>,
    pub(in crate::store) transport: BackupHttpTransport,
    pub(in crate::store) signer: S,
    pub(in crate::store) guard: G,
}

#[derive(Debug, Error)]
/// Operation refusal or uncertainty. No variant grants retry or renewed authority.
pub enum Error<S, G> {
    /// Received evidence could not be retained under original custody. No retry is allowed.
    #[error("received certificate could not be recorded: {source}")]
    AfterResponse {
        /// Original typed refusal.
        source: Box<Error<S, G>>,
        /// Bounded sensitive bytes, requiring independent authentication on recovery.
        received: crate::store::ReceivedCertificate,
    },
    #[error(transparent)]
    /// Original local files, ownership or finite admission refused.
    Capture(#[from] OperationStoreError),
    #[error(transparent)]
    /// Original immutable approval or historical evidence refused.
    CapturePlan(#[from] intent::Error),
    #[error(transparent)]
    /// Retained custody, authentication or fixed policy refused.
    Replay(#[from] super::replay::Error),
    #[error(transparent)]
    /// One-shot HTTP refused or its outcome is uncertain.
    Http(#[from] BackupHttpError),
    #[error("connection differs from the original approved profile")]
    /// Connection differs from the original approved profile.
    Binding,
    #[error("original live deadline or clock refused")]
    /// Original deadline or host clock refused.
    Deadline,
    #[error("external signing failed; reservation remains spent")]
    /// External signature failed; its reservation stays spent.
    Signer(S),
    #[error("original application authority refused")]
    /// Original application witness is unavailable for this action.
    Guard(G),
    #[error("the original call is no longer pending")]
    /// The original call no longer has an unresolved outcome.
    Resolved,
}

impl<S, G> Error<S, G> {
    /// Preserve these original bytes before dropping the error. They grant no authority.
    #[must_use]
    pub fn received_certificate(&self) -> Option<&crate::store::ReceivedCertificate> {
        match self {
            Self::AfterResponse { received, .. } => Some(received),
            Self::Http(error) => error.received_certificate(),
            _ => None,
        }
    }
    fn after_response(self, request_id: [u8; 32], certificate: Vec<u8>) -> Self {
        Self::AfterResponse {
            source: Box::new(self),
            received: crate::store::ReceivedCertificate {
                request_id,
                certificate,
            },
        }
    }
}

impl<S, G> From<crate::store::capture::procedure::Error> for Error<S, G> {
    fn from(error: crate::store::capture::procedure::Error) -> Self {
        Self::Replay(error.into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Result of one fixed advance or explicit observation; spending is never refunded.
pub enum Progress<T = crate::store::BackupSummary> {
    /// One original management call or observation was durably recorded.
    Advanced,
    /// An original reservation is pending; never repeat its signing or submission.
    NeedsAttention(Step),
    /// Exact completed receipt, verified against original evidence and payload.
    Complete(Box<T>),
}

pub(in crate::store) fn now() -> Option<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_nanos()
        .try_into()
        .ok()
}
impl<S: Signer, G: Guard> Connection<S, G> {
    pub(in crate::store) fn admit(
        &self,
        intent: Plan<'_>,
        action: Action,
        reference: u64,
        signing: bool,
    ) -> Result<(), Error<S::Error, G::Error>> {
        let profile = intent.profile();
        if let Some(manager) = &profile.manager {
            if self.manager.as_ref().is_none_or(|(authority, codec)| {
                authority != manager || codec.protocol() != manager.protocol
            }) || profile
                .codec
                .is_none_or(|codec| codec.protocol() != manager.protocol)
            {
                return Err(intent::Error::ManagerExchange.into());
            }
        } else if self.manager.is_some() {
            return Err(Error::Binding);
        }
        self.transport
            .admit_binding(intent.target().network, intent.trust_root())?;
        if profile.transport != self.transport.commitment()
            || profile.root_der != self.transport.root_key_der()
            || profile.signer != self.signer.protocol()
            || profile.guard != self.guard.protocol()
        {
            return Err(Error::Binding);
        }
        let admission = intent.admission(reference)?;
        let current = now().ok_or(Error::Deadline)?;
        let deadline = if signing {
            admission.prepare_deadline_ns
        } else {
            admission.submit_deadline_ns
        };
        if current < reference || current >= deadline {
            return Err(Error::Deadline);
        }
        self.guard
            .admit(intent, action, &profile.application)
            .map_err(Error::Guard)
    }

    async fn sign(
        &mut self,
        intent: Plan<'_>,
        action: Action,
        reference: u64,
        signing: Signing<'_>,
    ) -> Result<Vec<u8>, Error<S::Error, G::Error>> {
        self.admit(intent, action, reference, true)?;
        let admission = intent.admission(reference)?;
        let current = now().ok_or(Error::Deadline)?;
        let remaining = admission
            .prepare_deadline_ns
            .checked_sub(current)
            .filter(|n| *n > 0)
            .ok_or(Error::Deadline)?;
        let envelope = tokio::time::timeout(
            Duration::from_nanos(remaining),
            self.signer.sign(SigningRequest {
                plan: intent,
                signing,
                deadline_ns: admission.prepare_deadline_ns,
                expires_ns: admission.submit_deadline_ns,
                envelope_bytes: intent.profile().request_bytes,
                network_requests: admission.prepare_requests,
            }),
        )
        .await
        .map_err(|_| Error::Deadline)?
        .map_err(Error::Signer)?;
        self.admit(intent, action, reference, true)?;
        if envelope.len() > intent.profile().request_bytes as usize {
            return Err(Error::Binding);
        }
        Ok(envelope)
    }
}

impl Journal {
    // Admit the whole known remainder before take or the first payload request.
    // This is a pure ceiling check, not another persisted reservation or schedule.
    pub(in crate::store) fn remaining(&self, calls: u32) -> Result<(), OperationStoreError> {
        self.check_manager_slots(calls)?;
        let a = self.allowance;
        let (local_frames, local_bytes) = if self
            .calls
            .first()
            .is_some_and(|c| c.step == Step::RestoreInitialStatus)
        {
            (0, 0)
        } else {
            (
                2,
                2 * FRAME_RESERVATION + crate::store::MANIFEST_LIMIT as u64,
            )
        };
        let fits = (|| {
            let network = calls.checked_mul(
                a.signer_network
                    .checked_add(1 + u32::from(a.manager_reply_bytes.is_some()))?,
            )?;
            let frames = calls.checked_mul(4)?.checked_add(local_frames)?;
            let evidence = u64::from(calls)
                .checked_mul(
                    4 * FRAME_RESERVATION + u64::from(a.request_bytes) + u64::from(a.result_bytes),
                )?
                .checked_add(local_bytes)?;
            Some(
                self.signatures.checked_add(calls)? <= a.signatures
                    && self.network.checked_add(network)? <= a.network
                    && self.frames_reserved.checked_add(frames)? <= FRAMES
                    && self.evidence_reserved.checked_add(evidence)? <= a.evidence,
            )
        })()
        .unwrap_or(false);
        if fits {
            Ok(())
        } else {
            Err(OperationStoreError::StatusAdmission)
        }
    }
}

pub(in crate::store) struct Applied {
    pub request_id: [u8; 32],
    pub rejected: bool,
    pub response: Vec<u8>,
}
#[expect(
    private_bounds,
    reason = "the recorder is sealed to the two concrete original intent types"
)]
impl<P: Intent> Recorder<P> {
    pub(in crate::store) fn read_role(
        &self,
        call: &super::Call,
        role: Role,
    ) -> Result<Vec<u8>, OperationStoreError> {
        let reference = call
            .reference(role)
            .ok_or(OperationStoreError::Incomplete)?;
        self.read(&call.step.name(role), reference)
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "one private dispatch takes the checked journal, fixed action and exact derived call plus source custody"
    )]
    pub(in crate::store) async fn run<S: Signer, G: Guard>(
        &mut self,
        mut journal: Journal,
        connection: &mut Connection<S, G>,
        step: Step,
        action: Action,
        exchange: RootExchange,
        effect: &(super::effect::Effect, usize),
        mut source: impl FnMut() -> Result<(), OperationStoreError>,
    ) -> Result<Option<Applied>, Error<S::Error, G::Error>> {
        let original = self.intent.clone();
        let intent = original.view();
        let call = journal.calls.last().filter(|c| c.step == step);
        let pending = match exchange {
            RootExchange::Execute => call.map(|call| &call.update),
            RootExchange::Lookup => {
                let call = call
                    .filter(|call| call.update.submitted)
                    .ok_or(Error::Resolved)?;
                call.manager_lookup.as_ref()
            }
        };
        if pending.is_some_and(|invocation| invocation.submitted || invocation.request.is_none()) {
            return Ok(None);
        }
        let reference = match pending {
            Some(invocation) => invocation.reference_ns,
            None => now().ok_or(Error::Deadline)?,
        };
        let execute_reference = call.map_or(reference, |call| call.update.reference_ns);
        let original_call =
            super::manager::Call::prepare(intent, step, execute_reference, &effect.0, effect.1)?;
        let (method, arguments, role, original_id) = match exchange {
            RootExchange::Execute => {
                let (method, args) = original_call.execute();
                (method, args, Role::Request, None)
            }
            RootExchange::Lookup => {
                let super::manager::Call::Root { binding, .. } = &original_call else {
                    return Err(Error::Binding);
                };
                let call = call.ok_or(Error::Resolved)?;
                let original = intent
                    .decode_request(execute_reference, self.read_role(call, Role::Request)?)?;
                (
                    "snapshot_lookup",
                    binding.lookup(),
                    Role::ManagerLookupRequest,
                    Some(original.request_id),
                )
            }
        };
        if pending.is_none() {
            source()?;
            connection.admit(intent, action, reference, true)?;
            let reserved = self.append(
                &journal,
                match exchange {
                    RootExchange::Execute => FactRecord::CallReserved {
                        step,
                        reference_ns: reference,
                    },
                    RootExchange::Lookup => FactRecord::ManagerLookupReserved {
                        step,
                        reference_ns: reference,
                    },
                },
                None,
            )?;
            let envelope = connection
                .sign(
                    intent,
                    action,
                    reference,
                    Signing::Call { method, arguments },
                )
                .await?;
            source()?;
            if self.check()? != reserved {
                return Err(OperationStoreError::Changed.into());
            }
            let prepared = intent.decode_request(reference, envelope)?;
            intent.authenticate_ingress(reference, &prepared, method, arguments)?;
            let request = ReferenceRecord::new(&prepared.envelope)?;
            journal = self.append(
                &reserved,
                match exchange {
                    RootExchange::Execute => FactRecord::CallPrepared { step, request },
                    RootExchange::Lookup => FactRecord::ManagerLookupPrepared { step, request },
                },
                Some(&prepared.envelope),
            )?;
        }
        let call = journal
            .calls
            .last()
            .ok_or(OperationStoreError::Incomplete)?;
        let prepared = intent.decode_request(reference, self.read_role(call, role)?)?;
        if original_id == Some(prepared.request_id) {
            return Err(Error::Binding);
        }
        intent.authenticate_ingress(reference, &prepared, method, arguments)?;
        source()?;
        connection.admit(intent, action, reference, false)?;
        let submitted = self.append(
            &journal,
            match exchange {
                RootExchange::Execute => FactRecord::CallSubmitted { step },
                RootExchange::Lookup => FactRecord::ManagerLookupSubmitted { step },
            },
            None,
        )?;
        source()?;
        connection.admit(intent, action, reference, false)?;
        source()?;
        if self.check()? != submitted {
            return Err(OperationStoreError::Changed.into());
        }
        let evidence = connection
            .transport
            .submit_expected(
                intent.requirements(),
                &intent.admission(reference)?,
                &prepared,
                method,
                arguments,
            )
            .await?;
        let retained = (|| -> Result<(), Error<S::Error, G::Error>> {
            source()?;
            if self.check()? != submitted {
                return Err(OperationStoreError::Changed.into());
            }
            // submit_expected authenticated these owned bytes against this exact
            // request/root/window. Retain them before applying procedure policy.
            let bytes = evidence.encode(intent.profile().result_bytes)?;
            let result = ReferenceRecord::new(&bytes)?;
            self.append(
                &submitted,
                match exchange {
                    RootExchange::Execute => FactRecord::CallResult { step, result },
                    RootExchange::Lookup => FactRecord::ManagerLookupResult { step, result },
                },
                Some(&bytes),
            )?;

            Ok(())
        })();
        if let Err(error) = retained {
            return Err(error.after_response(prepared.request_id, evidence.proof));
        }
        Ok(original_call
            .result(exchange, evidence.rejected, evidence.response)?
            .map(|(rejected, response)| Applied {
                request_id: original_id.unwrap_or(prepared.request_id),
                rejected,
                response,
            }))
    }
    pub(in crate::store) async fn observe<S: Signer, G: Guard>(
        &mut self,
        mut journal: Journal,
        connection: &mut Connection<S, G>,
        mut source: impl FnMut() -> Result<(), OperationStoreError>,
    ) -> Result<Option<Applied>, Error<S::Error, G::Error>> {
        let original_intent = self.intent.clone();
        let intent = original_intent.view();
        let call = journal
            .calls
            .last()
            .ok_or(OperationStoreError::Incomplete)?;
        let step = call.step;
        if !call.update.submitted || call.update.result.is_some() {
            return Err(Error::Resolved);
        }
        source()?;
        if call
            .observation
            .as_ref()
            .is_some_and(|o| o.submitted || o.request.is_none())
        {
            return Ok(None);
        }
        let original = intent.decode_request(
            call.update.reference_ns,
            self.read_role(call, Role::Request)?,
        )?;
        if call.observation.is_none() {
            let reference = now().ok_or(Error::Deadline)?;
            connection.admit(intent, Action::Inspect, reference, true)?;
            let reserved = self.append(
                &journal,
                FactRecord::ObservationReserved {
                    step,
                    reference_ns: reference,
                },
                None,
            )?;
            let envelope = connection
                .sign(
                    intent,
                    Action::Inspect,
                    reference,
                    Signing::Observation {
                        request_id: original.request_id,
                    },
                )
                .await?;
            source()?;
            if self.check()? != reserved {
                return Err(OperationStoreError::Changed.into());
            }
            intent.authenticate_lookup(reference, &original, &envelope)?;
            journal = self.append(
                &reserved,
                FactRecord::ObservationPrepared {
                    step,
                    request: ReferenceRecord::new(&envelope)?,
                },
                Some(&envelope),
            )?;
        }
        let call = journal
            .calls
            .last()
            .ok_or(OperationStoreError::Incomplete)?;
        let observation = call
            .observation
            .as_ref()
            .ok_or(OperationStoreError::Incomplete)?;
        let reference = observation.reference_ns;
        let lookup = self.read_role(call, Role::ObservationRequest)?;
        intent.authenticate_lookup(reference, &original, &lookup)?;
        source()?;
        connection.admit(intent, Action::Inspect, reference, false)?;
        let submitted = self.append(&journal, FactRecord::ObservationSubmitted { step }, None)?;
        source()?;
        connection.admit(intent, Action::Inspect, reference, false)?;
        source()?;
        if self.check()? != submitted {
            return Err(OperationStoreError::Changed.into());
        }
        let certificate = connection
            .transport
            .read_state_certificate(
                intent.requirements().effective_canister(),
                &lookup,
                original.request_id,
                intent.admission(reference)?.submit_deadline_ns,
                intent.profile().result_bytes as usize,
            )
            .await?;
        let retained = (|| -> Result<_, Error<S::Error, G::Error>> {
            source()?;
            if self.check()? != submitted {
                return Err(OperationStoreError::Changed.into());
            }
            let observed = intent.observe(reference, &original, &certificate)?;
            if now().is_none_or(|n| {
                n >= intent
                    .admission(reference)
                    .map_or(0, |a| a.submit_deadline_ns)
            }) {
                return Err(Error::Deadline);
            }
            let settled = self.append(
                &submitted,
                FactRecord::ObservationResult {
                    step,
                    certificate: ReferenceRecord::new(&certificate)?,
                },
                Some(&certificate),
            )?;

            let _ = settled;
            Ok(observed)
        })();
        let observed =
            retained.map_err(|error| error.after_response(original.request_id, certificate))?;
        Ok(match observed {
            crate::store::capture::CaptureObservation::Replied(response) => Some(Applied {
                request_id: original.request_id,
                rejected: false,
                response,
            }),
            crate::store::capture::CaptureObservation::Rejected(response) => Some(Applied {
                request_id: original.request_id,
                rejected: true,
                response,
            }),
            _ => None,
        })
    }
    pub(in crate::store) fn import_reply(
        &mut self,
        journal: &Journal,
        effect: &(super::effect::Effect, usize),
        exchange: RootExchange,
        certificate: &[u8],
    ) -> Result<Option<Applied>, super::replay::Error> {
        let intent = self.intent.view();
        let call = journal
            .calls
            .last()
            .ok_or(OperationStoreError::Incomplete)?;
        let execute_reference = call.update.reference_ns;
        let original_call = super::manager::Call::prepare(
            intent,
            call.step,
            execute_reference,
            &effect.0,
            effect.1,
        )?;
        let original =
            intent.decode_request(execute_reference, self.read_role(call, Role::Request)?)?;
        let (method, arguments) = original_call.execute();
        intent.authenticate_ingress(execute_reference, &original, method, arguments)?;
        let original_id = original.request_id;
        let (request, invocation) = match exchange {
            RootExchange::Execute => (original, &call.update),
            RootExchange::Lookup => {
                let super::manager::Call::Root { binding, .. } = &original_call else {
                    return Err(super::replay::Error::History);
                };
                let lookup = call
                    .manager_lookup
                    .as_ref()
                    .ok_or(OperationStoreError::Incomplete)?;
                let request = intent.decode_request(
                    lookup.reference_ns,
                    self.read_role(call, Role::ManagerLookupRequest)?,
                )?;
                intent.authenticate_ingress(
                    lookup.reference_ns,
                    &request,
                    "snapshot_lookup",
                    binding.lookup(),
                )?;
                if request.request_id == original_id {
                    return Err(super::replay::Error::History);
                }
                (request, lookup)
            }
        };
        if !invocation.submitted || invocation.result.is_some() {
            return Err(super::replay::Error::History);
        }
        let (rejected, response) = intent
            .certificate()
            .response(
                &intent.admission(invocation.reference_ns)?,
                &request,
                certificate,
                &intent.profile().root_der,
            )
            .map_err(intent::Error::from)?;
        let evidence = StatusEvidence {
            rejected,
            response,
            proof: certificate.to_vec(),
        };
        let bytes = evidence.encode(intent.profile().result_bytes)?;
        let result = ReferenceRecord::new(&bytes)?;
        let step = call.step;
        let fact = match exchange {
            RootExchange::Execute => FactRecord::CallResult { step, result },
            RootExchange::Lookup => FactRecord::ManagerLookupResult { step, result },
        };
        self.append(journal, fact, Some(&bytes))?;
        Ok(original_call
            .result(exchange, rejected, evidence.response)?
            .map(|(rejected, response)| Applied {
                request_id: original_id,
                rejected,
                response,
            }))
    }
    pub(in crate::store) fn import_observation(
        &mut self,
        journal: &Journal,
        certificate: &[u8],
    ) -> Result<Option<Applied>, super::replay::Error> {
        let intent = self.intent.view();
        let call = journal
            .calls
            .last()
            .ok_or(OperationStoreError::Incomplete)?;
        let observation = call
            .observation
            .as_ref()
            .ok_or(OperationStoreError::Incomplete)?;
        if !call.update.submitted
            || call.update.result.is_some()
            || !observation.submitted
            || observation.result.is_some()
        {
            return Err(super::replay::Error::History);
        }
        let original = intent.decode_request(
            call.update.reference_ns,
            self.read_role(call, Role::Request)?,
        )?;
        intent.authenticate_lookup(
            observation.reference_ns,
            &original,
            &self.read_role(call, Role::ObservationRequest)?,
        )?;
        let observed = intent.observe(observation.reference_ns, &original, certificate)?;
        self.append(
            journal,
            FactRecord::ObservationResult {
                step: call.step,
                certificate: ReferenceRecord::new(certificate)?,
            },
            Some(certificate),
        )?;
        Ok(match observed {
            crate::store::capture::CaptureObservation::Replied(response) => Some(Applied {
                request_id: original.request_id,
                rejected: false,
                response,
            }),
            crate::store::capture::CaptureObservation::Rejected(response) => Some(Applied {
                request_id: original.request_id,
                rejected: true,
                response,
            }),
            _ => None,
        })
    }
}

impl<S, G> From<crate::store::restore::procedure::Error> for Error<S, G> {
    fn from(error: crate::store::restore::procedure::Error) -> Self {
        Self::Replay(error.into())
    }
}

#[cfg(test)]
mod tests;
