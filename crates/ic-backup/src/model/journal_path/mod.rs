//! Pure admission for exact normalized absolute UTF-8 journal locations.

use std::path::{Component, Path, PathBuf};

pub(super) fn is_canonical(path: &Path, maximum_bytes: usize) -> bool {
    let normalized: PathBuf = path.components().collect();
    path.to_str()
        .is_some_and(|value| value.len() <= maximum_bytes && !value.contains('\0'))
        && path.is_absolute()
        && path.file_name().is_some()
        && normalized.as_os_str() == path.as_os_str()
        && path
            .components()
            .all(|part| matches!(part, Component::RootDir | Component::Normal(_)))
}
