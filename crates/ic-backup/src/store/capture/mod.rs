//! Local capture intent and exclusive original custody. Direct-host status and snapshot continuation.

mod advance;
mod ledger;
pub use advance::{
    CaptureApplicationGuard, CaptureCall, CaptureCallAdmissionRecord, CaptureCallEvidence,
    CaptureObservation, CaptureObservationError, CaptureObservationPolicy, CaptureProvider,
    CaptureSettlementError, CaptureSigner, CaptureSigningRequest, CaptureSnapshotPhase,
    CaptureSnapshotView, CaptureStatusView, CaptureTiming, DirectCaptureError,
    DirectCaptureProvider, PreparedCaptureCall, PreparedStatus, SnapshotAdmissionRecord,
    SnapshotAdvanceError, SnapshotProgress, StatusAdmissionRecord, StatusAdvanceError,
    StatusCertificateError, StatusEvidence, StatusHttpError, StatusHttpTransport, StatusProgress,
    StatusProvider, StatusRequestError, authenticate_status_evidence, authenticate_status_request,
    observe_capture_certificate, status_certificate_evidence, status_certificate_observation,
};
mod plan;
mod snapshot;
pub use snapshot::{SnapshotReplyError, decode_snapshot_list, decode_taken_snapshot};
mod status;
mod transfer;
pub use ledger::{CaptureStatusPhase, LedgerError};
pub use plan::{CaptureExecution, CaptureLimitsRecord, CapturePlan, CaptureRequest};
pub use status::{CaptureStatusError, CaptureStatusReplyError};
pub use transfer::{
    CaptureTransfer, ReadbackDataError, ReadbackMetadataError, TransferDataError, TransferError,
    TransferMetadataError, UploadedSnapshotReadback,
};

use super::{
    BackupStore, CREATE, PRIVATE, StoreError, manifest::lower_hex, open_optional, read_optional,
};
use ic_host_artifacts::artifact::Sha256Digest;
use ic_host_fs::{
    capability::Directory,
    durable::{ExclusiveFileLock, create_retained_at},
    identity::FileIdentity,
};
use serde::Serialize;
use std::{
    ffi::OsStr,
    io::{self, Seek, SeekFrom, Write},
    os::{fd::AsFd, unix::fs::MetadataExt},
    path::Path,
};
use thiserror::Error;

/// Held original capture intent. Dropping releases process exclusion, not target ownership.
///
/// Direct-host status and snapshot continuation use durable attempt reservations.
/// There is no reconciliation, download, completion or owner-release API yet.
#[derive(Debug)]
pub struct CaptureOperation {
    store: Directory,
    operations: Directory,
    directory: Directory,
    target: Directory,
    lock: ExclusiveFileLock,
    target_lock_file: std::fs::File,
    ledger_file: std::fs::File,
    plan: CapturePlan,
    usable: bool,
}

impl CaptureOperation {
    /// Read original intent after re-admitting exact retained files and ownership.
    ///
    /// # Errors
    /// Refuses replaced custody, missing/changed records or any unrecognized ledger suffix.
    /// This never repairs files or interprets absent history as unused attempts.
    pub fn inspect(&self) -> Result<&CapturePlan, CaptureError> {
        self.check()?;
        Ok(&self.plan)
    }

    fn check(&self) -> Result<ledger::State, CaptureError> {
        if !self.usable {
            return Err(CaptureError::Unusable);
        }
        let request = self.plan.request();
        exact(&self.store, "format", super::FORMAT, 128)?;
        same_directory(&self.store, "operations", &self.operations)?;
        same_directory(&self.operations, &request.operation_id, &self.directory)?;
        let target = target_directory(&self.store, &self.plan, false)?;
        same_identity(&target, &self.target)?;
        let current_target_lock =
            open_optional(&self.target, "lock")?.ok_or(CaptureError::Incomplete)?;
        if FileIdentity::from_file(&current_target_lock)?
            != FileIdentity::from_file(&self.target_lock_file)?
        {
            return Err(CaptureError::Changed);
        }
        exact(&self.target, "lock", b"", 0)?;
        for entry in self.target.entries(2)? {
            if !matches!(entry?.name.to_str(), Some("lock" | "owner.json")) {
                return Err(CaptureError::Changed);
            }
        }
        let current_lock =
            open_optional(&self.directory, "lock")?.ok_or(CaptureError::Incomplete)?;
        if FileIdentity::from_file(&current_lock)? != FileIdentity::from_file(self.lock.file())? {
            return Err(CaptureError::Changed);
        }
        exact(&self.directory, "lock", b"", 0)?;
        exact(
            &self.directory,
            "plan.json",
            self.plan.encoded(),
            plan::PLAN_LIMIT,
        )?;
        let state = ledger::admit(&self.directory, &self.ledger_file, &self.plan)?;
        if let Some(reference) = &state.request {
            self.evidence("status-request.bin", reference)?;
        }
        if let Some(reference) = &state.result {
            self.evidence("status-result.bin", reference)?;
        }
        for (name, reference) in state.snapshot.evidence() {
            if let Some(reference) = reference {
                self.evidence(name, reference)?;
            }
        }
        exact(&self.target, "owner.json", &owner(&self.plan)?, 1024)?;
        for entry in self.directory.entries(9)? {
            let entry = entry?;
            let admitted = match entry.name.to_str() {
                Some("lock" | "plan.json" | "ledger.bin") => true,
                Some("status-request.bin") => state.request.is_some(),
                Some("status-result.bin") => state.result.is_some(),
                Some(name) => state
                    .snapshot
                    .evidence()
                    .iter()
                    .any(|(expected, reference)| name == *expected && reference.is_some()),
                None => false,
            };
            if !admitted {
                return Err(CaptureError::Changed);
            }
        }
        Ok(state)
    }
}

impl BackupStore {
    /// Persist one approved capture intent and claim its target, without network effects.
    ///
    /// Target ownership is durable before operation creation. Any failure retains
    /// partial files and may leave the target occupied; it never resets that claim.
    /// The original target cannot receive another operation until future qualified
    /// terminal succession is implemented. Independent targets do not share a lock.
    ///
    /// # Errors
    /// Refuses wrong approval before writes, occupied IDs/targets, contention,
    /// unsafe paths and IO failures. Partial creation requires original recovery;
    /// this API does not adopt or repair it.
    pub fn create_capture(
        &self,
        plan: CapturePlan,
        approval: Sha256Digest,
    ) -> Result<CaptureOperation, CaptureError> {
        if plan.digest() != approval {
            return Err(CaptureError::ApprovalMismatch);
        }
        exact(&self.root, "format", super::FORMAT, 128)?;
        let owner_bytes = owner(&plan)?;
        let ledger_bytes = ledger::initial(&plan)?;
        let operations = child(&self.root, "operations", false)?;
        let id = &plan.request().operation_id;
        if operations
            .open_directory(OsStr::new(id), PRIVATE)?
            .is_some()
        {
            return Err(CaptureError::Occupied);
        }
        let target = target_directory(&self.root, &plan, true)?;
        let (target_lock, created_lock) = lock_target(&target)?;
        if read_optional(&target, "owner.json", 1024)?.is_some() {
            return Err(CaptureError::Occupied);
        }
        if !created_lock {
            return Err(CaptureError::Incomplete);
        }
        exact(&target, "lock", b"", 0)?;
        for entry in target.entries(1)? {
            if entry?.name != OsStr::new("lock") {
                return Err(CaptureError::Changed);
            }
        }
        retain(&target, "owner.json", &owner_bytes)?;
        let directory = child(&operations, id, true)?;
        let parent = directory.open_syncable()?;
        let file = create_retained_at(parent.as_fd(), OsStr::new("lock"), 0o600)?;
        file.sync_all()?;
        let lock = acquire(file)?;
        retain(&directory, "plan.json", plan.encoded())?;
        retain(&directory, "ledger.bin", &ledger_bytes)?;
        directory.sync()?;
        let target_lock_file = target_lock.file().try_clone()?;
        let ledger_file =
            open_optional(&directory, "ledger.bin")?.ok_or(CaptureError::Incomplete)?;
        let operation = CaptureOperation {
            store: self.root.try_clone()?,
            operations,
            directory,
            target,
            lock,
            target_lock_file,
            ledger_file,
            plan,
            usable: true,
        };
        operation.check()?;
        drop(target_lock);
        Ok(operation)
    }

    /// Open only complete original capture intent under a nonblocking operation lock.
    ///
    /// No files are created or synchronized. Original status reservations remain
    /// spent; unknown or partial events refuse rather than project zero spending.
    ///
    /// # Errors
    /// Refuses wrong original commitment, partial/changed records, unsafe custody
    /// or another holder. Reopening does not free or replace the durable target owner.
    pub fn open_capture(
        &self,
        operation_id: &str,
        approval: Sha256Digest,
    ) -> Result<CaptureOperation, CaptureError> {
        if !lower_hex(operation_id, 32) {
            return Err(CaptureError::InvalidPlan);
        }
        let operations = child(&self.root, "operations", false)?;
        let directory = child(&operations, operation_id, false)?;
        let lock = acquire(open_optional(&directory, "lock")?.ok_or(CaptureError::Incomplete)?)?;
        let bytes = read_optional(&directory, "plan.json", plan::PLAN_LIMIT)?
            .ok_or(CaptureError::Incomplete)?;
        let plan = CapturePlan::decode(bytes)?;
        if plan.digest() != approval || plan.request().operation_id != operation_id {
            return Err(CaptureError::ApprovalMismatch);
        }
        let target = target_directory(&self.root, &plan, false)?;
        let target_lock_file = open_optional(&target, "lock")?.ok_or(CaptureError::Incomplete)?;
        let ledger_file =
            open_optional(&directory, "ledger.bin")?.ok_or(CaptureError::Incomplete)?;
        let operation = CaptureOperation {
            store: self.root.try_clone()?,
            operations,
            directory,
            target,
            lock,
            target_lock_file,
            ledger_file,
            plan,
            usable: true,
        };
        operation.check()?;
        Ok(operation)
    }
}

fn child(parent: &Directory, name: &str, create: bool) -> Result<Directory, CaptureError> {
    parent
        .open_directory(OsStr::new(name), if create { CREATE } else { PRIVATE })?
        .ok_or(CaptureError::Incomplete)
}
fn target_directory(
    root: &Directory,
    plan: &CapturePlan,
    create: bool,
) -> Result<Directory, CaptureError> {
    let targets = child(root, "targets", false)?;
    let network = child(&targets, &plan.request().target.network.to_string(), create)?;
    child(&network, &plan.request().target.canister.to_text(), create)
}
fn acquire(file: std::fs::File) -> Result<ExclusiveFileLock, CaptureError> {
    ExclusiveFileLock::try_acquire(file).map_err(|error| {
        if error.kind() == io::ErrorKind::WouldBlock {
            CaptureError::Locked
        } else {
            CaptureError::Io(error)
        }
    })
}
fn lock_target(target: &Directory) -> Result<(ExclusiveFileLock, bool), CaptureError> {
    if let Some(file) = open_optional(target, "lock")? {
        return Ok((acquire(file)?, false));
    }
    // A missing established lock must never be recreated alongside retained authority.
    if target.entries(1)?.next().transpose()?.is_some() {
        return Err(CaptureError::Incomplete);
    }
    let parent = target.open_syncable()?;
    let file = create_retained_at(parent.as_fd(), OsStr::new("lock"), 0o600)?;
    file.sync_all()?;
    Ok((acquire(file)?, true))
}
fn retain(directory: &Directory, name: &str, bytes: &[u8]) -> Result<(), CaptureError> {
    let parent = directory.open_syncable()?;
    let mut file = create_retained_at(parent.as_fd(), OsStr::new(name), 0o600)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}
fn exact(
    directory: &Directory,
    name: &str,
    expected: &[u8],
    limit: usize,
) -> Result<(), CaptureError> {
    let bytes = read_optional(directory, name, limit)?.ok_or(CaptureError::Incomplete)?;
    if bytes != expected {
        return Err(CaptureError::Changed);
    }
    Ok(())
}
fn same_identity(first: &Directory, second: &Directory) -> Result<(), CaptureError> {
    if FileIdentity::from_metadata(&first.open_syncable()?.metadata()?)
        != FileIdentity::from_metadata(&second.open_syncable()?.metadata()?)
    {
        return Err(CaptureError::Changed);
    }
    Ok(())
}
fn same_directory(parent: &Directory, name: &str, held: &Directory) -> Result<(), CaptureError> {
    same_identity(&child(parent, name, false)?, held)
}

#[derive(Serialize)]
struct OwnerRecord<'a> {
    format: &'a str,
    operation_id: &'a str,
    plan_sha256: String,
}
fn owner(plan: &CapturePlan) -> Result<Vec<u8>, CaptureError> {
    Ok(serde_json::to_vec(&OwnerRecord {
        format: "ic-backup/local-target-owner/1",
        operation_id: &plan.request().operation_id,
        plan_sha256: plan.digest().to_string(),
    })?)
}
/// Capture intent admission or local persistence refusal. No error authorizes retrying effects.
#[derive(Debug, Error)]
pub enum CaptureError {
    /// Invalid identity, encoding generation or finite limits.
    #[error("invalid capture plan")]
    InvalidPlan,
    /// Caller-selected approval differs from retained exact intent.
    #[error("capture plan approval mismatch")]
    ApprovalMismatch,
    /// Target or operation already has retained evidence.
    #[error("capture target or operation is occupied")]
    Occupied,
    /// Another process/handle currently holds exclusion.
    #[error("capture operation is locked")]
    Locked,
    /// Missing original evidence; opening never repairs it.
    #[error("capture intent is incomplete")]
    Incomplete,
    /// Original record bytes, ledger or namespace no longer agree.
    #[error("capture intent or custody changed")]
    Changed,
    /// Original live handle encountered uncertain persistence; reopen for diagnosis.
    #[error("capture session is unusable after uncertain persistence")]
    Unusable,
    /// Unsupported route, changed admission or unaffordable first status call.
    #[error("status admission is invalid or exceeds original authority")]
    StatusAdmission,
    /// Bounded framing, exact chain or supported-transition admission failed.
    #[error(transparent)]
    Ledger(#[from] LedgerError),
    /// Strict JSON admission/encoding failure.
    #[error("invalid capture record encoding")]
    Encoding(#[from] serde_json::Error),
    /// Private store and bounded file admission.
    #[error(transparent)]
    Store(#[from] StoreError),
    /// Held-directory admission failure.
    #[error(transparent)]
    Filesystem(#[from] ic_host_fs::capability::CapabilityError),
    /// Original native IO failure; all partial files remain retained.
    #[error(transparent)]
    Io(#[from] io::Error),
}

impl CaptureOperation {
    fn evidence(
        &self,
        name: &str,
        reference: &ledger::EvidenceRecord,
    ) -> Result<Vec<u8>, CaptureError> {
        let bytes = read_optional(&self.directory, name, reference.bytes as usize)?
            .ok_or(CaptureError::Incomplete)?;
        if bytes.len() != reference.bytes as usize
            || Sha256Digest::compute(&bytes).as_bytes() != &reference.sha256
        {
            return Err(CaptureError::Changed);
        }
        Ok(bytes)
    }

    fn commit(
        &mut self,
        event: &ledger::EventRecord,
        evidence: Option<(&str, &[u8])>,
    ) -> Result<(), CaptureError> {
        self.commit_with(event, evidence, |file, bytes| {
            file.write_all(bytes)?;
            file.sync_all()
        })
    }

    // Fault injection is confined to the filesystem write/sync boundary in native tests.
    fn commit_with(
        &mut self,
        event: &ledger::EventRecord,
        evidence: Option<(&str, &[u8])>,
        write: impl FnOnce(&mut std::fs::File, &[u8]) -> io::Result<()>,
    ) -> Result<(), CaptureError> {
        let state = self.check()?;
        let frame = state.candidate(event, &self.plan)?;
        match (event, evidence) {
            (ledger::EventRecord::SnapshotPrepared { call, request, .. }, Some((name, bytes)))
                if name == advance::snapshots::names(*call)?.0
                    && *request == ledger::EvidenceRecord::new(bytes) => {}
            (ledger::EventRecord::SnapshotSettled { call, result, .. }, Some((name, bytes)))
                if name == advance::snapshots::names(*call)?.1
                    && *result == ledger::EvidenceRecord::new(bytes) => {}
            (
                ledger::EventRecord::SnapshotStarted { .. }
                | ledger::EventRecord::SnapshotTakeReserved { .. }
                | ledger::EventRecord::SnapshotSubmissionReserved { .. },
                None,
            ) => {}
            (
                ledger::EventRecord::Prepared { request, .. },
                Some(("status-request.bin", bytes)),
            ) if *request == ledger::EvidenceRecord::new(bytes) => {}
            (ledger::EventRecord::Settled { result, .. }, Some(("status-result.bin", bytes)))
                if *result == ledger::EvidenceRecord::new(bytes) => {}
            (
                ledger::EventRecord::PreparationReserved { .. }
                | ledger::EventRecord::SubmissionReserved { .. },
                None,
            ) => {}
            _ => return Err(CaptureError::Changed),
        }
        let mut file = self
            .directory
            .resolve_parent(Path::new("ledger.bin"), PRIVATE)?
            .ok_or(CaptureError::Incomplete)?
            .open_regular_file_for_update(0o177)?
            .ok_or(CaptureError::Incomplete)?;
        if file.metadata()?.nlink() != 1
            || FileIdentity::from_file(&file)? != FileIdentity::from_file(&self.ledger_file)?
        {
            return Err(CaptureError::Changed);
        }
        if self.check()? != state {
            return Err(CaptureError::Changed);
        }
        // No fallible persistence below can leave live dispatch authority behind.
        self.usable = false;
        file.sync_all()?;
        if state.request.is_some() {
            open_optional(&self.directory, "status-request.bin")?
                .ok_or(CaptureError::Incomplete)?
                .sync_all()?;
        }
        if state.result.is_some() {
            open_optional(&self.directory, "status-result.bin")?
                .ok_or(CaptureError::Incomplete)?
                .sync_all()?;
        }
        for (name, reference) in state.snapshot.evidence() {
            if reference.is_some() {
                open_optional(&self.directory, name)?
                    .ok_or(CaptureError::Incomplete)?
                    .sync_all()?;
            }
        }
        self.directory.sync()?;
        if let Some((name, bytes)) = evidence {
            retain(&self.directory, name, bytes)?;
        }
        file.seek(SeekFrom::Start(state.bytes))?;
        write(&mut file, &frame)?;
        // Closing admission checks the entire resulting prefix and evidence again.
        self.usable = true;
        let checked = self.check();
        self.usable = checked.is_ok();
        checked?;
        Ok(())
    }
}
