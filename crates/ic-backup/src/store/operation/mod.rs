//! Common capture call/observation facts, not effect authority.
//! The fixed procedure authenticates referenced evidence and applies policy before
//! selecting another effect. An observation reference alone does not prove termination.
pub(in crate::store) mod framing;
use framing::{Bounds, FRAME_BYTES, FRAMES, Frames, LEDGER_BYTES, LedgerError, encode_frame};
pub(in crate::store) mod intent;
use crate::store::BackupDigest;
use intent::Intent;
use serde::{Deserialize, Serialize};
use std::io::Read;

const DOMAIN: &[u8] = b"ic-backup/operation-journal/1\0";
const FRAME_RESERVATION: u64 = FRAME_BYTES as u64 + 36;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Fixed capture/restore step for diagnostics and evidence identity, never a caller-selected command.
pub enum Step {
    /// Authenticate stopped state, module and actual controller.
    InitialStatus,
    /// Inspect original snapshot capacity.
    ListSnapshots,
    /// Take one snapshot without replacement or uninstalling code.
    TakeSnapshot,
    /// Read the exact retained snapshot metadata.
    Metadata,
    /// Read one contiguous deterministic payload ordinal.
    Data(u32),
    /// Recheck stopped state, module and controller with a distinct request.
    FinalStatus,
    /// Authenticate stopped source-module state before restore.
    RestoreInitialStatus,
    /// Inspect capacity before non-replacing allocation.
    RestoreListSnapshots,
    /// Allocate a fresh uploaded-metadata snapshot.
    AllocateSnapshot,
    /// Upload one original payload ordinal.
    Upload(u32),
    /// Compare exact restorable destination metadata.
    ReadbackMetadata,
    /// Compare one destination ordinal with original source bytes.
    Readback(u32),
    /// Recheck stopped state, module and sender before rewind.
    BeforeLoadStatus,
    /// Load the verified destination under live application admission.
    LoadSnapshot,
    /// Authenticate stopped state, module and sender after load.
    AfterLoadStatus,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::store) struct ReferenceRecord {
    pub(in crate::store) bytes: u32,
    pub(in crate::store) sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(in crate::store) enum FactRecord {
    Created,
    LoadedStopped {
        completed_at_ns: u64,
    },
    CallReserved {
        step: Step,
        reference_ns: u64,
    },
    CallPrepared {
        step: Step,
        request: ReferenceRecord,
    },
    CallSubmitted {
        step: Step,
    },
    CallResult {
        step: Step,
        result: ReferenceRecord,
    },
    ObservationReserved {
        step: Step,
        reference_ns: u64,
    },
    ObservationPrepared {
        step: Step,
        request: ReferenceRecord,
    },
    ObservationSubmitted {
        step: Step,
    },
    ObservationResult {
        step: Step,
        certificate: ReferenceRecord,
    },
    MaterializationStarted,
    PublishReady {
        completed_at_ns: u64,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FrameRecord {
    sequence: u32,
    fact: FactRecord,
}

// These are projections of the immutable intent/profile, never per-call input or
// a persisted mutable allowance. The immutable intent decoder owns their bounds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::store) struct Allowance {
    pub(in crate::store) signatures: u32,
    pub(in crate::store) network: u32,
    pub(in crate::store) observations: u32,
    pub(in crate::store) evidence: u64,
    pub(in crate::store) signer_network: u32,
    pub(in crate::store) request_bytes: u32,
    pub(in crate::store) result_bytes: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::store) struct Invocation {
    pub(in crate::store) reference_ns: u64,
    pub(in crate::store) request: Option<ReferenceRecord>,
    pub(in crate::store) submitted: bool,
    pub(in crate::store) result: Option<ReferenceRecord>,
}
impl Invocation {
    fn reserved(reference_ns: u64) -> Result<Self, LedgerError> {
        if reference_ns == 0 {
            return Err(LedgerError::Transition);
        }
        Ok(Self {
            reference_ns,
            request: None,
            submitted: false,
            result: None,
        })
    }
    fn prepare(&mut self, request: &ReferenceRecord, maximum: u32) -> Result<(), LedgerError> {
        if self.request.is_some() || self.submitted || request.bytes == 0 || request.bytes > maximum
        {
            return Err(LedgerError::Transition);
        }
        self.request = Some(request.clone());
        Ok(())
    }
    fn submit(&mut self) -> Result<(), LedgerError> {
        if self.request.is_none() || self.submitted {
            return Err(LedgerError::Transition);
        }
        self.submitted = true;
        Ok(())
    }
    fn settle(&mut self, result: &ReferenceRecord, maximum: u32) -> Result<(), LedgerError> {
        if !self.submitted || self.result.is_some() || result.bytes == 0 || result.bytes > maximum {
            return Err(LedgerError::Transition);
        }
        self.result = Some(result.clone());
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::store) struct Call {
    pub(in crate::store) step: Step,
    pub(in crate::store) update: Invocation,
    pub(in crate::store) observation: Option<Invocation>,
}
impl Call {
    // This only establishes that evidence exists for semantic replay. It does not
    // infer a terminal outcome, or policy success, from an observation certificate.
    fn has_outcome_evidence(&self) -> bool {
        self.update.result.is_some()
            || self
                .observation
                .as_ref()
                .is_some_and(|o| o.result.is_some())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::store) struct Journal {
    pub(in crate::store) allowance: Allowance,
    pub(in crate::store) calls: Vec<Call>,
    pub(in crate::store) materialization: bool,
    pub(in crate::store) completed_at_ns: Option<u64>,
    pub(in crate::store) signatures: u32,
    pub(in crate::store) network: u32,
    pub(in crate::store) observations: u32,
    pub(in crate::store) evidence_reserved: u64,
    pub(in crate::store) frames_reserved: u32,
    pub(in crate::store) head: BackupDigest,
    pub(in crate::store) bytes: u64,
    pub(in crate::store) count: u32,
}
impl Journal {
    pub(in crate::store) fn allowance(intent: &impl Intent) -> Allowance {
        let limits = intent.view().limits();
        let profile = intent.view().profile();
        Allowance {
            signatures: limits.signing_requests,
            network: limits.network_requests,
            observations: limits.observations,
            evidence: limits.evidence_bytes,
            signer_network: profile.signer_network,
            request_bytes: profile.request_bytes,
            result_bytes: profile.result_bytes,
        }
    }

    pub(in crate::store) fn create(intent: &impl Intent) -> Result<Vec<u8>, LedgerError> {
        Self::initial(intent.view().encoded(), Self::allowance(intent))
    }

    pub(in crate::store) fn open(
        reader: impl Read,
        intent: &impl Intent,
    ) -> Result<Self, LedgerError> {
        Self::read(reader, intent.view().encoded(), Self::allowance(intent))
    }

    fn initial(intent: &[u8], allowance: Allowance) -> Result<Vec<u8>, LedgerError> {
        Self::validate_allowance(allowance)?;
        let bytes = encode_frame(Self::seed(intent), &Self::payload(0, FactRecord::Created)?)?;
        if bytes.len() as u64 > allowance.evidence {
            return Err(LedgerError::ByteLimit);
        }
        Ok(bytes)
    }

    fn read(reader: impl Read, intent: &[u8], allowance: Allowance) -> Result<Self, LedgerError> {
        let initial_bytes = Self::initial(intent, allowance)?.len() as u64;
        let seed = Self::seed(intent);
        let mut frames = Frames::new(
            reader,
            seed,
            Bounds {
                frame_bytes: FRAME_BYTES,
                frames: FRAMES,
                bytes: LEDGER_BYTES.min(allowance.evidence),
            },
        );
        let mut state = Self {
            allowance,
            calls: Vec::new(),
            materialization: false,
            completed_at_ns: None,
            signatures: 0,
            network: 0,
            observations: 0,
            evidence_reserved: initial_bytes,
            frames_reserved: 1,
            head: seed,
            bytes: 0,
            count: 0,
        };
        while let Some(bytes) = frames.next()? {
            let record: FrameRecord =
                serde_json::from_slice(&bytes).map_err(|_| LedgerError::Transition)?;
            if record.sequence != state.count
                || Self::payload(record.sequence, record.fact.clone())? != bytes
            {
                return Err(LedgerError::Transition);
            }
            if state.count == 0 {
                if record.fact != FactRecord::Created {
                    return Err(LedgerError::Transition);
                }
            } else {
                state.apply(&record.fact, allowance)?;
            }
            state.count += 1;
            state.bytes = frames.bytes;
            state.head = frames.head;
        }
        if state.count == 0 {
            return Err(LedgerError::Empty);
        }
        Ok(state)
    }

    // Candidate construction is pure. Only the store can retain/sync this frame,
    // invalidate a handle on uncertain writes and authorize an external invocation.
    pub(in crate::store) fn candidate(&self, fact: FactRecord) -> Result<Vec<u8>, LedgerError> {
        let allowance = self.allowance;
        let mut next = self.clone();
        next.apply(&fact, allowance)?;
        let frame = encode_frame(self.head, &Self::payload(self.count, fact)?)?;
        if self.count >= FRAMES
            || self
                .bytes
                .checked_add(frame.len() as u64)
                .is_none_or(|n| n > allowance.evidence.min(LEDGER_BYTES))
        {
            return Err(LedgerError::ByteLimit);
        }
        Ok(frame)
    }

    fn apply(&mut self, fact: &FactRecord, allowance: Allowance) -> Result<(), LedgerError> {
        if self.completed_at_ns.is_some() {
            return Err(LedgerError::Transition);
        }
        match fact {
            FactRecord::CallReserved { step, reference_ns } => {
                let previous = self.calls.last();
                let ordered = match (previous.map(|call| call.step), step) {
                    (None, Step::RestoreInitialStatus)
                    | (Some(Step::RestoreInitialStatus), Step::RestoreListSnapshots)
                    | (Some(Step::RestoreListSnapshots), Step::AllocateSnapshot)
                    | (Some(Step::AllocateSnapshot), Step::Upload(0))
                    | (Some(Step::Upload(_)), Step::ReadbackMetadata)
                    | (Some(Step::ReadbackMetadata), Step::Readback(0))
                    | (Some(Step::Readback(_)), Step::BeforeLoadStatus)
                    | (Some(Step::BeforeLoadStatus), Step::LoadSnapshot)
                    | (Some(Step::LoadSnapshot), Step::AfterLoadStatus) => !self.materialization,
                    (Some(Step::Upload(a)), Step::Upload(b))
                    | (Some(Step::Readback(a)), Step::Readback(b)) => {
                        !self.materialization && a.checked_add(1) == Some(*b)
                    }
                    (None, Step::InitialStatus)
                    | (Some(Step::InitialStatus), Step::ListSnapshots)
                    | (Some(Step::ListSnapshots), Step::TakeSnapshot)
                    | (Some(Step::TakeSnapshot), Step::Metadata)
                    | (Some(Step::Metadata), Step::Data(0)) => !self.materialization,
                    (Some(Step::Data(a)), Step::Data(b)) => {
                        !self.materialization && a.checked_add(1) == Some(*b)
                    }
                    (Some(Step::Data(_)), Step::FinalStatus) => self.materialization,
                    _ => false,
                };
                if !ordered || previous.is_some_and(|call| !call.has_outcome_evidence()) {
                    return Err(LedgerError::Transition);
                }
                if *step == Step::RestoreInitialStatus {
                    self.frames_reserved += 1;
                    self.evidence_reserved = self
                        .evidence_reserved
                        .checked_add(FRAME_RESERVATION)
                        .ok_or(LedgerError::ByteLimit)?;
                }
                let update = Invocation::reserved(*reference_ns)?;
                self.reserve(allowance, false)?;
                self.calls.push(Call {
                    step: *step,
                    update,
                    observation: None,
                });
            }
            FactRecord::CallPrepared { step, request } => self
                .call(*step)?
                .update
                .prepare(request, allowance.request_bytes)?,
            FactRecord::CallSubmitted { step } => self.call(*step)?.update.submit()?,
            FactRecord::CallResult { step, result } => self
                .call(*step)?
                .update
                .settle(result, allowance.result_bytes)?,
            FactRecord::ObservationReserved { step, reference_ns } => {
                let call = self.call(*step)?;
                if !call.update.submitted
                    || call.update.result.is_some()
                    || call.observation.is_some()
                {
                    return Err(LedgerError::Transition);
                }
                let observation = Invocation::reserved(*reference_ns)?;
                self.reserve(allowance, true)?;
                self.call(*step)?.observation = Some(observation);
            }
            FactRecord::ObservationPrepared { step, request } => self
                .observation(*step)?
                .prepare(request, allowance.request_bytes)?,
            FactRecord::ObservationSubmitted { step } => self.observation(*step)?.submit()?,
            FactRecord::ObservationResult { step, certificate } => self
                .observation(*step)?
                .settle(certificate, allowance.result_bytes)?,
            FactRecord::MaterializationStarted => {
                if self.materialization
                    || !self.calls.last().is_some_and(|call| {
                        matches!(call.step, Step::Data(_)) && call.has_outcome_evidence()
                    })
                {
                    return Err(LedgerError::Transition);
                }
                self.materialization = true;
                self.frames_reserved = self
                    .frames_reserved
                    .checked_add(2)
                    .ok_or(LedgerError::FrameCount)?;
                self.evidence_reserved = self
                    .evidence_reserved
                    .checked_add(2 * FRAME_RESERVATION + crate::store::MANIFEST_LIMIT as u64)
                    .ok_or(LedgerError::ByteLimit)?;
            }
            FactRecord::PublishReady { completed_at_ns } => {
                if *completed_at_ns == 0
                    || !self.materialization
                    || !self.calls.last().is_some_and(|call| {
                        call.step == Step::FinalStatus && call.has_outcome_evidence()
                    })
                {
                    return Err(LedgerError::Transition);
                }
                self.completed_at_ns = Some(*completed_at_ns);
            }
            FactRecord::LoadedStopped { completed_at_ns } => {
                if *completed_at_ns == 0
                    || self.materialization
                    || !self.calls.last().is_some_and(|c| {
                        c.step == Step::AfterLoadStatus && c.has_outcome_evidence()
                    })
                {
                    return Err(LedgerError::Transition);
                }
                self.completed_at_ns = Some(*completed_at_ns);
            }
            FactRecord::Created => return Err(LedgerError::Transition),
        }
        if self.signatures > allowance.signatures
            || self.network > allowance.network
            || self.observations > allowance.observations
        {
            return Err(LedgerError::Transition);
        }
        if self.evidence_reserved > allowance.evidence {
            return Err(LedgerError::ByteLimit);
        }
        if self.frames_reserved > FRAMES {
            return Err(LedgerError::FrameCount);
        }
        Ok(())
    }

    fn call(&mut self, step: Step) -> Result<&mut Call, LedgerError> {
        self.calls
            .last_mut()
            .filter(|call| call.step == step)
            .ok_or(LedgerError::Transition)
    }
    fn observation(&mut self, step: Step) -> Result<&mut Invocation, LedgerError> {
        let call = self.call(step)?;
        if call.update.result.is_some() {
            return Err(LedgerError::Transition);
        }
        call.observation.as_mut().ok_or(LedgerError::Transition)
    }
    fn reserve(&mut self, allowance: Allowance, observation: bool) -> Result<(), LedgerError> {
        self.signatures = self
            .signatures
            .checked_add(1)
            .ok_or(LedgerError::Transition)?;
        self.network = self
            .network
            .checked_add(allowance.signer_network)
            .and_then(|n| n.checked_add(1))
            .ok_or(LedgerError::Transition)?;
        self.observations += u32::from(observation);
        self.frames_reserved = self
            .frames_reserved
            .checked_add(4)
            .ok_or(LedgerError::FrameCount)?;
        let bytes = 4 * FRAME_RESERVATION
            + u64::from(allowance.request_bytes)
            + u64::from(allowance.result_bytes);
        self.evidence_reserved = self
            .evidence_reserved
            .checked_add(bytes)
            .ok_or(LedgerError::ByteLimit)?;
        Ok(())
    }
    fn validate_allowance(a: Allowance) -> Result<(), LedgerError> {
        if !(1..=8192).contains(&a.signatures)
            || !(1..=8192).contains(&a.network)
            || a.observations > a.network
            || a.signer_network >= a.network
            || !(1..=1024 * 1024 * 1024).contains(&a.evidence)
            || !(1..=crate::store::limits::REQUEST_BYTES).contains(&a.request_bytes)
            || !(1..=crate::store::limits::RESULT_BYTES).contains(&a.result_bytes)
        {
            return Err(LedgerError::Transition);
        }
        Ok(())
    }
    fn seed(intent: &[u8]) -> BackupDigest {
        BackupDigest::compute(&[DOMAIN, intent].concat())
    }
    fn payload(sequence: u32, fact: FactRecord) -> Result<Vec<u8>, LedgerError> {
        serde_json::to_vec(&FrameRecord { sequence, fact }).map_err(|_| LedgerError::Transition)
    }
}

mod completion;
pub(in crate::store) mod custody;
mod dispatch;
pub(in crate::store) mod replay;
pub(in crate::store) mod runtime;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod procedure_tests;

mod facade;
#[cfg(test)]
pub(in crate::store) mod fixtures;
pub use Step as OperationStep;
pub use custody::CaptureOperation;
pub use facade::{OperationOptions, OperationView};
pub use replay::Error as OperationReplayError;
pub use runtime::{
    Action as OperationAction, Connection as BackupConnection, Error as OperationAdvanceError,
    Guard as BackupApplicationGuard, Progress as OperationProgress, Signer as BackupSigner,
    Signing as OperationSigningContent, SigningRequest as OperationSigningRequest,
};
