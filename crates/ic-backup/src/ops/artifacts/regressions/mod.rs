use super::*;
use crate::test_support::temp_dir;
use std::fs;

#[test]
fn reader_hash_retries_interruption_and_preserves_exact_empty_and_multibuffer_bytes() {
    use std::io::{Cursor, Read};

    struct InterruptedSource {
        bytes: Cursor<Vec<u8>>,
        reads: usize,
    }
    impl Read for InterruptedSource {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            self.reads += 1;
            if self.reads == 1 || self.reads == 3 {
                return Err(io::ErrorKind::Interrupted.into());
            }
            self.bytes.read(output)
        }
    }
    for bytes in [Vec::new(), b"abc".to_vec(), vec![0x9a; 200_001]] {
        let mut source = InterruptedSource {
            bytes: Cursor::new(bytes.clone()),
            reads: 0,
        };
        let checksum = checksum_reader(&mut source).unwrap();
        assert_eq!(checksum, ArtifactChecksumRecord::from_bytes(&bytes));
        assert_eq!(source.bytes.position(), u64::try_from(bytes.len()).unwrap());
        if bytes == b"abc" {
            assert_eq!(
                checksum.hash(),
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
            );
            assert_eq!(source.reads, 4); // Interruption, bytes, interruption, EOF.
        }
    }
}

#[test]
fn reader_hash_preserves_io_errors_and_rejects_invalid_counts_without_a_checksum() {
    use std::io::{Cursor, Read};

    struct FailingSource(Cursor<Vec<u8>>);
    impl Read for FailingSource {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            if self.0.position() == self.0.get_ref().len() as u64 {
                return Err(io::Error::from_raw_os_error(13));
            }
            self.0.read(output)
        }
    }
    struct InvalidSource;
    impl Read for InvalidSource {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            Ok(output.len() + 1)
        }
    }
    for bytes in [Vec::new(), b"retained prefix".to_vec()] {
        let mut source = FailingSource(Cursor::new(bytes.clone()));
        assert!(
            matches!(checksum_reader(&mut source), Err(ArtifactError::Io(error)) if error.raw_os_error() == Some(13))
        );
        assert_eq!(source.0.position(), u64::try_from(bytes.len()).unwrap());
    }
    assert!(
        matches!(checksum_reader(&mut InvalidSource), Err(ArtifactError::Io(error)) if error.kind() == io::ErrorKind::InvalidData)
    );
}

#[cfg(unix)]
#[test]
fn non_utf8_tree_names_reject_instead_of_collapsing_path_identity() {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};
    let root = temp_dir("ic-backup-non-utf8");
    let name = OsStr::from_bytes(b"\xff");
    if let Err(error) = fs::write(root.join(name), b"bytes") {
        // APFS rejects this raw basename before the traversal owner can inspect it.
        // Other hosts and other IO failures must still fail the real fixture.
        assert!(
            cfg!(target_os = "macos")
                && error.raw_os_error() == Some(rustix::io::Errno::ILSEQ.raw_os_error()),
            "write non UTF-8 name: {error:?}"
        );
        eprintln!("filesystem rejected the raw name; canonical UTF-8 admission checked directly");
        fs::remove_dir(&root).expect("remove empty successful fixture");
        return;
    }
    assert!(matches!(
        checksum_directory(&root),
        Err(ArtifactError::NonUtf8Path { .. })
    ));
    fs::remove_dir_all(root).expect("remove successful fixture");
}
