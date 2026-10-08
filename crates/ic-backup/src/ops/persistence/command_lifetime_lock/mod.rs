//! One-command descriptor custody and exact existing-sidecar quiescence admission.

use super::file_lock::{self, FileLockError};
use crate::model::command_custody::{CommandCustodyRecord, CommandCustodyRecordError};
use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    process::{Child, Command},
    thread,
    time::{Duration, Instant},
};
use thiserror::Error;

const QUIESCENCE_GRACE: Duration = Duration::from_millis(250);
/// Child environment variable containing its inherited custody descriptor number.
pub const COMMAND_CUSTODY_DESCRIPTOR_ENV: &str = "IC_BACKUP_COMMAND_CUSTODY_FD";

/// Exclusive custody for one journal operation's local command attempt.
///
/// Persist [`Self::record`] and dispatch intent before spawning. This primitive
/// does not consume durable paid-call authority or qualify a backend's retry policy.
#[derive(Debug)]
pub struct CommandLifetimeLock {
    file: File,
    path: PathBuf,
    record: CommandCustodyRecord,
    dispatched: bool,
}

impl CommandLifetimeLock {
    /// Acquire an operation's regular no-follow sidecar without waiting.
    ///
    /// The journal parent must already exist. The journal leaf may be absent;
    /// aliases of an explicitly selected parent resolve to one location.
    ///
    /// # Errors
    /// Rejects unsafe journal/sidecar entries, contention, invalid identity and IO failures.
    pub fn acquire(
        journal: &Path,
        operation_sequence: u64,
    ) -> Result<Self, CommandLifetimeLockError> {
        let journal = resolve_journal(journal)?;
        let path = lock_path(&journal, operation_sequence);
        let file = file_lock::acquire(&path).map_err(|error| project_error(&path, error))?;
        let (device, inode) = file_identity(&file.metadata()?)?;
        let record = CommandCustodyRecord::new(journal, operation_sequence, device, inode)?;
        Ok(Self {
            file,
            path,
            record,
            dispatched: false,
        })
    }

    /// Return exact custody evidence for durable retention before dispatch.
    #[must_use]
    pub fn record(&self) -> &CommandCustodyRecord {
        &self.record
    }

    /// Return the retained sidecar path; dropping custody never removes it.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Attempt one trusted command with owned descriptor inheritance.
    ///
    /// Consumes this guard's single spawn allowance before the spawn attempt.
    /// The command value is consumed and dropped, so its parent-side descriptor
    /// copy cannot outlive spawn. The owner descriptor remains close-on-exec.
    /// The selected backend must preserve inherited custody in its descendants.
    /// Caller code owns argv admission, output/deadline limits and child reaping.
    ///
    /// # Errors
    /// Rejects repeated dispatch, replaced sidecars, unsupported hosts and spawn failures.
    pub fn spawn(&mut self, mut command: Command) -> Result<Child, CommandLifetimeLockError> {
        if self.dispatched {
            return Err(CommandLifetimeLockError::AlreadyDispatched {
                path: self.path.clone(),
            });
        }
        require_identity(&self.record, &fs::symlink_metadata(&self.path)?, &self.path)?;
        self.dispatched = true;
        #[cfg(unix)]
        {
            use command_fds::CommandFdExt;
            use std::os::fd::AsRawFd;
            let descriptor =
                rustix::io::fcntl_dupfd_cloexec(&self.file, 3).map_err(io::Error::from)?;
            command.env(
                COMMAND_CUSTODY_DESCRIPTOR_ENV,
                descriptor.as_raw_fd().to_string(),
            );
            command.preserved_fds(vec![descriptor]);
            let child = command.spawn();
            drop(command);
            child.map_err(CommandLifetimeLockError::from)
        }
        #[cfg(not(unix))]
        {
            let _ = command;
            Err(io::Error::from(io::ErrorKind::Unsupported).into())
        }
    }

    /// Close owner custody and admit exact quiescence within a bounded grace period.
    ///
    /// Success returns an exclusive non-spawning guard. Contention means another
    /// inherited holder remains; command exit alone is insufficient. Failures
    /// retain the sidecar and do not establish that a remote effect failed.
    ///
    /// # Errors
    /// Returns in-flight custody, changed/missing/unsafe sidecars and IO failures.
    pub fn finish(self) -> Result<CommandQuiescenceGuard, CommandLifetimeLockError> {
        let Self { file, record, .. } = self;
        drop(file);
        let deadline = Instant::now() + QUIESCENCE_GRACE;
        loop {
            match CommandQuiescenceGuard::acquire(&record) {
                Err(CommandLifetimeLockError::InFlight { .. }) if Instant::now() < deadline => {
                    thread::sleep(Duration::from_millis(5));
                }
                result => return result,
            }
        }
    }
}

/// Fresh exclusive local quiescence for retained exact custody evidence.
///
/// This guard cannot spawn commands. It excludes cooperating holders of the same
/// sidecar until dropped. Drop explicitly releases this non-spawning exclusion,
/// even if an unrelated fork temporarily retained a descriptor copy. Terminal
/// completion still requires its own journal proof.
#[derive(Debug)]
pub struct CommandQuiescenceGuard {
    file: File,
    record: CommandCustodyRecord,
}

impl CommandQuiescenceGuard {
    /// Observe an existing sidecar and hold exact local custody without creating it.
    ///
    /// # Errors
    /// Rejects active holders, changed/missing/unsafe identities and filesystem failures.
    pub fn acquire(expected: &CommandCustodyRecord) -> Result<Self, CommandLifetimeLockError> {
        let path = lock_path(expected.journal(), expected.operation_sequence());
        let file =
            file_lock::acquire_existing(&path).map_err(|error| project_error(&path, error))?;
        require_identity(expected, &file.metadata()?, &path)?;
        Ok(Self {
            file,
            record: expected.clone(),
        })
    }

    /// Return the exact retained evidence freshly admitted by this guard.
    #[must_use]
    pub fn record(&self) -> &CommandCustodyRecord {
        &self.record
    }
}

impl Drop for CommandQuiescenceGuard {
    fn drop(&mut self) {
        // Unlike dispatched command custody, this guard cannot intentionally
        // transfer its lock to a descendant. Closing alone would keep exclusion
        // alive until all transient fork/dup references also close.
        #[cfg(unix)]
        file_lock::unlock(&self.file);
    }
}

fn resolve_journal(path: &Path) -> Result<PathBuf, CommandLifetimeLockError> {
    let name = path
        .file_name()
        .ok_or_else(|| CommandLifetimeLockError::InvalidJournal {
            path: path.to_path_buf(),
        })?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let parent = parent.canonicalize()?;
    if !parent.is_dir() {
        return Err(io::Error::from(io::ErrorKind::NotADirectory).into());
    }
    let journal = parent.join(name);
    match fs::symlink_metadata(&journal) {
        Ok(metadata) if !metadata.is_file() => {
            return Err(CommandLifetimeLockError::InvalidJournal { path: journal });
        }
        Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(error.into()),
        _ => {}
    }
    Ok(journal)
}

fn lock_path(journal: &Path, operation_sequence: u64) -> PathBuf {
    let mut path = journal.as_os_str().to_os_string();
    path.push(format!(".command-{operation_sequence}.lock"));
    PathBuf::from(path)
}

fn file_identity(metadata: &fs::Metadata) -> io::Result<(u64, u64)> {
    if !metadata.is_file() {
        return Err(io::Error::from(io::ErrorKind::InvalidInput));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok((metadata.dev(), metadata.ino()))
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }
}

fn require_identity(
    expected: &CommandCustodyRecord,
    metadata: &fs::Metadata,
    path: &Path,
) -> Result<(), CommandLifetimeLockError> {
    if !metadata.is_file() {
        return Err(CommandLifetimeLockError::IdentityChanged {
            path: path.to_path_buf(),
        });
    }
    let (device, inode) = file_identity(metadata)?;
    if expected.matches_file(device, inode) {
        return Ok(());
    }
    Err(CommandLifetimeLockError::IdentityChanged {
        path: path.to_path_buf(),
    })
}

fn project_error(path: &Path, error: FileLockError) -> CommandLifetimeLockError {
    match error {
        FileLockError::Locked => CommandLifetimeLockError::InFlight {
            path: path.to_path_buf(),
        },
        FileLockError::UnsafeEntry { kind } => CommandLifetimeLockError::UnsafeEntry {
            path: path.to_path_buf(),
            kind,
        },
        FileLockError::Io(error) => CommandLifetimeLockError::Io(error),
    }
}

/// Typed custody admission, dispatch or quiescence failure.
#[derive(Debug, Error)]
pub enum CommandLifetimeLockError {
    /// An owner or inherited command still holds custody.
    #[error("command custody remains in flight: {path:?}")]
    InFlight {
        /// Retained sidecar path.
        path: PathBuf,
    },
    /// The retained sidecar is no longer the expected regular file.
    #[error("command custody identity changed: {path:?}")]
    IdentityChanged {
        /// Changed sidecar path.
        path: PathBuf,
    },
    /// One guard already attempted its single command spawn.
    #[error("command spawn already attempted: {path:?}")]
    AlreadyDispatched {
        /// Retained sidecar path.
        path: PathBuf,
    },
    /// The sidecar has an unsafe entry type.
    #[error("unsafe command custody entry at {path:?}: {kind}")]
    UnsafeEntry {
        /// Sidecar path.
        path: PathBuf,
        /// Observed entry kind.
        kind: String,
    },
    /// The selected journal location is unsafe or incomplete.
    #[error("invalid command journal location: {path:?}")]
    InvalidJournal {
        /// Rejected location.
        path: PathBuf,
    },
    /// Retained custody evidence fails model admission.
    #[error(transparent)]
    Record(#[from] CommandCustodyRecordError),
    /// A filesystem or process operation failed.
    #[error(transparent)]
    Io(#[from] io::Error),
}

#[cfg(all(test, unix))]
mod tests;
