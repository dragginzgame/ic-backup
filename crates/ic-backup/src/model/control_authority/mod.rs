//! Canonical bounded controller identities used by IC status decoding.

use crate::model::attempt_journal::OperationBindingRecord;
use thiserror::Error;

/// Maximum controllers in the maintained IC controller-set boundary.
pub const MAX_CONTROLLERS: usize = 10;

/// Canonical bounded known controller set, including an explicitly known empty set.
///
/// This validates identities, not observation authenticity. Missing/unknown controller
/// evidence must fail at the provider boundary, never default into a success result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ControllerSet {
    principals: Vec<String>,
}
impl ControllerSet {
    /// Normalize and sort exact principals; equivalent duplicate entries reject.
    /// # Errors
    /// Rejects excessive counts, malformed principals and canonical duplicates.
    pub fn new(principals: Vec<String>) -> Result<Self, ControlObservationError> {
        let principals = super::principal::canonical_set(
            principals,
            MAX_CONTROLLERS,
            || ControlObservationError::TooManyControllers,
            || ControlObservationError::InvalidPrincipal,
            || ControlObservationError::DuplicateController,
        )?;
        Ok(Self { principals })
    }
    /// Read canonical controllers; ordering grants no routing or lifecycle semantics.
    #[must_use]
    pub fn principals(&self) -> &[String] {
        &self.principals
    }
    /// Check the exact canonical original caller, without delegating through other controllers.
    #[must_use]
    pub fn contains_caller(&self, binding: &OperationBindingRecord) -> bool {
        self.principals
            .binary_search_by(|principal| principal.as_str().cmp(binding.caller()))
            .is_ok()
    }
}

/// Invalid bounded controller identities; status decoding retains this public error.
#[derive(Debug, Eq, Error, PartialEq)]
pub enum ControlObservationError {
    /// Controller principal failed bounded canonical admission.
    #[error("invalid control observation principal")]
    InvalidPrincipal,
    /// Full set exceeds the maintained bound.
    #[error("controller set exceeds {MAX_CONTROLLERS}")]
    TooManyControllers,
    /// Equivalent principal appears twice.
    #[error("duplicate controller principal")]
    DuplicateController,
}

#[cfg(test)]
mod tests;
