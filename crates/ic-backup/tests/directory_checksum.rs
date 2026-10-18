//! Product publication must independently verify declared Host tree identities.

use ic_host_artifacts::artifact::{Sha256Digest, tree::checksum_relative_files};
use std::path::PathBuf;

#[cfg(unix)]
mod support;

fn entry(name: &str, bytes: &[u8]) -> (PathBuf, Sha256Digest) {
    (name.into(), Sha256Digest::compute(bytes))
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
