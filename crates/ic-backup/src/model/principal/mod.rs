//! Pure bounded principal text admission shared by owning record boundaries.

pub(super) fn canonical_text(value: &str) -> Option<String> {
    if value.len() > 63 {
        return None;
    }
    ic_principal::Principal::from_text(value)
        .ok()
        .map(|principal| principal.to_text())
}
