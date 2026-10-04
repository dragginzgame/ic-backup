use super::*;
use crate::model::artifacts::{ArtifactChecksumRecord, ChecksumError};
use crate::test_support::temp_path;
use std::fs;

const EMPTY_SHA256: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

// Ensure empty-byte checksums match the standard SHA-256 vector.
#[test]
fn byte_checksum_matches_sha256_vector() {
    let checksum = ArtifactChecksumRecord::from_bytes(&[]);

    assert_eq!(checksum.algorithm(), "sha256");
    assert_eq!(checksum.hash(), EMPTY_SHA256);
}

// Ensure file checksums use the same implementation as byte checksums.
#[test]
fn file_checksum_matches_byte_checksum() {
    let path = temp_path("canic-backup-checksum");
    fs::write(&path, b"canic backup artifact").expect("write temp artifact");

    let from_file = checksum_file(&path).expect("checksum file");
    let from_bytes = ArtifactChecksumRecord::from_bytes(b"canic backup artifact");

    fs::remove_file(&path).expect("remove temp artifact");
    assert_eq!(from_file, from_bytes);
}

// Ensure directory checksums are stable regardless of file creation order.
#[test]
fn directory_checksum_is_order_independent() {
    let first = temp_path("canic-backup-dir-a");
    let second = temp_path("canic-backup-dir-b");
    fs::create_dir_all(first.join("nested")).expect("create first");
    fs::create_dir_all(second.join("nested")).expect("create second");

    fs::write(first.join("a.txt"), b"a").expect("write first a");
    fs::write(first.join("nested/b.txt"), b"b").expect("write first b");
    fs::write(second.join("nested/b.txt"), b"b").expect("write second b");
    fs::write(second.join("a.txt"), b"a").expect("write second a");

    let first_checksum = checksum_directory(&first).expect("checksum first");
    let second_checksum = checksum_directory(&second).expect("checksum second");

    fs::remove_dir_all(first).expect("remove first");
    fs::remove_dir_all(second).expect("remove second");
    assert_eq!(first_checksum, second_checksum);
}

// Ensure checksum verification reports mismatches.
#[test]
fn checksum_verify_rejects_mismatch() {
    let checksum = ArtifactChecksumRecord::from_bytes(b"actual");

    let err = checksum
        .verify(EMPTY_SHA256)
        .expect_err("different hash should fail");

    assert!(matches!(err, ChecksumError::ChecksumMismatch { .. }));
}

#[test]
fn checksum_verification_accepts_equivalent_hex_case() {
    let lower = ArtifactChecksumRecord::from_bytes(&[]);
    let upper = ArtifactChecksumRecord::from_hash(&lower.hash().to_ascii_uppercase())
        .expect("normalize uppercase checksum");
    assert_eq!(lower, upper);
    lower
        .verify(&upper.hash().to_ascii_uppercase())
        .expect("uppercase expected hash");
    upper.verify(lower.hash()).expect("uppercase observed hash");
    assert!(matches!(
        lower.verify(
            &ArtifactChecksumRecord::from_bytes(b"different")
                .hash()
                .to_ascii_uppercase()
        ),
        Err(ChecksumError::ChecksumMismatch { .. })
    ));
    assert!(matches!(
        lower.verify("invalid"),
        Err(ChecksumError::InvalidHash(_))
    ));
}

#[cfg(unix)]
#[test]
fn filesystem_checksums_reject_symlinked_files_and_entries() {
    let root = temp_path("canic-backup-checksum-symlink");
    fs::create_dir_all(root.join("tree")).expect("create checksum tree");
    fs::write(root.join("source"), b"source").expect("write source");
    std::os::unix::fs::symlink(root.join("source"), root.join("linked-file"))
        .expect("create file symlink");
    std::os::unix::fs::symlink(root.join("source"), root.join("tree/linked-entry"))
        .expect("create tree symlink");

    let file_error =
        checksum_file(&root.join("linked-file")).expect_err("symlinked file must reject");
    let tree_error =
        checksum_directory(&root.join("tree")).expect_err("symlinked directory entry must reject");

    assert!(matches!(file_error, ArtifactError::Io(_)));
    assert!(matches!(tree_error, ArtifactError::UnsupportedEntry { .. }));
    fs::remove_dir_all(root).expect("remove fixture");
}
