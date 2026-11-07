//! Pure bounded principal text admission shared by owning record boundaries.

pub(super) fn canonical_parts(value: &str) -> Option<(String, ic_platform::Principal)> {
    if value.len() > 63 {
        return None;
    }
    ic_platform::Principal::from_text(value)
        .ok()
        .map(|principal| (principal.to_text(), principal))
}

pub(super) fn canonical_text(value: &str) -> Option<String> {
    canonical_parts(value).map(|(text, _)| text)
}
