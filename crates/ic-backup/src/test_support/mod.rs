//! Local filesystem regression helpers, independent of IC executors.

pub mod consistency;
pub mod control_authority;
pub mod execution_settlement;
pub mod fence_acquisition;
pub mod fence_reconciliation;
pub mod ic_mutation;
pub mod ic_observation;
pub mod ic_snapshot_upload;
pub mod ic_snapshot_upload_data_observation;
pub mod local_restore_source;
pub mod membership;
pub mod ready;
pub mod restore_safety;
pub mod snapshot_read;

#[cfg(unix)]
use std::{
    fs,
    os::unix::fs::{FileTypeExt, PermissionsExt},
    path::Path,
    process::{Child, Command},
    time::Duration,
};

use std::path::PathBuf;

#[cfg(unix)]
const CRASH_BARRIER_TIMEOUT: Duration = Duration::from_secs(30);

// Allocate one private test root without reclaiming an existing entry.
pub fn temp_dir(prefix: &str) -> PathBuf {
    let selected_temp = std::env::temp_dir()
        .canonicalize()
        .expect("resolve selected fixture parent");
    ic_host_fs::testing::create_fixture_directory(&selected_temp, prefix)
        .expect("allocate fixture directory")
}

/// Create and verify a private real FIFO using the host's portable Unix utility.
#[cfg(unix)]
pub fn create_private_fifo(path: &Path) {
    assert!(
        Command::new("mkfifo")
            .args(["-m", "600"])
            .arg(path)
            .status()
            .expect("host mkfifo utility")
            .success(),
        "create private fixture FIFO"
    );
    let metadata = fs::symlink_metadata(path).expect("fixture FIFO metadata");
    assert!(metadata.file_type().is_fifo());
    assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
}

/// Kill one test child only after both sides acknowledge the named crash barrier.
#[cfg(unix)]
pub fn kill_child_at_acknowledged_barrier(child: &mut Child, root: &Path) {
    ic_host_process::testing::crash_barrier::kill_child_at_acknowledged_barrier(
        child,
        root,
        CRASH_BARRIER_TIMEOUT,
    )
    .expect("kill and reap child at acknowledged barrier");
}

/// Signal that a test child reached its barrier, then wait to be killed.
#[cfg(unix)]
pub fn hold_at_acknowledged_barrier(root: &Path) -> ! {
    ic_host_process::testing::crash_barrier::hold_at_acknowledged_barrier(
        root,
        CRASH_BARRIER_TIMEOUT,
    )
}

#[cfg(unix)]
pub fn wait_for_child_path(child: &mut Child, path: &Path, description: &str) {
    ic_host_process::testing::crash_barrier::wait_for_child_path(
        child,
        path,
        description,
        CRASH_BARRIER_TIMEOUT,
    )
    .expect("wait for direct child marker");
}

#[cfg(unix)]
pub fn wait_for_path(path: &Path, description: &str) {
    ic_host_process::testing::crash_barrier::wait_for_path(
        path,
        description,
        CRASH_BARRIER_TIMEOUT,
    )
    .expect("wait for parent marker");
}

/// Encode a bounded test fixture before publishing through the production owner.
pub(crate) fn create_json_durable(
    path: &std::path::Path,
    value: &impl serde::Serialize,
) -> Result<(), crate::ops::persistence::PersistenceError> {
    let bytes = crate::ops::persistence::json::encode_json(value, 16 * 1024 * 1024)?;
    crate::ops::persistence::json::create_encoded(path, &bytes)
}

/// Replace a bounded test fixture through the production publication owner.
pub(crate) fn write_json_durable(
    path: &std::path::Path,
    value: &impl serde::Serialize,
) -> Result<(), crate::ops::persistence::PersistenceError> {
    let bytes = crate::ops::persistence::json::encode_json(value, 16 * 1024 * 1024)?;
    crate::ops::persistence::json::write_encoded(path, &bytes)
}

/// Broad finite admission for trusted fixture oracles, independent of product limits.
pub(crate) const WIRE_ORACLE_LIMITS: ic_platform::candid::DecodeLimits =
    ic_platform::candid::DecodeLimits {
        reply_bytes: 64 * 1024 * 1024,
        decoding_quota: 512 * 1024 * 1024,
        skipping_quota: 64 * 1024 * 1024,
        type_entries: 4096,
        header_bytes: 64 * 1024,
    };
