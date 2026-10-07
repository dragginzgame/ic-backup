//! Exclusive integration fixture roots; failed journeys retain their evidence.

use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

/// Reserve a fresh root before a public integration journey writes any children.
pub(crate) fn temp_root(prefix: &str) -> PathBuf {
    static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);
    let parent = std::env::temp_dir()
        .canonicalize()
        .expect("resolve integration fixture parent");
    let prefix = format!("{prefix}-{}", std::process::id());
    reserve_root(&parent, &prefix, &NEXT_ROOT)
        .unwrap_or_else(|error| panic!("reserve fixture beneath {}: {error}", parent.display()))
}

/// Skip occupied names without replacing or removing their retained evidence.
pub(crate) fn reserve_root(
    parent: &Path,
    prefix: &str,
    sequence: &AtomicU64,
) -> io::Result<PathBuf> {
    for _ in 0..128 {
        let root = parent.join(format!(
            "{prefix}-{}",
            sequence.fetch_add(1, Ordering::Relaxed)
        ));
        match fs::create_dir(&root) {
            Ok(()) => {
                eprintln!(
                    "Integration fixture retained until success: {}",
                    root.display()
                );
                return Ok(root);
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(io::Error::new(
                    error.kind(),
                    format!("{}: {error}", root.display()),
                ));
            }
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        format!(
            "128 occupied fixture names beneath {} for {prefix}",
            parent.display()
        ),
    ))
}
