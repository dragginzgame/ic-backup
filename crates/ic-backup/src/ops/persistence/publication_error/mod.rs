//! Backup-owned publication failure and recovery evidence.

use std::{error::Error, fmt, io};

/// File publication failure with explicit visibility and retained cleanup evidence.
///
/// `AfterPublication` means the new output is visible, but final durability is
/// uncertain; callers must reconcile it before considering another write.
#[derive(Debug)]
pub enum PublicationError {
    /// The staged producer failed before publication was attempted.
    Producer {
        /// Original producer failure.
        source: io::Error,
        /// Failure to clean up the still-owned staging entry, if any.
        cleanup_error: Option<io::Error>,
    },
    /// Admission, synchronization, or rename failed before publication.
    BeforePublication {
        /// Original filesystem failure.
        source: io::Error,
        /// Failure to clean up the still-owned staging entry, if any.
        cleanup_error: Option<io::Error>,
    },
    /// The new output is visible, but cleanup or final directory sync failed.
    AfterPublication {
        /// Original completion failure.
        source: io::Error,
    },
}

impl PublicationError {
    pub(crate) fn from_host(error: ic_host_fs::durable::NamedWriteError<io::Error>) -> Self {
        use ic_host_fs::durable::NamedWriteError;
        match error {
            NamedWriteError::Producer {
                source,
                cleanup_error,
            } => Self::Producer {
                source,
                cleanup_error,
            },
            NamedWriteError::BeforePublication {
                source,
                cleanup_error,
            } => Self::BeforePublication {
                source,
                cleanup_error,
            },
            NamedWriteError::AfterPublication { source } => Self::AfterPublication { source },
        }
    }
}

impl fmt::Display for PublicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (description, source, cleanup_error) = match self {
            Self::Producer {
                source,
                cleanup_error,
            } => ("staged producer failed", source, cleanup_error.as_ref()),
            Self::BeforePublication {
                source,
                cleanup_error,
            } => (
                "publication failed before rename",
                source,
                cleanup_error.as_ref(),
            ),
            Self::AfterPublication { source } => {
                ("output published but completion failed", source, None)
            }
        };
        write!(formatter, "{description}: {source}")?;
        if let Some(cleanup_error) = cleanup_error {
            write!(formatter, "; staging cleanup failed: {cleanup_error}")?;
        }
        Ok(())
    }
}

impl Error for PublicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Producer { source, .. }
            | Self::BeforePublication { source, .. }
            | Self::AfterPublication { source } => Some(source),
        }
    }
}

#[cfg(test)]
mod tests;
