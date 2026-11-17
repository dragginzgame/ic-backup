//! Backup commitment encoding; SHA-256 itself remains owned by `sha2`.
use sha2::{Digest, Sha256};
use std::{fmt, str::FromStr};

/// A Backup SHA-256 commitment, encoded as exactly 64 lowercase hexadecimal digits.
///
/// This value identifies bytes; it grants no provenance, custody or effect authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BackupDigest([u8; 32]);

impl BackupDigest {
    pub(super) fn check(self, expected: Self) -> Result<(), super::StoreError> {
        if self == expected {
            Ok(())
        } else {
            Err(super::StoreError::DigestMismatch {
                expected,
                actual: self,
            })
        }
    }
    /// Hash the complete supplied bytes without additional framing.
    #[must_use]
    pub fn compute(bytes: &[u8]) -> Self {
        Self(Sha256::digest(bytes).into())
    }

    /// Retain an already computed commitment.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Exact digest bytes, without hexadecimal encoding.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for BackupDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// A commitment was not exactly 64 lowercase hexadecimal digits.
#[derive(Debug, thiserror::Error)]
#[error("expected 64 lowercase hexadecimal SHA-256 digits")]
pub struct DigestParseError;

impl FromStr for BackupDigest {
    type Err = DigestParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if !super::manifest::lower_hex(value, 64) {
            return Err(DigestParseError);
        }
        let mut bytes = [0; 32];
        for (byte, pair) in bytes.iter_mut().zip(value.as_bytes().as_chunks::<2>().0) {
            let digit = |b: u8| if b <= b'9' { b - b'0' } else { b - b'a' + 10 };
            *byte = digit(pair[0]) * 16 + digit(pair[1]);
        }
        Ok(Self(bytes))
    }
}

#[cfg(test)]
mod tests;
