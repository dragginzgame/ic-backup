//! Local filesystem regression helpers, independent of IC executors.

#[cfg(unix)]
use std::{
    fs,
    path::Path,
    process::Child,
    thread,
    time::{Duration, Instant},
};

use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

// Build a unique temporary directory path for tests that create their own layout.
pub fn temp_dir(prefix: &str) -> PathBuf {
    std::env::temp_dir()
        .canonicalize()
        .unwrap()
        .join(unique_name(prefix))
}

// Build a unique temporary file path for tests that only need one artifact.
pub fn temp_path(prefix: &str) -> PathBuf {
    temp_dir(prefix)
}

// Include process and timestamp data so parallel test runs do not collide.
fn unique_name(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after epoch")
        .as_nanos();
    format!("{prefix}-{}-{nanos}", std::process::id())
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
    let deadline = Instant::now() + Duration::from_secs(5);
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
    let deadline = Instant::now() + Duration::from_secs(5);
    while !path.is_file() {
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {description}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}
