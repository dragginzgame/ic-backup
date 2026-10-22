//! Product publication must independently verify declared Host tree identities.

use ic_backup::ops::artifacts::ArtifactError;
use ic_host_artifacts::artifact::{
    Sha256Digest,
    tree::{DirectoryChecksumError, checksum_relative_files},
};
use std::path::PathBuf;

#[cfg(unix)]
mod support;

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

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
#[test]
fn declared_checksum_requires_fresh_bytes_before_publication_and_recovery() {
    use ic_backup::{
        model::artifacts::ChecksumError,
        ops::persistence::{ArtifactCommitOutcome, PersistenceError, commit_artifact_directory},
    };
    use ic_host_fs::read::tree::checksum::checksum_directory;
    use std::fs;

    let root = support::temp_root("ic-backup-declared-publication");
    let temporary = root.join("snapshot.tmp");
    let canonical = root.join("snapshot");
    fs::create_dir_all(temporary.join("a")).unwrap();
    fs::write(temporary.join("a/leaf"), b"a").unwrap();
    fs::write(temporary.join("a-file"), b"b").unwrap();
    let expected =
        checksum_relative_files(vec![entry("a-file", b"b"), entry("a/leaf", b"a")]).unwrap();

    // Declared digests grant no custody: publication independently reads bytes.
    fs::write(temporary.join("a/leaf"), b"changed").unwrap();
    assert!(matches!(
        commit_artifact_directory(&temporary, &canonical, &expected.to_string()),
        Err(PersistenceError::Checksum(
            ChecksumError::ChecksumMismatch { .. }
        ))
    ));
    assert_eq!(fs::read(temporary.join("a/leaf")).unwrap(), b"changed");
    assert_eq!(fs::read(temporary.join("a-file")).unwrap(), b"b");
    assert!(!canonical.exists());

    // Restore only this owned fixture; no failed production evidence is repaired.
    fs::write(temporary.join("a/leaf"), b"a").unwrap();
    assert_eq!(
        commit_artifact_directory(&temporary, &canonical, &expected.to_string()).unwrap(),
        ArtifactCommitOutcome::Published
    );
    assert!(!temporary.exists());
    assert_eq!(checksum_directory(&canonical, u64::MAX).unwrap(), expected);
    assert_eq!(
        commit_artifact_directory(&temporary, &canonical, &expected.to_string()).unwrap(),
        ArtifactCommitOutcome::Recovered
    );

    // Recovery likewise cannot accept changed bytes under the declaration.
    fs::write(canonical.join("a-file"), b"drifted").unwrap();
    assert!(matches!(
        commit_artifact_directory(&temporary, &canonical, &expected.to_string()),
        Err(PersistenceError::Checksum(
            ChecksumError::ChecksumMismatch { .. }
        ))
    ));
    assert_eq!(fs::read(canonical.join("a-file")).unwrap(), b"drifted");
    assert!(!temporary.exists());
    fs::remove_dir_all(root).unwrap();
}
