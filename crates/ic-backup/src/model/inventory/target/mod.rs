//! Passive physical target admission; role and module claims are integration declarations.

use super::InventoryRecordError;
use crate::model::artifacts::ArtifactChecksumRecord;
use serde::{Deserialize, Deserializer, Serialize};

/// Maximum optional role size in UTF-8 bytes, without role-based authority semantics.
pub const MAX_INVENTORY_ROLE_BYTES: usize = 256;

/// Passive physical inventory row supplied by an integration or explicit operator selection.
#[derive(Clone, Debug)]
pub struct InventoryTargetRequest {
    /// Exact physical principal text.
    pub canister_id: String,
    /// Exact parent inside the declared forest, or no declared parent.
    pub parent_canister_id: Option<String>,
    /// Opaque optional application label; never a selector or authority substitute.
    pub role: Option<String>,
    /// Optional declared exact module SHA-256 digest, not a fresh observation.
    pub module_hash: Option<String>,
}

/// Canonical immutable declared physical target with required nullable fields.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "TargetFields")]
pub struct InventoryTargetRecord {
    canister_id: String,
    parent_canister_id: Option<String>,
    role: Option<String>,
    module_hash: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TargetFields {
    canister_id: String,
    #[serde(deserialize_with = "required_text")]
    parent_canister_id: Option<String>,
    #[serde(deserialize_with = "required_text")]
    role: Option<String>,
    #[serde(deserialize_with = "required_text")]
    module_hash: Option<String>,
}
fn required_text<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<String>, D::Error> {
    Option::deserialize(deserializer)
}
impl TryFrom<TargetFields> for InventoryTargetRecord {
    type Error = InventoryRecordError;
    fn try_from(fields: TargetFields) -> Result<Self, Self::Error> {
        Self::new(&InventoryTargetRequest {
            canister_id: fields.canister_id,
            parent_canister_id: fields.parent_canister_id,
            role: fields.role,
            module_hash: fields.module_hash,
        })
    }
}
impl InventoryTargetRecord {
    /// Normalize equivalent principals/hashes and admit bounded opaque role text.
    ///
    /// # Errors
    /// Rejects malformed principals/hashes and excessive role bytes.
    pub fn new(request: &InventoryTargetRequest) -> Result<Self, InventoryRecordError> {
        if request
            .role
            .as_ref()
            .is_some_and(|role| role.len() > MAX_INVENTORY_ROLE_BYTES)
        {
            return Err(InventoryRecordError::RoleTooLarge);
        }
        Ok(Self {
            canister_id: crate::model::principal::canonical_text(&request.canister_id)
                .ok_or(InventoryRecordError::InvalidPrincipal("canister_id"))?,
            parent_canister_id: request
                .parent_canister_id
                .as_deref()
                .map(|value| {
                    crate::model::principal::canonical_text(value)
                        .ok_or(InventoryRecordError::InvalidPrincipal("parent_canister_id"))
                })
                .transpose()?,
            role: request.role.clone(),
            module_hash: request
                .module_hash
                .as_deref()
                .map(|value| {
                    ArtifactChecksumRecord::from_hash(value).map(|hash| hash.hash().to_owned())
                })
                .transpose()?,
        })
    }
    /// Read the exact canonical physical principal.
    #[must_use]
    pub fn canister_id(&self) -> &str {
        &self.canister_id
    }
    /// Read the optional canonical declared parent.
    #[must_use]
    pub fn parent_canister_id(&self) -> Option<&str> {
        self.parent_canister_id.as_deref()
    }
    /// Read the opaque optional role label.
    #[must_use]
    pub fn role(&self) -> Option<&str> {
        self.role.as_deref()
    }
    /// Read the optional canonical declared module digest.
    #[must_use]
    pub fn module_hash(&self) -> Option<&str> {
        self.module_hash.as_deref()
    }
}
