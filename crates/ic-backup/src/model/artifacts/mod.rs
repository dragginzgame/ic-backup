//! Canonical SHA-256 artifact metadata; filesystem effects belong to ops.

use ic_host_artifacts::artifact::Sha256Digest;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Maintained checksum record with a lowercase, validated SHA-256 digest.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "ChecksumFields")]
pub struct ArtifactChecksumRecord {
    algorithm: String,
    hash: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChecksumFields {
    algorithm: String,
    hash: String,
}

impl TryFrom<ChecksumFields> for ArtifactChecksumRecord {
    type Error = ChecksumError;

    fn try_from(value: ChecksumFields) -> Result<Self, Self::Error> {
        if value.algorithm != "sha256" {
            return Err(ChecksumError::UnsupportedAlgorithm(value.algorithm));
        }
        Self::from_hash(&value.hash)
    }
}

impl ArtifactChecksumRecord {
    /// Compute checksum metadata from exact bytes.
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            algorithm: "sha256".to_owned(),
            hash: Sha256Digest::compute(bytes).to_string(),
        }
    }

    /// Validate and normalize an existing digest.
    ///
    /// # Errors
    /// Returns [`ChecksumError::InvalidHash`] unless the input has 64 hex digits.
    pub fn from_hash(hash: &str) -> Result<Self, ChecksumError> {
        Ok(Self {
            algorithm: "sha256".to_owned(),
            hash: canonical_hash(hash)?,
        })
    }

    pub(crate) fn from_digest(digest: [u8; 32]) -> Self {
        Self {
            algorithm: "sha256".to_owned(),
            hash: Sha256Digest::from_bytes(digest).to_string(),
        }
    }

    /// Return the maintained algorithm identifier.
    #[must_use]
    pub fn algorithm(&self) -> &str {
        &self.algorithm
    }

    /// Return the canonical lowercase hexadecimal digest.
    #[must_use]
    pub fn hash(&self) -> &str {
        &self.hash
    }

    /// Compare a digest, accepting equivalent uppercase and lowercase hex.
    ///
    /// # Errors
    /// Returns a typed malformed-hash or checksum-mismatch error.
    pub fn verify(&self, expected_hash: &str) -> Result<(), ChecksumError> {
        let expected = canonical_hash(expected_hash)?;
        if self.hash == expected {
            Ok(())
        } else {
            Err(ChecksumError::ChecksumMismatch {
                expected,
                actual: self.hash.clone(),
            })
        }
    }
}

pub(crate) fn canonical_hash(hash: &str) -> Result<String, ChecksumError> {
    let normalized = hash.to_ascii_lowercase();
    normalized
        .parse::<Sha256Digest>()
        .map_err(|_| ChecksumError::InvalidHash(hash.to_owned()))?;
    Ok(normalized)
}

/// Typed checksum-record validation failure.
#[derive(Debug, Error)]
pub enum ChecksumError {
    /// The expected and observed exact bytes have different digests.
    #[error("checksum mismatch: expected {expected}, actual {actual}")]
    ChecksumMismatch {
        /// Canonical expected digest.
        expected: String,
        /// Canonical observed digest.
        actual: String,
    },
    /// A digest is not exactly 64 hexadecimal digits.
    #[error("invalid SHA-256 checksum: {0}")]
    InvalidHash(String),
    /// Only the maintained SHA-256 algorithm is accepted.
    #[error("unsupported checksum algorithm {0}")]
    UnsupportedAlgorithm(String),
}

#[cfg(test)]
mod tests;
