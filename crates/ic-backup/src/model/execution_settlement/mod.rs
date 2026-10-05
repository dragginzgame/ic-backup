//! Immutable exact original journal fingerprints; not a backup/restore terminal receipt.

use crate::model::{
    artifacts::ArtifactChecksumRecord, attempt_journal::AttemptJournalRecord,
    effect_graph::MAX_EFFECT_OPERATIONS,
};
use serde::{Deserialize, Deserializer, Serialize, de};
use std::fmt;
use thiserror::Error;

/// Maximum raw and canonical checkpoint bytes, including all 8,192 possible journal rows.
pub const MAX_EXECUTION_SETTLEMENT_BYTES: u64 = 2 * 1024 * 1024;

/// One exact chronological journal fingerprint under its opaque original operation identity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionSettlementJournalRecord {
    operation_sequence: u64,
    history: ArtifactChecksumRecord,
}
impl ExecutionSettlementJournalRecord {
    /// Project original sequence and exact history, without granting receipt authenticity.
    #[must_use]
    pub fn from_journal(journal: &AttemptJournalRecord) -> Self {
        Self {
            operation_sequence: journal.authority().binding().operation_sequence(),
            history: journal.digest(),
        }
    }
    /// Read the original opaque operation identity.
    #[must_use]
    pub const fn operation_sequence(&self) -> u64 {
        self.operation_sequence
    }
    /// Read full original authority/reservation/receipt history identity.
    #[must_use]
    pub const fn history(&self) -> &ArtifactChecksumRecord {
        &self.history
    }
}

/// Strict v1 immutable exact journal checkpoint, with no copied counters or completion flags.
///
/// Structural decoding authenticates no receipts. Current admission requires the full
/// exact original journal set, causal Applied prerequisites and every operation Applied.
/// This is local ledger settlement, not full backup/restore completion, application
/// safety, command quiescence, fresh authority or fence/reference-release permission.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "SettlementFields")]
pub struct ExecutionSettlementRecord {
    version: u16,
    plan_intent: ArtifactChecksumRecord,
    journals: Vec<ExecutionSettlementJournalRecord>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SettlementFields {
    version: u16,
    plan_intent: ArtifactChecksumRecord,
    #[serde(deserialize_with = "bounded_journals")]
    journals: Vec<ExecutionSettlementJournalRecord>,
}
impl TryFrom<SettlementFields> for ExecutionSettlementRecord {
    type Error = ExecutionSettlementError;
    fn try_from(fields: SettlementFields) -> Result<Self, Self::Error> {
        if fields.version != 1 {
            return Err(ExecutionSettlementError::UnsupportedVersion(fields.version));
        }
        Self::new(fields.plan_intent, fields.journals)
    }
}
impl ExecutionSettlementRecord {
    /// Declare bounded unique journal fingerprints in canonical sequence order.
    ///
    /// Declaration alone proves no journal presence or Applied outcome. Policy and
    /// persistence independently admit original retained journals before publication/replay.
    /// # Errors
    /// Rejects empty/excessive rows or repeated operation identity, including different hashes.
    pub fn new(
        plan_intent: ArtifactChecksumRecord,
        mut journals: Vec<ExecutionSettlementJournalRecord>,
    ) -> Result<Self, ExecutionSettlementError> {
        if journals.is_empty() || journals.len() > MAX_EFFECT_OPERATIONS {
            return Err(ExecutionSettlementError::InvalidJournalCount);
        }
        journals.sort_by_key(ExecutionSettlementJournalRecord::operation_sequence);
        if journals
            .windows(2)
            .any(|rows| rows[0].operation_sequence == rows[1].operation_sequence)
        {
            return Err(ExecutionSettlementError::DuplicateOperation);
        }
        Ok(Self {
            version: 1,
            plan_intent,
            journals,
        })
    }
    /// Read original full plan identity, including all original allowances.
    #[must_use]
    pub const fn plan_intent(&self) -> &ArtifactChecksumRecord {
        &self.plan_intent
    }
    /// Read exact canonical fingerprint set; no row is a dispatch/completion receipt.
    #[must_use]
    pub fn journals(&self) -> &[ExecutionSettlementJournalRecord] {
        &self.journals
    }
    /// Hash NUL-terminated v1 domain, 64 ASCII plan hash, u64 row count and
    /// ascending u64 operation identity/64 ASCII history hash pairs, all integers big-endian.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        let mut bytes = b"ic-backup/execution-settlement/v1\0".to_vec();
        bytes.extend_from_slice(self.plan_intent.hash().as_bytes());
        bytes.extend_from_slice(&(self.journals.len() as u64).to_be_bytes());
        for row in &self.journals {
            bytes.extend_from_slice(&row.operation_sequence.to_be_bytes());
            bytes.extend_from_slice(row.history.hash().as_bytes());
        }
        ArtifactChecksumRecord::from_bytes(&bytes)
    }
}
fn bounded_journals<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<ExecutionSettlementJournalRecord>, D::Error> {
    struct Rows;
    impl<'de> de::Visitor<'de> for Rows {
        type Value = Vec<ExecutionSettlementJournalRecord>;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                formatter,
                "at most {MAX_EFFECT_OPERATIONS} original journal fingerprints"
            )
        }
        fn visit_seq<A: de::SeqAccess<'de>>(
            self,
            mut sequence: A,
        ) -> Result<Self::Value, A::Error> {
            let mut rows = Vec::new();
            while let Some(row) = sequence.next_element()? {
                if rows.len() == MAX_EFFECT_OPERATIONS {
                    return Err(de::Error::custom("too many journal fingerprints"));
                }
                rows.push(row);
            }
            Ok(rows)
        }
    }
    deserializer.deserialize_seq(Rows)
}
/// Typed strict declaration rejection; original journals remain the only spending owners.
#[derive(Debug, Eq, Error, PartialEq)]
pub enum ExecutionSettlementError {
    /// Historical or unknown generation cannot become a current checkpoint.
    #[error("unsupported execution settlement version {0}")]
    UnsupportedVersion(u16),
    /// A checkpoint requires one through 8,192 unique original rows.
    #[error("invalid execution settlement journal count")]
    InvalidJournalCount,
    /// Same operation was declared more than once.
    #[error("duplicate execution settlement operation")]
    DuplicateOperation,
}
#[cfg(test)]
mod tests;
