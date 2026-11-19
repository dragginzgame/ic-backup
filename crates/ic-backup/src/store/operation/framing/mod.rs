//! Bounded hash-chained framing shared by the capture recorder and retained restore status contract.

#[cfg(test)]
pub(super) mod tests;

use crate::store::BackupDigest;
use std::io::{self, Read};
use thiserror::Error;

pub(in crate::store) const FRAME_BYTES: usize = 16 * 1024;
pub(in crate::store) const FRAMES: u32 = 65_536;
pub(in crate::store) const LEDGER_BYTES: u64 = 128 * 1024 * 1024;
#[derive(Clone, Copy)]
pub(in crate::store) struct Bounds {
    pub(in crate::store) frame_bytes: usize,
    pub(in crate::store) frames: u32,
    pub(in crate::store) bytes: u64,
}

fn chain(previous: BackupDigest, length: &[u8; 4], payload: &[u8]) -> BackupDigest {
    BackupDigest::compute(&[previous.as_bytes().as_slice(), length, payload].concat())
}

pub(in crate::store) fn encode_frame(
    head: BackupDigest,
    payload: &[u8],
) -> Result<Vec<u8>, LedgerError> {
    if payload.is_empty() || payload.len() > FRAME_BYTES {
        return Err(LedgerError::FrameSize);
    }
    let length = (payload.len() as u32).to_le_bytes();
    Ok([
        length.as_slice(),
        payload,
        chain(head, &length, payload).as_bytes(),
    ]
    .concat())
}

pub(in crate::store) struct Frames<R> {
    reader: R,
    pub(in crate::store) head: BackupDigest,
    count: u32,
    pub(in crate::store) bytes: u64,
    bounds: Bounds,
}
impl<R: Read> Frames<R> {
    pub(in crate::store) fn new(reader: R, head: BackupDigest, bounds: Bounds) -> Self {
        Self {
            reader,
            head,
            count: 0,
            bytes: 0,
            bounds,
        }
    }

    pub(in crate::store) fn next(&mut self) -> Result<Option<Vec<u8>>, LedgerError> {
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
