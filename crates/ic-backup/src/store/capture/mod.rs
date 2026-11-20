//! Fixed direct-host capture with immutable approval, original custody and one-shot calls.

pub(in crate::store) mod advance;
mod arguments;
pub(in crate::store) mod assembly;
pub(in crate::store) mod completion;
pub(in crate::store) mod intent;
pub(in crate::store) mod plan;
pub(in crate::store) mod procedure;
mod snapshot;
pub(in crate::store) mod source;
pub(in crate::store) mod status;
mod succession;
#[cfg(test)]
pub(in crate::store) mod tests;
pub(in crate::store) mod transfer;

use super::{BackupStore, PRIVATE, manifest::lower_hex, open_optional, read_optional};
use crate::store::BackupDigest;
pub(in crate::store) use crate::store::OperationLimits;
pub use crate::store::operation::custody::OperationStoreError;
pub use crate::store::operation::{
    BackupApplicationGuard, BackupConnection, BackupSigner, CaptureOperation, OperationAction,
    OperationAdvanceError, OperationOptions, OperationProgress, OperationReplayError,
    OperationSigningContent, OperationSigningRequest, OperationStep, OperationView,
    framing::LedgerError, intent::Error as OperationPlanError,
};
pub(in crate::store) use advance::CaptureObservation;
pub use advance::{
    BackupHttpError, BackupHttpTransport, CaptureLookupError, CaptureObservationError,
    CaptureTiming, ReceivedCertificate, StatusCertificateError, StatusRequestError,
};
pub(in crate::store) use advance::{
    CertificateRequirements, authenticate_direct_request, certificate_observation,
};
pub(in crate::store) use advance::{PreparedStatus, StatusAdmissionRecord, StatusEvidence};
pub use arguments::MetadataReadError;
use ic_host_fs::{Directory, Lock};
pub use intent::CapturePlan;
pub(in crate::store) use plan::CapturePolicy;
pub use plan::CaptureRequest;
pub use procedure::Error as CapturePolicyError;
use serde::Serialize;
pub use snapshot::{SnapshotReplyError, decode_snapshot_list, decode_taken_snapshot};
pub use source::{
    CaptureSource, CaptureSourceError, RestoreTransfer, RestoreTransferError,
    RestoreTransferRequest,
};
pub use status::{CaptureStatusError, CaptureStatusReplyError};
use std::{
    ffi::OsStr,
    io::{self, Write},
};
pub use succession::CaptureSuccessionError;
pub(in crate::store) use transfer::CaptureTransfer;
pub use transfer::{DataReadError, TransferDataError, TransferError, TransferMetadataError};

pub(in crate::store) fn child(
    parent: &Directory,
    name: &str,
    create: bool,
) -> Result<Directory, OperationStoreError> {
    match parent.open_directory(OsStr::new(name), PRIVATE)? {
        Some(directory) => Ok(directory),
        None if create => Ok(parent.create_directory(OsStr::new(name), 0o700, PRIVATE)?),
        None => Err(OperationStoreError::Incomplete),
    }
}

pub(in crate::store) fn target_for(
    root: &Directory,
    target: super::BackupTarget,
    create: bool,
) -> Result<Directory, OperationStoreError> {
    let targets = child(root, "targets", false)?;
    let network = child(&targets, &target.network.to_string(), create)?;

    child(&network, &target.canister.to_text(), create)
}

pub(in crate::store) fn acquire(file: std::fs::File) -> Result<Lock, OperationStoreError> {
    Lock::acquire(file).map_err(|error| {
        if error.kind() == io::ErrorKind::WouldBlock {
            OperationStoreError::Locked
        } else {
            OperationStoreError::Io(error)
        }
    })
}

pub(in crate::store) fn lock_target(
    target: &Directory,
) -> Result<(Lock, bool), OperationStoreError> {
    if let Some(file) = open_optional(target, "lock")? {
        return Ok((acquire(file)?, false));
    }
    // A missing established lock must never be recreated alongside retained authority.
    if !target.entries(1)?.is_empty() {
        return Err(OperationStoreError::Incomplete);
    }
    let file = target.create_file(OsStr::new("lock"), 0o600)?;
    file.sync_all()?;

    Ok((acquire(file)?, true))
}

pub(in crate::store) fn retain(
    directory: &Directory,
    name: &str,
    bytes: &[u8],
) -> Result<(), OperationStoreError> {
    let mut file = directory.create_file(OsStr::new(name), 0o600)?;
    file.write_all(bytes)?;
    file.sync_all()?;

    Ok(())
}

pub(in crate::store) fn exact(
    directory: &Directory,
    name: &str,
    expected: &[u8],
    limit: usize,
) -> Result<(), OperationStoreError> {
    let bytes = read_optional(directory, name, limit)?.ok_or(OperationStoreError::Incomplete)?;
    if bytes != expected {
        return Err(OperationStoreError::Changed);
    }

    Ok(())
}

pub(in crate::store) fn same_identity(
    first: &Directory,
    second: &Directory,
) -> Result<(), OperationStoreError> {
    if first.identity()? != second.identity()? {
        return Err(OperationStoreError::Changed);
    }

    Ok(())
}

pub(in crate::store) fn same_directory(
    parent: &Directory,
    name: &str,
    held: &Directory,
) -> Result<(), OperationStoreError> {
    same_identity(&child(parent, name, false)?, held)
}

///
/// OwnerRecord
/// Binds an operation to its target.
///

#[derive(Serialize)]
struct OwnerRecord<'a> {
    format: &'a str,
    operation_id: &'a str,
    plan_sha256: String,
}

pub(in crate::store) fn owner(plan: &CapturePlan) -> Result<Vec<u8>, OperationStoreError> {
    owner_bytes(&plan.request().operation_id, plan.digest())
}

pub(in crate::store) fn owner_bytes(
    id: &str,
    digest: BackupDigest,
) -> Result<Vec<u8>, OperationStoreError> {
    Ok(serde_json::to_vec(&OwnerRecord {
        format: "ic-backup/local-target-owner/1",
        operation_id: id,
        plan_sha256: digest.to_string(),
    })?)
}
