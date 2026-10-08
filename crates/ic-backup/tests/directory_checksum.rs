//! Public declared-file framing, independent of filesystem traversal/publication.

use ic_backup::{
    model::artifacts::ArtifactChecksumRecord,
    ops::artifacts::{ArtifactError, DirectoryChecksumError, checksum_relative_files},
};
use std::path::PathBuf;

#[cfg(unix)]
mod support;

fn entry(name: &str, bytes: &[u8]) -> (PathBuf, ArtifactChecksumRecord) {
    (name.into(), ArtifactChecksumRecord::from_bytes(bytes))
}

#[test]
fn declared_files_keep_original_golden_order_and_empty_framing() {
    let files = vec![entry("a.txt", b"a"), entry("nested/b.txt", b"b")];
    let original = checksum_relative_files(files.clone()).unwrap();
    assert_eq!(
        original.hash(),
        "e4d330f138b8f1b3044e84b5dcbe4fd1cb7e043d0c20810c083d791b6de01266"
    );
    let mut reversed = files;
    reversed.reverse();
    assert_eq!(checksum_relative_files(reversed).unwrap(), original);
    assert_eq!(
        checksum_relative_files(Vec::new()).unwrap(),
        ArtifactChecksumRecord::from_bytes(&[])
    );
    // Path component ordering differs from sorting the complete text bytes.
    assert_eq!(
        checksum_relative_files(vec![entry("a-file", b"b"), entry("a/leaf", b"a")])
            .unwrap()
            .hash(),
        "8ebe1da155356d34952f1515a80ee0c223ee530c1f87f6d10efa48f875b7c9b0"
    );
}

#[test]
fn exact_unicode_and_legal_filename_bytes_keep_canonical_digest_text() {
    let mut files = vec![
        entry("slash\\name", b"three"),
        entry("line\nname", b"two"),
        entry("café/雪", b"one"),
    ];
    files[0].1 =
        ArtifactChecksumRecord::from_hash(&files[0].1.hash().to_ascii_uppercase()).unwrap();
    assert_eq!(
        checksum_relative_files(files).unwrap().hash(),
        "95a10c85036a57ecbae066721f0b5b47b69617f8c5d1788e694c22bbbd1b0dc0"
    );
    let composed = checksum_relative_files(vec![entry("café", b"same")]).unwrap();
    let decomposed = checksum_relative_files(vec![entry("cafe\u{301}", b"same")]).unwrap();
    assert_ne!(composed, decomposed);
}

#[test]
fn malformed_and_duplicate_paths_return_exact_typed_refusals() {
    for name in [
        "",
        "/absolute",
        ".",
        "..",
        "./a",
        "a/./b",
        "a/../b",
        "a//b",
        "a/",
        "a\0b",
    ] {
        assert_eq!(
            checksum_relative_files(vec![entry(name, b"a")]).unwrap_err(),
            DirectoryChecksumError::InvalidRelativePath { path: name.into() }
        );
    }
    for bytes in [b"a".as_slice(), b"different".as_slice()] {
        assert_eq!(
            checksum_relative_files(vec![entry("a", b"a"), entry("a", bytes)]).unwrap_err(),
            DirectoryChecksumError::DuplicatePath { path: "a".into() }
        );
    }
    let refused = DirectoryChecksumError::DuplicatePath { path: "a".into() };
    let ArtifactError::Io(error) = ArtifactError::from(refused.clone()) else {
        panic!("existing filesystem error boundary");
    };
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert_eq!(
        error
            .get_ref()
            .unwrap()
            .downcast_ref::<DirectoryChecksumError>(),
        Some(&refused)
    );
}

#[cfg(unix)]
#[test]
fn non_utf8_identity_rejects_without_lossy_substitution() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};

    let path = PathBuf::from(OsString::from_vec(vec![0xff]));
    let refused = checksum_relative_files(vec![(
        path.clone(),
        ArtifactChecksumRecord::from_bytes(b"a"),
    )])
    .unwrap_err();
    assert_eq!(
        refused,
        DirectoryChecksumError::NonUtf8Path { path: path.clone() }
    );
    assert!(
        matches!(ArtifactError::from(refused), ArtifactError::NonUtf8Path { path: actual } if actual == path)
    );
    checksum_relative_files(vec![entry("\u{fffd}", b"a")]).unwrap();
}

#[cfg(unix)]
#[test]
fn filesystem_and_retained_declared_files_share_one_framing() {
    use ic_backup::ops::artifacts::{checksum_directory, checksum_file};
    use std::fs;

    let root = support::temp_root("ic-backup-directory-framing");
    fs::create_dir(root.join("nested")).unwrap();
    fs::write(root.join("a.txt"), b"a").unwrap();
    fs::write(root.join("nested/b.txt"), b"b").unwrap();
    let files = vec![
        ("a.txt".into(), checksum_file(&root.join("a.txt")).unwrap()),
        (
            "nested/b.txt".into(),
            checksum_file(&root.join("nested/b.txt")).unwrap(),
        ),
    ];
    let tree = checksum_directory(&root).unwrap();
    assert_eq!(checksum_relative_files(files.clone()).unwrap(), tree);
    fs::remove_dir_all(root).unwrap();
    // Composition consumes declared identities, without reopening the removed tree.
    assert_eq!(checksum_relative_files(files).unwrap(), tree);
}

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
#[test]
fn declared_checksum_requires_fresh_bytes_before_publication_and_recovery() {
    use ic_backup::{
        model::artifacts::ChecksumError,
        ops::{
            artifacts::checksum_directory,
            persistence::{ArtifactCommitOutcome, PersistenceError, commit_artifact_directory},
        },
    };
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
        commit_artifact_directory(&temporary, &canonical, expected.hash()),
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
        commit_artifact_directory(&temporary, &canonical, expected.hash()).unwrap(),
        ArtifactCommitOutcome::Published
    );
    assert!(!temporary.exists());
    assert_eq!(checksum_directory(&canonical).unwrap(), expected);
    assert_eq!(
        commit_artifact_directory(&temporary, &canonical, expected.hash()).unwrap(),
        ArtifactCommitOutcome::Recovered
    );

    // Recovery likewise cannot accept changed bytes under the declaration.
    fs::write(canonical.join("a-file"), b"drifted").unwrap();
    assert!(matches!(
        commit_artifact_directory(&temporary, &canonical, expected.hash()),
        Err(PersistenceError::Checksum(
            ChecksumError::ChecksumMismatch { .. }
        ))
    ));
    assert_eq!(fs::read(canonical.join("a-file")).unwrap(), b"drifted");
    assert!(!temporary.exists());
    fs::remove_dir_all(root).unwrap();
}
