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

/// Publish a finite private JSON fixture; this is test setup, not product authority.
#[allow(
    dead_code,
    reason = "this shared module also compiles in integration binaries without JSON fixtures"
)]
pub(crate) fn create_json_durable(
    path: &std::path::Path,
    value: &impl serde::Serialize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ic_host_artifacts::artifact::BoundedWriter::new(Vec::new(), 16 * 1024 * 1024);
    serde_json::to_writer_pretty(&mut writer, value)?;
    let bytes = writer.into_inner();
    ic_host_fs::durable::write_bytes(
        path,
        ic_host_fs::durable::WriteOptions {
            mode: ic_host_fs::durable::PublicationMode::CreateNew,
            permissions: 0o600,
        },
        &bytes,
    )?;
    Ok(())
}

/// Decode fixture evidence with Host's bounded, regular no-follow reader.
#[allow(
    dead_code,
    reason = "shared support also compiles in binaries without JSON fixtures"
)]
pub(crate) fn read_json<T: serde::de::DeserializeOwned>(
    path: &std::path::Path,
    max_bytes: u64,
) -> Result<T, Box<dyn std::error::Error>> {
    let bytes = ic_host_fs::read::read_file_no_follow(path, usize::try_from(max_bytes)?)?;
    Ok(serde_json::from_slice(&bytes)?)
}
