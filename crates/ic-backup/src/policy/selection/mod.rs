//! Explicit physical selection and bounded parent-edge expansion; no live discovery.

use crate::model::{
    artifacts::ArtifactChecksumRecord,
    inventory::{
        InventoryRecord, InventoryRecordError, InventoryTargetRecord, MAX_INVENTORY_TARGETS,
    },
};
use std::collections::{BTreeSet, VecDeque};
use thiserror::Error;

/// Exact expansion selected by the caller; no implicit role or root special case.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionExpansion {
    /// Include exactly the explicitly named physical targets.
    Exact,
    /// Include named targets and their direct children only.
    DirectChildren,
    /// Include named targets and every declared descendant.
    Descendants,
}

/// Passive bounded selection request, without a non-neutral default.
#[derive(Clone, Debug)]
pub struct SelectionRequest {
    /// Nonempty exact principal list; equivalent duplicates reject.
    pub canister_ids: Vec<String>,
    /// Explicit requested expansion over declared parent edges.
    pub expansion: SelectionExpansion,
}

/// Read-only projection bound to the entire declared inventory, not fresh membership.
#[derive(Clone, Debug)]
pub struct SelectionView<'a> {
    /// Digest of the original full declared inventory, including unselected targets.
    pub inventory: ArtifactChecksumRecord,
    /// Unique targets in canonical principal order, not dispatch order.
    ///
    /// Parent links retain original meaning even if a parent is outside this selection.
    pub targets: Vec<&'a InventoryTargetRecord>,
}

/// Resolve exact identities and expand only the requested validated declared edges.
///
/// # Errors
/// Rejects empty/excessive requests, malformed/absent identities and duplicate selectors.
pub fn select<'a>(
    inventory: &'a InventoryRecord,
    request: &SelectionRequest,
) -> Result<SelectionView<'a>, SelectionError> {
    if request.canister_ids.is_empty() {
        return Err(SelectionError::EmptySelection);
    }
    if request.canister_ids.len() > MAX_INVENTORY_TARGETS {
        return Err(SelectionError::TooManySelectors);
    }
    let mut selected = BTreeSet::new();
    let mut queue = VecDeque::new();
    for id in &request.canister_ids {
        let target = inventory.target(id)?;
        if !selected.insert(target.canister_id()) {
            return Err(SelectionError::DuplicateSelector(
                target.canister_id().into(),
            ));
        }
        queue.push_back(target.canister_id());
    }
    if request.expansion != SelectionExpansion::Exact {
        while let Some(parent) = queue.pop_front() {
            for child in inventory
                .targets()
                .iter()
                .filter(|target| target.parent_canister_id() == Some(parent))
            {
                if selected.insert(child.canister_id())
                    && request.expansion == SelectionExpansion::Descendants
                {
                    queue.push_back(child.canister_id());
                }
            }
        }
    }
    Ok(SelectionView {
        inventory: inventory.digest(),
        targets: inventory
            .targets()
            .iter()
            .filter(|target| selected.contains(target.canister_id()))
            .collect(),
    })
}

/// Typed declared selection rejection, before any effects.
#[derive(Debug, Error)]
pub enum SelectionError {
    /// At least one exact physical selector is required.
    #[error("selection contains no targets")]
    EmptySelection,
    /// Selector count exceeds the inventory bound.
    #[error("selection exceeds {MAX_INVENTORY_TARGETS} selectors")]
    TooManySelectors,
    /// Equivalent principal was explicitly selected more than once.
    #[error("duplicate physical selector {0}")]
    DuplicateSelector(String),
    /// Owning inventory boundary rejected an identity.
    #[error(transparent)]
    Inventory(#[from] InventoryRecordError),
}

#[cfg(test)]
mod tests;
