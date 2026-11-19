//! Shared bounded framing regressions retained from the original capture ledger.

use super::*;
use std::io::Cursor;

fn bounds() -> Bounds {
    Bounds {
        frame_bytes: FRAME_BYTES,
        frames: FRAMES,
        bytes: LEDGER_BYTES,
    }
}

fn frame(head: BackupDigest, payload: &[u8]) -> (Vec<u8>, BackupDigest) {
    let bytes = encode_frame(head, payload).unwrap();
    let digest = chain(head, &(payload.len() as u32).to_le_bytes(), payload);
    (bytes, digest)
}
struct HeaderOnly {
    bytes: Cursor<[u8; 4]>,
}
impl Read for HeaderOnly {
    fn read(&mut self, into: &mut [u8]) -> io::Result<usize> {
        assert!(
            self.bytes.position() < 4,
            "attempted body read before admission"
        );
        self.bytes.read(into)
    }
}

#[test]
fn oversized_header_refuses_before_any_body_read_or_allocation() {
    for size in [0, FRAME_BYTES as u32 + 1, u32::MAX] {
        let reader = HeaderOnly {
            bytes: Cursor::new(size.to_le_bytes()),
        };
        let mut frames = Frames::new(reader, seed(b"approved intent"), bounds());
        assert!(matches!(frames.next(), Err(LedgerError::FrameSize)));
    }
    let reader = HeaderOnly {
        bytes: Cursor::new(30_u32.to_le_bytes()),
    };
    let mut bounds = bounds();
    bounds.bytes = 65;
    let mut frames = Frames::new(reader, seed(b"approved intent"), bounds);
    assert!(matches!(frames.next(), Err(LedgerError::ByteLimit)));
}

#[test]
fn frame_and_cumulative_bounds_are_independent_and_exact() {
    let (bytes, _) = frame(seed(b"approved intent"), &vec![0x42; FRAME_BYTES]);
    let mut bounds = bounds();
    bounds.bytes = bytes.len() as u64;
    let mut frames = Frames::new(bytes.as_slice(), seed(b"approved intent"), bounds);
    assert_eq!(frames.next().unwrap().unwrap().len(), FRAME_BYTES);
    assert!(frames.next().unwrap().is_none());
    bounds.bytes -= 1;
    let mut frames = Frames::new(bytes.as_slice(), seed(b"approved intent"), bounds);
    assert!(matches!(frames.next(), Err(LedgerError::ByteLimit)));

    let (first, head) = frame(seed(b"approved intent"), b"a");
    let (second, _) = frame(head, b"b");
    let combined = [first.as_slice(), second.as_slice()].concat();
    bounds.bytes = combined.len() as u64 - 1;
    let mut frames = Frames::new(combined.as_slice(), seed(b"approved intent"), bounds);
    frames.next().unwrap();
    assert!(matches!(frames.next(), Err(LedgerError::ByteLimit)));
    bounds.bytes = first.len() as u64;
    let mut frames = Frames::new(combined.as_slice(), seed(b"approved intent"), bounds);
    frames.next().unwrap();
    assert!(matches!(frames.next(), Err(LedgerError::ByteLimit)));
}

#[test]
fn production_frame_count_limit_accepts_exactly_its_bound() {
    let mut bytes = Vec::new();
    let mut head = seed(b"approved intent");
    for _ in 0..FRAMES {
        let (next, next_head) = frame(head, b"x");
        bytes.extend(next);
        head = next_head;
    }
    let bounds = Bounds {
        frame_bytes: FRAME_BYTES,
        frames: FRAMES,
        bytes: LEDGER_BYTES,
    };
    let mut frames = Frames::new(bytes.as_slice(), seed(b"approved intent"), bounds);
    for _ in 0..FRAMES {
        assert_eq!(frames.next().unwrap().unwrap(), b"x");
    }
    assert!(frames.next().unwrap().is_none());
    assert_eq!(frames.count, FRAMES);
    // Only the EOF probe is needed to reject an additional frame, even a partial one.
    bytes.push(1);
    let mut frames = Frames::new(bytes.as_slice(), seed(b"approved intent"), bounds);
    for _ in 0..FRAMES {
        frames.next().unwrap();
    }
    assert!(matches!(frames.next(), Err(LedgerError::FrameCount)));
}

struct ShortReads {
    reader: Cursor<Vec<u8>>,
    interrupt: bool,
}
impl Read for ShortReads {
    fn read(&mut self, into: &mut [u8]) -> io::Result<usize> {
        self.interrupt = !self.interrupt;
        if self.interrupt {
            return Err(io::ErrorKind::Interrupted.into());
        }
        let len = into.len().min(2);
        self.reader.read(&mut into[..len])
    }
}

#[test]
fn interrupted_short_reads_and_original_io_errors_are_handled() {
    let bytes = initial_bytes(b"approved intent", LEDGER_BYTES).unwrap();
    let mut frames = Frames::new(
        ShortReads {
            reader: Cursor::new(bytes),
            interrupt: false,
        },
        seed(b"approved intent"),
        bounds(),
    );
    assert_eq!(frames.next().unwrap().unwrap(), CREATED);
    assert!(frames.next().unwrap().is_none());
    struct Refusal;
    impl Read for Refusal {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::ErrorKind::PermissionDenied.into())
        }
    }
    assert!(
        matches!(Frames::new(Refusal, seed(b"approved intent"), bounds()).next(), Err(LedgerError::Io(e)) if e.kind() == io::ErrorKind::PermissionDenied)
    );
}
