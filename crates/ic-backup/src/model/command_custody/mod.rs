//! Validated v1 evidence binding one command sidecar to an exact journal operation.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Maximum encoded bytes admitted when reading a standalone custody record.
pub const MAX_COMMAND_CUSTODY_RECORD_BYTES: u64 = 32 * 1024;

/// Retained local custody identity; this record alone grants no effect authority.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "CustodyFields")]
pub struct CommandCustodyRecord {
    version: u16,
    journal: PathBuf,
    operation_sequence: u64,
    device: u64,
    inode: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CustodyFields {
    version: u16,
    journal: PathBuf,
    operation_sequence: u64,
    device: u64,
    inode: u64,
}

impl TryFrom<CustodyFields> for CommandCustodyRecord {
    type Error = CommandCustodyRecordError;
    fn try_from(fields: CustodyFields) -> Result<Self, Self::Error> {
        if fields.version != 1 {
            return Err(CommandCustodyRecordError::UnsupportedVersion(
                fields.version,
            ));
        }
        Self::new(
            fields.journal,
            fields.operation_sequence,
            fields.device,
            fields.inode,
        )
    }
}

impl CommandCustodyRecord {
    pub(crate) fn new(
        journal: PathBuf,
        operation_sequence: u64,
        device: u64,
        inode: u64,
    ) -> Result<Self, CommandCustodyRecordError> {
        if !super::journal_path::is_canonical(&journal, 4096) {
            return Err(CommandCustodyRecordError::InvalidJournal { journal });
        }
        if inode == 0 {
            return Err(CommandCustodyRecordError::UnknownFileIdentity);
        }
        Ok(Self {
            version: 1,
            journal,
            operation_sequence,
            device,
            inode,
        })
    }

    /// Return the resolved journal location bound to this custody identity.
    #[must_use]
    pub fn journal(&self) -> &Path {
        &self.journal
    }

    /// Return the exact journal operation sequence.
    #[must_use]
    pub const fn operation_sequence(&self) -> u64 {
        self.operation_sequence
    }

    /// Return the observed sidecar filesystem device identity.
    #[must_use]
    pub const fn device(&self) -> u64 {
        self.device
    }

    /// Return the observed sidecar inode identity.
    #[must_use]
    pub const fn inode(&self) -> u64 {
        self.inode
    }

    pub(crate) const fn matches_file(&self, device: u64, inode: u64) -> bool {
        self.device == device && self.inode == inode
    }
}

/// Invalid maintained-generation local custody evidence.
#[derive(Debug, Error)]
pub enum CommandCustodyRecordError {
    /// The serialized generation is not maintained.
    #[error("unsupported command custody version {0}")]
    UnsupportedVersion(u16),
    /// The journal location is not bounded, canonical, absolute UTF-8.
    #[error("invalid command journal location: {journal:?}")]
    InvalidJournal {
        /// Rejected journal path.
        journal: PathBuf,
    },
    /// No usable inode identity was observed.
    #[error("command custody file identity is unknown")]
    UnknownFileIdentity,
}

#[cfg(test)]
mod tests;
