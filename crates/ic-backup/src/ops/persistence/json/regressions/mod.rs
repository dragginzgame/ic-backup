use super::*;
use crate::test_support::temp_dir;
use std::path::PathBuf;

#[test]
fn create_only_publication_preserves_an_existing_record() {
    let root = temp_dir("ic-backup-create-json");
    let path = root.join("journal.json");
    create_json_durable(&path, &serde_json::json!({"intent": "original"})).expect("create record");
    let bytes = fs::read(&path).expect("read exact bytes");
    let error = create_json_durable(&path, &serde_json::json!({"intent": "different"}))
        .expect_err("existing record must not be replaced");
    assert!(
        matches!(error, PersistenceError::Publication(ic_host_fs::durable::NamedWriteError::BeforePublication { source: ref error, .. }) if error.kind() == io::ErrorKind::AlreadyExists)
    );
    assert_eq!(fs::read(&path).expect("read retained bytes"), bytes);
    fs::remove_dir_all(root).expect("remove successful fixture");
}

#[cfg(unix)]
#[test]
fn record_reads_enforce_exact_byte_limits_and_reject_symlinks() {
    let root = temp_dir("ic-backup-record-read");
    let path = root.join("journal.json");
    let value = serde_json::json!({"intent": "snapshot"});
    create_json_durable(&path, &value).expect("create record");
    let length = fs::metadata(&path).expect("record metadata").len();
    let decoded: serde_json::Value = read_json(&path, length).expect("exact bound accepts");
    assert_eq!(decoded, value);
    for bound in [0, length - 1] {
        assert!(
            matches!(read_json::<serde_json::Value>(&path, bound), Err(PersistenceError::RecordTooLarge { limit }) if limit == bound)
        );
    }
    let linked = root.join("linked.json");
    std::os::unix::fs::symlink(&path, &linked).expect("create record link");
    assert!(matches!(
        read_json::<serde_json::Value>(&linked, length),
        Err(PersistenceError::Io(_))
    ));
    assert!(
        matches!(read_json::<serde_json::Value>(&root, length), Err(PersistenceError::Io(ref error)) if error.kind() == io::ErrorKind::InvalidInput)
    );
    fs::write(&path, b"malformed").expect("write malformed record");
    assert!(matches!(
        read_json::<serde_json::Value>(&path, length),
        Err(PersistenceError::Json(_))
    ));
    fs::remove_dir_all(root).expect("remove successful fixture");
}

#[cfg(unix)]
#[test]
fn shared_record_reads_preserve_extreme_limits_missing_files_and_parent_aliases() {
    let root = temp_dir("ic-backup-shared-record-read");
    fs::create_dir_all(&root).expect("create fixture root");
    let path = root.join("record.json");
    fs::write(&path, b"null").expect("write exact JSON");
    assert_eq!(
        read_json::<serde_json::Value>(&path, u64::MAX).expect("large declared limit"),
        serde_json::Value::Null
    );
    assert!(matches!(
        read_json::<serde_json::Value>(&path, 0),
        Err(PersistenceError::RecordTooLarge { limit: 0 })
    ));
    assert_eq!(fs::read(&path).expect("original bytes"), b"null");
    let alias = root.join("parent-alias");
    std::os::unix::fs::symlink(&root, &alias).expect("selected parent alias");
    assert_eq!(
        read_json::<serde_json::Value>(&alias.join("record.json"), 4)
            .expect("parent selection remains caller owned"),
        serde_json::Value::Null
    );
    assert!(matches!(
        read_json::<serde_json::Value>(&root.join("missing"), 4),
        Err(PersistenceError::Io(error)) if error.kind() == io::ErrorKind::NotFound
    ));
    let dangling = root.join("dangling");
    std::os::unix::fs::symlink("missing", &dangling).expect("dangling final link");
    assert!(matches!(
        read_json::<serde_json::Value>(&dangling, 4),
        Err(PersistenceError::Io(error))
            if error.raw_os_error() == Some(rustix::io::Errno::LOOP.raw_os_error())
    ));
    fs::write(&path, []).expect("empty file");
    assert!(matches!(
        read_json::<serde_json::Value>(&path, 0),
        Err(PersistenceError::Json(error)) if error.is_eof()
    ));
    fs::remove_dir_all(root).expect("remove successful fixture");
}

#[cfg(unix)]
#[test]
fn shared_record_reads_reject_real_fifo_and_device_without_waiting_for_a_writer() {
    use std::{process::Command, sync::mpsc, thread, time::Duration};
    let root = temp_dir("ic-backup-shared-record-fifo");
    fs::create_dir_all(&root).expect("create fixture root");
    let path = root.join("record.fifo");
    assert!(
        Command::new("/usr/bin/mkfifo")
            .args(["-m", "600"])
            .arg(&path)
            .env_clear()
            .status()
            .expect("create native FIFO")
            .success()
    );
    let (send, receive) = mpsc::channel();
    let reader = thread::spawn(move || {
        send.send(read_json::<serde_json::Value>(&path, 1024))
            .expect("return actual FIFO admission");
    });
    // Bound only the blocking-open regression, not ordinary filesystem read latency.
    assert!(matches!(
        receive
            .recv_timeout(Duration::from_secs(5))
            .expect("record admission must not wait for a FIFO writer"),
        Err(PersistenceError::Io(error)) if error.kind() == io::ErrorKind::InvalidInput
    ));
    reader.join().expect("join completed admission");
    assert!(matches!(
        read_json::<serde_json::Value>(Path::new("/dev/null"), 1024),
        Err(PersistenceError::Io(error)) if error.kind() == io::ErrorKind::InvalidInput
    ));
    fs::remove_dir_all(root).expect("remove successful fixture");
}

#[cfg(unix)]
#[test]
fn new_machine_records_and_directories_have_private_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let root = temp_dir("ic-backup-private-json");
    let parent = root.join("private");
    let path = parent.join("journal.json");
    create_json_durable(&path, &serde_json::json!({"intent": "snapshot"}))
        .expect("create private record");
    assert_eq!(
        fs::metadata(&path)
            .expect("file metadata")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert_eq!(
        fs::metadata(&parent)
            .expect("parent metadata")
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    fs::remove_dir_all(root).expect("remove successful fixture");
}

#[cfg(unix)]
#[test]
fn process_death_preserves_exact_publication_sides() {
    use crate::test_support::{hold_at_acknowledged_barrier, kill_child_at_acknowledged_barrier};
    use std::process::Command;
    const ROOT_ENV: &str = "IC_BACKUP_TEST_JSON_CRASH_ROOT";
    const OP_ENV: &str = "IC_BACKUP_TEST_JSON_CRASH_OPERATION";
    const SIDE_ENV: &str = "IC_BACKUP_TEST_JSON_CRASH_SIDE";
    let new = serde_json::json!({"new": "complete"});
    if let Some(root) = std::env::var_os(ROOT_ENV) {
        let root = PathBuf::from(root);
        let path = root.join("journal.json");
        let operation = std::env::var(OP_ENV).expect("child operation");
        let side = std::env::var(SIDE_ENV).expect("child barrier");
        if operation == "create" {
            create_json_durable_at_barriers(
                &path,
                &new,
                || {
                    if side == "before" {
                        hold_at_acknowledged_barrier(&root);
                    }
                },
                || {
                    if side == "after" {
                        hold_at_acknowledged_barrier(&root);
                    }
                },
            )
            .expect("create at crash barrier");
        } else {
            write_json_durable_at_barriers(&path, &new, |barrier| {
                if matches!(
                    (side.as_str(), barrier),
                    ("before", DurableWriteBarrier::BeforeRename)
                        | ("after", DurableWriteBarrier::AfterDirectorySync)
                ) {
                    hold_at_acknowledged_barrier(&root);
                }
            })
            .expect("replace at crash barrier");
        }
        panic!("child passed acknowledged crash barrier");
    }
    for operation in ["create", "replace"] {
        for side in ["before", "after"] {
            let root = temp_dir("ic-backup-json-crash");
            fs::create_dir(&root).expect("create crash fixture");
            let path = root.join("journal.json");
            let previous = serde_json::json!({"previous": true});
            if operation == "replace" {
                create_json_durable(&path, &previous).expect("create previous record");
            }
            let mut child = Command::new(std::env::current_exe().expect("test executable"))
                .args(["--exact", "ops::persistence::json::regressions::process_death_preserves_exact_publication_sides", "--nocapture"])
                .env(ROOT_ENV, &root).env(OP_ENV, operation).env(SIDE_ENV, side)
                .spawn().expect("spawn crash child");
            kill_child_at_acknowledged_barrier(&mut child, &root);
            if side == "after" || operation == "replace" {
                let actual: serde_json::Value =
                    read_json(&path, 1024).expect("read surviving record");
                assert_eq!(
                    actual,
                    if side == "after" {
                        new.clone()
                    } else {
                        previous
                    }
                );
            } else {
                assert!(!path.exists());
            }
            fs::remove_dir_all(root).expect("remove verified crash fixture");
        }
    }
}
