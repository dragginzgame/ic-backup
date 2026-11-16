//! Create-only exact manifest publication. The recorder owns completion policy.
use super::{OperationStoreError, open_optional};
use crate::store::{MANIFEST_LIMIT, StoreError, payload::unchanged_file};
use ic_host_fs::{
    capability::Directory,
    durable::{create_retained_at, publish_retained_at},
};
use std::{ffi::OsStr, io::Write, os::fd::AsFd};
pub(in crate::store) fn publish_manifest_file(
    bundle: &Directory,
    bytes: &[u8],
) -> Result<(), OperationStoreError> {
    if let Some(file) = open_optional(bundle, "manifest.json")? {
        verify_manifest_file(bundle, "manifest.json", &file, bytes)?;
        file.sync_all()?;
        bundle.sync()?;
        return Ok(());
    }
    let parent = bundle.open_syncable()?;
    let file = if let Some(file) = open_optional(bundle, "manifest.pending")? {
        file
    } else {
        let mut file = create_retained_at(parent.as_fd(), OsStr::new("manifest.pending"), 0o600)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        file
    };
    verify_manifest_file(bundle, "manifest.pending", &file, bytes)?;
    publish_retained_at(
        parent.as_fd(),
        OsStr::new("manifest.pending"),
        OsStr::new("manifest.json"),
        file,
    )
    .map_err(StoreError::from)?;
    Ok(())
}
pub(in crate::store) fn verify_manifest_file(
    bundle: &Directory,
    name: &str,
    file: &std::fs::File,
    expected: &[u8],
) -> Result<(), OperationStoreError> {
    let before = file.metadata()?;
    let reader = unchanged_file(bundle, name, file, &before)?;
    let bytes =
        ic_host_fs::read::read_opened_file(reader, MANIFEST_LIMIT).map_err(StoreError::from)?;
    if bytes != expected {
        return Err(OperationStoreError::Changed);
    }
    unchanged_file(bundle, name, file, &before)?;
    Ok(())
}
