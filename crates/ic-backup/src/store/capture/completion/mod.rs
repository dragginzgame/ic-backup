//! Create-only exact manifest publication. The recorder owns completion policy.

use super::{OperationStoreError, open_optional};
use crate::store::{BackupDigest, MANIFEST_LIMIT, StoreError, payload::unchanged_file};
use ic_host_artifacts::{Identity, Length};
use ic_host_fs::Directory;
use std::ffi::OsStr;

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
    let written = if let Some(file) = open_optional(bundle, "manifest.pending")? {
        verify_manifest_file(bundle, "manifest.pending", &file, bytes)?;

        bundle.admit_retained(
            OsStr::new("manifest.pending"),
            OsStr::new("manifest.json"),
            &file,
            Identity {
                bytes: bytes.len() as u64,
                sha256: *BackupDigest::compute(bytes).as_bytes(),
            },
            crate::store::FILE_POLICY.forbidden,
        )?
    } else {
        let written = bundle
            .stage(
                OsStr::new("manifest.pending"),
                OsStr::new("manifest.json"),
                ic_host_fs::Policy {
                    destination: ic_host_fs::Destination::CreateNew,
                    permissions: 0o600,
                    retention: ic_host_fs::Retention::Retain,
                },
            )
            .map_err(StoreError::from)?
            .write_from(bytes, Length::Exactly(bytes.len() as u64))
            .map_err(StoreError::from)?;
        // Retain the existing exact readback before publication authority is sealed.
        let file =
            open_optional(bundle, "manifest.pending")?.ok_or(OperationStoreError::Incomplete)?;
        verify_manifest_file(bundle, "manifest.pending", &file, bytes)?;

        written
    };
    crate::store::commit(written.seal().map_err(StoreError::from)?)?;

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
    let (bytes, _) = ic_host_artifacts::read(reader, Length::AtMost(MANIFEST_LIMIT as u64))
        .map_err(StoreError::from)?;
    if bytes != expected {
        return Err(OperationStoreError::Changed);
    }
    unchanged_file(bundle, name, file, &before)?;

    Ok(())
}
