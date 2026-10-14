//! Pure bounded principal text admission shared by owning record boundaries.

pub(super) fn canonical_text(value: &str) -> Option<String> {
    if value.len() > 63 {
        return None;
    }
    ic_principal::Principal::from_text(value)
        .ok()
        .map(|principal| principal.to_text())
}

pub(super) fn canonical_set<E>(
    mut principals: Vec<String>,
    limit: usize,
    too_many: fn() -> E,
    invalid: fn() -> E,
    duplicate: fn() -> E,
) -> Result<Vec<String>, E> {
    if principals.len() > limit {
        return Err(too_many());
    }
    for principal in &mut principals {
        *principal = canonical_text(principal).ok_or_else(invalid)?;
    }
    principals.sort();
    if principals.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(duplicate());
    }
    Ok(principals)
}
