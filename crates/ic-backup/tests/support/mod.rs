//! Exclusive integration fixture roots; failed journeys retain their evidence.

use std::path::PathBuf;

/// Reserve a fresh root before a public integration journey writes any children.
pub(crate) fn temp_root(prefix: &str) -> PathBuf {
    let selected_temp = std::env::temp_dir()
        .canonicalize()
        .unwrap_or_else(|error| panic!("resolve integration fixture parent: {error}"));
    let root = ic_host_fs::testing::create_fixture_directory(&selected_temp, prefix)
        .unwrap_or_else(|error| panic!("reserve integration fixture {prefix}: {error}"));
    eprintln!(
        "Integration fixture retained until success: {}",
        root.display()
    );
    root
}
