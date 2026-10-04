//! Immutable operation identity and finite attempt allowance; no fresh authority.

use super::AttemptJournalRecordError;
use crate::model::artifacts::ArtifactChecksumRecord;
use serde::{Deserialize, Serialize};

/// Maximum total mutation and reconciliation-observation attempts per journal.
pub const MAX_OPERATION_ATTEMPTS: u32 = 1024;

/// Passive operation identity selected by the integration before effects.
#[derive(Clone, Debug)]
pub struct OperationBindingRequest {
    /// Canonical immutable plan/intent digest supplied by its owner.
    pub intent: String,
    /// Exact operation sequence within that intent.
    pub operation_sequence: u64,
    /// Qualified network identity fingerprint; an endpoint label is insufficient.
    pub network: String,
    /// Exact selected caller principal; credentials are never retained here.
    pub caller: String,
    /// Exact physical canister principal selected for this operation.
    pub target: String,
    /// Opaque exact release evidence digest supplied by the integration.
    pub release: String,
    /// Canonical exact mutating request digest supplied by its owning codec.
    pub request: String,
}

/// Canonical immutable declared operation identity, not observed live authority.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "BindingFields")]
pub struct OperationBindingRecord {
    intent: String,
    operation_sequence: u64,
    network: String,
    caller: String,
    target: String,
    release: String,
    request: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BindingFields {
    intent: String,
    operation_sequence: u64,
    network: String,
    caller: String,
    target: String,
    release: String,
    request: String,
}

impl TryFrom<BindingFields> for OperationBindingRecord {
    type Error = AttemptJournalRecordError;
    fn try_from(fields: BindingFields) -> Result<Self, Self::Error> {
        Self::new(&OperationBindingRequest {
            intent: fields.intent,
            operation_sequence: fields.operation_sequence,
            network: fields.network,
            caller: fields.caller,
            target: fields.target,
            release: fields.release,
            request: fields.request,
        })
    }
}

impl OperationBindingRecord {
    /// Canonicalize exact declared identities without IO or authorizing dispatch.
    ///
    /// # Errors
    /// Rejects malformed principal text or SHA-256 digest fields.
    pub fn new(request: &OperationBindingRequest) -> Result<Self, AttemptJournalRecordError> {
        Ok(Self {
            intent: super::canonical_hash(&request.intent)?,
            operation_sequence: request.operation_sequence,
            network: super::canonical_hash(&request.network)?,
            caller: crate::model::principal::canonical_text(&request.caller)
                .ok_or(AttemptJournalRecordError::InvalidPrincipal)?,
            target: crate::model::principal::canonical_text(&request.target)
                .ok_or(AttemptJournalRecordError::InvalidPrincipal)?,
            release: super::canonical_hash(&request.release)?,
            request: super::canonical_hash(&request.request)?,
        })
    }
    /// Read immutable intent identity.
    #[must_use]
    pub fn intent(&self) -> &str {
        &self.intent
    }
    /// Read the exact operation sequence.
    #[must_use]
    pub const fn operation_sequence(&self) -> u64 {
        self.operation_sequence
    }
    /// Read the exact declared network fingerprint.
    #[must_use]
    pub fn network(&self) -> &str {
        &self.network
    }
    /// Read the canonical selected caller principal.
    #[must_use]
    pub fn caller(&self) -> &str {
        &self.caller
    }
    /// Read the canonical selected physical target principal.
    #[must_use]
    pub fn target(&self) -> &str {
        &self.target
    }
    /// Read the integration-owned release digest.
    #[must_use]
    pub fn release(&self) -> &str {
        &self.release
    }
    /// Read the exact canonical mutating request digest.
    #[must_use]
    pub fn request(&self) -> &str {
        &self.request
    }
}

/// Immutable finite allowances; consumption never refunds or replenishes these limits.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "BudgetFields")]
pub struct AttemptBudgetRecord {
    mutations: u32,
    observations: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BudgetFields {
    mutations: u32,
    observations: u32,
}

impl TryFrom<BudgetFields> for AttemptBudgetRecord {
    type Error = AttemptJournalRecordError;
    fn try_from(fields: BudgetFields) -> Result<Self, Self::Error> {
        Self::new(fields.mutations, fields.observations)
    }
}

impl AttemptBudgetRecord {
    /// Admit finite independent mutation and observation allowances, including zero.
    ///
    /// # Errors
    /// Rejects overflow or a total greater than [`MAX_OPERATION_ATTEMPTS`].
    pub fn new(mutations: u32, observations: u32) -> Result<Self, AttemptJournalRecordError> {
        if mutations
            .checked_add(observations)
            .is_none_or(|total| total > MAX_OPERATION_ATTEMPTS)
        {
            return Err(AttemptJournalRecordError::BudgetTooLarge);
        }
        Ok(Self {
            mutations,
            observations,
        })
    }
    /// Read the immutable mutation-attempt ceiling.
    #[must_use]
    pub const fn mutations(&self) -> u32 {
        self.mutations
    }
    /// Read the immutable reconciliation-observation ceiling.
    #[must_use]
    pub const fn observations(&self) -> u32 {
        self.observations
    }
}

/// Named exact declared operation and budget admitted by a journal owner.
///
/// This data alone is not fresh permission, settlement evidence or a dispatch permit.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AttemptAuthorityRecord {
    binding: OperationBindingRecord,
    budget: AttemptBudgetRecord,
}

impl AttemptAuthorityRecord {
    /// Bind already validated operation identity to immutable finite limits.
    #[must_use]
    pub const fn new(binding: OperationBindingRecord, budget: AttemptBudgetRecord) -> Self {
        Self { binding, budget }
    }
    /// Read canonical declared identity.
    #[must_use]
    pub const fn binding(&self) -> &OperationBindingRecord {
        &self.binding
    }
    /// Read immutable ceilings.
    #[must_use]
    pub const fn budget(&self) -> &AttemptBudgetRecord {
        &self.budget
    }
    /// Hash exact immutable authority using the maintained domain-separated encoding.
    ///
    /// Encoding is the ASCII domain (including its NUL), four fixed 64-byte
    /// lowercase digest strings in intent/network/release/request order, big-endian
    /// u64 operation sequence, u8-length-prefixed canonical ASCII caller/target
    /// text, then big-endian u32 mutation/observation limits. Mutable events are excluded.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        let mut bytes = b"ic-backup/attempt-authority/v1\0".to_vec();
        for digest in [
            &self.binding.intent,
            &self.binding.network,
            &self.binding.release,
            &self.binding.request,
        ] {
            bytes.extend_from_slice(digest.as_bytes());
        }
        bytes.extend_from_slice(&self.binding.operation_sequence.to_be_bytes());
        for principal in [&self.binding.caller, &self.binding.target] {
            // Admission bounds canonical ASCII text to at most 63 bytes.
            bytes.push(principal.len().to_le_bytes()[0]);
            bytes.extend_from_slice(principal.as_bytes());
        }
        bytes.extend_from_slice(&self.budget.mutations.to_be_bytes());
        bytes.extend_from_slice(&self.budget.observations.to_be_bytes());
        ArtifactChecksumRecord::from_bytes(&bytes)
    }
}
