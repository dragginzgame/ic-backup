//! Canonical bounded physical inventories; declarations do not establish live membership.

use crate::model::evidence_digest::EvidenceDigest;
mod target;
pub use target::{InventoryTargetRecord, InventoryTargetRequest, MAX_INVENTORY_ROLE_BYTES};

use crate::model::{
    artifacts::{ArtifactChecksumRecord, ChecksumError},
    bounded_sequence,
};
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

/// Maximum exact physical targets in one declared inventory.
pub const MAX_INVENTORY_TARGETS: usize = 1024;

/// Immutable v1 declared forest with canonical unique principals and closed parent links.
///
/// No root is assigned a privileged role. A parentless entry may be any selected
/// physical canister; multiple disconnected roots are allowed. Membership,
/// permissions, current module state and application consistency stay integration-owned.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "InventoryFields")]
pub struct InventoryRecord {
    version: u16,
    targets: Vec<InventoryTargetRecord>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InventoryFields {
    version: u16,
    #[serde(deserialize_with = "bounded_targets")]
    targets: Vec<InventoryTargetRecord>,
}

impl TryFrom<InventoryFields> for InventoryRecord {
    type Error = InventoryRecordError;
    fn try_from(fields: InventoryFields) -> Result<Self, Self::Error> {
        if fields.version != 1 {
            return Err(InventoryRecordError::UnsupportedVersion(fields.version));
        }
        Self::new(fields.targets)
    }
}

impl InventoryRecord {
    /// Validate and sort a nonempty exact declared forest without IO.
    ///
    /// # Errors
    /// Rejects excessive counts, duplicate identities, missing parents and cycles.
    pub fn new(mut targets: Vec<InventoryTargetRecord>) -> Result<Self, InventoryRecordError> {
        check_count(targets.len())?;
        targets.sort_by(|a, b| a.canister_id().cmp(b.canister_id()));
        for pair in targets.windows(2) {
            if pair[0].canister_id() == pair[1].canister_id() {
                return Err(InventoryRecordError::DuplicateTarget(
                    pair[0].canister_id().into(),
                ));
            }
        }
        let record = Self {
            version: 1,
            targets,
        };
        record.validate_forest()?;
        Ok(record)
    }

    fn validate_forest(&self) -> Result<(), InventoryRecordError> {
        // Admit every parent before walking any chain, preserving missing-parent
        // precedence. Indices avoid repeated principal lookup along shared ancestry.
        let parents = self
            .targets
            .iter()
            .map(|target| {
                target
                    .parent_canister_id()
                    .map(|parent| {
                        self.targets
                            .binary_search_by(|row| row.canister_id().cmp(parent))
                            .map_err(|_| InventoryRecordError::MissingParent {
                                canister_id: target.canister_id().into(),
                                parent: parent.into(),
                            })
                    })
                    .transpose()
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut visited = vec![None; self.targets.len()];
        for start in 0..self.targets.len() {
            let mut current = Some(start);
            while let Some(index) = current {
                if let Some(origin) = visited[index] {
                    // A mark from this walk is a cycle; an earlier walk already
                    // reached a root. Canonical starts preserve the reported ID.
                    if origin == start {
                        return Err(InventoryRecordError::Cycle(
                            self.targets[index].canister_id().into(),
                        ));
                    }
                    break;
                }
                visited[index] = Some(start);
                current = parents[index];
            }
        }
        Ok(())
    }

    /// Read targets in canonical ASCII principal order; this is not an effect order.
    #[must_use]
    pub fn targets(&self) -> &[InventoryTargetRecord] {
        &self.targets
    }

    /// Resolve an exact principal through canonical inventory identity admission.
    ///
    /// # Errors
    /// Rejects malformed principals or absent exact physical targets.
    pub fn target(
        &self,
        canister_id: &str,
    ) -> Result<&InventoryTargetRecord, InventoryRecordError> {
        let id = super::principal::canonical_text(canister_id)
            .ok_or(InventoryRecordError::InvalidPrincipal("canister_id"))?;
        self.canonical_target(&id)
            .ok_or(InventoryRecordError::UnknownTarget(id))
    }

    // Internal callers have already admitted canonical text at their owning boundary.
    pub(super) fn canonical_target(&self, id: &str) -> Option<&InventoryTargetRecord> {
        self.targets
            .binary_search_by(|target| target.canister_id().cmp(id))
            .ok()
            .map(|index| &self.targets[index])
    }

    /// Hash exact canonical declared fields using the documented v1 binary encoding.
    ///
    /// Equivalent input order/case has one digest. Optional fields have explicit
    /// tags, so null, empty strings and the text "null" remain distinct. This hash
    /// proves declared equality only, without fresh membership or authority.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        let mut bytes = EvidenceDigest::new(b"ic-backup/inventory/v1\0");
        #[expect(
            clippy::cast_possible_truncation,
            reason = "validated target count is at most 1024"
        )]
        let count = self.targets.len() as u32;
        bytes.u32_be(count);
        for target in &self.targets {
            bytes.bytes_u32(target.canister_id().as_bytes());
            for field in [
                target.parent_canister_id(),
                target.role(),
                target.module_hash(),
            ] {
                match field {
                    None => bytes.byte(0),
                    Some(value) => {
                        bytes.byte(1);
                        bytes.bytes_u32(value.as_bytes());
                    }
                }
            }
        }
        bytes.finish()
    }
}

pub(super) fn check_count(count: usize) -> Result<(), InventoryRecordError> {
    if count == 0 {
        return Err(InventoryRecordError::EmptyInventory);
    }
    if count > MAX_INVENTORY_TARGETS {
        return Err(InventoryRecordError::TooManyTargets);
    }
    Ok(())
}

fn bounded_targets<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<InventoryTargetRecord>, D::Error> {
    bounded_sequence::deserialize::<_, _, MAX_INVENTORY_TARGETS>(
        deserializer,
        "a bounded physical target list",
        InventoryRecordError::TooManyTargets,
    )
}

/// Typed declared identity, graph or resource-bound rejection.
#[derive(Debug, Error)]
pub enum InventoryRecordError {
    /// Only product generation v1 is maintained.
    #[error("unsupported inventory version {0}")]
    UnsupportedVersion(u16),
    /// Exact inventory cannot be empty.
    #[error("inventory contains no targets")]
    EmptyInventory,
    /// Target count exceeds the maintained bound.
    #[error("inventory exceeds {MAX_INVENTORY_TARGETS} targets")]
    TooManyTargets,
    /// Principal text fails canonical admission at this named field.
    #[error("invalid inventory principal in {0}")]
    InvalidPrincipal(&'static str),
    /// Role text exceeds its UTF-8 byte bound.
    #[error("inventory role exceeds {MAX_INVENTORY_ROLE_BYTES} bytes")]
    RoleTooLarge,
    /// Module digest fails canonical SHA-256 admission.
    #[error(transparent)]
    Checksum(#[from] ChecksumError),
    /// Equivalent physical target appeared more than once.
    #[error("duplicate inventory target {0}")]
    DuplicateTarget(String),
    /// Selected exact principal is absent.
    #[error("unknown inventory target {0}")]
    UnknownTarget(String),
    /// Parent relationship points outside the declared forest.
    #[error("inventory target {canister_id} has absent parent {parent}")]
    MissingParent {
        /// Exact child principal.
        canister_id: String,
        /// Exact missing parent principal.
        parent: String,
    },
    /// Parent edges form a cycle, including self-parenting.
    #[error("inventory cycle at {0}")]
    Cycle(String),
}

#[cfg(test)]
mod tests;
