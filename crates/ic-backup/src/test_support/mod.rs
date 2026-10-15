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
    thread,
    time::{Duration, Instant},
};

use std::path::PathBuf;

#[cfg(unix)]
const CRASH_BARRIER_TIMEOUT: Duration = Duration::from_secs(30);

// Allocate one private test root without reclaiming an existing entry.
pub fn temp_dir(prefix: &str) -> PathBuf {
    ic_host_fs::testing::create_fixture_directory(&std::env::temp_dir(), prefix)
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
    let ready_path = root.join("barrier-ready");
    let acknowledge_path = root.join("barrier-acknowledged");
    let armed_path = root.join("barrier-armed");
    wait_for_child_path(child, &ready_path, "child barrier");
    fs::write(&acknowledge_path, b"acknowledged\n").expect("acknowledge child barrier");
    wait_for_child_path(child, &armed_path, "armed child barrier");
    child.kill().expect("kill child at acknowledged barrier");
    child.wait().expect("reap killed child");
}

/// Signal that a test child reached its barrier, then wait to be killed.
#[cfg(unix)]
pub fn hold_at_acknowledged_barrier(root: &Path) -> ! {
    let ready_path = root.join("barrier-ready");
    let acknowledge_path = root.join("barrier-acknowledged");
    let armed_path = root.join("barrier-armed");
    fs::write(&ready_path, b"ready\n").expect("signal child barrier");
    wait_for_path(&acknowledge_path, "parent barrier acknowledgement");
    fs::write(&armed_path, b"armed\n").expect("arm child crash");
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

#[cfg(unix)]
pub fn wait_for_child_path(child: &mut Child, path: &Path, description: &str) {
    let deadline = Instant::now() + CRASH_BARRIER_TIMEOUT;
    while !path.is_file() {
        assert!(
            child.try_wait().expect("inspect crash child").is_none(),
            "crash child exited before {description}"
        );
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {description}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(unix)]
pub fn wait_for_path(path: &Path, description: &str) {
    let deadline = Instant::now() + CRASH_BARRIER_TIMEOUT;
    while !path.is_file() {
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {description}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}
