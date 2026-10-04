//! Maintained v1 restore dependencies and model-owned retention transitions.

use crate::model::artifacts::{ArtifactChecksumRecord, ChecksumError};
use serde::{Deserialize, Deserializer, Serialize, de};
use std::{
    collections::BTreeSet,
    fmt,
    path::{Path, PathBuf},
};
use thiserror::Error;

/// Maximum references admitted into one layout's retained dependency record.
pub const MAX_RESTORE_REFERENCES: usize = 1024;
/// Maximum UTF-8 bytes in a normalized absolute journal location.
pub const MAX_JOURNAL_PATH_BYTES: usize = 4096;

/// One exact journal location bound to immutable restore-intent digest bytes.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "ReferenceFields")]
pub struct RestoreReferenceRecord {
    journal: PathBuf,
    authority: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReferenceFields {
    journal: PathBuf,
    authority: String,
}

impl TryFrom<ReferenceFields> for RestoreReferenceRecord {
    type Error = RestoreReferenceError;
    fn try_from(fields: ReferenceFields) -> Result<Self, Self::Error> {
        Self::new(fields.journal, &fields.authority)
    }
}

impl RestoreReferenceRecord {
    /// Bind a normalized absolute journal location to a SHA-256 intent digest.
    ///
    /// # Errors
    /// Rejects relative, noncanonical, non-UTF-8 or excessive paths and invalid hashes.
    pub fn new(journal: PathBuf, authority: &str) -> Result<Self, RestoreReferenceError> {
        if !super::journal_path::is_canonical(&journal, MAX_JOURNAL_PATH_BYTES) {
            return Err(RestoreReferenceError::InvalidJournal { journal });
        }
        let authority = ArtifactChecksumRecord::from_hash(authority)?
            .hash()
            .to_owned();
        Ok(Self { journal, authority })
    }

    /// Return the resolved journal location, which need not currently exist.
    #[must_use]
    pub fn journal(&self) -> &Path {
        &self.journal
    }

    /// Return the canonical immutable restore-intent digest.
    #[must_use]
    pub fn authority(&self) -> &str {
        &self.authority
    }
}

/// Durable unfinished restore dependencies for one backup layout.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "ReferencesFields")]
pub struct RestoreReferencesRecord {
    version: u16,
    restores: Vec<RestoreReferenceRecord>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReferencesFields {
    version: u16,
    #[serde(deserialize_with = "read_bounded_references")]
    restores: Vec<RestoreReferenceRecord>,
}

impl TryFrom<ReferencesFields> for RestoreReferencesRecord {
    type Error = RestoreReferenceError;
    fn try_from(mut fields: ReferencesFields) -> Result<Self, Self::Error> {
        if fields.version != 1 {
            return Err(RestoreReferenceError::UnsupportedVersion(fields.version));
        }
        let mut journals = BTreeSet::new();
        for entry in &fields.restores {
            if !journals.insert(entry.journal()) {
                return Err(RestoreReferenceError::DuplicateJournal {
                    journal: entry.journal.clone(),
                });
            }
        }
        fields
            .restores
            .sort_by(|left, right| left.journal.cmp(&right.journal));
        Ok(Self {
            version: 1,
            restores: fields.restores,
        })
    }
}

impl RestoreReferencesRecord {
    /// Construct an empty maintained-generation dependency record.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            version: 1,
            restores: Vec::new(),
        }
    }

    /// Read retained dependencies in canonical journal-path order.
    #[must_use]
    pub fn entries(&self) -> &[RestoreReferenceRecord] {
        &self.restores
    }

    /// Report whether there are no retained restore dependencies.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.restores.is_empty()
    }

    pub(crate) fn retain(
        &mut self,
        reference: RestoreReferenceRecord,
    ) -> Result<bool, RestoreReferenceError> {
        if let Some(existing) = self
            .restores
            .iter()
            .find(|entry| entry.journal == reference.journal)
        {
            if existing != &reference {
                return Err(RestoreReferenceError::AuthorityConflict {
                    journal: reference.journal,
                });
            }
            return Ok(false);
        }
        if self.restores.len() == MAX_RESTORE_REFERENCES {
            return Err(RestoreReferenceError::TooManyReferences {
                limit: MAX_RESTORE_REFERENCES,
            });
        }
        self.restores.push(reference);
        self.restores
            .sort_by(|left, right| left.journal.cmp(&right.journal));
        Ok(true)
    }
}

fn read_bounded_references<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<RestoreReferenceRecord>, D::Error> {
    struct ReferencesVisitor;
    impl<'de> de::Visitor<'de> for ReferencesVisitor {
        type Value = Vec<RestoreReferenceRecord>;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a bounded list of restore references")
        }
        fn visit_seq<A: de::SeqAccess<'de>>(
            self,
            mut sequence: A,
        ) -> Result<Self::Value, A::Error> {
            let mut entries = Vec::new();
            while let Some(entry) = sequence.next_element()? {
                if entries.len() == MAX_RESTORE_REFERENCES {
                    return Err(de::Error::custom("restore reference count exceeds limit"));
                }
                entries.push(entry);
            }
            Ok(entries)
        }
    }
    deserializer.deserialize_seq(ReferencesVisitor)
}

/// Typed schema or immutable-reference transition failure.
#[derive(Debug, Error)]
pub enum RestoreReferenceError {
    /// The journal path is not an exact bounded absolute UTF-8 location.
    #[error("invalid restore journal location: {journal:?}")]
    InvalidJournal {
        /// Rejected journal location.
        journal: PathBuf,
    },
    /// An intent digest is malformed.
    #[error(transparent)]
    Checksum(#[from] ChecksumError),
    /// The persisted reference generation is not maintained.
    #[error("unsupported restore references version {0}")]
    UnsupportedVersion(u16),
    /// A decoded record repeats one journal location.
    #[error("duplicate restore journal location: {journal:?}")]
    DuplicateJournal {
        /// Repeated journal location.
        journal: PathBuf,
    },
    /// An existing journal location is bound to different intent bytes.
    #[error("restore authority conflict at {journal:?}")]
    AuthorityConflict {
        /// Retained journal location.
        journal: PathBuf,
    },
    /// Retaining another dependency would exceed the maintained count bound.
    #[error("restore reference count exceeds {limit}")]
    TooManyReferences {
        /// Maximum admitted entries.
        limit: usize,
    },
}

#[cfg(test)]
mod tests;
