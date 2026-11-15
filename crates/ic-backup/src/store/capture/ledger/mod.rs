//! Bounded chained frames and held original-ledger admission. Private ordered status and snapshot transitions.

use super::advance::{
    CaptureCall, StatusAdmissionRecord, arguments,
    snapshots::{CaptureSnapshotPhase, SnapshotAdmissionRecord, SnapshotState},
};
use super::{CaptureError, CapturePlan, open_optional};
use ic_host_artifacts::artifact::Sha256Digest;
use ic_host_fs::{capability::Directory, identity::FileIdentity};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{self, Read},
};
use thiserror::Error;

mod lookup;
use lookup::LookupState;

const CREATED: &[u8] = b"{\"sequence\":0,\"kind\":\"created\"}";
const FRAME_BYTES: usize = 16 * 1024;
const FRAMES: u32 = 65_536;
const LEDGER_BYTES: u64 = 128 * 1024 * 1024;

#[derive(Clone, Copy)]
struct Bounds {
    frame_bytes: usize,
    frames: u32,
    bytes: u64,
}
impl Bounds {
    fn for_plan(plan: &CapturePlan) -> Self {
        Self {
            frame_bytes: FRAME_BYTES,
            frames: FRAMES,
            bytes: LEDGER_BYTES.min(plan.request().limits.evidence_bytes),
        }
    }
}

fn seed(plan: &CapturePlan) -> Sha256Digest {
    Sha256Digest::compute(&[b"ic-backup/local-ledger/1\0".as_slice(), plan.encoded()].concat())
}
fn chain(previous: Sha256Digest, length: &[u8; 4], payload: &[u8]) -> Sha256Digest {
    Sha256Digest::compute(&[previous.as_bytes().as_slice(), length, payload].concat())
}

pub(super) fn initial(plan: &CapturePlan) -> Result<Vec<u8>, LedgerError> {
    let length = (CREATED.len() as u32).to_le_bytes();
    let digest = chain(seed(plan), &length, CREATED);
    let bytes = [length.as_slice(), CREATED, digest.as_bytes().as_slice()].concat();
    if bytes.len() as u64 > Bounds::for_plan(plan).bytes {
        return Err(LedgerError::ByteLimit);
    }
    Ok(bytes)
}

pub(super) fn admit(
    directory: &Directory,
    held: &File,
    plan: &CapturePlan,
) -> Result<State, CaptureError> {
    admit_with(directory, held, plan, |file, plan| replay(file, plan))
}

// The injected reader is also used for deterministic native closing-custody tests.
fn admit_with<T>(
    directory: &Directory,
    held: &File,
    plan: &CapturePlan,
    read: impl FnOnce(&mut File, &CapturePlan) -> Result<T, LedgerError>,
) -> Result<T, CaptureError> {
    let mut file = open_optional(directory, "ledger.bin")?.ok_or(CaptureError::Incomplete)?;
    let original = FileIdentity::from_file(held)?;
    let before = file.metadata()?;
    if FileIdentity::from_metadata(&before) != original {
        return Err(CaptureError::Changed);
    }
    if before.len() > Bounds::for_plan(plan).bytes {
        return Err(LedgerError::ByteLimit.into());
    }
    let result = read(&mut file, plan)?;
    let after = file.metadata()?;
    let named = open_optional(directory, "ledger.bin")?.ok_or(CaptureError::Incomplete)?;
    if FileIdentity::from_metadata(&after) != original
        || after.len() != before.len()
        || FileIdentity::from_file(&named)? != original
        || named.metadata()?.len() != before.len()
    {
        return Err(CaptureError::Changed);
    }
    Ok(result)
}

/// Retained initial-status stage; this projection grants no dispatch authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureStatusPhase {
    /// Original intent exists; no status preparation is reserved.
    Created,
    /// Signing allowance is consumed; the original signer outcome is unresolved.
    PreparationReserved,
    /// Original request bytes are retained; submission is not yet reserved.
    Prepared,
    /// Submission allowance is consumed; the original outcome is unresolved.
    SubmissionReserved,
    /// A result is retained; inspection alone does not authenticate or validate it.
    Settled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct EvidenceRecord {
    pub bytes: u32,
    pub sha256: [u8; 32],
}
impl EvidenceRecord {
    pub fn new(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.len() as u32,
            sha256: *Sha256Digest::compute(bytes).as_bytes(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum EventRecord {
    LookupSettled {
        root_key_der: Vec<u8>,
        result: EvidenceRecord,
    },
    LookupStarted {
        admission: super::advance::CaptureLookupAdmissionRecord,
    },
    LookupPrepared {
        request: EvidenceRecord,
    },
    LookupSubmissionReserved,
    LookupObserved {
        certificate: EvidenceRecord,
    },
    SnapshotStarted {
        sequence: u32,
        admission: SnapshotAdmissionRecord,
        arguments_sha256: [u8; 32],
    },
    SnapshotTakeReserved {
        sequence: u32,
        arguments_sha256: [u8; 32],
    },
    SnapshotPrepared {
        sequence: u32,
        call: CaptureCall,
        request: EvidenceRecord,
    },
    SnapshotSubmissionReserved {
        sequence: u32,
        call: CaptureCall,
    },
    SnapshotSettled {
        sequence: u32,
        call: CaptureCall,
        result: EvidenceRecord,
    },
    PreparationReserved {
        sequence: u32,
        admission: StatusAdmissionRecord,
        arguments_sha256: [u8; 32],
    },
    Prepared {
        sequence: u32,
        request: EvidenceRecord,
    },
    SubmissionReserved {
        sequence: u32,
    },
    Settled {
        sequence: u32,
        result: EvidenceRecord,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct State {
    pub lookup: Option<LookupState>,
    pub snapshot: SnapshotState,
    pub phase: CaptureStatusPhase,
    pub admission: Option<StatusAdmissionRecord>,
    pub request: Option<EvidenceRecord>,
    pub result: Option<EvidenceRecord>,
    pub head: Sha256Digest,
    pub bytes: u64,
    count: u32,
}
impl State {
    fn apply(&mut self, event: &EventRecord, plan: &CapturePlan) -> Result<(), LedgerError> {
        if matches!(
            event,
            EventRecord::LookupStarted { .. }
                | EventRecord::LookupPrepared { .. }
                | EventRecord::LookupSubmissionReserved
                | EventRecord::LookupObserved { .. }
                | EventRecord::LookupSettled { .. }
        ) {
            self.apply_lookup(event, plan)?;
        } else {
            self.apply_original(event, plan)?;
        }
        self.lookup_budget(plan)
    }
    fn apply_original(
        &mut self,
        event: &EventRecord,
        plan: &CapturePlan,
    ) -> Result<(), LedgerError> {
        if matches!(
            event,
            EventRecord::SnapshotStarted { .. }
                | EventRecord::SnapshotTakeReserved { .. }
                | EventRecord::SnapshotPrepared { .. }
                | EventRecord::SnapshotSubmissionReserved { .. }
                | EventRecord::SnapshotSettled { .. }
        ) {
            if self.phase != CaptureStatusPhase::Settled {
                return Err(LedgerError::Transition);
            }
            return self.snapshot.apply(
                event,
                plan,
                self.admission.as_ref().ok_or(LedgerError::Transition)?,
            );
        }
        match (self.phase, event) {
            (
                CaptureStatusPhase::Created,
                EventRecord::PreparationReserved {
                    sequence: 1,
                    admission,
                    arguments_sha256,
                },
            ) => {
                admission
                    .validate(plan)
                    .map_err(|_| LedgerError::Transition)?;
                let args = arguments(plan).map_err(|_| LedgerError::Transition)?;
                if Sha256Digest::compute(&args).as_bytes() != arguments_sha256 {
                    return Err(LedgerError::Transition);
                }
                self.admission = Some(admission.clone());
                self.phase = CaptureStatusPhase::PreparationReserved;
            }
            (
                CaptureStatusPhase::PreparationReserved,
                EventRecord::Prepared {
                    sequence: 2,
                    request,
                },
            ) => {
                let cap = self
                    .admission
                    .as_ref()
                    .ok_or(LedgerError::Transition)?
                    .request_bytes;
                if !(46..=cap).contains(&request.bytes) {
                    return Err(LedgerError::Transition);
                }
                self.request = Some(request.clone());
                self.phase = CaptureStatusPhase::Prepared;
            }
            (CaptureStatusPhase::Prepared, EventRecord::SubmissionReserved { sequence: 3 }) => {
                self.phase = CaptureStatusPhase::SubmissionReserved
            }
            (
                CaptureStatusPhase::SubmissionReserved,
                EventRecord::Settled {
                    sequence: 4,
                    result,
                },
            ) => {
                let cap = self
                    .admission
                    .as_ref()
                    .ok_or(LedgerError::Transition)?
                    .result_bytes;
                if !(7..=cap).contains(&result.bytes) {
                    return Err(LedgerError::Transition);
                }
                self.result = Some(result.clone());
                self.phase = CaptureStatusPhase::Settled;
            }
            _ => return Err(LedgerError::Transition),
        }
        Ok(())
    }
    pub fn candidate(
        &self,
        event: &EventRecord,
        plan: &CapturePlan,
    ) -> Result<Vec<u8>, CaptureError> {
        let mut next = self.clone();
        next.apply(event, plan)?;
        let payload = serde_json::to_vec(event)?;
        if payload.len() > FRAME_BYTES {
            return Err(LedgerError::FrameSize.into());
        }
        if self.count == FRAMES {
            return Err(LedgerError::FrameCount.into());
        }
        if self.bytes + payload.len() as u64 + 36 > Bounds::for_plan(plan).bytes {
            return Err(LedgerError::ByteLimit.into());
        }
        let length = (payload.len() as u32).to_le_bytes();
        Ok([
            length.as_slice(),
            &payload,
            chain(self.head, &length, &payload).as_bytes(),
        ]
        .concat())
    }
}

fn replay(reader: impl Read, plan: &CapturePlan) -> Result<State, LedgerError> {
    let mut frames = Frames::new(reader, seed(plan), Bounds::for_plan(plan));
    let mut state = State {
        lookup: None,
        snapshot: SnapshotState::new(),
        phase: CaptureStatusPhase::Created,
        admission: None,
        request: None,
        result: None,
        head: seed(plan),
        bytes: 0,
        count: 0,
    };
    while let Some(payload) = frames.next()? {
        if frames.count == 1 {
            if payload != CREATED {
                return Err(LedgerError::Transition);
            }
        } else {
            let event: EventRecord =
                serde_json::from_slice(&payload).map_err(|_| LedgerError::Transition)?;
            if serde_json::to_vec(&event).map_err(|_| LedgerError::Transition)? != payload {
                return Err(LedgerError::Transition);
            }
            state.apply(&event, plan)?;
        }
    }
    if frames.count == 0 {
        return Err(LedgerError::Empty);
    }
    state.head = frames.head;
    state.bytes = frames.bytes;
    state.count = frames.count;
    Ok(state)
}

struct Frames<R> {
    reader: R,
    head: Sha256Digest,
    count: u32,
    bytes: u64,
    bounds: Bounds,
}
impl<R: Read> Frames<R> {
    fn new(reader: R, head: Sha256Digest, bounds: Bounds) -> Self {
        Self {
            reader,
            head,
            count: 0,
            bytes: 0,
            bounds,
        }
    }

    fn next(&mut self) -> Result<Option<Vec<u8>>, LedgerError> {
        let mut length = [0; 4];
        // One-byte EOF probe distinguishes a complete prefix from any partial tail.
        loop {
            match self.reader.read(&mut length[..1]) {
                Ok(0) => return Ok(None),
                Ok(_) => break,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) => return Err(error.into()),
            }
        }
        if self.count == self.bounds.frames {
            return Err(LedgerError::FrameCount);
        }
        if self.bounds.bytes.saturating_sub(self.bytes) < 4 {
            return Err(LedgerError::ByteLimit);
        }
        read_exact(&mut self.reader, &mut length[1..])?;
        let size = u32::from_le_bytes(length) as usize;
        if size == 0 || size > self.bounds.frame_bytes {
            return Err(LedgerError::FrameSize);
        }
        let next_bytes = self
            .bytes
            .checked_add(size as u64 + 36)
            .ok_or(LedgerError::ByteLimit)?;
        if next_bytes > self.bounds.bytes {
            return Err(LedgerError::ByteLimit);
        }
        // Every allocating read follows both the per-frame and cumulative bounds.
        let mut payload = vec![0; size];
        read_exact(&mut self.reader, &mut payload)?;
        let mut digest = [0; 32];
        read_exact(&mut self.reader, &mut digest)?;
        let expected = chain(self.head, &length, &payload);
        if expected.as_bytes() != &digest {
            return Err(LedgerError::Chain);
        }
        self.head = expected;
        self.count += 1;
        self.bytes = next_bytes;
        Ok(Some(payload))
    }
}
fn read_exact(reader: &mut impl Read, bytes: &mut [u8]) -> Result<(), LedgerError> {
    reader.read_exact(bytes).map_err(|error| {
        if error.kind() == io::ErrorKind::UnexpectedEof {
            LedgerError::Truncated
        } else {
            LedgerError::Io(error)
        }
    })
}

/// Original ledger admission failed. No error grants fresh attempt authority.
#[derive(Debug, Error)]
pub enum LedgerError {
    /// No created frame was retained.
    #[error("capture ledger is empty")]
    Empty,
    /// An incomplete header, payload or chain digest remains retained.
    #[error("capture ledger has a partial frame")]
    Truncated,
    /// Declared payload is empty or exceeds 16 KiB.
    #[error("capture ledger frame size exceeds its admitted bounds")]
    FrameSize,
    /// More than 65,536 frames would be read.
    #[error("capture ledger frame count exceeds its admitted bound")]
    FrameCount,
    /// Ledger exceeds 128 MiB or the original plan's smaller evidence allowance.
    #[error("capture ledger exceeds its original byte allowance")]
    ByteLimit,
    /// The chained digest does not bind the exact original plan and prefix.
    #[error("capture ledger chain mismatch")]
    Chain,
    /// Unsupported event, sequence or noncanonical initial record.
    #[error("unsupported capture ledger transition")]
    Transition,
    /// Original reader failure; no tail truncation or retry is performed.
    #[error(transparent)]
    Io(#[from] io::Error),
}

#[cfg(test)]
mod tests;

impl SnapshotState {
    fn apply(
        &mut self,
        event: &EventRecord,
        plan: &CapturePlan,
        status: &StatusAdmissionRecord,
    ) -> Result<(), LedgerError> {
        use CaptureSnapshotPhase as Phase;
        match (self.phase, event) {
            (
                Phase::Created,
                EventRecord::SnapshotStarted {
                    sequence: 5,
                    admission,
                    arguments_sha256,
                },
            ) => {
                admission
                    .validate(plan, status)
                    .map_err(|_| LedgerError::Transition)?;
                if Sha256Digest::compute(
                    &CaptureCall::ListSnapshots
                        .arguments(plan)
                        .map_err(|_| LedgerError::Transition)?,
                )
                .as_bytes()
                    != arguments_sha256
                {
                    return Err(LedgerError::Transition);
                }
                self.admission = Some(admission.clone());
                self.phase = Phase::ListPreparationReserved;
            }
            (
                Phase::ListPreparationReserved,
                EventRecord::SnapshotPrepared {
                    sequence: 6,
                    call: CaptureCall::ListSnapshots,
                    request,
                },
            ) => {
                let admission = self.admission.as_ref().ok_or(LedgerError::Transition)?;
                if !(46..=admission.list.request_bytes).contains(&request.bytes) {
                    return Err(LedgerError::Transition);
                }
                self.list_request = Some(request.clone());
                self.phase = Phase::ListPrepared;
            }
            (
                Phase::ListPrepared,
                EventRecord::SnapshotSubmissionReserved {
                    sequence: 7,
                    call: CaptureCall::ListSnapshots,
                },
            ) => self.phase = Phase::ListSubmissionReserved,
            (
                Phase::ListSubmissionReserved,
                EventRecord::SnapshotSettled {
                    sequence: 8,
                    call: CaptureCall::ListSnapshots,
                    result,
                },
            ) => {
                let admission = self.admission.as_ref().ok_or(LedgerError::Transition)?;
                if !(7..=admission.list.result_bytes).contains(&result.bytes) {
                    return Err(LedgerError::Transition);
                }
                self.list_result = Some(result.clone());
                self.phase = Phase::ListSettled;
            }
            (
                Phase::ListSettled,
                EventRecord::SnapshotTakeReserved {
                    sequence: 9,
                    arguments_sha256,
                },
            ) => {
                if Sha256Digest::compute(
                    &CaptureCall::TakeSnapshot
                        .arguments(plan)
                        .map_err(|_| LedgerError::Transition)?,
                )
                .as_bytes()
                    != arguments_sha256
                {
                    return Err(LedgerError::Transition);
                }
                self.phase = Phase::TakePreparationReserved;
            }
            (
                Phase::TakePreparationReserved,
                EventRecord::SnapshotPrepared {
                    sequence: 10,
                    call: CaptureCall::TakeSnapshot,
                    request,
                },
            ) => {
                let admission = self.admission.as_ref().ok_or(LedgerError::Transition)?;
                if !(46..=admission.take.request_bytes).contains(&request.bytes) {
                    return Err(LedgerError::Transition);
                }
                self.take_request = Some(request.clone());
                self.phase = Phase::TakePrepared;
            }
            (
                Phase::TakePrepared,
                EventRecord::SnapshotSubmissionReserved {
                    sequence: 11,
                    call: CaptureCall::TakeSnapshot,
                },
            ) => self.phase = Phase::TakeSubmissionReserved,
            (
                Phase::TakeSubmissionReserved,
                EventRecord::SnapshotSettled {
                    sequence: 12,
                    call: CaptureCall::TakeSnapshot,
                    result,
                },
            ) => {
                let admission = self.admission.as_ref().ok_or(LedgerError::Transition)?;
                if !(7..=admission.take.result_bytes).contains(&result.bytes) {
                    return Err(LedgerError::Transition);
                }
                self.take_result = Some(result.clone());
                self.phase = Phase::Settled;
            }
            _ => return Err(LedgerError::Transition),
        }
        Ok(())
    }
}
