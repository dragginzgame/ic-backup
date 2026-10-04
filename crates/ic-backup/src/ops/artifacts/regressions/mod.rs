use super::*;
use crate::test_support::temp_path;
use std::fs;

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
    fs::create_dir(&root).expect("create tree");
    fs::write(root.join(OsStr::from_bytes(b"\xff")), b"bytes").expect("write non UTF-8 name");
    assert!(matches!(
        checksum_directory(&root),
        Err(ArtifactError::NonUtf8Path { .. })
    ));
    fs::remove_dir_all(root).expect("remove successful fixture");
}
