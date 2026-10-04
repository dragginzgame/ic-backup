//! Canonical SHA-256 artifact metadata; filesystem effects belong to ops.

use crate::hash::{hex_bytes, sha256_hex};
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
            hash: sha256_hex(bytes),
        }
    }

    /// Validate and normalize an existing digest.
    ///
    /// # Errors
    /// Returns [`ChecksumError::InvalidHash`] unless the input has 64 hex digits.
    pub fn from_hash(hash: &str) -> Result<Self, ChecksumError> {
        validate_hash(hash)?;
        Ok(Self {
            algorithm: "sha256".to_owned(),
            hash: hash.to_ascii_lowercase(),
        })
    }

    pub(crate) fn from_digest(digest: [u8; 32]) -> Self {
        Self {
            algorithm: "sha256".to_owned(),
            hash: hex_bytes(digest),
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
        validate_hash(expected_hash)?;
        if self.hash.eq_ignore_ascii_case(expected_hash) {
            Ok(())
        } else {
            Err(ChecksumError::ChecksumMismatch {
                expected: expected_hash.to_ascii_lowercase(),
                actual: self.hash.clone(),
            })
        }
    }
}

fn validate_hash(hash: &str) -> Result<(), ChecksumError> {
    if hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ChecksumError::InvalidHash(hash.to_owned()));
    }
    Ok(())
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
