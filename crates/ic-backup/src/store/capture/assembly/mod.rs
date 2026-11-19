//! Closed payload-member hashing retained by completion.

use super::{OperationStoreError, open_optional};
use crate::store::{BackupDigest, StoreError, payload::unchanged_file};
use ic_host_artifacts::Length;
use ic_host_fs::{Directory, Kind as EntryKind};
use std::fs::File;

fn names(directory: &Directory, limit: u64) -> Result<Vec<String>, OperationStoreError> {
    let mut names = Vec::new();
    let mut total = 0_u64;
    for entry in directory.entries(1027)? {
        let name = entry.name.to_str().ok_or(OperationStoreError::Changed)?;
        let valid = matches!(name, "module.wasm" | "heap.bin" | "stable.bin")
            || name
                .strip_prefix("chunk-")
                .and_then(|s| s.strip_suffix(".bin"))
                .is_some_and(|s| super::lower_hex(s, 64));
        if !valid || entry.kind != EntryKind::File {
            return Err(OperationStoreError::Changed);
        }
        let file = open_optional(directory, name)?.ok_or(OperationStoreError::Incomplete)?;
        total = total
            .checked_add(file.metadata()?.len())
            .filter(|n| *n <= limit)
            .ok_or(OperationStoreError::Changed)?;
        names.push(name.to_owned());
    }
    names.sort();
    Ok(names)
}

pub(in crate::store) fn tree_members(
    directory: &Directory,
    limit: u64,
) -> Result<Vec<crate::store::manifest::MemberRecord>, OperationStoreError> {
    let original = names(directory, limit)?;
    let mut members = Vec::with_capacity(original.len());
    let mut held: Vec<(String, File, std::fs::Metadata)> = Vec::with_capacity(original.len());
    let mut total = 0_u64;
    for name in &original {
        let mut file = open_optional(directory, name)?.ok_or(OperationStoreError::Incomplete)?;
        let before = file.metadata()?;
        total = total
            .checked_add(before.len())
            .filter(|n| *n <= limit)
            .ok_or(OperationStoreError::Changed)?;
        let identity =
            ic_host_artifacts::copy(&mut file, std::io::sink(), Length::Exactly(before.len()))
                .map_err(StoreError::from)?;
        members.push(crate::store::manifest::MemberRecord {
            name: name.clone(),
            bytes: identity.bytes,
            sha256: BackupDigest::from_bytes(identity.sha256).to_string(),
        });
        held.push((name.clone(), file, before));
    }
    for (name, file, before) in &held {
        unchanged_file(directory, name, file, before)?;
    }
    if names(directory, limit)? != original {
        return Err(OperationStoreError::Changed);
    }
    Ok(members)
}
