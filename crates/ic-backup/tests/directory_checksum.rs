//! Product publication must independently verify declared Host tree identities.

use ic_backup::ops::artifacts::ArtifactError;
use ic_host_artifacts::artifact::{
    Sha256Digest,
    tree::{DirectoryChecksumError, checksum_relative_files},
};
use std::path::PathBuf;

fn entry(name: &str, bytes: &[u8]) -> (PathBuf, Sha256Digest) {
    (name.into(), Sha256Digest::compute(bytes))
}

#[test]
fn host_checksum_errors_convert_through_public_backup_boundary() {
    let cases = [
        (
            vec![entry("../outside", b"bytes")],
            DirectoryChecksumError::InvalidRelativePath {
                path: "../outside".into(),
            },
        ),
        (
            vec![entry("duplicate", b"a"), entry("duplicate", b"b")],
            DirectoryChecksumError::DuplicatePath {
                path: "duplicate".into(),
            },
        ),
    ];
    for (entries, expected) in cases {
        // Use the consumer's Host producer and Backup's public From implementation.
        let refused = checksum_relative_files(entries).unwrap_err();
        assert_eq!(refused, expected);
        let ArtifactError::Io(error) = ArtifactError::from(refused) else {
            panic!("declaration refusal must retain the IO boundary");
        };
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(
            error
                .get_ref()
                .unwrap()
                .downcast_ref::<DirectoryChecksumError>(),
            Some(&expected)
        );
    }
}

#[cfg(unix)]
#[test]
fn host_non_utf8_checksum_error_preserves_exact_path_in_backup() {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

    // No native filesystem filename admission is needed to exercise this boundary.
    let path = PathBuf::from(OsStr::from_bytes(b"raw-\xff"));
    let refused =
        checksum_relative_files(vec![(path.clone(), Sha256Digest::compute(b"bytes"))]).unwrap_err();
    assert_eq!(
        refused,
        DirectoryChecksumError::NonUtf8Path { path: path.clone() }
    );
    assert!(matches!(
        ArtifactError::from(refused),
        ArtifactError::NonUtf8Path { path: actual } if actual == path
    ));
}
