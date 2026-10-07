//! Exact local stream length/error admission; these readers perform no IC effects.

use super::*;
use std::io::Cursor;

#[test]
fn exact_stream_admission_preserves_empty_and_multibuffer_identity_and_rejects_size_changes() {
    for bytes in [Vec::new(), b"abc".to_vec(), vec![0x9a; 200_001]] {
        let length = u64::try_from(bytes.len()).unwrap();
        let mut reader = Cursor::new(bytes.clone());
        let checksum = checksum_exact_reader(&mut reader, length).unwrap();
        assert_eq!(checksum, ArtifactChecksumRecord::from_bytes(&bytes));
        assert_eq!(reader.position(), length);
        if length > 0 {
            let mut smaller = Cursor::new(&bytes[..bytes.len() - 1]);
            assert!(matches!(
                checksum_exact_reader(&mut smaller, length),
                Err(IcSnapshotArtifactError::FileShape)
            ));
            assert_eq!(smaller.position(), length - 1);
        }
        let mut growing = bytes;
        growing.extend_from_slice(&[1; 32]);
        let mut reader = Cursor::new(growing);
        assert!(matches!(
            checksum_exact_reader(&mut reader, length),
            Err(IcSnapshotArtifactError::FileShape)
        ));
        assert_eq!(reader.position(), length + 1);
    }
}

#[test]
fn exact_stream_admission_retries_interruption_and_preserves_io_and_invalid_count_errors() {
    struct Interrupted(Cursor<Vec<u8>>, bool);
    impl Read for Interrupted {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            if !std::mem::replace(&mut self.1, true) {
                return Err(io::ErrorKind::Interrupted.into());
            }
            self.0.read(output)
        }
    }
    struct Failing(Cursor<Vec<u8>>);
    impl Read for Failing {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            if self.0.position() == u64::try_from(self.0.get_ref().len()).unwrap() {
                return Err(io::Error::from_raw_os_error(13));
            }
            self.0.read(output)
        }
    }
    struct Invalid;
    impl Read for Invalid {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            Ok(output.len() + 1)
        }
    }
    let mut interrupted = Interrupted(Cursor::new(b"abc".to_vec()), false);
    assert_eq!(
        checksum_exact_reader(&mut interrupted, 3).unwrap(),
        ArtifactChecksumRecord::from_bytes(b"abc")
    );
    assert_eq!(interrupted.0.position(), 3);
    for bytes in [Vec::new(), b"retained prefix".to_vec()] {
        let mut failing = Failing(Cursor::new(bytes));
        assert!(
            matches!(checksum_exact_reader(&mut failing, 20), Err(IcSnapshotArtifactError::Artifact(ArtifactError::Io(error))) if error.raw_os_error() == Some(13))
        );
        assert_eq!(
            failing.0.position(),
            u64::try_from(failing.0.get_ref().len()).unwrap()
        );
    }
    assert!(
        matches!(checksum_exact_reader(Invalid, 3), Err(IcSnapshotArtifactError::Artifact(ArtifactError::Io(error))) if error.kind() == io::ErrorKind::InvalidData)
    );
}
