//! Ordered passive identity checks shared by original-operation settlements.

use crate::model::artifacts::ArtifactChecksumRecord;

pub(super) struct SettlementIdentity<'a> {
    pub authority: &'a ArtifactChecksumRecord,
    pub mutation_attempt: u32,
    pub observation_attempt: u32,
    pub challenge: &'a ArtifactChecksumRecord,
}

impl SettlementIdentity<'_> {
    pub fn compare(&self, expected: &Self) -> Result<(), SettlementIdentityMismatch> {
        if self.authority != expected.authority {
            return Err(SettlementIdentityMismatch::Authority);
        }
        if self.mutation_attempt != expected.mutation_attempt
            || self.observation_attempt != expected.observation_attempt
        {
            return Err(SettlementIdentityMismatch::Attempt);
        }
        if self.challenge != expected.challenge {
            return Err(SettlementIdentityMismatch::Challenge);
        }
        Ok(())
    }
}

/// Passive settlement claims differ from their independently qualified original evidence.
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum SettlementIdentityMismatch {
    /// Full original operation authority differs.
    #[error("settlement authority differs")]
    Authority,
    /// Original mutation or observation attempt differs.
    #[error("settlement attempts differ")]
    Attempt,
    /// Caller-owned qualification challenge differs.
    #[error("settlement challenge differs")]
    Challenge,
}
