//! Backup-owned journal binding over Host Process's inherited descriptor custody.

use crate::model::command_custody::{CommandCustodyRecord, CommandCustodyRecordError};
#[cfg(unix)]
use ic_host_process::custody::{CommandCustody, CustodyError, FileIdentity, QuiescenceGuard};
#[cfg(unix)]
use std::time::Duration;
use std::{
    fs, io,
    path::{Path, PathBuf},
    process::{Child, Command},
};
use thiserror::Error;

/// Child environment variable containing its inherited custody descriptor number.
pub const COMMAND_CUSTODY_DESCRIPTOR_ENV: &str = "IC_BACKUP_COMMAND_CUSTODY_FD";

/// Exclusive custody bound to one exact journal operation.
///
/// Persist the record and dispatch intent before spawning; paid-call authority,
/// argv, output bounds, deadlines and reaping remain integration-owned.
#[derive(Debug)]
pub struct CommandLifetimeLock {
    #[cfg(unix)]
    custody: CommandCustody,
    path: PathBuf,
    record: CommandCustodyRecord,
}

impl CommandLifetimeLock {
    /// Acquire the operation's regular no-follow sidecar without waiting.
    /// # Errors
    /// Rejects unsafe journals, contention, invalid identity and native I/O failures.
    pub fn acquire(
        journal: &Path,
        operation_sequence: u64,
    ) -> Result<Self, CommandLifetimeLockError> {
        let journal = super::journal_path::resolve_journal(
            journal,
            || CommandLifetimeLockError::InvalidJournal {
                path: journal.to_path_buf(),
            },
            |path| CommandLifetimeLockError::InvalidJournal { path },
        )?;
        let path = lock_path(&journal, operation_sequence);
        #[cfg(unix)]
        {
            let custody =
                CommandCustody::acquire(&path).map_err(|error| project_error(&path, error))?;
            let identity = custody.identity();
            let record = CommandCustodyRecord::new(
                journal,
                operation_sequence,
                identity.device,
                identity.inode,
            )?;
            Ok(Self {
                custody,
                path,
                record,
            })
        }
        #[cfg(not(unix))]
        {
            let _ = path;
            Err(io::Error::from(io::ErrorKind::Unsupported).into())
        }
    }

    /// Read exact Backup-owned evidence to retain before dispatch.
    #[must_use]
    pub fn record(&self) -> &CommandCustodyRecord {
        &self.record
    }

    /// Read the retained sidecar path; dropping custody never removes it.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Attempt one command with Host-owned descriptor inheritance and no retry.
    ///
    /// A failed spawn consumes the single allowance. Descendants must preserve
    /// the inherited descriptor; callers retain admission, deadlines and reaping.
    /// # Errors
    /// Rejects repeated dispatch, replaced sidecars and native spawn failures.
    pub fn spawn(&mut self, command: Command) -> Result<Child, CommandLifetimeLockError> {
        #[cfg(unix)]
        {
            self.custody
                .spawn(command, COMMAND_CUSTODY_DESCRIPTOR_ENV)
                .map_err(|error| project_error(&self.path, error))
        }
        #[cfg(not(unix))]
        {
            let _ = command;
            Err(io::Error::from(io::ErrorKind::Unsupported).into())
        }
    }

    /// Close owner custody and probe exact quiescence with Backup's bounded grace.
    ///
    /// Only contention is retried for 250 ms at 5 ms intervals. Failure preserves
    /// the sidecar and establishes no remote effect outcome or retry authority.
    /// # Errors
    /// Rejects active inherited holders, changed identities and native I/O failures.
    pub fn finish(self) -> Result<CommandQuiescenceGuard, CommandLifetimeLockError> {
        #[cfg(unix)]
        {
            let guard = self
                .custody
                .finish(Duration::from_millis(250), Duration::from_millis(5))
                .map_err(|error| project_error(&self.path, error))?;
            Ok(CommandQuiescenceGuard {
                _guard: guard,
                record: self.record,
            })
        }
        #[cfg(not(unix))]
        {
            Err(io::Error::from(io::ErrorKind::Unsupported).into())
        }
    }
}

/// Fresh exclusive quiescence bound to Backup's retained original journal record.
///
/// Host owns descriptor exclusion and explicit unlock on drop. This guard cannot
/// spawn; terminal settlement and reference-release admission remain separate.
#[derive(Debug)]
pub struct CommandQuiescenceGuard {
    #[cfg(unix)]
    _guard: QuiescenceGuard,
    record: CommandCustodyRecord,
}

impl CommandQuiescenceGuard {
    /// Admit the exact retained existing sidecar without creating it.
    /// # Errors
    /// Rejects active holders, replaced/missing/unsafe sidecars and I/O failures.
    pub fn acquire(expected: &CommandCustodyRecord) -> Result<Self, CommandLifetimeLockError> {
        #[cfg(unix)]
        {
            let path = lock_path(expected.journal(), expected.operation_sequence());
            let identity = FileIdentity {
                device: expected.device(),
                inode: expected.inode(),
            };
            let guard = QuiescenceGuard::acquire(&path, identity)
                .map_err(|error| project_error(&path, error))?;
            Ok(Self {
                _guard: guard,
                record: expected.clone(),
            })
        }
        #[cfg(not(unix))]
        {
            let _ = expected;
            Err(io::Error::from(io::ErrorKind::Unsupported).into())
        }
    }

    /// Read the exact retained evidence freshly admitted by this guard.
    #[must_use]
    pub fn record(&self) -> &CommandCustodyRecord {
        &self.record
    }
}

fn lock_path(journal: &Path, operation_sequence: u64) -> PathBuf {
    let mut path = journal.as_os_str().to_os_string();
    path.push(format!(".command-{operation_sequence}.lock"));
    PathBuf::from(path)
}

#[cfg(unix)]
fn project_error(path: &Path, error: CustodyError) -> CommandLifetimeLockError {
    use ic_host_fs::durable::RegularFileLockError;
    match error {
        CustodyError::InFlight { path } => CommandLifetimeLockError::InFlight { path },
        CustodyError::IdentityChanged { path } => {
            CommandLifetimeLockError::IdentityChanged { path }
        }
        CustodyError::AlreadyDispatched { path } => {
            CommandLifetimeLockError::AlreadyDispatched { path }
        }
        CustodyError::Io(error) => CommandLifetimeLockError::Io(error),
        CustodyError::Lock(RegularFileLockError::NotRegular) => {
            let kind = match fs::symlink_metadata(path) {
                Ok(metadata) if metadata.file_type().is_symlink() => "Symlink",
                Ok(metadata) if metadata.is_dir() => "Directory",
                _ => "Special",
            };
            CommandLifetimeLockError::UnsafeEntry {
                path: path.to_path_buf(),
                kind: kind.into(),
            }
        }
        CustodyError::Lock(error) => CommandLifetimeLockError::Io(error.into()),
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
