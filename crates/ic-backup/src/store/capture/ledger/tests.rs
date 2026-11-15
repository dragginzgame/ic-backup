use super::*;
use crate::store::{
    BackupStore,
    tests::{root, write_file},
};
use std::{fs, io::Cursor, os::unix::fs::symlink};

fn plan() -> CapturePlan {
    CapturePlan::decode(include_bytes!("../../tests/fixtures/capture-plan.json").to_vec()).unwrap()
}

// Fixtures construct exact bytes independently of the production writer.
fn frame(previous: Sha256Digest, payload: &[u8]) -> (Vec<u8>, Sha256Digest) {
    let length = u32::try_from(payload.len()).unwrap().to_le_bytes();
    let mut input = previous.as_bytes().to_vec();
    input.extend_from_slice(&length);
    input.extend_from_slice(payload);
    let digest = Sha256Digest::compute(&input);
    let mut bytes = length.to_vec();
    bytes.extend_from_slice(payload);
    bytes.extend_from_slice(digest.as_bytes());
    (bytes, digest)
}

#[test]
fn golden_frame_is_unchanged_and_every_incomplete_prefix_refuses() {
    let p = plan();
    let golden = include_str!("../../tests/fixtures/capture-created.hex").trim();
    let bytes: Vec<_> = golden
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect();
    assert_eq!(initial(&p).unwrap(), bytes);
    replay(bytes.as_slice(), &p).unwrap();
    for end in 0..bytes.len() {
        let error = replay(&bytes[..end], &p).unwrap_err();
        if end == 0 {
            assert!(matches!(error, LedgerError::Empty));
        } else {
            assert!(
                matches!(error, LedgerError::Truncated),
                "prefix {end}: {error}"
            );
        }
    }
    for tail in 1..=3 {
        let mut partial = bytes.clone();
        partial.extend(vec![1; tail]);
        assert!(matches!(
            replay(partial.as_slice(), &p),
            Err(LedgerError::Truncated)
        ));
    }
}

#[test]
fn chain_binds_plan_header_payload_and_previous_frame() {
    let p = plan();
    let bytes = initial(&p).unwrap();
    for offset in [4, 10, bytes.len() - 1] {
        let mut altered = bytes.clone();
        altered[offset] ^= 1;
        assert!(matches!(
            replay(altered.as_slice(), &p),
            Err(LedgerError::Chain)
        ));
    }
    let mut changed = p.request().clone();
    changed.limits.network_requests += 1;
    let changed = CapturePlan::new(changed).unwrap();
    assert!(matches!(
        replay(bytes.as_slice(), &changed),
        Err(LedgerError::Chain)
    ));

    let (first, head) = frame(seed(p.encoded()), b"one");
    let (second, _) = frame(head, b"two");
    let joined = [first.as_slice(), second.as_slice()].concat();
    let mut frames = Frames::new(joined.as_slice(), seed(p.encoded()), Bounds::for_plan(&p));
    assert_eq!(frames.next().unwrap().unwrap(), b"one");
    assert_eq!(frames.next().unwrap().unwrap(), b"two");
    assert!(frames.next().unwrap().is_none());
    let reordered = [second.as_slice(), first.as_slice()].concat();
    let mut frames = Frames::new(
        reordered.as_slice(),
        seed(p.encoded()),
        Bounds::for_plan(&p),
    );
    assert!(matches!(frames.next(), Err(LedgerError::Chain)));
    assert_eq!(
        (frames.count, frames.bytes, frames.head),
        (0, 0, seed(p.encoded()))
    );
}

#[test]
fn valid_hash_does_not_admit_unsupported_or_ambiguous_transitions() {
    let p = plan();
    for payload in [
        b"{\"sequence\":1,\"kind\":\"created\"}".as_slice(),
        b"{\"sequence\":0,\"kind\":\"reserved\"}",
        b"{\"sequence\":0,\"kind\":\"created\",\"unknown\":1}",
        b"{\"sequence\":0,\"sequence\":0,\"kind\":\"created\"}",
        b"{ \"sequence\":0,\"kind\":\"created\"}",
        b"not json",
    ] {
        let (bytes, _) = frame(seed(p.encoded()), payload);
        assert!(matches!(
            replay(bytes.as_slice(), &p),
            Err(LedgerError::Transition)
        ));
    }
    let (first, head) = frame(seed(p.encoded()), CREATED);
    let (second, _) = frame(head, CREATED);
    assert!(matches!(
        replay([first, second].concat().as_slice(), &p),
        Err(LedgerError::Transition)
    ));
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
    let p = plan();
    for size in [0, FRAME_BYTES as u32 + 1, u32::MAX] {
        let reader = HeaderOnly {
            bytes: Cursor::new(size.to_le_bytes()),
        };
        let mut frames = Frames::new(reader, seed(p.encoded()), Bounds::for_plan(&p));
        assert!(matches!(frames.next(), Err(LedgerError::FrameSize)));
    }
    let reader = HeaderOnly {
        bytes: Cursor::new(30_u32.to_le_bytes()),
    };
    let mut bounds = Bounds::for_plan(&p);
    bounds.bytes = 65;
    let mut frames = Frames::new(reader, seed(p.encoded()), bounds);
    assert!(matches!(frames.next(), Err(LedgerError::ByteLimit)));
}

#[test]
fn frame_and_cumulative_bounds_are_independent_and_exact() {
    let p = plan();
    let (bytes, _) = frame(seed(p.encoded()), &vec![0x42; FRAME_BYTES]);
    let mut bounds = Bounds::for_plan(&p);
    bounds.bytes = bytes.len() as u64;
    let mut frames = Frames::new(bytes.as_slice(), seed(p.encoded()), bounds);
    assert_eq!(frames.next().unwrap().unwrap().len(), FRAME_BYTES);
    assert!(frames.next().unwrap().is_none());
    bounds.bytes -= 1;
    let mut frames = Frames::new(bytes.as_slice(), seed(p.encoded()), bounds);
    assert!(matches!(frames.next(), Err(LedgerError::ByteLimit)));

    let (first, head) = frame(seed(p.encoded()), b"a");
    let (second, _) = frame(head, b"b");
    let combined = [first.as_slice(), second.as_slice()].concat();
    bounds.bytes = combined.len() as u64 - 1;
    let mut frames = Frames::new(combined.as_slice(), seed(p.encoded()), bounds);
    frames.next().unwrap();
    assert!(matches!(frames.next(), Err(LedgerError::ByteLimit)));
    bounds.bytes = first.len() as u64;
    let mut frames = Frames::new(combined.as_slice(), seed(p.encoded()), bounds);
    frames.next().unwrap();
    assert!(matches!(frames.next(), Err(LedgerError::ByteLimit)));
}

#[test]
fn production_frame_count_limit_accepts_exactly_its_bound() {
    let p = plan();
    let mut bytes = Vec::new();
    let mut head = seed(p.encoded());
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
    let mut frames = Frames::new(bytes.as_slice(), seed(p.encoded()), bounds);
    for _ in 0..FRAMES {
        assert_eq!(frames.next().unwrap().unwrap(), b"x");
    }
    assert!(frames.next().unwrap().is_none());
    assert_eq!(frames.count, FRAMES);
    // Only the EOF probe is needed to reject an additional frame, even a partial one.
    bytes.push(1);
    let mut frames = Frames::new(bytes.as_slice(), seed(p.encoded()), bounds);
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
    let p = plan();
    replay(
        ShortReads {
            reader: Cursor::new(initial(&p).unwrap()),
            interrupt: false,
        },
        &p,
    )
    .unwrap();
    struct Refusal;
    impl Read for Refusal {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::ErrorKind::PermissionDenied.into())
        }
    }
    assert!(
        matches!(replay(Refusal, &p), Err(LedgerError::Io(e)) if e.kind() == io::ErrorKind::PermissionDenied)
    );
}

#[test]
fn closing_admission_refuses_changed_file_and_unsafe_name() {
    for mutation in 0..4 {
        let path = root("ledger-closing");
        let store = BackupStore::initialize(&path).unwrap();
        let p = plan();
        let digest = p.digest();
        let operation = store.create_capture(p, digest).unwrap();
        let named = path
            .join("operations")
            .join(operation.plan.request().operation_id.clone())
            .join("ledger.bin");
        let retained = path.with_extension("original-ledger");
        let result = admit_with(
            &operation.directory,
            &operation.ledger_file,
            &operation.plan,
            |file, plan| {
                replay(file, plan)?;
                let original = fs::read(&named).unwrap();
                match mutation {
                    0 => {
                        fs::rename(&named, &retained).unwrap();
                        write_file(&named, &original);
                    }
                    1 => {
                        let file = fs::OpenOptions::new().write(true).open(&named).unwrap();
                        file.set_len(1).unwrap();
                    }
                    2 => {
                        fs::rename(&named, &retained).unwrap();
                        symlink(&retained, &named).unwrap();
                    }
                    _ => {
                        let file = fs::OpenOptions::new().write(true).open(&named).unwrap();
                        file.set_len(original.len() as u64 + 1).unwrap();
                    }
                }
                Ok(())
            },
        );
        assert!(matches!(
            result,
            Err(CaptureError::Changed | CaptureError::Store(_))
        ));
    }
}

#[test]
fn sparse_oversized_ledger_refuses_under_original_plan_budget() {
    let path = root("ledger-oversized");
    let store = BackupStore::initialize(&path).unwrap();
    let mut request = plan().request().clone();
    request.limits.evidence_bytes = 1 << 30;
    let p = CapturePlan::new(request).unwrap();
    let digest = p.digest();
    let operation = store.create_capture(p, digest).unwrap();
    let named = path
        .join("operations")
        .join(operation.plan.request().operation_id.clone())
        .join("ledger.bin");
    fs::OpenOptions::new()
        .write(true)
        .open(&named)
        .unwrap()
        .set_len(LEDGER_BYTES + 1)
        .unwrap();
    assert!(matches!(
        operation.inspect(),
        Err(CaptureError::Ledger(LedgerError::ByteLimit))
    ));
    assert_eq!(fs::metadata(named).unwrap().len(), LEDGER_BYTES + 1);
}

#[test]
fn initial_status_events_refuse_noncanonical_wrong_arguments_and_out_of_order_transitions() {
    let p = plan();
    let first = initial(&p).unwrap();
    let head = Sha256Digest::from_bytes(first[first.len() - 32..].try_into().unwrap());
    let admission = super::super::advance::StatusAdmissionRecord {
        provider: [1; 32],
        application: [2; 32],
        prepare_deadline_ns: 10,
        submit_deadline_ns: 20,
        prepare_requests: 1,
        submit_requests: 1,
        request_bytes: 4096,
        result_bytes: 4096,
    };
    let event = EventRecord::PreparationReserved {
        sequence: 1,
        admission: admission.clone(),
        arguments_sha256: *Sha256Digest::compute(&arguments(&p).unwrap()).as_bytes(),
    };
    let payload = serde_json::to_vec(&event).unwrap();
    let (second, _) = frame(head, &payload);
    let accepted = [first.as_slice(), &second].concat();
    assert_eq!(
        replay(accepted.as_slice(), &p).unwrap().phase,
        CaptureStatusPhase::PreparationReserved
    );
    for payload in [
        serde_json::to_vec_pretty(&event).unwrap(),
        serde_json::to_vec(&EventRecord::PreparationReserved {
            sequence: 1,
            admission,
            arguments_sha256: [0; 32],
        })
        .unwrap(),
        b"{\"kind\":\"submission_reserved\",\"sequence\":3}".to_vec(),
        b"{\"kind\":\"submission_reserved\",\"sequence\":3,\"extra\":0}".to_vec(),
        b"{\"kind\":\"submission_reserved\",\"sequence\":3,\"sequence\":3}".to_vec(),
    ] {
        let (second, _) = frame(head, &payload);
        let bytes = [first.as_slice(), &second].concat();
        assert!(matches!(
            replay(bytes.as_slice(), &p),
            Err(LedgerError::Transition)
        ));
    }
}
