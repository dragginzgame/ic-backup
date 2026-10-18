use super::*;
use crate::test_support::temp_dir;
use std::fs;

// Ensure directory checksums are stable regardless of file creation order.
#[test]
fn directory_checksum_is_order_independent() {
    let first = temp_dir("ic-backup-dir-a");
    let second = temp_dir("ic-backup-dir-b");
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

#[cfg(unix)]
#[test]
fn filesystem_checksums_reject_symlinked_files_and_entries() {
    let root = temp_dir("ic-backup-checksum-symlink");
    fs::create_dir_all(root.join("tree")).expect("create checksum tree");
    fs::create_dir(root.join("real-directory")).expect("create nested source");
    fs::write(root.join("source"), b"source").expect("write source");
    fs::write(root.join("real-directory/source"), b"nested").expect("write nested source");
    std::os::unix::fs::symlink(root.join("source"), root.join("linked-file"))
        .expect("create file symlink");
    std::os::unix::fs::symlink(root.join("source"), root.join("tree/linked-entry"))
        .expect("create tree symlink");
    std::os::unix::fs::symlink(root.join("real-directory"), root.join("linked-directory"))
        .expect("create intermediate symlink");

    assert!(matches!(
        checksum_directory(&root.join("source")),
        Err(ArtifactError::UnsupportedEntry { .. })
    ));
    let file_error =
        checksum_directory(&root.join("linked-file")).expect_err("symlinked file must reject");
    let tree_error =
        checksum_directory(&root.join("tree")).expect_err("symlinked directory entry must reject");
    let nested_error = checksum_directory(&root.join("linked-directory/source"))
        .expect_err("symlinked intermediate directory must reject");

    assert!(matches!(file_error, ArtifactError::Io(_)));
    assert!(matches!(tree_error, ArtifactError::UnsupportedEntry { .. }));
    assert!(matches!(nested_error, ArtifactError::Io(_)));
    fs::remove_dir_all(root).expect("remove fixture");
}
