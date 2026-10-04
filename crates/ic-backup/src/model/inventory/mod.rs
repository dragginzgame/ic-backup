//! Canonical bounded physical inventories; declarations do not establish live membership.

mod target;
pub use target::{InventoryTargetRecord, InventoryTargetRequest, MAX_INVENTORY_ROLE_BYTES};

use crate::model::artifacts::{ArtifactChecksumRecord, ChecksumError};
use serde::{Deserialize, Deserializer, Serialize, de};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};
use thiserror::Error;

/// Maximum exact physical targets in one declared inventory.
pub const MAX_INVENTORY_TARGETS: usize = 1024;
/// Maximum encoded input and canonical output bytes admitted by inventory persistence.
pub const MAX_INVENTORY_BYTES: u64 = 1024 * 1024;

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
        let parents: BTreeMap<_, _> = targets
            .iter()
            .map(|target| (target.canister_id(), target.parent_canister_id()))
            .collect();
        for target in &targets {
            if let Some(parent) = target.parent_canister_id()
                && !parents.contains_key(parent)
            {
                return Err(InventoryRecordError::MissingParent {
                    canister_id: target.canister_id().into(),
                    parent: parent.into(),
                });
            }
        }
        for target in &targets {
            let mut current = Some(target.canister_id());
            let mut seen = BTreeSet::new();
            while let Some(id) = current {
                if !seen.insert(id) {
                    return Err(InventoryRecordError::Cycle(id.into()));
                }
                current = parents[id];
            }
        }
        Ok(Self {
            version: 1,
            targets,
        })
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
        self.targets
            .binary_search_by(|target| target.canister_id().cmp(&id))
            .map(|index| &self.targets[index])
            .map_err(|_| InventoryRecordError::UnknownTarget(id))
    }

    /// Hash exact canonical declared fields using the documented v1 binary encoding.
    ///
    /// Equivalent input order/case has one digest. Optional fields have explicit
    /// tags, so null, empty strings and the text "null" remain distinct. This hash
    /// proves declared equality only, without fresh membership or authority.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        let mut bytes = b"ic-backup/inventory/v1\0".to_vec();
        #[expect(
            clippy::cast_possible_truncation,
            reason = "validated target count is at most 1024"
        )]
        let count = self.targets.len() as u32;
        bytes.extend_from_slice(&count.to_be_bytes());
        for target in &self.targets {
            append_text(&mut bytes, target.canister_id());
            for field in [
                target.parent_canister_id(),
                target.role(),
                target.module_hash(),
            ] {
                match field {
                    None => bytes.push(0),
                    Some(value) => {
                        bytes.push(1);
                        append_text(&mut bytes, value);
                    }
                }
            }
        }
        ArtifactChecksumRecord::from_bytes(&bytes)
    }
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "all admitted text is bounded to at most 256 UTF-8 bytes"
)]
fn append_text(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u32).to_be_bytes());
    bytes.extend_from_slice(value.as_bytes());
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
    struct TargetsVisitor;
    impl<'de> de::Visitor<'de> for TargetsVisitor {
        type Value = Vec<InventoryTargetRecord>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a bounded physical target list")
        }
        fn visit_seq<A: de::SeqAccess<'de>>(
            self,
            mut sequence: A,
        ) -> Result<Self::Value, A::Error> {
            let mut targets = Vec::new();
            while targets.len() < MAX_INVENTORY_TARGETS {
                match sequence.next_element()? {
                    Some(target) => targets.push(target),
                    None => return Ok(targets),
                }
            }
            if sequence.next_element::<de::IgnoredAny>()?.is_some() {
                return Err(de::Error::custom(InventoryRecordError::TooManyTargets));
            }
            Ok(targets)
        }
    }
    deserializer.deserialize_seq(TargetsVisitor)
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
