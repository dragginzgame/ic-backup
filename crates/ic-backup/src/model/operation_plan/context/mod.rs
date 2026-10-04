//! Canonical declared network, caller and release context; no fresh permission.

use super::{OperationPlanError, canonical_hash};
use serde::{Deserialize, Serialize};

/// Passive integration-owned exact execution context declarations.
#[derive(Clone, Debug)]
pub struct PlanContextRequest {
    /// Qualified network fingerprint digest, not an endpoint label.
    pub network: String,
    /// Exact selected caller principal; credentials are excluded.
    pub caller: String,
    /// Exact opaque release evidence digest supplied by its owner.
    pub release: String,
}
/// Immutable canonical declared context shared by every operation in one plan.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "ContextFields")]
pub struct PlanContextRecord {
    network: String,
    caller: String,
    release: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ContextFields {
    network: String,
    caller: String,
    release: String,
}
impl TryFrom<ContextFields> for PlanContextRecord {
    type Error = OperationPlanError;
    fn try_from(fields: ContextFields) -> Result<Self, Self::Error> {
        Self::new(&PlanContextRequest {
            network: fields.network,
            caller: fields.caller,
            release: fields.release,
        })
    }
}
impl PlanContextRecord {
    /// Canonicalize exact declarations without IO or fresh authorization.
    ///
    /// # Errors
    /// Rejects malformed principal or SHA-256 digest fields.
    pub fn new(request: &PlanContextRequest) -> Result<Self, OperationPlanError> {
        Ok(Self {
            network: canonical_hash(&request.network)?,
            caller: crate::model::principal::canonical_text(&request.caller)
                .ok_or(OperationPlanError::InvalidPrincipal("caller"))?,
            release: canonical_hash(&request.release)?,
        })
    }
    /// Read the declared canonical network fingerprint.
    #[must_use]
    pub fn network(&self) -> &str {
        &self.network
    }
    /// Read the exact canonical selected caller.
    #[must_use]
    pub fn caller(&self) -> &str {
        &self.caller
    }
    /// Read the integration-owned release evidence digest.
    #[must_use]
    pub fn release(&self) -> &str {
        &self.release
    }
}
