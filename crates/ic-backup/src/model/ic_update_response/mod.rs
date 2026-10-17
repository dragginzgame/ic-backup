//! Shared passive IC update-response fields, bounded admission and redacted formatting.

use crate::model::{
    artifacts::ArtifactChecksumRecord, attempt_journal::MAX_OPERATION_ATTEMPTS,
    operation_plan::PlanContextRecord,
};
use std::fmt;

/// Passive provider fields; actual authentication and association are integration-owned.
#[derive(Clone)]
pub struct IcUpdateResponseInput {
    /// Full original authority digest, including plan/context/operation/limits.
    pub authority: ArtifactChecksumRecord,
    /// Original already reserved mutation attempt, never a new call number.
    pub mutation_attempt: u32,
    /// Actual authenticated network/caller/release claimed by the provider.
    pub context: PlanContextRecord,
    /// Actual response routing target claimed by the provider; canonicalized on admission.
    pub target: String,
    /// Exact raw Candid reply, retained under the existing codec byte ceilings.
    pub reply: Vec<u8>,
    /// Opaque provider evidence binding original request/attempt and all actual fields.
    pub evidence: ArtifactChecksumRecord,
}

impl IcUpdateResponseInput {
    pub(crate) fn admit<E>(
        &mut self,
        max_reply_bytes: usize,
        invalid_attempt: E,
        reply_too_large: E,
        invalid_target: E,
    ) -> Result<(), E> {
        if self.mutation_attempt == 0 || self.mutation_attempt > MAX_OPERATION_ATTEMPTS {
            return Err(invalid_attempt);
        }
        if self.reply.len() > max_reply_bytes {
            return Err(reply_too_large);
        }
        self.target =
            crate::model::principal::canonical_text(&self.target).ok_or(invalid_target)?;
        Ok(())
    }

    pub(crate) fn fmt_redacted(
        &self,
        formatter: &mut fmt::Formatter<'_>,
        name: &str,
    ) -> fmt::Result {
        formatter
            .debug_struct(name)
            .field("authority", &self.authority)
            .field("mutation_attempt", &self.mutation_attempt)
            .field("target", &self.target)
            .field("reply_bytes", &self.reply.len())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests;
