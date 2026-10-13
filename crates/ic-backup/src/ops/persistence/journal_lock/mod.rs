//! Module: `persistence::journal_lock`
//!
//! Responsibility: serialize mutation of a persisted journal with a sidecar lock.
//! Does not own: journal validation, workflow policy, or domain error projection.

use std::{fs, io, path::Path, path::PathBuf};

use super::file_lock::{self, FileLockError};
use thiserror::Error as ThisError;

///
/// `JournalLockError`
///
/// Typed failure to acquire exclusive access to a journal's regular sidecar file.
///

#[derive(Debug, ThisError)]
pub enum JournalLockError {
    /// Another owner holds the sidecar lock.
    #[error("filesystem authority is locked: {lock_path}")]
    Locked {
        /// Sidecar path.
        lock_path: String,
    },

    /// The sidecar was replaced by a non-regular entry.
    #[error("unsafe lock entry at {lock_path}: {kind}")]
    UnsafeEntry {
        /// Sidecar path.
        lock_path: String,
        /// Observed entry kind.
        kind: String,
    },

    /// Filesystem IO failed.
    #[error(transparent)]
    Io(#[from] io::Error),
}

///
/// `JournalLock`
///
/// Exclusive journal mutation guard. Dropping it releases the host file lock.
///

#[derive(Debug)]
pub struct JournalLock {
    journal: PathBuf,
    #[cfg(unix)]
    file: fs::File,
}

impl JournalLock {
    /// Return the canonical journal location protected by this real sidecar guard.
    #[must_use]
    pub fn journal(&self) -> &Path {
        &self.journal
    }

    /// Acquire the journal sidecar without waiting or following its symlink.
    ///
    /// # Errors
    /// Returns typed lock contention, unsafe-entry or filesystem failure.
    pub fn acquire(journal_path: &Path) -> Result<Self, JournalLockError> {
        let name = journal_path
            .file_name()
            .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?;
        let parent = journal_path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        // Sidecars also serialize directory-shaped artifact publication locations.
        let journal = parent.canonicalize()?.join(name);
        let path = journal_lock_path(&journal);

        #[cfg(unix)]
        {
            acquire_supported(&path, journal)
        }

        #[cfg(not(unix))]
        {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!(
                    "journal locking is unsupported on this host: {}",
                    path.display()
                ),
            )
            .into())
        }
    }
}

#[cfg(unix)]
impl Drop for JournalLock {
    fn drop(&mut self) {
        file_lock::unlock(&self.file);
    }
}

#[cfg(unix)]
fn acquire_supported(path: &Path, journal: PathBuf) -> Result<JournalLock, JournalLockError> {
    match file_lock::acquire(path) {
        Ok(file) => Ok(JournalLock { journal, file }),
        Err(FileLockError::Locked) => Err(JournalLockError::Locked {
            lock_path: path.to_string_lossy().to_string(),
        }),
        Err(FileLockError::UnsafeEntry { kind }) => Err(JournalLockError::UnsafeEntry {
            lock_path: path.to_string_lossy().to_string(),
            kind,
        }),
        Err(FileLockError::Io(error)) => Err(JournalLockError::Io(error)),
    }
}

fn journal_lock_path(path: &Path) -> PathBuf {
    let mut lock_path = path.as_os_str().to_os_string();
    lock_path.push(".lock");
    PathBuf::from(lock_path)
}

#[cfg(test)]
mod tests;
