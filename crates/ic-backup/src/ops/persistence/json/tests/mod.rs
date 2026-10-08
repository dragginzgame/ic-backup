use super::*;
use crate::test_support::temp_dir;
use serde::Serializer;

struct FailingSerialize;

impl Serialize for FailingSerialize {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Err(serde::ser::Error::custom(
            "intentional serialization failure",
        ))
    }
}

#[test]
fn bounded_json_size_uses_exact_pretty_encoding_and_original_limits() {
    for value in [
        serde_json::Value::Null,
        serde_json::json!({"escaped": "\n\t\"\\", "utf8": "é雪", "nested": [1, 2]}),
        serde_json::json!({"state": "a".repeat(100_001)}),
    ] {
        let size = serde_json::to_vec_pretty(&value)
            .expect("original encoding")
            .len() as u64;
        for limit in [size, size + 1, u64::MAX] {
            check_json_size(&value, limit).expect("inclusive exact encoding budget");
        }
        for limit in [0, size - 1] {
            assert!(matches!(
                check_json_size(&value, limit),
                Err(PersistenceError::RecordTooLarge { limit: actual }) if actual == limit
            ));
        }
    }
}

#[test]
fn bounded_json_size_preserves_serializer_errors() {
    assert!(matches!(
        check_json_size(&FailingSerialize, u64::MAX),
        Err(PersistenceError::Json(error)) if !error.is_io()
    ));
}

#[test]
fn durable_json_replaces_the_complete_document() {
    let root = temp_dir("canic-backup-durable-json-replace");
    let path = root.join("journal.json");
    fs::create_dir_all(&root).expect("create temp root");
    fs::write(&path, b"previous-document-with-more-bytes").expect("write previous document");

    write_json_durable(&path, &serde_json::json!({"state": "ready"})).expect("replace document");

    let written = fs::read_to_string(&path).expect("read replaced document");
    let decoded: serde_json::Value = serde_json::from_str(&written).expect("decode document");
    assert_eq!(decoded, serde_json::json!({"state": "ready"}));
    assert_no_staging_file(&root, "journal.json");
    fs::remove_dir_all(root).expect("remove temp root");
}

#[test]
fn serialization_failure_preserves_the_previous_document() {
    let root = temp_dir("canic-backup-durable-json-serialize");
    let path = root.join("journal.json");
    fs::create_dir_all(&root).expect("create temp root");
    fs::write(&path, b"previous-document").expect("write previous document");

    let error = write_json_durable(&path, &FailingSerialize)
        .expect_err("serialization failure should reject");

    assert!(matches!(error, PersistenceError::Json(_)));
    assert_eq!(
        fs::read(&path).expect("read previous document"),
        b"previous-document"
    );
    assert_no_staging_file(&root, "journal.json");
    fs::remove_dir_all(root).expect("remove temp root");
}

#[test]
fn rename_failure_removes_the_staging_file() {
    let root = temp_dir("canic-backup-durable-json-rename");
    let path = root.join("journal.json");
    fs::create_dir_all(&path).expect("create conflicting target directory");

    let error = write_json_durable(&path, &serde_json::json!({"state": "ready"}))
        .expect_err("rename over directory should reject");

    assert!(matches!(
        error,
        PersistenceError::Publication(
            ic_host_fs::durable::NamedWriteError::BeforePublication { .. }
        )
    ));
    assert!(path.is_dir());
    assert_no_staging_file(&root, "journal.json");
    fs::remove_dir_all(root).expect("remove temp root");
}

#[test]
fn serialization_runs_once_before_parent_creation_even_when_it_fails() {
    use std::cell::Cell;

    struct Probe<'a> {
        parent: &'a Path,
        calls: &'a Cell<u8>,
        reject: bool,
    }
    impl Serialize for Probe<'_> {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            assert!(!self.parent.exists());
            self.calls.set(self.calls.get() + 1);
            if self.reject {
                Err(serde::ser::Error::custom("failed preflight"))
            } else {
                serializer.serialize_u8(7)
            }
        }
    }

    for create in [false, true] {
        for reject in [false, true] {
            let root = temp_dir("ic-backup-json-preflight");
            fs::create_dir(&root).unwrap();
            let parent = root.join("private/nested");
            let path = parent.join("record.json");
            let calls = Cell::new(0);
            let value = Probe {
                parent: &parent,
                calls: &calls,
                reject,
            };
            let result = if create {
                create_json_durable(&path, &value)
            } else {
                write_json_durable(&path, &value)
            };
            assert_eq!(calls.get(), 1);
            if reject {
                assert!(matches!(result, Err(PersistenceError::Json(_))));
                assert!(!root.join("private").exists());
            } else {
                result.unwrap();
                assert_eq!(fs::read(&path).unwrap(), b"7");
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    for directory in [root.join("private"), parent] {
                        assert_eq!(
                            fs::metadata(directory).unwrap().permissions().mode() & 0o777,
                            0o700
                        );
                    }
                }
            }
            fs::remove_dir_all(root).unwrap();
        }
    }
}

#[cfg(unix)]
#[test]
fn foreign_staging_replacement_retains_typed_cleanup_evidence() {
    use ic_host_fs::durable::NamedWriteError;

    for mode in [PublicationMode::Replace, PublicationMode::CreateNew] {
        let root = temp_dir("ic-backup-json-stage-custody");
        fs::create_dir(&root).unwrap();
        let path = root.join("record.json");
        let displaced = root.join("displaced");
        if mode == PublicationMode::Replace {
            fs::write(&path, b"previous").unwrap();
        }
        let mut foreign = None;
        let error = publish_bytes_at_barriers(&path, b"new", mode, |barrier| {
            if barrier == DurableWriteBarrier::BeforeRename {
                let stage = fs::read_dir(&root)
                    .unwrap()
                    .map(Result::unwrap)
                    .find(|entry| entry.path() != path)
                    .unwrap()
                    .path();
                fs::rename(&stage, &displaced).unwrap();
                fs::write(&stage, b"foreign").unwrap();
                foreign = Some(stage);
            }
        })
        .unwrap_err();
        assert!(matches!(error, PersistenceError::Publication(
            NamedWriteError::BeforePublication { source, cleanup_error: Some(cleanup) }
        ) if source.kind() == io::ErrorKind::InvalidData && cleanup.kind() == io::ErrorKind::InvalidData));
        assert_eq!(fs::read(displaced).unwrap(), b"new");
        assert_eq!(fs::read(foreign.unwrap()).unwrap(), b"foreign");
        if mode == PublicationMode::Replace {
            assert_eq!(fs::read(path).unwrap(), b"previous");
        } else {
            assert!(!path.exists());
        }
        fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(unix)]
#[test]
fn held_parent_publication_does_not_switch_to_a_replaced_path() {
    let root = temp_dir("ic-backup-json-parent-custody");
    fs::create_dir(&root).unwrap();
    let parent = root.join("parent");
    let moved = root.join("moved");
    let path = parent.join("record.json");
    let mut durable = false;
    write_json_durable_at_barriers(&path, &serde_json::json!({"new": true}), |barrier| {
        if barrier == DurableWriteBarrier::BeforeRename {
            fs::rename(&parent, &moved).unwrap();
            fs::create_dir(&parent).unwrap();
            fs::write(&path, b"foreign").unwrap();
        } else {
            durable = true;
        }
    })
    .unwrap();
    assert!(durable);
    assert_eq!(fs::read(path).unwrap(), b"foreign");
    assert_eq!(
        read_json::<serde_json::Value>(&moved.join("record.json"), 1024).unwrap(),
        serde_json::json!({"new": true})
    );
    fs::remove_dir_all(root).unwrap();
}

fn assert_no_staging_file(root: &Path, target_name: &str) {
    let staging_files = fs::read_dir(root)
        .expect("read temp root")
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name() != target_name)
        .collect::<Vec<_>>();
    assert!(staging_files.is_empty(), "staging files remain");
}
