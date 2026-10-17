//! Canonical journal-parent resolution with caller-owned invalid-path errors.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

// Resolve the existing parent, allowing an absent journal but never a leaf alias.
pub(super) fn resolve_journal<E: From<io::Error>>(
    path: &Path,
    missing_name: impl FnOnce() -> E,
    not_regular: impl FnOnce(PathBuf) -> E,
) -> Result<PathBuf, E> {
    let name = path.file_name().ok_or_else(missing_name)?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let parent = parent.canonicalize()?;
    if !parent.is_dir() {
        return Err(io::Error::from(io::ErrorKind::NotADirectory).into());
    }
    let journal = parent.join(name);
    match fs::symlink_metadata(&journal) {
        Ok(metadata) if !metadata.is_file() => {
            return Err(not_regular(journal));
        }
        Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(error.into()),
        _ => {}
    }
    Ok(journal)
}
