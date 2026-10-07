use super::*;
use crate::test_support::temp_path;
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
fn staging_copy_retries_interruption_and_preserves_multichunk_identity() {
    use std::io::{Cursor, Read, Write};

    struct InterruptedSource(Cursor<Vec<u8>>, bool);
    impl Read for InterruptedSource {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            if !std::mem::replace(&mut self.1, true) {
                return Err(io::ErrorKind::Interrupted.into());
            }
            self.0.read(output)
        }
    }
    struct ShortSink(Vec<u8>);
    impl Write for ShortSink {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            let accepted = bytes.len().min(7);
            self.0.extend_from_slice(&bytes[..accepted]);
            Ok(accepted)
        }
        fn flush(&mut self) -> io::Result<()> {
            panic!("copying does not own flushing")
        }
    }

    for bytes in [Vec::new(), b"abc".to_vec(), vec![0x9a; 200_001]] {
        let mut source = InterruptedSource(Cursor::new(bytes.clone()), false);
        let mut sink = ShortSink(Vec::new());
        let checksum = copy_from_reader(&mut source, &mut sink).expect("copy exact bytes");
        assert_eq!(sink.0, bytes);
        assert_eq!(checksum, ArtifactChecksumRecord::from_bytes(&bytes));
        if bytes == b"abc" {
            assert_eq!(
                checksum.hash(),
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
            );
        }
    }
}

#[cfg(unix)]
#[test]
fn staging_copy_preserves_partial_output_and_original_source_or_sink_error() {
    use std::io::{Cursor, Read, Write};

    struct FailingSource(Cursor<Vec<u8>>);
    impl Read for FailingSource {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            if self.0.position() == self.0.get_ref().len() as u64 {
                return Err(io::Error::from_raw_os_error(13));
            }
            self.0.read(output)
        }
    }
    struct FailingSink(Vec<u8>);
    impl Write for FailingSink {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self.0.len() == 3 {
                return Err(io::Error::from_raw_os_error(28));
            }
            let accepted = bytes.len().min(3 - self.0.len());
            self.0.extend_from_slice(&bytes[..accepted]);
            Ok(accepted)
        }
        fn flush(&mut self) -> io::Result<()> {
            panic!("copying does not own flushing")
        }
    }

    let mut retained_prefix = Vec::new();
    assert!(matches!(
        copy_from_reader(
            &mut FailingSource(Cursor::new(b"retained prefix".to_vec())),
            &mut retained_prefix,
        ),
        Err(ArtifactError::Io(error)) if error.raw_os_error() == Some(13)
    ));
    assert_eq!(retained_prefix, b"retained prefix");

    let mut sink = FailingSink(Vec::new());
    assert!(matches!(
        copy_from_reader(&mut Cursor::new(b"partial write"), &mut sink),
        Err(ArtifactError::Io(error)) if error.raw_os_error() == Some(28)
    ));
    assert_eq!(sink.0, b"par");
}

#[cfg(unix)]
#[test]
fn staging_copy_rejects_impossible_stream_counts_without_panicking_or_discarding_prefix() {
    use ic_host_artifacts::artifact::WriterError;
    use std::io::{Read, Write};

    struct InvalidSource;
    impl Read for InvalidSource {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            Ok(output.len() + 1)
        }
    }
    struct InvalidSink(Vec<u8>);
    impl Write for InvalidSink {
        fn write(&mut self, input: &[u8]) -> io::Result<usize> {
            if self.0.is_empty() {
                self.0.extend_from_slice(&input[..2]);
                Ok(2)
            } else {
                Ok(input.len() + 1)
            }
        }
        fn flush(&mut self) -> io::Result<()> {
            panic!("copying does not own flushing")
        }
    }
    let mut destination = b"retained".to_vec();
    assert!(
        matches!(copy_from_reader(&mut InvalidSource, &mut destination),
        Err(ArtifactError::Io(error)) if error.kind() == io::ErrorKind::InvalidData)
    );
    assert_eq!(destination, b"retained");
    let mut sink = InvalidSink(Vec::new());
    let Err(ArtifactError::Io(error)) = copy_from_reader(&mut b"abcdef".as_slice(), &mut sink)
    else {
        panic!("invalid sink count must preserve typed IO failure")
    };
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert_eq!(
        error.get_ref().unwrap().downcast_ref::<WriterError>(),
        Some(&WriterError::InvalidWriteCount {
            offered: 4,
            written: 5
        })
    );
    assert_eq!(sink.0, b"ab");
}

#[cfg(unix)]
#[test]
fn staging_preserves_exact_bytes_checksums_and_private_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let root = temp_path("ic-backup-stage");
    fs::create_dir_all(root.join("source/nested")).expect("create source");
    fs::write(root.join("source/nested/state.bin"), b"state bytes").expect("write source");
    let destination = root.join("staged");
    let expected = checksum_directory(&root.join("source")).expect("source checksum");
    let actual = stage_relative_path(&root, Path::new("source"), &destination).expect("stage tree");
    assert_eq!(actual, expected);
    assert_eq!(
        fs::read(destination.join("nested/state.bin")).expect("staged bytes"),
        b"state bytes"
    );
    assert_eq!(
        fs::metadata(&destination)
            .expect("directory permissions")
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(destination.join("nested/state.bin"))
            .expect("file permissions")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert!(
        matches!(stage_relative_path(&root, Path::new("source"), &destination), Err(ArtifactError::Io(ref error)) if error.kind() == io::ErrorKind::AlreadyExists)
    );
    let escaped = root.join("escaped");
    assert!(matches!(
        stage_relative_path(&root, Path::new("../source"), &escaped),
        Err(ArtifactError::UnsupportedEntry { .. })
    ));
    assert!(!escaped.exists());
    assert_eq!(
        checksum_relative_path(&root, Path::new("source")).expect("relative checksum"),
        expected
    );
    fs::remove_dir_all(root).expect("remove successful fixture");
}

#[cfg(unix)]
#[test]
fn non_utf8_tree_names_reject_instead_of_collapsing_path_identity() {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};
    let root = temp_path("ic-backup-non-utf8");
    let name = OsStr::from_bytes(b"\xff");
    assert!(matches!(
        require_utf8_tree_name(name, &root),
        Err(ArtifactError::NonUtf8Path { path }) if path == root.join(name)
    ));
    require_utf8_tree_name(OsStr::new("exact-é-name"), &root).expect("valid UTF-8 name");
    fs::create_dir(&root).expect("create tree");
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
