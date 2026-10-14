//! Shared canonical admission for passive, inventory-backed target evidence rows.

use super::{
    inventory::{InventoryRecord, MAX_INVENTORY_TARGETS},
    principal::canonical_text,
};

pub(super) trait ObservedTargetRow {
    fn target(&self) -> &str;
    fn target_mut(&mut self) -> &mut String;
}

pub(super) fn canonicalize<T: ObservedTargetRow, E>(
    mut rows: Vec<T>,
    inventory: &InventoryRecord,
    invalid_count: fn() -> E,
    invalid_principal: fn() -> E,
    absent: fn() -> E,
    duplicate: fn() -> E,
) -> Result<Vec<T>, E> {
    if rows.is_empty() || rows.len() > MAX_INVENTORY_TARGETS {
        return Err(invalid_count());
    }
    for row in &mut rows {
        let canonical = canonical_text(row.target()).ok_or_else(invalid_principal)?;
        *row.target_mut() = canonical;
        if inventory.target(row.target()).is_err() {
            return Err(absent());
        }
    }
    rows.sort_by(|left, right| left.target().cmp(right.target()));
    if rows
        .windows(2)
        .any(|pair| pair[0].target() == pair[1].target())
    {
        return Err(duplicate());
    }
    Ok(rows)
}
