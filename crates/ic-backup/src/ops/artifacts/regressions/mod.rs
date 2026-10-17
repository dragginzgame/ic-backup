use super::*;
use crate::test_support::temp_dir;
use std::fs;

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

#[test]
fn host_declaration_refusals_keep_product_error_kinds_and_typed_causes() {
    let path = PathBuf::from("duplicate");
    let refused = DirectoryChecksumError::DuplicatePath { path: path.clone() };
    let ArtifactError::Io(error) = ArtifactError::from(refused.clone()) else {
        panic!("declaration refusal must retain the IO boundary");
    };
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert_eq!(
        error
            .get_ref()
            .unwrap()
            .downcast_ref::<DirectoryChecksumError>(),
        Some(&refused)
    );
    assert!(matches!(
        ArtifactError::from(DirectoryChecksumError::NonUtf8Path { path: path.clone() }),
        ArtifactError::NonUtf8Path { path: actual } if actual == path
    ));
}
