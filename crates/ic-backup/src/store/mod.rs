//! Private local-store initialization, completion inventory and payload integrity.
//!
//! Callers exclude external namespace/content writers. Held directories resist
//! path substitution; they do not make files immutable. Concurrent capture may
//! publish new manifests, so a scan is not a point-in-time fleet snapshot.
//!
//! ```no_run
//! use ic_backup::store::{BackupStore, BackupTarget, ScanLimits};
//! use std::{num::NonZeroUsize, path::Path};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let store = BackupStore::open(Path::new("/private/backups"))?;
//! // The integration supplies its exact network commitment and canonical canister.
//! let target = BackupTarget {
//!     network: "11".repeat(32).parse()?,
//!     canister: "rrkah-fqaaa-aaaaa-aaaaq-cai".parse()?,
//! };
//! let limits = ScanLimits {
//!     entries: NonZeroUsize::new(1_000).unwrap(),
//!     bytes: NonZeroUsize::new(16 * 1024 * 1024).unwrap(),
//!     results: NonZeroUsize::new(100).unwrap(),
//! };
//! let history = store.list_backups(Some(&target), limits)?;
//! let latest = store.latest_completed_backup(&target, limits)?;
//! // Completion declarations; restore will require fresh verification.
//! # let _ = (history, latest);
//! # Ok(())
//! # }
//! ```

mod digest;
mod limits;
pub use digest::{BackupDigest, DigestParseError};
pub use limits::OperationLimits;
mod capture;
mod manifest;
mod operation;
mod payload;
mod restore;
pub use operation::intent::Plan as OperationPlan;
pub use restore::persistence::LoadedStopped;
pub use restore::{
    RestoreOperation, RestorePersistenceError, RestorePlan, RestorePlanError, RestorePolicyError,
    RestoreRequest,
};
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(in crate::store) use capture::CapturePolicy;
pub use capture::{
    BackupApplicationGuard, BackupConnection, BackupHttpError, BackupHttpTransport, BackupSigner,
    CaptureLookupError, CaptureObservationError, CaptureOperation, CapturePlan, CapturePolicyError,
    CaptureRequest, CaptureSource, CaptureSourceError, CaptureStatusError, CaptureStatusReplyError,
    CaptureSuccessionError, CaptureTiming, DataReadError, LedgerError, MetadataReadError,
    OperationAction, OperationAdvanceError, OperationOptions, OperationPlanError,
    OperationProgress, OperationReplayError, OperationSigningContent, OperationSigningRequest,
    OperationStep, OperationStoreError, OperationView, ReceivedCertificate, RestoreTransfer,
    RestoreTransferError, RestoreTransferRequest, SnapshotReplyError, StatusCertificateError,
    StatusRequestError, TransferDataError, TransferError, TransferMetadataError,
};
pub use manifest::ManifestError;
pub use payload::{PayloadReadError, PayloadSource};

use ic_host_artifacts::{Error as ArtifactError, Length};
use ic_host_fs::{Access, Directory, FilePolicy, Kind as EntryKind};
use ic_platform::Principal;
use std::{ffi::OsStr, fs::File, io, num::NonZeroUsize, path::Path};
use thiserror::Error;

const FORMAT: &[u8] = b"ic-backup/host-store/1\n";
const PRIVATE: u32 = 0o077;
const FILE_POLICY: FilePolicy = FilePolicy {
    forbidden: 0o177,
    single_link: true,
};
const MANIFEST_LIMIT: usize = 1024 * 1024;

/// An exact network instance and canister, independent of display names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackupTarget {
    /// Application-provisioned commitment to the network instance and trust binding.
    pub network: BackupDigest,
    /// Canonical Platform canister identity.
    pub canister: Principal,
}

/// Explicit scan limits; exceeding any limit returns an error, never partial success.
#[derive(Clone, Copy, Debug)]
pub struct ScanLimits {
    /// Maximum bundle-directory entries visited, including unfinished bundles.
    pub entries: NonZeroUsize,
    /// Cumulative manifest bytes read, including nonmatching targets.
    pub bytes: NonZeroUsize,
    /// Maximum returned rows. Latest-only queries return at most one row.
    pub results: NonZeroUsize,
}

/// A completion declaration, not fresh verification or restore authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackupSummary {
    /// Opaque capture operation identity, equal to its bundle directory name.
    pub bundle_id: String,
    /// Exact source network and canister.
    pub target: BackupTarget,
    /// Source snapshot ID represented as canonical lowercase hexadecimal.
    pub snapshot_id_hex: String,
    /// Snapshot capture time in nanoseconds since the Unix epoch, declared by IC metadata.
    pub captured_at_ns: u64,
    /// Original completion time from the operator host's clock, in Unix nanoseconds.
    pub completed_at_ns: u64,
    /// Integration-owned release commitment.
    pub release: BackupDigest,
    /// Declared module SHA-256.
    pub module_sha256: BackupDigest,
    /// Declared total payload bytes, excluding operation evidence.
    pub payload_bytes: u64,
    /// SHA-256 of the exact manifest bytes observed by this scan.
    pub manifest_sha256: BackupDigest,
}

impl BackupSummary {
    fn recency(&self) -> (u64, u64, &str) {
        (self.captured_at_ns, self.completed_at_ns, &self.bundle_id)
    }
}

/// A held local-store directory. No network client or background work is installed.
#[derive(Debug)]
pub struct BackupStore {
    root: Directory,
}

impl BackupStore {
    /// Initialize an existing empty private directory, preserving partial work on failure.
    ///
    /// The caller creates the directory and excludes external writers. A retained
    /// `format.pending` file claims initialization before child creation. An occupied
    /// or interrupted store is never reset. No payload or completion is manufactured.
    ///
    /// # Errors
    /// Refuses missing/unsafe/nonprivate directories, occupied roots and IO failures.
    /// Publication errors preserve whether the final format name became visible.
    pub fn initialize(path: &Path) -> Result<Self, StoreError> {
        let root = Directory::open(path, PRIVATE)?.ok_or(StoreError::MissingStore)?;
        match root.entries(1) {
            Ok(entries) if entries.is_empty() => (),
            Ok(_) | Err(ic_host_fs::Error::EntryLimit) => return Err(StoreError::Occupied),
            Err(error) => return Err(error.into()),
        }
        let stage = root
            .stage(
                OsStr::new("format.pending"),
                OsStr::new("format"),
                ic_host_fs::Policy {
                    destination: ic_host_fs::Destination::CreateNew,
                    permissions: 0o600,
                    retention: ic_host_fs::Retention::Retain,
                },
            )?
            .write_from(FORMAT, Length::Exactly(FORMAT.len() as u64))?
            .seal()?;
        for name in ["bundles", "operations", "targets"] {
            root.create_directory(OsStr::new(name), 0o700, PRIVATE)?;
        }
        commit(stage)?;
        Ok(Self { root })
    }

    /// Open an initialized private store without creating or repairing anything.
    ///
    /// Handles stay attached to the selected directory if its pathname is renamed.
    /// The caller owns continuity and excludes external writes to admitted contents.
    ///
    /// # Errors
    /// Refuses missing, partial, unsupported or unsafe roots. An empty directory
    /// is not interpreted as an empty initialized store.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let root = Directory::open(path, PRIVATE)?.ok_or(StoreError::MissingStore)?;
        let Some(format) = read_optional(&root, "format", 128)? else {
            return Err(StoreError::IncompleteStore);
        };
        if format != FORMAT {
            return Err(StoreError::UnsupportedFormat);
        }
        let mut count = 0;
        for entry in root.entries(4)? {
            let expected = match entry.name.to_str() {
                Some("format") => EntryKind::File,
                Some("bundles" | "operations" | "targets") => EntryKind::Directory,
                _ => return Err(StoreError::InvalidLayout),
            };
            if entry.kind != expected {
                return Err(StoreError::InvalidLayout);
            }
            if expected == EntryKind::Directory {
                root.open_directory(&entry.name, PRIVATE)?
                    .ok_or(StoreError::IncompleteStore)?;
            }
            count += 1;
        }
        if count != 4 {
            return Err(StoreError::IncompleteStore);
        }
        Ok(Self { root })
    }

    /// List matching completion declarations, newest capture first, without reading payloads.
    ///
    /// A bundle without a manifest is unfinished and omitted. Malformed manifests
    /// fail the entire scan even for other targets. No result establishes current
    /// integrity, authenticated provenance, durable replay or application safety.
    ///
    /// # Errors
    /// Returns traversal, schema and budget errors without a partial success result.
    pub fn list_backups(
        &self,
        target: Option<&BackupTarget>,
        limits: ScanLimits,
    ) -> Result<Vec<BackupSummary>, StoreError> {
        let mut rows = Vec::new();
        self.scan(target, limits, |row| {
            if rows.len() == limits.results.get() {
                return Err(StoreError::ResultLimit);
            }
            rows.push(row);
            Ok(())
        })?;
        rows.sort_by(|a, b| b.recency().cmp(&a.recency()));
        Ok(rows)
    }

    /// Find the latest completed capture for one exact target in this store.
    ///
    /// Scans all candidates with bounded work and retains only the best row. `None`
    /// means no matching completion was observed, not that the fleet has never had
    /// a backup elsewhere. A concurrently published completion may appear next time.
    ///
    /// # Errors
    /// Refuses incomplete scans or invalid records instead of returning an older
    /// backup or an authoritative absence result.
    pub fn latest_completed_backup(
        &self,
        target: &BackupTarget,
        limits: ScanLimits,
    ) -> Result<Option<BackupSummary>, StoreError> {
        let mut latest: Option<BackupSummary> = None;
        self.scan(Some(target), limits, |row| {
            if latest
                .as_ref()
                .is_none_or(|old| row.recency() > old.recency())
            {
                latest = Some(row);
            }
            Ok(())
        })?;
        Ok(latest)
    }

    fn scan(
        &self,
        target: Option<&BackupTarget>,
        limits: ScanLimits,
        mut accept: impl FnMut(BackupSummary) -> Result<(), StoreError>,
    ) -> Result<(), StoreError> {
        let bundles = self
            .root
            .open_directory(OsStr::new("bundles"), PRIVATE)?
            .ok_or(StoreError::IncompleteStore)?;
        let mut remaining = limits.bytes.get();
        for entry in bundles.entries(limits.entries.get())? {
            let id = entry
                .name
                .to_str()
                .filter(|id| manifest::lower_hex(id, 32))
                .ok_or(StoreError::InvalidLayout)?;
            if entry.kind != EntryKind::Directory {
                return Err(StoreError::InvalidLayout);
            }
            let bundle = bundles
                .open_directory(&entry.name, PRIVATE)?
                .ok_or(StoreError::InvalidLayout)?;
            let Some(bytes) =
                read_optional(&bundle, "manifest.json", MANIFEST_LIMIT.min(remaining))?
            else {
                continue;
            };
            remaining -= bytes.len();
            let row = manifest::decode(id, &bytes)
                .map_err(|source| StoreError::Manifest {
                    bundle: id.into(),
                    source,
                })?
                .summary;
            if target.is_none_or(|target| *target == row.target) {
                accept(row)?;
            }
        }
        Ok(())
    }
}

fn read_optional(
    root: &Directory,
    name: &str,
    limit: usize,
) -> Result<Option<Vec<u8>>, StoreError> {
    open_optional(root, name)?
        .map(|file| {
            ic_host_artifacts::read(file, Length::AtMost(limit as u64))
                .map(|(bytes, _)| bytes)
                .map_err(StoreError::Read)
        })
        .transpose()
}

fn open_optional(root: &Directory, name: &str) -> Result<Option<File>, StoreError> {
    Ok(root.open_file(OsStr::new(name), Access::Read, FILE_POLICY)?)
}

fn commit(stage: ic_host_fs::Sealed) -> Result<(), StoreError> {
    match stage.commit() {
        ic_host_fs::Commit::Committed(_) => Ok(()),
        ic_host_fs::Commit::NotPublished { cause, .. } => Err(StoreError::NotPublished {
            cause,
            cleanup: None,
        }),
        ic_host_fs::Commit::PublishedUnconfirmed {
            receipt,
            phase,
            cause,
            sync_error,
        } => Err(StoreError::PublishedUnconfirmed {
            receipt: Box::new(receipt),
            phase,
            cause,
            sync_error,
        }),
    }
}

/// A local refusal, preserving original IO/publication errors and retained files.
#[derive(Debug, Error)]
pub enum StoreError {
    /// Store root is absent; reads never create it.
    #[error("backup store does not exist")]
    MissingStore,
    /// Initialization encountered prior entries.
    #[error("backup store is occupied; original files were retained")]
    Occupied,
    /// Required initialized structure is absent.
    #[error("backup store initialization is incomplete")]
    IncompleteStore,
    /// The store marker names another format.
    #[error("unsupported backup store format")]
    UnsupportedFormat,
    /// Unexpected names, kinds, aliases or missing observed entries.
    #[error("invalid backup store layout")]
    InvalidLayout,
    /// Closing verification observed changed file identity, extent or directory custody.
    #[error("backup file or directory custody changed during verification")]
    CustodyChanged,
    /// Returned rows exceed the caller's allowance.
    #[error("backup result limit exceeded")]
    ResultLimit,
    /// Declared payload exceeds the caller's limit, before payload traversal.
    #[error("payload declares {declared} bytes, exceeding the {limit} byte limit")]
    PayloadLimit {
        /// Complete payload size from the admitted manifest.
        declared: u64,
        /// Caller-selected maximum.
        limit: u64,
    },
    /// Exact payload length/hash admission failed; the original file is retained.
    #[error("payload verification failed for {member}")]
    Payload {
        /// Validated member name from the manifest, never an arbitrary path.
        member: String,
        /// Original bounded read, exact-length or digest failure.
        #[source]
        source: ArtifactError,
    },
    /// Bounded manifest decoding or semantic validation failed.
    #[error("invalid completion manifest for bundle {bundle}")]
    Manifest {
        /// Validated bundle identity, never an arbitrary path.
        bundle: String,
        /// Original typed record refusal.
        #[source]
        source: ManifestError,
    },
    /// Host directory/file admission, including entry limits.
    #[error(transparent)]
    Filesystem(#[from] ic_host_fs::Error),
    /// Host bounded byte read or manifest digest mismatch; no partial result.
    #[error(transparent)]
    Read(#[from] ArtifactError),
    /// Native initialization IO; partial entries remain retained.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// Original publication phase, including visibility before failed durability.
    #[error(transparent)]
    Publication(#[from] ic_host_fs::AttemptFailure),
    /// Publication refused before this attempt made the destination visible.
    #[error("publication refused before visibility: {cause}")]
    NotPublished {
        /// Original refusal; no visibility was granted by this attempt.
        #[source]
        cause: ic_host_fs::Error,
        /// Cleanup refusal for an explicitly discarded temporary owner file.
        cleanup: Option<ic_host_fs::Error>,
    },
    /// Destination became visible; its cleanup or durability remains unconfirmed.
    #[error("publication visible but {phase:?} unconfirmed: {cause}")]
    PublishedUnconfirmed {
        /// Exact destination and admitted bytes.
        receipt: Box<ic_host_fs::Receipt>,
        /// First unconfirmed phase.
        phase: ic_host_fs::Phase,
        /// Original phase refusal.
        #[source]
        cause: ic_host_fs::Error,
        /// Additional directory sync refusal after cleanup failed.
        sync_error: Option<io::Error>,
    },
    /// Complete bytes differ from the caller's original commitment.
    #[error("backup digest mismatch: expected {expected}, received {actual}")]
    DigestMismatch {
        /// Original trusted commitment.
        expected: BackupDigest,
        /// Complete observed commitment.
        actual: BackupDigest,
    },
    /// A payload member differs from its admitted manifest commitment.
    #[error("payload digest mismatch for {member}: expected {expected}, received {actual}")]
    PayloadDigest {
        /// Admitted payload member name.
        member: String,
        /// Original manifest commitment.
        expected: BackupDigest,
        /// Complete observed commitment.
        actual: BackupDigest,
    },
}

pub use capture::transfer::readback::ReadbackMetadataError;
pub(in crate::store) use capture::{PreparedStatus, StatusAdmissionRecord, StatusEvidence};
