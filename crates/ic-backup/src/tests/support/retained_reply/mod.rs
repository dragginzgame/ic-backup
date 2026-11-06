//! Exact native fixture byte retention; callers own suspension timing.

use std::{
    fs::File,
    io::{self, Write},
    path::Path,
};

// Preserve create/truncate policy and permissions; sync the containing directory.
// Callers retain the synced descriptor
// for the original qualification lifetime.
pub(crate) fn write_synced(path: &Path, bytes: &[u8]) -> io::Result<File> {
    let mut file = File::create(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    File::open(path.parent().expect("fixture reply has a parent"))?.sync_all()?;
    Ok(file)
}
