//! Fixed direct-host capture with immutable approval, original custody and one-shot calls.
pub(in crate::store) mod advance;
mod arguments;
pub(in crate::store) mod assembly;
pub(in crate::store) mod completion;
pub(in crate::store) mod intent;
pub(in crate::store) mod plan;
pub(in crate::store) mod procedure;
pub(in crate::store) use advance::CaptureObservation;
pub use arguments::MetadataReadError;
pub(in crate::store) use transfer::CaptureTransfer;
mod snapshot;
pub(in crate::store) mod source;
pub(in crate::store) mod status;
mod succession;
pub(in crate::store) mod transfer;
use super::{BackupStore, CREATE, PRIVATE, manifest::lower_hex, open_optional, read_optional};
pub(in crate::store) use crate::store::OperationLimits;
pub use crate::store::operation::framing::LedgerError;
pub use crate::store::operation::intent::Error as OperationPlanError;
pub use crate::store::operation::{
    BackupApplicationGuard, BackupConnection, BackupSigner, CaptureOperation, OperationAction,
    OperationAdvanceError, OperationOptions, OperationProgress, OperationReplayError,
    OperationSigningContent, OperationSigningRequest, OperationStep, OperationView,
};
pub use advance::{
    BackupHttpError, BackupHttpTransport, CaptureLookupError, CaptureObservationError,
    CaptureTiming, ReceivedCertificate, StatusCertificateError, StatusRequestError,
};
pub(in crate::store) use advance::{
    CertificateRequirements, authenticate_direct_request, certificate_observation,
};
use ic_host_artifacts::artifact::Sha256Digest;
use ic_host_fs::{
    capability::Directory,
    durable::{ExclusiveFileLock, create_retained_at},
    identity::FileIdentity,
};
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
    os::fd::AsFd,
};
pub use succession::CaptureSuccessionError;
pub use transfer::{DataReadError, TransferDataError, TransferError, TransferMetadataError};
pub(in crate::store) fn child(
    parent: &Directory,
    name: &str,
    create: bool,
) -> Result<Directory, OperationStoreError> {
    parent
        .open_directory(OsStr::new(name), if create { CREATE } else { PRIVATE })?
        .ok_or(OperationStoreError::Incomplete)
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
pub(in crate::store) fn acquire(
    file: std::fs::File,
) -> Result<ExclusiveFileLock, OperationStoreError> {
    ExclusiveFileLock::try_acquire(file).map_err(|error| {
        if error.kind() == io::ErrorKind::WouldBlock {
            OperationStoreError::Locked
        } else {
            OperationStoreError::Io(error)
        }
    })
}
pub(in crate::store) fn lock_target(
    target: &Directory,
) -> Result<(ExclusiveFileLock, bool), OperationStoreError> {
    if let Some(file) = open_optional(target, "lock")? {
        return Ok((acquire(file)?, false));
    }
    // A missing established lock must never be recreated alongside retained authority.
    if target.entries(1)?.next().transpose()?.is_some() {
        return Err(OperationStoreError::Incomplete);
    }
    let parent = target.open_syncable()?;
    let file = create_retained_at(parent.as_fd(), OsStr::new("lock"), 0o600)?;
    file.sync_all()?;
    Ok((acquire(file)?, true))
}
pub(in crate::store) fn retain(
    directory: &Directory,
    name: &str,
    bytes: &[u8],
) -> Result<(), OperationStoreError> {
    let parent = directory.open_syncable()?;
    let mut file = create_retained_at(parent.as_fd(), OsStr::new(name), 0o600)?;
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
    if FileIdentity::from_metadata(&first.open_syncable()?.metadata()?)
        != FileIdentity::from_metadata(&second.open_syncable()?.metadata()?)
    {
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
    digest: Sha256Digest,
) -> Result<Vec<u8>, OperationStoreError> {
    Ok(serde_json::to_vec(&OwnerRecord {
        format: "ic-backup/local-target-owner/1",
        operation_id: id,
        plan_sha256: digest.to_string(),
    })?)
}
pub use crate::store::operation::custody::OperationStoreError;

#[cfg(test)]
pub(in crate::store) mod test_fixtures;

pub(in crate::store) use advance::{PreparedStatus, StatusAdmissionRecord, StatusEvidence};
