//! Product error projection for exact local streams; these readers perform no IC effects.

use super::*;
use std::io;
use std::io::Cursor;

#[test]
fn exact_stream_identity_and_shape_errors_keep_product_contract() {
    for bytes in [Vec::new(), b"abc".to_vec(), vec![0x9a; 200_001]] {
        let length = u64::try_from(bytes.len()).unwrap();
        let mut reader = Cursor::new(bytes.clone());
        let checksum = checksum_exact_reader(&mut reader, length).unwrap();
        assert_eq!(checksum, ArtifactChecksumRecord::from_bytes(&bytes));
        if length > 0 {
            let mut smaller = Cursor::new(&bytes[..bytes.len() - 1]);
            assert!(matches!(
                checksum_exact_reader(&mut smaller, length),
                Err(IcSnapshotArtifactError::FileShape)
            ));
        }
        let mut growing = bytes;
        growing.extend_from_slice(&[1; 32]);
        let mut reader = Cursor::new(growing);
        assert!(matches!(
            checksum_exact_reader(&mut reader, length),
            Err(IcSnapshotArtifactError::FileShape)
        ));
    }
}

#[test]
fn exact_stream_preserves_original_io_error() {
    struct Failing(Cursor<Vec<u8>>);
    impl Read for Failing {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            if self.0.position() == u64::try_from(self.0.get_ref().len()).unwrap() {
                return Err(io::Error::from_raw_os_error(13));
            }
            self.0.read(output)
        }
    }
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
}
