//! Exact native fixture byte retention; callers own parent sync and suspension timing.

use std::{
    fs::File,
    io::{self, Write},
    path::Path,
};

// Preserve create/truncate policy and permissions; callers retain the synced descriptor
// for the original qualification lifetime.
pub(crate) fn write_synced(path: &Path, bytes: &[u8]) -> io::Result<File> {
    let mut file = File::create(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(file)
}
