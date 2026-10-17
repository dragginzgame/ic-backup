//! Stable parent-side layout locks and durable unfinished restore dependencies.

use crate::{
    model::restore_references::{RestoreReferenceRecord, RestoreReferencesRecord},
    ops::persistence::{JournalLock, JournalLockError, PersistenceError, read_json},
};
use ic_host_artifacts::artifact::Sha256Digest;
#[cfg(unix)]
use ic_host_fs::read::open_directory_no_follow;
use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

const REFERENCES_FILE: &str = "restore-references.json";
/// Maximum bytes decoded or published for one restore-dependency record.
pub const MAX_RESTORE_REFERENCE_BYTES: u64 = 1024 * 1024;

/// Exclusive access to an existing resolved backup layout and its dependencies.
///
/// All layout writers/removers must use the same parent-side lock. The operator
/// owns the trusted parent; this guard does not fence arbitrary filesystem writers.
#[derive(Debug)]
pub struct BackupLayoutGuard {
    root: PathBuf,
    directory: File,
    _lock: JournalLock,
}

impl BackupLayoutGuard {
    /// Resolve and exclusively lock an existing directory without recreating it.
    ///
    /// An explicitly selected root symlink resolves once. The lock remains beside
    /// the resolved directory, including after that directory is removed.
    ///
    /// # Errors
    /// Returns lock contention, unsafe lock entries, missing roots or IO failures.
    pub fn acquire(root: &Path) -> Result<Self, JournalLockError> {
        let root = root.canonicalize()?;
        let parent = root
            .parent()
            .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?;
        let name = root
            .file_name()
            .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?;
        let key = Sha256Digest::compute(name.as_encoded_bytes()).to_string();
        let lock = JournalLock::acquire(&parent.join(format!(".ic-backup-layout-{key}")))?;
        let directory = open_directory_no_follow(&root)?;
        Ok(Self {
            root,
            directory,
            _lock: lock,
        })
    }

    /// Return the resolved root protected by this guard.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Read bounded, validated restore dependencies; absent records mean empty.
    ///
    /// # Errors
    /// Rejects replaced layouts, unsafe entries, invalid records and IO failures.
    pub fn restore_references(&self) -> Result<RestoreReferencesRecord, PersistenceError> {
        self.check_root()?;
        let path = self.root.join(REFERENCES_FILE);
        match fs::symlink_metadata(&path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(RestoreReferencesRecord::empty());
            }
            Err(error) => return Err(error.into()),
            Ok(metadata) if !metadata.is_file() => {
                return Err(PersistenceError::InvalidRestoreReferences { path });
            }
            Ok(_) => {}
        }
        read_json(&path, MAX_RESTORE_REFERENCE_BYTES)
    }

    /// Report retained dependencies even if their external journals are missing.
    ///
    /// # Errors
    /// Returns the same conservative read failures as [`Self::restore_references`].
    pub fn has_restore_references(&self) -> Result<bool, PersistenceError> {
        Ok(!self.restore_references()?.is_empty())
    }

    /// Durably retain exact immutable restore intent before publishing its journal.
    ///
    /// Resolve the existing journal parent once; the journal itself may be absent.
    /// This binds local custody only and does not establish restore authority.
    /// Release requires the separate terminal/custody provider boundary.
    ///
    /// # Errors
    /// Rejects conflicting intent, invalid locations, unsafe records, limits and IO failures.
    pub fn retain_restore(
        &self,
        journal: &Path,
        authority: &str,
    ) -> Result<RestoreReferenceRecord, PersistenceError> {
        self.retain_with(journal, authority, super::json::write_encoded)
    }

    fn retain_with(
        &self,
        journal: &Path,
        authority: &str,
        write: impl FnOnce(&Path, &[u8]) -> Result<(), PersistenceError>,
    ) -> Result<RestoreReferenceRecord, PersistenceError> {
        let mut references = self.restore_references()?;
        let reference = RestoreReferenceRecord::new(
            super::journal_path::resolve_journal(
                journal,
                || io::Error::from(io::ErrorKind::InvalidInput).into(),
                |path| PersistenceError::InvalidRestoreReferences { path },
            )?,
            authority,
        )?;
        let path = self.root.join(REFERENCES_FILE);
        if references.retain(reference.clone())? {
            let bytes = super::json::encode_json(&references, MAX_RESTORE_REFERENCE_BYTES)?;
            write(&path, &bytes)?;
        } else {
            // Complete durability after a rename whose response was lost.
            sync_reference(&path)?;
            self.directory.sync_all()?;
        }
        Ok(reference)
    }

    pub(super) fn sync_references(&self) -> Result<(), PersistenceError> {
        self.check_root()?;
        let path = self.root.join(REFERENCES_FILE);
        match fs::symlink_metadata(&path) {
            Ok(_) => sync_reference(&path)?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        self.directory.sync_all()?;
        Ok(())
    }

    pub(super) fn check_root(&self) -> Result<(), PersistenceError> {
        match ic_host_fs::durable::admit_directory(&self.root, &self.directory) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::InvalidInput => {
                Err(PersistenceError::LayoutChanged {
                    path: self.root.clone(),
                })
            }
            Err(error) => Err(error.into()),
        }
    }
}

#[cfg(not(unix))]
fn open_directory_no_follow(_path: &Path) -> io::Result<File> {
    Err(io::Error::from(io::ErrorKind::Unsupported))
}

fn sync_reference(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use rustix::fs::{FileType, Mode, OFlags, fstat, open};
        let fd = open(
            path,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(io::Error::from)?;
        let metadata = fstat(&fd).map_err(io::Error::from)?;
        if !FileType::from_raw_mode(metadata.st_mode).is_file() {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }
        File::from(fd).sync_all()
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }
}

#[cfg(all(test, unix))]
mod tests;
