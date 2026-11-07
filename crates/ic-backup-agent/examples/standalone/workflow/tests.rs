//! Retained dispatch evidence stays private and cannot be replaced or aliased.

use super::retain;
use std::{
    fs, io,
    os::unix::fs::{PermissionsExt, symlink},
};

#[test]
fn private_retained_bytes_refuse_duplicate_names_and_symlink_aliases() {
    let root = crate::tests::support::root("retained-evidence");
    let path = root.join("signed-original.cbor");
    let bytes = [0, 255, 17];
    retain(&path, &bytes).unwrap();
    assert_eq!(fs::read(&path).unwrap(), bytes);
    assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o177, 0);

    let error = retain(&path, b"replacement").unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
    assert!(error.raw_os_error().is_some());
    assert_eq!(fs::read(&path).unwrap(), bytes);

    let alias = root.join("signed-alias.cbor");
    symlink(&path, &alias).unwrap();
    assert_eq!(
        retain(&alias, b"replacement").unwrap_err().kind(),
        io::ErrorKind::AlreadyExists
    );
    assert_eq!(fs::read_link(&alias).unwrap(), path);
    assert_eq!(fs::read(&path).unwrap(), bytes);

    let missing = root.join("missing-parent");
    assert_eq!(
        retain(&missing.join("signed.cbor"), &bytes)
            .unwrap_err()
            .kind(),
        io::ErrorKind::NotFound
    );
    assert!(!missing.exists());
}
