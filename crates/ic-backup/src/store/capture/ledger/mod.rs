//! Bounded chained frames and held original-ledger admission. No append authority.

use super::{CaptureError, CapturePlan, open_optional};
use ic_host_artifacts::artifact::Sha256Digest;
use ic_host_fs::{capability::Directory, identity::FileIdentity};
use std::{
    fs::File,
    io::{self, Read},
};
use thiserror::Error;

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
) -> Result<(), CaptureError> {
    admit_with(directory, held, plan, |file, plan| replay(file, plan))
}

// The injected reader is also used for deterministic native closing-custody tests.
fn admit_with(
    directory: &Directory,
    held: &File,
    plan: &CapturePlan,
    read: impl FnOnce(&mut File, &CapturePlan) -> Result<(), LedgerError>,
) -> Result<(), CaptureError> {
    let mut file = open_optional(directory, "ledger.bin")?.ok_or(CaptureError::Incomplete)?;
    let original = FileIdentity::from_file(held)?;
    let before = file.metadata()?;
    if FileIdentity::from_metadata(&before) != original {
        return Err(CaptureError::Changed);
    }
    if before.len() > Bounds::for_plan(plan).bytes {
        return Err(LedgerError::ByteLimit.into());
    }
    read(&mut file, plan)?;
    let after = file.metadata()?;
    let named = open_optional(directory, "ledger.bin")?.ok_or(CaptureError::Incomplete)?;
    if FileIdentity::from_metadata(&after) != original
        || after.len() != before.len()
        || FileIdentity::from_file(&named)? != original
        || named.metadata()?.len() != before.len()
    {
        return Err(CaptureError::Changed);
    }
    Ok(())
}

fn replay(reader: impl Read, plan: &CapturePlan) -> Result<(), LedgerError> {
    let mut frames = Frames::new(reader, seed(plan), Bounds::for_plan(plan));
    while let Some(payload) = frames.next()? {
        // The only implemented transition is the exact original created record.
        // A valid hash on an unknown/later record is not permission to ignore it.
        if frames.count != 1 || payload != CREATED {
            return Err(LedgerError::Transition);
        }
    }
    if frames.count == 0 {
        return Err(LedgerError::Empty);
    }
    Ok(())
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
